# Visible Diagnostics display cadence

November 3, 2026. Follow-up to
[hidden Settings and freshness work](hidden-settings-cpu-validation.md).

## Evidence and scope

Run 2 of `diagnostics/swiftui_profile_20261102.trace` targets
`dist/hidden-settings-20261102/S4 MK3 Bridge.app`. The user identifies Settings
closure at 31.8 seconds. From 32–43 seconds, its 100-microsecond Time Profiler
sample weights sum to 62.8 ms: approximately 0.57% of one CPU core. There are
zero SwiftUI view-body updates in that interval, while telemetry decoding
continues. The earlier hidden-window fix is working.

With Diagnostics visible at 15–30 seconds, sampled native-app CPU is about
10.07% of one core, versus 14.11% in the corresponding original-run interval.
Different user actions and view durations prevent treating the cross-run
comparison as a controlled benchmark. These traces do not measure the Rust
child.

The new run contains 169 platter-group cause edges: 135 changing snapshots,
33 timestamp-only changes, and one broader state change. The remaining target
is visible presentation work, not a continuing hidden render loop.

## Implementation

The raw `LivePlatterDiagnostics.state` publisher still receives every valid
snapshot. Its separate `PlatterDiagnosticsDisplay` is the only object observed
by the live view. Raw receipt notifications therefore do not invalidate that
view.

Routine display changes are limited to four publications per second. An
incoming burst replaces the retained latest state without moving an already
pending display refresh, so the next refresh presents the newest sample
without debounce starvation. No refresh repeats without new input.

Presentation equality uses all rendered wheel fields, including the formatted
one-decimal speed and bend values, deck, mode and muted state. Sub-tenth
differences or timestamp-only updates do not publish unchanged readouts.
The exact unrounded telemetry remains available in the raw state and capture.

Opening the view and bridge start/stop update its presentation immediately.
The independent one-shot stale deadline still follows the latest raw receipt,
not the slower display cadence. Hiding the view cancels both pending display
work and its stale deadline. Reopening uses the latest raw snapshot and checks
its current freshness.

No Rust, motor, MIDI, mapping, screen-output, log-view, or preference-format
changes are part of this work.

## Verification

All 512 Rust tests and 79 Swift tests passed. Formatting, strict Clippy,
all-target build, build-script syntax, plist validation and diff whitespace
checks passed. Production Swift SourceKit diagnostics were clean. The existing
SwiftPM warnings for unhandled `Assets/AppIcon.png` and
`build-jog-diagnostic.sh` remain.

Focused regressions establish:

- A synchronous burst of 100 differing snapshots delivers all 100 raw
  notifications but only one pending display publication, containing the newest
  sample, deck, mode, speed and bend values.
- Sub-tenth differences with identical formatted values do not publish another
  fresh readout. The independent stale deadline still publishes the transition.
- Hidden cancellation, reopening freshness and absence of repeated expiration
  updates remain covered.
- The real mounted SwiftUI platter view responds to bridge stop without waiting
  for a new snapshot.
- The actual subprocess pipe delivers 100 valid snapshots and the capture file
  retains all 100 distinct sequence records; malformed telemetry remains omitted.

Temporary mutations removing coalescing, duplicate suppression and the mounted
stop handler each caused their corresponding regression to fail. All mutations
were restored before the complete passing suite. An initial new capture-count
assertion also counted the capture header's event list; the corrected assertion
counts actual `S4_PLATTER_STATUS` records, not header mentions.

Twelve window-backed PNGs under `.omo/evidence/live-diagnostics-20261103/`
cover Diagnostics and the fresh/stale readout states at 760 and 1000 points in
light and dark appearances. Parent inspection and two scoped read-only checks
collectively inspected all twelve; no concrete content, overlap or theme
regression was found. The layout and formatting are unchanged. These static
captures do not establish the live cadence or CPU reduction; the runtime
regressions establish publication behavior. Native offscreen tab-strip
compositing and the harness's Unknown version remain capture limitations.

## Portable comparison build

`dist/live-diagnostics-20261103/` contains the app, ZIP and DMG. The build used
`CARGO_PROFILE_RELEASE_STRIP=none` and a separate `S4_BRIDGE_OUTPUT_DIR`;
earlier packages remain untouched. Version remains `0.2.1-beta.1`, build `7`.
Deep/strict signature verification, ZIP integrity and DMG checksum verification
passed. The bundled Rust executable and all three default configuration files
are byte-identical to the `hidden-settings-20261102` build.

| Artifact | SHA-256 |
| --- | --- |
| Native executable | `1ae71834ba95e35792eace256295a642deef8f776a2a2efc855b9d7df1a14a49` |
| ZIP | `4b40bc1f33b8df054fccf5fd39a6c43c52f139db48c20b19679a157221d22715` |
| DMG | `d926c57b00b8d559bea6e5b8f8a267d71fb954c46bc6fd6d95884adf243b41f7` |

No HID, MIDI, screen or motor session was started for build verification.
Subsequent user-supplied profiling evidence is recorded below; it does not
establish individual hardware-control or musical-response behavior.

## Run 3 profiling follow-up

Run 3 of `diagnostics/swiftui_profile_20261102.trace` confirms the comparison
app at `dist/live-diagnostics-20261103/S4 MK3 Bridge.app`, PID 59023. It was
recorded on November 2, 2026, from 19:35:50 to 19:36:30 local time, for
40.545547 seconds.

For the matching active Diagnostics interval at 15–27 seconds:

| Recording | Sampled native-app CPU time | Estimated share of one CPU core |
| --- | ---: | ---: |
| Run 2 | 1,317.8 ms over 12 seconds | 10.98% |
| Run 3 | 545.4 ms over 12 seconds | 4.55% |

This is approximately 58.6% lower observed CPU. Different controller activity
and user actions prevent treating it as a controlled benchmark. These are
Time Profiler sample-weight estimates for the native app, not total system
usage or measurements of the Rust child.

Run 3 has 52 routine platter-group body evaluations from 13.2937 to 27.0606
seconds, approximately 3.70 per second. The shortest observed interval is about
253.6 ms, consistent with the four-per-second ceiling.

From 28–31 seconds, telemetry decoding continues but there are zero SwiftUI
view-body updates and zero cause-graph rows. None of the sampled stacks in
that interval pass through SwiftUI, SwiftUICore or AttributeGraph. Sampled
native-app CPU time is 14.6 ms over three seconds, approximately 0.49% of one
core. Settings visibility changes and its content unmounts around
36.16–36.18 seconds, so the earlier quiet interval demonstrates stable-value
suppression rather than merely closing the window.

The remaining active-readout cost is primarily SwiftUI dependency and layout
processing; telemetry parsing and log handling are much smaller contributors.

## Repeat capture procedure

Use `dist/live-diagnostics-20261103/S4 MK3 Bridge.app` for repeat recordings,
preserving `dist/hidden-settings-20261102/` for comparison. Both use version
`0.2.1-beta.1`, build `7`; select the executable by its full path in Instruments.

1. Quit the previous bridge app normally.
2. Select
   `dist/live-diagnostics-20261103/S4 MK3 Bridge.app/Contents/MacOS/S4Bridge`
   as the Instruments target.
3. Keep the same preferences, log level and controller activity as run 2.
4. Record a settled interval with Diagnostics visible. Note its start/end times.
5. Close Settings, note the closure time, and record a hidden interval.
6. Reopen Diagnostics and confirm the readouts show the current sample rather
   than replaying the intermediate hidden samples.
7. Stop recording and save a new run or trace without replacing the earlier runs.

The visible platter view should not evaluate at every telemetry receipt, and
stable displayed values should not cause routine readout updates. Compare
matching active and stable intervals rather than whole recordings with
different startup, window and shutdown activity.
