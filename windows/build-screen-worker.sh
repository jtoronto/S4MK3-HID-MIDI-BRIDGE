#!/usr/bin/env bash
# macOS-only offline cross-build; never executes Windows code or opens devices.
# Reproduces the production worker's Rust 1.88.0 / cargo-xwin 0.20.2 environment.
# Retained tools: Clang 22.1.8, LLD 20.1.5, SDK 10.0.26100, CRT 14.44.17.14.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:?Usage: S4_WINDOWS_TOOL_ROOT=/retained/tools bash windows/build-screen-worker.sh NEW_OUTPUT}"
mkdir -p "$(dirname "$output")"
output="$(cd "$(dirname "$output")" && pwd)/$(basename "$output")"
tools="${S4_WINDOWS_TOOL_ROOT:?Set the retained LLVM/xwin/sysroot tools directory}"
tools="$(cd "$tools" && pwd)"
for input in bin/cargo-xwin llvm/bin/clang llvm/bin/lld-link sysroot/lib/rustlib/x86_64-pc-windows-msvc/lib xwin/xwin/DONE; do
    [[ -e "$tools/$input" ]] || { echo "Missing build input: $tools/$input" >&2; exit 1; }
done
command -v jq >/dev/null
[[ "$(rustc --version)" == "rustc 1.88.0 (6b00bc388 2025-06-23)" ]]
[[ "$(cargo --version)" == "cargo 1.88.0 (873a06493 2025-05-10)" ]]
# mkdir atomically refuses every existing output, including a dangling symlink.
mkdir "$output"
mkdir -p "$output/runtime/tests"
(
    cd "$tools"
    printf '%s\n' \
        '0779bca286743510d1ecab1323da28dceeb32b0a28e890447d0228a354de6097  bin/cargo-xwin' \
        '305697d3868243dca6e4aabf4f628df7de135d701eb4fd78f6a5d7528fef4395  llvm/bin/clang' \
        '94171bbd7d8ee1aaa30a2deb67bd9fdfed5275bf39cbf0b20486b3f06af8ca07  llvm/bin/lld-link' |
        shasum -a 256 -c -
) > "$output/TOOL-HASH-CHECK.txt"
cp -cR "$tools/xwin" "$output/xwin"
export PATH="$tools/bin:$tools/llvm/bin:$PATH"
export XWIN_CACHE_DIR="$output/xwin"
export XWIN_ARCH=x86_64 XWIN_VARIANT=desktop XWIN_VERSION=17
export XWIN_SDK_VERSION=10.0.26100 XWIN_CRT_VERSION=14.44.17.14
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="--sysroot=$tools/sysroot -C target-feature=+crt-static"
export CARGO_TARGET_DIR="$output/target" CARGO_NET_OFFLINE=true
(
    rustc --version --verbose
    cargo --version --verbose
    cargo-xwin --version
    clang --version
    lld-link --version
    printf '%s\n' \
        "XWIN_ARCH=$XWIN_ARCH XWIN_VARIANT=$XWIN_VARIANT XWIN_VERSION=$XWIN_VERSION" \
        "XWIN_SDK_VERSION=$XWIN_SDK_VERSION XWIN_CRT_VERSION=$XWIN_CRT_VERSION" \
        "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=$CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS" \
        "CARGO_TARGET_DIR=$CARGO_TARGET_DIR XWIN_CACHE_DIR=$XWIN_CACHE_DIR"
) > "$output/TOOLCHAIN.txt"
cd "$root"
{
    find windows/encdr-screen-worker \
        \( -path windows/encdr-screen-worker/target -o -path windows/encdr-screen-worker/cross-build \) -prune \
        -o -type f -print
    printf '%s\n' windows/build-screen-worker.sh
} | LC_ALL=C sort | while IFS= read -r input; do
    shasum -a 256 "$input"
done > "$output/SOURCE-SHA256SUMS"
cd "$root/windows/encdr-screen-worker"
cargo xwin build --locked --offline --release --target x86_64-pc-windows-msvc \
    2>&1 | tee "$output/build.log"
cargo xwin test --locked --offline --release --target x86_64-pc-windows-msvc \
    --no-run --message-format=json > "$output/test-artifacts.jsonl" 2> "$output/test-build.log"
cp "$output/target/x86_64-pc-windows-msvc/release/s4-encdr-screen-worker.exe" "$output/runtime/"
for name in s4-encdr-screen-worker cli; do
    artifact="$(jq -ers --arg name "$name" '
        [.[] | select(.reason == "compiler-artifact" and .profile.test
                      and .target.name == $name and .executable != null)]
        | if length == 1 then .[0].executable else error("expected one test artifact for " + $name) end
    ' "$output/test-artifacts.jsonl")"
    cp "$artifact" "$output/runtime/tests/$name.exe"
done
cp SOURCE-NOTICE "$output/runtime/"
encdr_manifest="$(cargo metadata --locked --offline --format-version 1 | jq -er '
    [.packages[] | select(.name == "encdr")]
    | if length == 1 then .[0].manifest_path else error("expected one Encdr package") end
')"
cp "$(dirname "$(dirname "$encdr_manifest")")/LICENSE" "$output/runtime/ENCDr-GPL-LICENSE"
cd "$root"
shasum -a 256 -c "$output/SOURCE-SHA256SUMS" > "$output/SOURCE-HASH-CHECK.txt"
(
    cd "$output/runtime"
    file *.exe tests/*.exe > BINARY-IDENTITY.txt
    shasum -a 256 *.exe tests/*.exe SOURCE-NOTICE ENCDr-GPL-LICENSE BINARY-IDENTITY.txt > SHA256SUMS
)
(
    cd "$output"
    shasum -a 256 SOURCE-SHA256SUMS SOURCE-HASH-CHECK.txt TOOLCHAIN.txt TOOL-HASH-CHECK.txt \
        build.log test-build.log test-artifacts.jsonl runtime/SHA256SUMS > BUILD-SHA256SUMS
)
echo "WINDOWS_SCREEN_WORKER_BUILD_READY $output/runtime"
