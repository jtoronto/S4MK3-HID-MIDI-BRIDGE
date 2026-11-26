#!/usr/bin/env bash
# Stage an already-packaged Windows host without building or installing anything.
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: bash windows/qa/ni-bridge/package.sh BUILT_HOST_DIR NEW_PACKAGE_DIR" >&2
    exit 2
fi
host="$1"
output="$2"
here="$(cd "$(dirname "$0")" && pwd)"
for file in S4Bridge.exe Resources/s4mk3-hid-midi-bridge.exe \
    "Resources/S4 MK3 Bridge Windows.djayMidiMapping" runtime-manifest.json; do
    [[ -f "$host/$file" ]] || { echo "Missing packaged host input: $host/$file" >&2; exit 1; }
done
[[ ! -e "$output" && ! -L "$output" ]] || { echo "Output already exists: $output" >&2; exit 1; }
host="$(cd "$host" && pwd -P)"
parent="$(cd "$(dirname "$output")" && pwd -P)"
case "$parent/" in "$host/"*) echo "Output must be outside the built host directory." >&2; exit 1 ;; esac
mkdir "$output"
cp -R "$host" "$output/app"
cp "$here/ATTENDED-RUN.md" "$here/Start-NI-Bridge.cmd" "$output/"
(
    cd "$output"
    shasum -a 256 app/S4Bridge.exe app/Resources/s4mk3-hid-midi-bridge.exe \
        app/runtime-manifest.json ATTENDED-RUN.md Start-NI-Bridge.cmd > SHA256SUMS
)
echo "NI_BRIDGE_PACKAGE_READY $output"
