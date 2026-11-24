# Release version replacement: 0.2.1-beta.1

October 26, 2026. Internal verification record. End-user notes are in
[release-0.2.1-beta.1.md](release-0.2.1-beta.1.md).

## Scope

After publication of 0.2.0-beta.2, the user requested 0.2.1-beta.1 and explicitly
selected replacement of the published release. Source and bundle metadata now
use release `0.2.1-beta.1`, numeric version `0.2.1` and build `7`. Controller
behavior, mapping and the requested 5.6%/enabled/100% backspin defaults are
unchanged. Explicit saved preferences remain preserved.

GitHub release ID 424776748 was confirmed mutable. The replacement uses a new
annotated tag and rebuilt installers, without rewriting or deleting the
historical `v0.2.0-beta.2` tag. The original local installers and
[validation record](release-0.2.0-beta.2-validation.md) remain preserved.

## Source checks

All 358 Rust tests passed in a single serial no-fail-fast run, followed by all
55 Swift tests. Formatting, strict locked all-target/all-feature Clippy, the
locked all-target build, build-script syntax, Info.plist lint and whitespace
checks passed. The two existing SwiftPM unhandled-file warnings remain.
The later README/image edits do not alter bundled source or resources.

## README correction and screenshot

The user requested a fresh Jog screenshot including Backspin, dark mode to
match the other images, and removal of accepted backspin functionality from
Known issues. README now describes backspins under ordinary Jog usage and
retains actual unresolved faults in its limitations list.

`docs/images/jog-settings.png` is a fresh 1960 x 2168 native dark-mode capture
of the production `BridgeSettingsView`. It shows Hardware, Backspin, Scratch,
Pitch bend and mapping instructions without clipped sections. The shown
backspin defaults are 5.6%, motor assistance enabled and strength 100%.

The capture used an isolated native harness with temporary preference/resource
directories, no bridge executable and an offline NI agent. No Start, Save or
hardware operation was invoked, and no real preferences were changed. The
capture helper was compiled with all production Swift sources; it was closed
after capture. A command-line capture omitted content, so the final image was
saved directly from the native computer capture backend and visually inspected
from its final repository path. It is not a recreated or generated mockup.

## Installers

Built separately with `CARGO_PROFILE_RELEASE_STRIP=none` and
`S4_BRIDGE_OUTPUT_DIR="$PWD/dist/Release-0.2.1-beta.1-20261026"`:

- `S4 MK3 Bridge.app`
- `S4 MK3 Bridge-arm64.zip`
- `S4 MK3 Bridge-0.2.1-beta.1-arm64.dmg`

Build, deep strict signatures, ZIP integrity and DMG verification passed.
The app, extracted ZIP app and read-only-mounted DMG app each passed release,
numeric/build version, bundled CLI version, mapping plist lint and byte
comparisons of all default configurations, font license and mapping.
The mapping matches the preceding accepted release byte-for-byte.
Temporary extraction and mount directories were cleaned up. No app was launched
and no HID, MIDI, motor, screen or audio activity was triggered by these checks.

```text
c4fdf7e1873039eacfd3aebc85770b3a2a16513d7ddf61b633c1d64f31bb48fb  S4 MK3 Bridge-arm64.zip
bba664421da9e6f6e74141b18dffca14c4a4cf8615d50b7c0c0adb29d822a186  S4 MK3 Bridge-0.2.1-beta.1-arm64.dmg
```
