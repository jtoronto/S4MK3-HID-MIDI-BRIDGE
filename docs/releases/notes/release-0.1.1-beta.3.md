# S4 MK3 Bridge 0.1.1-beta.3

October 21, 2026. macOS build 4.

## Keep the bridge running through brief input gaps

This update fixes an error that could close the bridge during a set when
controller readings paused for 50 ms. The bridge now keeps control and LED
processing running instead of exiting.

Jog tension clears during the gap. Once fresh readings return, each wheel
must move again before assistance or braking resumes automatically. You
don't need to stop and restart the bridge. Your saved tension setting and
the existing touch rules are preserved.

This handles a temporary input gap, not a USB disconnect or a hardware I/O
error. Those still stop the bridge. The cause of the original delay remains
unknown, and physical timeout-recovery testing is still pending.

## Updating

Quit the old bridge app, open **S4 MK3 Bridge-0.1.1-beta.3-arm64.dmg**, and
replace the app in **Applications**. The ZIP is an alternative.

Your saved preferences are preserved. No mapping reinstall or tension
setting change is needed when updating from beta.2. Launch the new app and
start the bridge normally. Diagnostics shows **0.1.1-beta.3 (build 4)**.

The app is for Apple Silicon Macs running macOS 12 or later. It remains
ad-hoc signed, not notarized; the README covers first-launch approval.

## Validation

The recovery source passed 254 Rust and 18 Swift tests before this release
version increment. Tests cover repeated gaps, stale queued readings, new
movement before force returns, independent touch gates, and fatal I/O errors.
See [timeout recovery](tension-timeout-recovery.md) for the evidence and limits.

After the version increment, all 254 Rust and 18 Swift tests passed again,
along with formatting, strict all-target/all-feature Clippy, the locked
all-target build, shell syntax, plist lint, and diff checks. Existing SwiftPM
unhandled-file warnings for the icon and diagnostic build script remain.

The portable app, ZIP and DMG were rebuilt with
`CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh`.
Strict deep signatures, ZIP/DMG integrity, all three bundled defaults,
release versions, and bundled bridge versions passed for the app and the
copies extracted from the ZIP and mounted read-only DMG. Both executables
passed the build's non-system absolute dependency check.

The production Swift controller initialized, saved and reloaded fresh,
temporary preferences from all three copies. No HID or MIDI ports were opened,
no motor commands were sent, and saved user preferences were not changed.
The generated mapping is byte-for-byte identical to beta.2.

Previous packages are preserved in
`dist/previous-beta2-before-beta3-20261021.b8BaWM/`.

## SHA-256

```text
7ef75461a8644b2148ec7b22567fe89a4d1fba45fac395e44f4f0c7df8f9ce09  S4 MK3 Bridge-arm64.zip
a043192cde59fd9184be83483436dc9e750740d009df92a30f119287a14131d5  S4 MK3 Bridge-0.1.1-beta.3-arm64.dmg
```
