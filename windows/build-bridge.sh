#!/usr/bin/env bash
# Cross-build the real worker and its native engine tests with retained tooling.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:?Usage: S4_WINDOWS_TOOL_ROOT=/path/to/retained/tools bash windows/build-bridge.sh NEW_OUTPUT}"
mkdir -p "$(dirname "$output")"
output="$(cd "$(dirname "$output")" && pwd)/$(basename "$output")"
[[ ! -e "$output" ]] || { echo "Output already exists: $output" >&2; exit 1; }
tool_root="${S4_WINDOWS_TOOL_ROOT:?Set S4_WINDOWS_TOOL_ROOT to the retained LLVM/xwin/sysroot directory}"
for path in bin/cargo-xwin llvm/bin/clang llvm/bin/lld-link sysroot/lib/rustlib/x86_64-pc-windows-msvc/lib xwin/xwin/DONE; do
    [[ -e "$tool_root/$path" ]] || { echo "Missing build input: $tool_root/$path" >&2; exit 1; }
done
mkdir -p "$output/runtime/tests"
# Keep all generated cache state out of the retained evidence/tool inputs.
cp -cR "$tool_root/xwin" "$output/xwin"
export PATH="$tool_root/bin:$tool_root/llvm/bin:$PATH"
export XWIN_CACHE_DIR="$output/xwin"
export XWIN_ARCH=x86_64 XWIN_VARIANT=desktop XWIN_VERSION=17
export XWIN_SDK_VERSION=10.0.26100 XWIN_CRT_VERSION=14.44.17.14
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="--sysroot=$tool_root/sysroot -C target-feature=+crt-static"
export CARGO_TARGET_DIR="$output/target"
export CARGO_NET_OFFLINE=true
cd "$root"
cargo xwin build --locked --release --bin s4mk3-hid-midi-bridge --target x86_64-pc-windows-msvc
cargo xwin test --locked --release --lib --bin s4mk3-hid-midi-bridge --test windows_engine_cli --target x86_64-pc-windows-msvc --no-run --message-format=json > "$output/test-artifacts.jsonl"
# The target laptop must not need rustc to run the real subprocess IPC tests.
# This standalone peer uses only stdio and a loopback test socket, never NI/HID.
rustc --edition=2024 --target x86_64-pc-windows-msvc \
    --sysroot "$tool_root/sysroot" \
    -C "linker=$tool_root/llvm/bin/lld-link" -C target-feature=+crt-static -C opt-level=2 \
    -L "native=$output/xwin/xwin/crt/lib/x86_64" \
    -L "native=$output/xwin/xwin/sdk/lib/um/x86_64" \
    -L "native=$output/xwin/xwin/sdk/lib/ucrt/x86_64" \
    src/screens/worker/ni/ipc/fixture.rs \
    -o "$output/runtime/tests/s4-screen-ipc-fixture.exe"
cp "$output/target/x86_64-pc-windows-msvc/release/s4mk3-hid-midi-bridge.exe" "$output/runtime/"
python3 -c '
import json, pathlib, shutil, sys
root = pathlib.Path(sys.argv[1])
artifacts = [json.loads(line) for line in (root / "test-artifacts.jsonl").read_text().splitlines()]
tests = [item for item in artifacts if item.get("reason") == "compiler-artifact" and item.get("profile", {}).get("test") and item.get("executable")]
if len(tests) != 3:
    raise SystemExit(f"Expected shared, production worker and CLI tests; got {len(tests)}")
for item in tests:
    kind = item["target"]["kind"]
    name = "engine-unit.exe" if "bin" in kind else ("engine-cli.exe" if "test" in kind else "shared-unit.exe")
    shutil.copy2(item["executable"], root / "runtime" / "tests" / name)
' "$output"
# Export with the real host-build generator. This mode opens no HID or MIDI.
CARGO_TARGET_DIR="$output/host-target" cargo run --locked --release --bin s4mk3-hid-midi-bridge -- \
    --generate-mapping "$output/runtime/S4 MK3 Bridge Windows.djayMidiMapping" --midi-backend loopbe
cp windows/test.ps1 "$output/runtime/test.ps1"
cp examples/led-config.json "$output/runtime/default-led-config.json"
cp examples/jog-config.json "$output/runtime/default-jog-config.json"
cp examples/tension-config.json "$output/runtime/default-tension-config.json"
(
    cd "$output/runtime"
    shasum -a 256 s4mk3-hid-midi-bridge.exe *.djayMidiMapping default-*.json tests/*.exe test.ps1 > SHA256SUMS
)
echo "WINDOWS_ENGINE_BUILD_READY $output/runtime"
