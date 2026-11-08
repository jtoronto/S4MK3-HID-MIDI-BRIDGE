#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
project="$root/windows/midi-probe"
cache="$root/diagnostics/windows-tools"
dotnet="${DOTNET:-dotnet}"
package="Windows.Devices.Midi2.0.99.92-preview.11.nupkg"
package_sha="13afc6e6da2adac309ab8c174352f46f1c0ebe2d79e65475bd6274defb73df74"
output="${S4_PROBE_OUTPUT_DIR:-$root/dist/windows-midi-probe}"

mkdir -p "$cache/nuget"
if [[ ! -f "$cache/nuget/$package" ]]; then
    curl --fail --location --silent --show-error \
        "https://github.com/microsoft/MIDI/releases/download/inbox-preview-11/$package" \
        --output "$cache/nuget/$package"
fi
actual="$(shasum -a 256 "$cache/nuget/$package")"
[[ "${actual%% *}" == "$package_sha" ]] || {
    printf 'MIDI API package checksum mismatch\n' >&2
    exit 1
}

export DOTNET_CLI_TELEMETRY_OPTOUT=1
"$dotnet" restore "$project/S4MidiProbe.csproj" -r win-x64 --locked-mode --tl:off \
    --source "$cache/nuget" --source https://api.nuget.org/v3/index.json

export CSWINRT_PACKAGE="${NUGET_PACKAGES:-$HOME/.nuget/packages}/microsoft.windows.cswinrt/2.2.0"
export WINEPREFIX="$cache/wine-cswinrt"
export CSWINRT_METADATA="${NUGET_PACKAGES:-$HOME/.nuget/packages}/microsoft.windows.sdk.net.ref/10.0.26100.57/winmd"
export MIDI_API_PACKAGE="${NUGET_PACKAGES:-$HOME/.nuget/packages}/windows.devices.midi2/0.99.92-preview.11"
export CSWINRT_GENERATED="$project/obj/cswinrt-generated/"
chmod +x "$project/cswinrt-wine.sh"

"$dotnet" publish "$project/S4MidiProbe.csproj" --no-restore \
    -c Release -r win-x64 --self-contained true --tl:off -o "$output" \
    "-p:CsWinRTExe=$project/cswinrt-wine.sh" \
    "-p:CsWinRTWindowsMetadata=$CSWINRT_METADATA" \
    "-p:CsWinRTGeneratedFilesDir=$CSWINRT_GENERATED" \
    "-p:IIDOptimizerInterimDir=$project/obj/iid-optimizer/" \
    "-p:S4ProbeDotnet=$dotnet"

printf 'PROBE_PUBLISHED %s\n' "$output"
