# Hidden Settings and diagnostics freshness

November 2, 2026. Follow-up to
[the earlier idle UI change](idle-ui-performance-validation.md).

This record describes the first comparison build. The subsequent display-cadence
change and measured run 3 results are recorded in
[visible Diagnostics validation](live-diagnostics-cpu-validation.md).

## Instruments evidence

The supplied recordings under `diagnostics/` target the native menu-bar
executable in `dist/tt-backspin-20261102/S4 MK3 Bridge.app`:

- `cpu_profiler_2026112.trace` contains two CPU Profiler runs, lasting 91.24
  and 195.08 seconds. Main-thread stacks account for 99.52% and 99.51% of
  sampled cycle weights. AttributeGraph appears in 68.71% and 58.01% of
  weighted stacks. Inclusive stack percentages overlap and are not Activity
  Monitor CPU percentages.
- `swiftui_profile_20261102.trace` contains 94,247 SwiftUI update records:
  6,337 view-body updates, 17 representable updates, and 87,893 other updates.
  The live platter view and group each evaluate 434 times, while the main
  Settings body evaluates seven times. There are 4,191 internal TimelineView
  updates and 3,756 time-triggered updates to its periodic schedule filter.
- The cause graph attributes 303 platter-group updates to `receivedAt` and
  129 to the snapshot itself. This is not evidence that every notification
  redraws pixels or that the whole Settings body is repeatedly rebuilt.

These exports do not establish exactly when the window was hidden, and do
not measure the Rust child process. They identify optimization targets, not
a measured improvement from the following change.

## Implementation

- Settings keeps its root state, selected tab, and controller, but mounts its
  content only while the native window is visible, not minimized, and has
  visible screen area. Window close, application hide and occlusion events
  update that visibility.
- Unmounting Diagnostics removes its live presentation and incremental log
  subscription. Telemetry decoding, bounded log collection, motor captures,
  and child-process control continue independently.
- Live diagnostics uses one cancellable main-queue deadline for the latest
  sample, only while its view is mounted. New samples replace that deadline;
  leaving Diagnostics cancels it. Expiration refreshes the presentation once,
  rather than using a periodic SwiftUI TimelineView.
- The existing strictly-greater-than-half-a-second stale boundary is retained.
  Reopening Diagnostics checks freshness immediately, including samples
  retained while hidden.

No motor, MIDI, mapping, screen-output, or saved preference format changes are
part of this work.

## Verification

All 512 Rust tests and 76 Swift tests passed. Required formatting, strict Clippy,
all-target build, build-script syntax, plist and diff whitespace checks passed.
Production Swift SourceKit diagnostics were clean. SwiftPM still reports the
existing unhandled `Assets/AppIcon.png` and `build-jog-diagnostic.sh` files;
the standalone test-file SourceKit context does not resolve XCTest, while the
package compiler and the complete test suite succeed.

The new regressions exercise deadline replacement, single expiration without
periodic updates, hidden cancellation, continued raw telemetry ingestion, and
the visible/minimized/occluded state combinations. A window-backed SwiftUI
regression mounts the real Settings view, hides and remounts its Diagnostics
subtree, and confirms retained log text reaches the native text view and the
retained sample is stale on reopening. Existing subprocess tests still cover
telemetry capture and stop behavior.

Temporary mutations established that these tests detect missing deadline
cancellation, ignored occlusion, and removal of the hosted visibility gate.
All mutations were restored before the complete passing run. An initial native
test fixture crashed during autorelease cleanup because its window used
release-on-close ownership; setting `isReleasedWhenClosed = false`, as in the
app, corrected the fixture. No production crash was observed.

Native window-backed content captures cover all six Settings tabs and fresh
and stale platter groups at 760 and 1000 points in light and dark appearances:
32 PNGs under `.omo/evidence/hidden-settings-20261102/`. The full-window
packaged app was also launched stopped and inspected: its Bridge content and
native tab strip were present. That process was then stopped without starting
the bridge. Two independent read-only visual checks inspected all 32 captures
and the eight earlier Jog/Diagnostics baseline captures; neither found a
concrete content, overlap, or paired-theme regression. These static checks do
not establish hide/remount timing or CPU improvement. Offscreen captures do
not reliably composite the native tab strip,
and scrolling beyond the viewport is not established by those captures.
Interactive desktop input was unavailable because Accessibility permission was
denied; interactive close/minimize/occlude transitions and actual CPU reduction
remain for the user's new Instruments capture.

## Portable comparison build

`dist/hidden-settings-20261102/` contains the app, ZIP and DMG. Prior builds
remain untouched. Version remains `0.2.1-beta.1`, numeric `0.2.1`, build `7`.
The app was built with `CARGO_PROFILE_RELEASE_STRIP=none` and a separate
`S4_BRIDGE_OUTPUT_DIR`.

Deep/strict code-signature verification, ZIP integrity and DMG checksum
verification passed. The bundled Rust executable and all three bundled default
configuration files are byte-identical to the profiled `tt-backspin-20261102`
build. No HID, MIDI, screen, or motor session was started for verification.

| Artifact | SHA-256 |
| --- | --- |
| Native executable | `7a6f3c4095bf880cc80fb65dc9ea21d3b2a2d8c2c601b5fb2b7ce5d02cde50d9` |
| Rust executable | `21753dc77918f0ad2447965735187833fff9206bace2d35ec39adcaf13144a01` |
| ZIP | `c79926531761fb8d875b59d9cab268fc90fb803cacac21fa224cf3e69e63138d` |
| DMG | `a9025e183e4b6a201b1a3b23eef429cd41b5c6df9bd85cceb0f755a13e27479a` |

## Comparative capture procedure

Use the comparison app under `dist/hidden-settings-20261102/`, not the previous
`tt-backspin-20261102` executable. Keep the same saved preferences, log level,
controller activity, and Instruments template for both captures.

1. Quit the previous bridge app normally before opening the comparison app.
2. In Instruments, select the comparison app's executable at
   `dist/hidden-settings-20261102/S4 MK3 Bridge.app/Contents/MacOS/S4Bridge`.
3. Start recording, then start the bridge normally if testing a running bridge.
4. Leave the Bridge tab visible for a baseline interval; note the elapsed time.
5. Select Diagnostics and leave it visible; note the elapsed time.
6. Close Settings without quitting the app; note the elapsed time.
7. Reopen Settings from the S4 menu and confirm Diagnostics is still selected,
   with current readings and retained log output.
8. Minimize and restore Settings, then fully cover and uncover it with another
   window. Note each interval.
9. Stop recording and save the trace under a new name in `diagnostics/`.

Compare main-thread cycle weights and SwiftUI activity for matching visible
and hidden intervals. The periodic TimelineView update path should be absent.
The bridge should keep receiving telemetry and any explicitly started motor
capture should continue throughout the hidden intervals.
