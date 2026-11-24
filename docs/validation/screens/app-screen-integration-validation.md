# App screen integration: internal validation record

October 22, 2026. This is a local rebuild, not a new public release.
The version remains 0.1.1-beta.3, numeric 0.1.1, build 4.
This historical record covers the initial screen integration committed as
`93632c6`, before screen preferences and NI driver handover were added. For the
current build, use the [screen-driver handover guide](screen-driver-handover.md).

## Use the updated app

1. Stop the standalone screen prototype with Control-C. Quit any older
   menu-bar bridge app before replacing its Applications copy.
2. Copy the rebuilt `dist/S4 MK3 Bridge.app` to Applications, or use
   `dist/S4 MK3 Bridge-0.1.1-beta.3-arm64.dmg`. The ZIP is an alternative.
   Keep the S4 powered and connected, with audio routed directly through it
   in Djay. Quit Traktor/Mixxx.
3. Open the updated app. Choose **Full controller + LEDs + screens**, then
   **Start bridge**. There is no separate screen switch or CLI to run.
   Existing LED, jog and tension preferences are retained.
4. Keep the already-installed **S4 Info Screen-20261022** mapping selected.
   If it was never installed, use **Install Djay mapping...**, save under a
   new name, accept Djay's import, and select it for **S4 MK3 MIDI Full**.
   Preserve custom mappings under their existing names.
5. Select HOTCUE or SAMPLES for JUMP / LOOP / KEY / RANGE; select STEMS or
   MUTE for the reactive mute/solo legend. Both use the accepted smooth font.
   MOVE press enters/leaves size selection; Shift in STEMS shows ONLY.
6. Use the app's **Stop bridge** or **Quit S4 MK3 Bridge** when finished.
   Do not start a separate prototype bridge alongside the app.

If screens stay dark, open Diagnostics. `SCREENS_DISABLED` with an interface-4
access error can mean NIHardwareAgent still owns the display interface.
Follow the [historical ownership instructions](screen-prototype-validation.md#local-investigation-commands)
to quit it temporarily and then use Stop/Start in the app.
The app never quits another application automatically. A screen failure does
not stop MIDI, LEDs or the saved tension service.

## Integration boundary

`RunPreferences.arguments` adds `--screens` only to the full profile.
The same argument builder serves Start, Save and restart, and Diagnostics'
command preview. No saved preference field or migration was added.
Minimal MIDI and HID probe do not receive the flag. Controller checks and
mapping exports have separate screen-free argument lists.

The mapping generator and the user-accepted raster implementation are
unchanged by this app integration. The app packages the smooth glyph atlas
inside its bridge executable and retains the OFL notice in Resources.

The previous app/ZIP/DMG are preserved in
`dist/previous-before-app-screens-20261022.FvxSbd/`. Standalone screen prototype
folders and the installed Applications copy were not modified during rebuilding.

## Source verification

All 299 Rust tests and 18 Swift tests passed. Required formatting, strict locked
all-target/all-feature Clippy, locked all-target build, shell syntax, plist
lint and diff checks passed; Swift diagnostics were clean.
The existing SwiftPM warnings for `Assets/AppIcon.png` and
`build-jog-diagnostic.sh` remain.

The existing real-child-process regressions now check the exact full-profile
arguments at Start, require screens across Save and restart, and verify that
controller checks and mapping exports do not receive the flag. Minimal/probe
argument expectations remain unchanged. No HID, motor or audio trial is part of
these regressions.

## Package and UI verification

`dist/S4 MK3 Bridge.app`, `dist/S4 MK3 Bridge-arm64.zip` and
`dist/S4 MK3 Bridge-0.1.1-beta.3-arm64.dmg` were rebuilt with
`CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh`.
The build checked system-only absolute dynamic-library dependencies.

The app and the independently extracted ZIP/mounted read-only DMG copies passed
deep strict signature verification. ZIP integrity and the DMG checksum passed.
All three copies have the source Info.plist/version, byte-identical native app
and bridge executables, matching LED/jog/tension defaults, the font's OFL
notice and a valid matching mapping plist. Each copied bridge exported all 36
page rasters; all 108 outputs matched the previously inspected and user-accepted
smooth-font rasters byte-for-byte. The bundled mapping also matches the installed
Info Screen mapping byte-for-byte, so no remapping is needed for this update.
The DMG was detached after verification.

A native harness compiled the actual `BridgeSettingsView` and `BridgeController`
against the rebuilt Resources with isolated, unsaved preferences. WindowServer
capture and accessibility inspection of the default light Bridge view showed
the complete **Full controller + LEDs + screens** label without clipping.
The model was stopped and its command preview included `--screens`.

The first offscreen `cacheDisplay` capture attempt produced incomplete/blank
SwiftUI layer output and is not valid UI evidence. The native capture backend
then failed during attempts to save further captures. A complete minimum-size/
dark-mode matrix is therefore not claimed; no product layout change was made
to work around a capture-tool failure. The owned QA process was stopped.
No HID or motor service was started during packaging or UI inspection.

## User acceptance

On October 22, 2026, the user tested the integrated app and reported
"Tested and it seems good!", then authorized committing the changes.
This records general acceptance of the app integration, following the earlier
visual acceptance of the smooth-font prototype. It does not claim an exhaustive
all-deck, held-control or audio-coexistence test matrix.

SHA-256:

| Artifact | Hash |
| --- | --- |
| Native app executable | `e2559e8fe45da69b219262469558de2c265a68c7694f604332ff134791c3af8e` |
| Bundled bridge | `89ec414ee7c267d2aed77402e27d5065836eaf16afd658235f1f301daded68e0` |
| ZIP | `7151ff909f3aac313a84a22f39feb2efc541918794e7e007796f6a34c5e716f3` |
| DMG | `4837f73df95dd69a77fc1fee2719a4519998006c0671c9a8f17e2c9f761108e0` |
