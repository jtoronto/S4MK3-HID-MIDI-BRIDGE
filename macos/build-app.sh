#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="S4 MK3 Bridge"
APP_VERSION="$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml")"
APP_BUNDLE="$ROOT/dist/$APP_NAME.app"
SWIFT_SOURCE="$ROOT/macos/S4BridgeApp.swift"
BRIDGE_NAME="s4mk3-hid-midi-bridge"
BRIDGE="$ROOT/target/release/$BRIDGE_NAME"
DMG="$ROOT/dist/$APP_NAME-$APP_VERSION-arm64.dmg"
STAGING="$(mktemp -d "${TMPDIR:-/tmp}/s4mk3-app.XXXXXX")"
STAGED_APP="$STAGING/$APP_NAME.app"

cleanup() {
    rm -rf "$STAGING"
}
trap cleanup EXIT

fail() {
    printf 'build-app: %s\n' "$*" >&2
    exit 1
}

[[ "$(uname -s)" == "Darwin" ]] || fail "must be run on macOS with the Apple SDK installed"
[[ "$(uname -m)" == "arm64" ]] || fail "only Apple Silicon arm64 builds are supported"

for tool in cargo swiftc xcrun otool codesign hdiutil sips iconutil ditto /usr/libexec/PlistBuddy; do
    command -v "$tool" >/dev/null 2>&1 || fail "required tool not found: $tool"
done

[[ -f "$SWIFT_SOURCE" ]] || fail "missing Swift app source: $SWIFT_SOURCE"
[[ -f "$ROOT/examples/led-config.json" ]] || fail "missing default LED config"
[[ -f "$ROOT/examples/jog-config.json" ]] || fail "missing default jog config"
[[ -f "$ROOT/macos/Info.plist" ]] || fail "missing app Info.plist"
[[ -f "$ROOT/macos/Assets/AppIcon.png" ]] || fail "missing app icon"

PLIST_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$ROOT/macos/Info.plist")"
PLIST_RELEASE="$(/usr/libexec/PlistBuddy -c 'Print :S4BridgeReleaseVersion' "$ROOT/macos/Info.plist")"
[[ "$PLIST_RELEASE" == "$APP_VERSION" ]] || fail "update Info.plist release ($PLIST_RELEASE) to match Cargo.toml ($APP_VERSION)"
[[ "$PLIST_VERSION" == "${APP_VERSION%%-*}" ]] || fail "update Info.plist numeric version ($PLIST_VERSION) to match Cargo.toml ($APP_VERSION)"

mkdir -p "$ROOT/dist"
[[ ! -e "$APP_BUNDLE" ]] || fail "output already exists; move or remove it first: $APP_BUNDLE"
[[ ! -e "$DMG" ]] || fail "output already exists; preserve it first: $DMG"

SDK="$(xcrun --sdk macosx --show-sdk-path)"
MACOSX_DEPLOYMENT_TARGET=12.0 cargo build --locked --release --manifest-path "$ROOT/Cargo.toml"
[[ -x "$BRIDGE" ]] || fail "release bridge was not created: $BRIDGE"

mkdir -p "$STAGED_APP/Contents/MacOS" "$STAGED_APP/Contents/Resources"
swiftc \
    -parse-as-library \
    -swift-version 5 \
    -target arm64-apple-macos12.0 \
    -sdk "$SDK" \
    -framework AppKit \
    -framework SwiftUI \
    "$SWIFT_SOURCE" \
    -o "$STAGED_APP/Contents/MacOS/S4Bridge"

cp "$ROOT/macos/Info.plist" "$STAGED_APP/Contents/Info.plist"
cp "$BRIDGE" "$STAGED_APP/Contents/Resources/$BRIDGE_NAME"
cp "$ROOT/examples/led-config.json" "$STAGED_APP/Contents/Resources/default-led-config.json"
cp "$ROOT/examples/jog-config.json" "$STAGED_APP/Contents/Resources/default-jog-config.json"

ICONSET="$STAGING/AppIcon.iconset"
mkdir "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$ROOT/macos/Assets/AppIcon.png" \
        --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    doubled=$((size * 2))
    sips -z "$doubled" "$doubled" "$ROOT/macos/Assets/AppIcon.png" \
        --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$STAGED_APP/Contents/Resources/AppIcon.icns"

check_linked_libraries() {
    local binary="$1"
    otool -L "$binary" |
        awk 'NR > 1 {
        path = $1
        if (path ~ /^\// && path !~ /^\/System\/Library\// && path !~ /^\/usr\/lib\//) {
            print "build-app: non-system absolute dependency: " path > "/dev/stderr"
            found = 1
        }
    }
    END { exit found }' ||
        fail "$binary links a non-system absolute dependency; bundle or remove it before distributing"
}

printf 'Checking linked libraries for non-system absolute dependencies...\n'
check_linked_libraries "$STAGED_APP/Contents/MacOS/S4Bridge"
check_linked_libraries "$STAGED_APP/Contents/Resources/$BRIDGE_NAME"

"$STAGED_APP/Contents/Resources/$BRIDGE_NAME" \
    --generate-mapping "$STAGED_APP/Contents/Resources/S4 MK3 Bridge.djayMidiMapping" \
    --jog-config "$STAGED_APP/Contents/Resources/default-jog-config.json"

codesign --force --sign - "$STAGED_APP/Contents/Resources/$BRIDGE_NAME"
codesign --force --sign - "$STAGED_APP"

mv "$STAGED_APP" "$APP_BUNDLE"
printf 'Created %s\n' "$APP_BUNDLE"
ditto -c -k --sequesterRsrc --keepParent "$APP_BUNDLE" "$ROOT/dist/$APP_NAME-arm64.zip"
printf 'Created %s\n' "$ROOT/dist/$APP_NAME-arm64.zip"

DMG_CONTENTS="$STAGING/dmg"
mkdir "$DMG_CONTENTS"
ditto "$APP_BUNDLE" "$DMG_CONTENTS/$APP_NAME.app"
ln -s /Applications "$DMG_CONTENTS/Applications"
hdiutil create -volname "$APP_NAME $APP_VERSION" \
    -srcfolder "$DMG_CONTENTS" -format UDZO "$DMG"
printf 'Created %s\n' "$DMG"
