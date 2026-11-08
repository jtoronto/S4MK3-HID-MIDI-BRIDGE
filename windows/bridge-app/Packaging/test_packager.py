"""Exercise the real packager with hardware-free, temporary package inputs."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import zipfile

import pytest


PROJECT = Path(__file__).resolve().parent / "S4Package.csproj"
JsonValue = dict[str, "JsonValue"] | list["JsonValue"] | str | int | bool | None
ENCDR_GIT_SOURCE = (
    "git+https://github.com/RufusIbiza/Encdr.git?"
    "rev=a5414658a69c4330d10b95f2359371dfc8550326#"
    "a5414658a69c4330d10b95f2359371dfc8550326"
)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value: dict[str, JsonValue]) -> None:
    path.write_text(json.dumps(value), encoding="utf-8")


def pe_image(marker: int) -> bytes:
    image = bytearray(71)
    struct.pack_into("<H", image, 0, 0x5A4D)
    struct.pack_into("<I", image, 0x3C, 64)
    struct.pack_into("<I", image, 64, 0x4550)
    struct.pack_into("<H", image, 68, 0x8664)
    image[70] = marker
    return bytes(image)


@pytest.fixture(scope="session")
def packager(tmp_path_factory: pytest.TempPathFactory) -> tuple[str, Path]:
    dotnet = os.environ.get("DOTNET") or shutil.which("dotnet")
    if not dotnet:
        pytest.fail("Set DOTNET to the .NET 10 SDK executable")
    build = tmp_path_factory.mktemp("screen-packager-build")
    source = build / "source"
    shutil.copytree(PROJECT.parent, source, ignore=shutil.ignore_patterns("bin", "obj", "__pycache__"))
    result = subprocess.run(
        [
            dotnet,
            "build",
            str(source / PROJECT.name),
            "-c",
            "Release",
            "--output",
            str(build / "bin"),
            "--source",
            str(Path.home() / ".nuget" / "packages"),
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=120,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    return dotnet, build / "bin" / "S4Package.dll"


@pytest.fixture
def inputs(tmp_path: Path) -> dict[str, Path]:
    app = tmp_path / "app"
    resources = app / "Resources"
    resources.mkdir(parents=True)
    (app / "S4Bridge.exe").write_bytes(pe_image(1))
    (resources / "s4mk3-hid-midi-bridge.exe").write_bytes(pe_image(2))
    (resources / "screen-access.json").write_text("{}", encoding="utf-8")
    (resources / "S4 MK3 Bridge Windows.djayMidiMapping").write_text(
        "<plist><string>LoopBe Internal MIDI</string></plist>", encoding="utf-8"
    )
    supervisor = tmp_path / "supervisor.exe"
    worker = tmp_path / "worker.exe"
    supervisor.write_bytes(pe_image(3))
    worker.write_bytes(pe_image(4))

    inventory = tmp_path / "engine-inventory"
    (inventory / "notices").mkdir(parents=True)
    write_json(inventory / "nuget-license-manifest.json", {"packages": []})
    write_json(
        inventory / "rust-license-manifest.json",
        {"packages": [{"name": "process-wrap", "license": "MIT", "notice_files": []}]},
    )
    cache = tmp_path / "packages"
    runtime = cache / "microsoft.netcore.app.runtime.win-x64" / "10.0.14"
    runtime.mkdir(parents=True)
    (runtime / "LICENSE.TXT").write_text("runtime license", encoding="utf-8")
    (runtime / "THIRD-PARTY-NOTICES.TXT").write_text("notices", encoding="utf-8")
    assets = tmp_path / "assets.json"
    write_json(assets, {"libraries": {}, "packageFolders": {str(cache) + "/": {}}})
    return {
        "app": app,
        "assets": assets,
        "engine_inventory": inventory,
        "supervisor": supervisor,
        "worker": worker,
    }


def screen_inventory(
    root: Path, supervisor: Path, worker: Path, source: str | None = None
) -> Path:
    root.mkdir()
    notice_path = "notices/rust/encdr/LICENSE"
    notice = root / notice_path
    notice.parent.mkdir(parents=True)
    notice.write_bytes(b"fixture license")
    cargo_manifest = b'[package]\nname = "encdr"\n'
    archive = root / "source.zip"
    with zipfile.ZipFile(archive, "w") as source_archive:
        source_archive.writestr("encdr/Cargo.toml", cargo_manifest)
    rust_manifest = root / "rust-license-manifest.json"
    write_json(
        rust_manifest,
        {
            "packages": [
                {
                    "name": "encdr",
                    "source": source,
                    "manifest_sha256": digest(cargo_manifest),
                    "license": "GPL-3.0-or-later",
                    "notice_files": [notice_path],
                    "notice_provenance": {notice_path: {"sha256": digest(notice.read_bytes())}},
                }
            ]
        },
    )
    write_json(
        root / "screen-helper-manifest.json",
        {
            "schema_version": 1,
            "supervisor_sha256": digest(supervisor.read_bytes()),
            "worker_sha256": digest(worker.read_bytes()),
            "source_archive": archive.name,
            "source_sha256": digest(archive.read_bytes()),
            "encdr_manifest_path": "encdr/Cargo.toml",
            "encdr_manifest_sha256": digest(cargo_manifest),
            "rust_license_manifest_sha256": digest(rust_manifest.read_bytes()),
        },
    )
    return root


def run_packager(
    packager: tuple[str, Path],
    inputs: dict[str, Path],
    screen_inventory_path: str,
    engine_inventory_path: str | None = None,
) -> subprocess.CompletedProcess[str]:
    dotnet, dll = packager
    return subprocess.run(
        [
            dotnet,
            str(dll),
            str(inputs["app"]),
            str(inputs["assets"]),
            str(inputs["engine_inventory"]) if engine_inventory_path is None else engine_inventory_path,
            "engine-integration",
            str(inputs["supervisor"]),
            str(inputs["worker"]),
            screen_inventory_path,
        ],
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )


def test_private_screen_package_succeeds_without_either_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path]
) -> None:
    result = run_packager(packager, inputs, "", engine_inventory_path="")
    assert result.returncode == 0, result.stderr
    app = inputs["app"]
    manifest = json.loads((app / "runtime-manifest.json").read_text(encoding="utf-8"))
    assert manifest["screen_access_enabled"] is True
    assert manifest["screen_inventory_included"] is False
    assert manifest["dependency_inventory_included"] is False
    assert not (app / "rust-license-manifest.json").exists()
    assert not (app / "nuget-license-manifest.json").exists()
    assert (app / "notices" / "dotnet-10.0.14" / "LICENSE.TXT").is_file()


def test_package_rejects_invalid_optional_engine_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path]
) -> None:
    write_json(inputs["engine_inventory"] / "rust-license-manifest.json", {"packages": []})
    result = run_packager(packager, inputs, "")
    assert result.returncode != 0
    assert not (inputs["app"] / "runtime-manifest.json").exists()


def test_package_rejects_mismatched_optional_nuget_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path], tmp_path: Path
) -> None:
    package = tmp_path / "packages" / "example" / "1.0"
    package.mkdir(parents=True)
    (package / "example.1.0.nupkg").write_bytes(b"actual package")
    write_json(
        inputs["assets"],
        {
            "libraries": {"example/1.0": {"type": "package", "path": "example/1.0"}},
            "packageFolders": {str(tmp_path / "packages") + "/": {}},
        },
    )
    write_json(
        inputs["engine_inventory"] / "nuget-license-manifest.json",
        {"packages": [{"name": "example", "version": "1.0", "package_sha256": "wrong", "license": "MIT"}]},
    )
    result = run_packager(packager, inputs, "")
    assert result.returncode != 0
    assert not (inputs["app"] / "runtime-manifest.json").exists()


def test_screen_inventory_can_be_used_without_engine_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path], tmp_path: Path
) -> None:
    inventory = screen_inventory(tmp_path / "screen-inventory", inputs["supervisor"], inputs["worker"])
    result = run_packager(packager, inputs, str(inventory), engine_inventory_path="")
    assert result.returncode == 0, result.stderr
    app = inputs["app"]
    manifest = json.loads((app / "runtime-manifest.json").read_text(encoding="utf-8"))
    assert manifest["screen_inventory_included"] is True
    assert manifest["dependency_inventory_included"] is False
    assert (app / "sources" / "screen-helper-source.zip").is_file()


def test_screen_package_succeeds_without_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path]
) -> None:
    result = run_packager(packager, inputs, "")
    assert result.returncode == 0, result.stderr
    app = inputs["app"]
    assert (app / "Resources" / "s4-encdr-screen-worker.exe").read_bytes() == inputs[
        "worker"
    ].read_bytes()
    manifest = json.loads((app / "runtime-manifest.json").read_text(encoding="utf-8"))
    assert manifest["screen_access_enabled"] is True
    assert manifest["screen_inventory_included"] is False
    assert not (app / "screen-helper-manifest.json").exists()
    assert not (app / "sources" / "screen-helper-source.zip").exists()
    with zipfile.ZipFile(str(app) + ".zip") as archive:
        assert "Resources/s4-encdr-screen-worker.exe" in archive.namelist()
        assert "sources/screen-helper-source.zip" not in archive.namelist()


@pytest.mark.parametrize("source", [None, ENCDR_GIT_SOURCE])
def test_screen_package_preserves_valid_optional_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path], tmp_path: Path, source: str | None
) -> None:
    inventory = screen_inventory(
        tmp_path / "screen-inventory", inputs["supervisor"], inputs["worker"], source
    )
    result = run_packager(packager, inputs, str(inventory))
    assert result.returncode == 0, result.stderr
    app = inputs["app"]
    assert (app / "sources" / "screen-helper-source.zip").read_bytes() == (
        inventory / "source.zip"
    ).read_bytes()
    assert (app / "screen-notices" / "rust" / "encdr" / "LICENSE").read_bytes() == (
        inventory / "notices" / "rust" / "encdr" / "LICENSE"
    ).read_bytes()
    manifest = json.loads((app / "runtime-manifest.json").read_text(encoding="utf-8"))
    assert manifest["screen_inventory_included"] is True
    assert manifest["dependency_inventory_included"] is True
    with zipfile.ZipFile(str(app) + ".zip") as archive:
        assert archive.read("sources/screen-helper-source.zip") == (inventory / "source.zip").read_bytes()


@pytest.mark.parametrize("corruption", ["worker", "source", "notice", "encdr-source"])
def test_screen_package_rejects_invalid_optional_inventory(
    packager: tuple[str, Path], inputs: dict[str, Path], tmp_path: Path, corruption: str
) -> None:
    source = "git+https://example.invalid/unrelated.git?rev=1#1" if corruption == "encdr-source" else None
    inventory = screen_inventory(
        tmp_path / "screen-inventory", inputs["supervisor"], inputs["worker"], source
    )
    if corruption == "worker":
        inputs["worker"].write_bytes(pe_image(5))
    if corruption == "source":
        (inventory / "source.zip").write_bytes(b"changed")
    if corruption == "notice":
        (inventory / "notices" / "rust" / "encdr" / "LICENSE").write_bytes(b"changed")
    result = run_packager(packager, inputs, str(inventory))
    assert result.returncode != 0
    assert not (inputs["app"] / "runtime-manifest.json").exists()
