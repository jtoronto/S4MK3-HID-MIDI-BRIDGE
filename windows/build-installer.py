#!/usr/bin/env python3
"""Build an optional per-user installer from build-app.sh's portable ZIP."""

import argparse
import hashlib
import json
import ntpath
import os
from pathlib import Path, PureWindowsPath
import re
import shutil
import stat
import subprocess
import sys
import tempfile
from zipfile import BadZipFile, ZipFile


class PackageError(Exception):
    """The input or compiler output is not a valid bridge package."""


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def zip_path(name: str) -> bool:
    is_reserved = getattr(ntpath, "isreserved", None)
    parts = name.rstrip("/").split("/")
    return all(
        part not in {"", ".", ".."}
        and not re.search(r'[\x00-\x1f\\:*?"<>|]', part)
        and not part.endswith((" ", "."))
        and not (is_reserved(part) if is_reserved else PureWindowsPath(part).is_reserved())
        for part in parts
    )


def extract_payload(archive: Path, destination: Path) -> str:
    """Extract only a complete match to build-app's verified runtime ledger."""
    checksum = archive.with_name(archive.name + ".sha256").read_text(encoding="utf-8")
    archive_hash = digest(archive)
    if checksum != f"{archive_hash}  {archive.name}\n":
        raise PackageError("Portable ZIP does not match its .sha256 sidecar.")
    with ZipFile(archive) as source:
        names = [entry.filename for entry in source.infolist()]
        if (
            any(not zip_path(name) for name in names)
            or len({name.rstrip("/").casefold() for name in names}) != len(names)
            or any(stat.S_ISLNK(entry.external_attr >> 16) for entry in source.infolist())
        ):
            raise PackageError("Portable ZIP contains an unsafe or conflicting path.")
        files = {name for name in names if not name.endswith("/")}
        manifest_name = "runtime-manifest.json"
        required = {
            manifest_name,
            "S4Bridge.exe",
            "Resources/s4mk3-hid-midi-bridge.exe",
            "Resources/S4 MK3 Bridge Windows.djayMidiMapping",
        }
        if not required.issubset(files):
            raise PackageError("Portable ZIP must include the full Windows bridge and mapping.")
        manifest = json.loads(source.read(manifest_name))
        if (
            manifest["version"] != 1
            or manifest["kind"] != "engine-integration"
            or manifest["rid"] != "win-x64"
            or manifest["host_only"] is not False
            or manifest["screen_access_enabled"] is not False
        ):
            raise PackageError("Expected an ordinary integrated Windows x64 build, not a diagnostic package.")
        records = manifest["files"]
        paths = {record["Path"] for record in records}
        if files - {manifest_name} != paths or len(paths) != len(records):
            raise PackageError("Portable ZIP does not match the runtime manifest file list.")
        for record in records:
            path = record["Path"]
            if source.getinfo(path).file_size != record["Bytes"]:
                raise PackageError(f"File size mismatch: {path}")
        for name in files:
            source.extract(name, destination)
        for record in records:
            path = record["Path"]
            if digest(destination / path) != record["Sha256"]:
                raise PackageError(f"File hash mismatch: {path}")
    return archive_hash


def compile_installer(
    compiler: Path, payload: Path, output: Path, version: str,
) -> None:
    script = Path(__file__).resolve().parent / "installer" / "S4Bridge.iss"
    if os.name == "nt":
        command = [str(compiler)]
        script_arg, payload_arg, output_arg = map(str, (script, payload, output))
    else:
        if not os.environ.get("WINEPREFIX"):
            raise PackageError("Set WINEPREFIX to an existing isolated Inno Setup prefix.")
        command = ["wine", str(compiler)]
        script_arg, payload_arg, output_arg = [
            subprocess.check_output(["winepath", "-w", str(path)], text=True).strip()
            for path in (script, payload, output)
        ]
    subprocess.run([
        *command, "/Qp", f"/O{output_arg}", f"/DPayloadDir={payload_arg}",
        f"/DBridgeVersion={version}", script_arg,
    ], check=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path, help="Portable ZIP with its .sha256 sidecar")
    parser.add_argument("--output", required=True, type=Path, help="Existing output directory")
    parser.add_argument("--iscc", required=True, type=Path, help="Inno Setup 7 ISCC.exe")
    parser.add_argument("--version", required=True, help="Version in the portable build, e.g. 0.3.0-beta.1")
    args = parser.parse_args()
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[A-Za-z0-9]+(?:[.-][A-Za-z0-9]+)*)?", args.version, re.ASCII):
        parser.error("Expected a dotted version, optionally with a prerelease suffix.")

    try:
        result = args.output.resolve() / f"S4-MK3-Bridge-{args.version}-win-x64-Setup.exe"
        sidecar = result.with_suffix(".exe.sha256")
        if result.exists() or sidecar.exists():
            raise PackageError("Installer output already exists; use a fresh output location.")
        with tempfile.TemporaryDirectory(prefix="s4-setup-", dir=result.parent) as staging:
            work = Path(staging)
            runtime = work / "payload"
            archive_hash = extract_payload(args.archive.resolve(strict=True), runtime)
            compile_installer(args.iscc.resolve(strict=True), runtime, work, args.version)
            candidate = work / result.name
            installer_hash = digest(candidate)
            with candidate.open("rb") as source, result.open("xb") as target:
                shutil.copyfileobj(source, target)
            with sidecar.open("x", encoding="utf-8") as target:
                target.write(f"{installer_hash}  {result.name}\n")
        print(f"PORTABLE_SHA256 {archive_hash}\nINSTALLER_SHA256 {installer_hash}\nCreated {result}")
    except (PackageError, OSError, KeyError, TypeError, ValueError, BadZipFile, subprocess.SubprocessError) as error:
        print(f"build-installer: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
