#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/windows/bridge-app"
DOTNET="${DOTNET:-dotnet}"
OUTPUT=""
ENGINE=""
MAPPING=""
INVENTORY="${S4_BRIDGE_LICENSE_INVENTORY:-}"
SCREEN_ACCESS=""
SCREEN_SUPERVISOR=""
SCREEN_WORKER=""
SCREEN_INVENTORY=""
MODE="engine-integration"
fail() { printf 'build-app: %s\n' "$*" >&2; exit 1; }
while (($#)); do
    case "$1" in
        --host-only) MODE="host-only"; shift ;;
        --output) OUTPUT="$2"; shift 2 ;;
        --engine) ENGINE="$2"; shift 2 ;;
        --mapping) MAPPING="$2"; shift 2 ;;
        --inventory) INVENTORY="$2"; shift 2 ;;
        --screen-access) SCREEN_ACCESS="$2"; shift 2 ;;
        --screen-supervisor) SCREEN_SUPERVISOR="$2"; shift 2 ;;
        --screen-worker) SCREEN_WORKER="$2"; shift 2 ;;
        --screen-inventory) SCREEN_INVENTORY="$2"; shift 2 ;;
        *) fail "Unknown argument: $1" ;;
    esac
done
[[ -n "$OUTPUT" ]] || fail "Pass --output with a fresh absolute directory."
[[ "$OUTPUT" == /* ]] || fail "--output must be absolute."
[[ ! -e "$OUTPUT" && ! -e "$OUTPUT.zip" ]] || fail "Output exists; preserve it and choose a new path."
if [[ -n "$INVENTORY" ]]; then
    [[ -d "$INVENTORY/notices" && -f "$INVENTORY/nuget-license-manifest.json" ]] ||
        fail "Pass --inventory pointing to the retained reviewed license/runtime inventory."
fi
[[ "$("$DOTNET" --version)" == "10.0.403" ]] || fail "Use retained .NET SDK 10.0.403 via DOTNET."
if [[ "$MODE" == "engine-integration" ]]; then
    [[ -f "$ENGINE" ]] || fail "Pass --engine with the actual worker-built win-x64 Rust binary, or explicitly select --host-only."
    [[ -f "$MAPPING" ]] || fail "Pass --mapping with that actual engine's generated Windows mapping."
else
    [[ -z "$ENGINE" && -z "$MAPPING" ]] || fail "--host-only must not include an engine or mapping."
fi
if [[ -n "$SCREEN_ACCESS" ]]; then
    [[ "$MODE" == "engine-integration" ]] || fail "Screen acceptance cannot be attached to a host-only handoff."
    [[ -f "$SCREEN_SUPERVISOR" ]] || fail "Pass --screen-supervisor with the built s4-screen-supervisor.exe."
    [[ -f "$SCREEN_WORKER" ]] || fail "Pass --screen-worker with the built s4-encdr-screen-worker.exe."
    if [[ -n "$SCREEN_INVENTORY" ]]; then
        [[ -d "$SCREEN_INVENTORY/notices" &&
           -f "$SCREEN_INVENTORY/screen-helper-manifest.json" &&
           -f "$SCREEN_INVENTORY/rust-license-manifest.json" ]] ||
            fail "Pass --screen-inventory with the reviewed production helper source and license inventory."
    fi
elif [[ -n "$SCREEN_SUPERVISOR" || -n "$SCREEN_WORKER" || -n "$SCREEN_INVENTORY" ]]; then
    fail "Screen helper binaries and inventory require --screen-access."
fi
export DOTNET_CLI_TELEMETRY_OPTOUT=1
export DOTNET_CLI_DO_NOT_USE_MSBUILD_SERVER=1
export MSBUILDDISABLENODEREUSE=1
WORK="$(mktemp -d "${TMPDIR:-/tmp}/s4-app-build.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
printf 'BUILD_WORKSPACE %s\n' "$WORK"
PACKAGE_CACHE="${NUGET_PACKAGES:-$HOME/.nuget/packages}"
COMMON_PROPS=("-p:ArtifactsPath=$WORK/dotnet" "-p:UseSharedCompilation=false")
SDK_PROPS=()
case "$(uname -s)" in
    Darwin|Linux)
        command -v wine >/dev/null || fail "Wine is required for the Windows SDK projection."
        export WINEPREFIX="${S4_BRIDGE_WINEPREFIX:-${WINEPREFIX:-}}"
        [[ -n "$WINEPREFIX" && -d "$WINEPREFIX" ]] ||
            fail "Set S4_BRIDGE_WINEPREFIX to an existing isolated Wine prefix."
        export CSWINRT_PACKAGE="$PACKAGE_CACHE/microsoft.windows.cswinrt/2.2.0"
        export CSWINRT_METADATA="$PACKAGE_CACHE/microsoft.windows.sdk.net.ref/10.0.26100.57/winmd"
        export MIDI_API_PACKAGE="$PACKAGE_CACHE/windows.devices.midi2/0.99.92-preview.11"
        export CSWINRT_GENERATED="$WORK/cswinrt/"
        SDK_PROPS=(
            "-p:CsWinRTExe=$ROOT/windows/midi-probe/cswinrt-wine.sh"
            "-p:CsWinRTWindowsMetadata=$CSWINRT_METADATA"
            "-p:CsWinRTGeneratedFilesDir=$CSWINRT_GENERATED"
            "-p:IIDOptimizerInterimDir=$WORK/iid/"
            "-p:S4ProbeDotnet=$DOTNET"
        )
        ;;
esac
mkdir -p "$OUTPUT"
# The retained global NuGet cache is the sole source: no network restore.
"$DOTNET" restore "$APP/S4Bridge.csproj" --locked-mode \
    --source "$PACKAGE_CACHE" "${COMMON_PROPS[@]}" --tl:off
"$DOTNET" publish "$APP/S4Bridge.csproj" --no-restore -c Release -r win-x64 \
    --self-contained true -p:PublishTrimmed=false -p:PublishSingleFile=false \
    "${COMMON_PROPS[@]}" "${SDK_PROPS[@]}" --tl:off -o "$OUTPUT"
if [[ "$MODE" == "engine-integration" ]]; then
    cp "$ENGINE" "$OUTPUT/Resources/s4mk3-hid-midi-bridge.exe"
    cp "$MAPPING" "$OUTPUT/Resources/S4 MK3 Bridge Windows.djayMidiMapping"
fi
if [[ -n "$SCREEN_ACCESS" ]]; then
    cp "$SCREEN_ACCESS" "$OUTPUT/Resources/screen-access.json"
fi
"$DOTNET" restore "$APP/Packaging/S4Package.csproj" --source "$PACKAGE_CACHE" "${COMMON_PROPS[@]}" --tl:off
"$DOTNET" run --project "$APP/Packaging/S4Package.csproj" --no-restore -c Release "${COMMON_PROPS[@]}" --tl:off -- \
    "$OUTPUT" "$WORK/dotnet/obj/S4Bridge/project.assets.json" "$INVENTORY" "$MODE" \
    "$SCREEN_SUPERVISOR" "$SCREEN_WORKER" "$SCREEN_INVENTORY"
printf 'Created %s and %s.zip\n' "$OUTPUT" "$OUTPUT"
