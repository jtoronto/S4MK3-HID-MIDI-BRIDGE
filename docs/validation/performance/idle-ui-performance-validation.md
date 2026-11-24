# Idle logging and native Settings performance

November 2, 2026. This change follows the response-curve comparison build.
It does not change motor commands, regulation, jog curves, MIDI mappings,
screen scheduling or saved preferences.

## Report and reproduction

The user reported high CPU and slow Settings scrolling while the bridge ran,
with unchanged `MOTOR_TT_STATE` Info records roughly every 100 ms, including
when TT was disabled on both sides.

The native regression `testDiagnosticTrafficDoesNotInvalidateSettings` reproduced
200 whole-controller `objectWillChange` notifications from 100 log appends and
100 valid platter snapshots. A subsequent preference edit produced notification
201. Settings observes this controller, so unrelated diagnostic traffic
invalidated the preference view tree, including the curve editors.

## Change

- Normal/Info retains initial TT state and boolean transitions, without periodic
  unchanged-state samples. Requested force is included in Debug records;
  Debug retains bounded periodic force telemetry and immediate transitions.
- Log and live-platter publishers now belong to separate observable objects.
  Only their corresponding diagnostic subviews observe them. Diagnostic traffic
  does not publish preference-controller changes.
- The Diagnostics tab content is mounted only while selected, avoiding hidden
  log layout and stale-status timeline work on other tabs.
- Malformed platter records are parsed before mutating published state.
- The existing bounded log and explicit full motor-capture file remain intact.

The native regression now checks zero preference notifications for the same
200 diagnostic updates, while both diagnostic publishers deliver all 100
updates and a real preference edit still notifies Settings. Invalid telemetry
does not produce a diagnostic update. Existing pipe, capture, stop and restart
tests exercise the actual subprocess path with isolated helper processes.

## Verification

All required source checks passed: formatting, Clippy with warnings denied,
all-target build, shell syntax, plist validation and diff whitespace checks.
All 504 Rust tests and 71 Swift tests passed. Rust and native production-source
diagnostics were clean.

The actual portable executables were compared using the built-in
`--replay-motor-fixture playback` fixture with identical custom curve settings:

| Observation | Previous curve build | Performance build |
| --- | ---: | ---: |
| Info TT-state records | 265 | 27 |
| Repeated unchanged Info state | Present | 0 |
| Force-bearing samples at Debug | 265 | 265 |

All 265 Debug force samples match the former Info samples exactly after
excluding timestamps and severity. All 17,159 structured runtime records
(configuration, motor reports, MIDI, routing and platter status) match
byte-for-byte across both builds and both log levels. This is offline
equivalence evidence, not a physical test.

Actual native Settings content was rendered at 760 and 1000 points in light
and dark appearances for Jog and Diagnostics (eight captures under
`.omo/evidence/idle-ui-20261102/`). The offscreen AppKit capture does not reliably
composite the native tab bar, so it cannot establish tab navigation or scrolling
performance. An initial capture without an AppKit window overlapped tab labels;
it was replaced by window-backed content captures, not treated as a product
defect.
Two independent read-only checks inspected all eight captures. Neither found a
concrete content-overlap or blank-render regression. The log region is visible
at both widths and themes. Lower Jog settings are outside the captured viewport,
and live platter fields show stopped placeholders; scrolling, running-state
presentation and native tab navigation are not established by these captures.

## Portable comparison build

`dist/idle-ui-20261102/` contains the app, ZIP and DMG, built with
`S4_BRIDGE_OUTPUT_DIR` and `CARGO_PROFILE_RELEASE_STRIP=none`. Prior packages
are preserved. Version remains `0.2.1-beta.1` (build 7).

Deep/strict code-signature verification, ZIP integrity and DMG checksum
verification passed. A hash command initially included the app directory and
returned an error after those checks; hashing explicit files succeeded.
Bundled LED, Jog and tension defaults are byte-identical to the preceding curve
build. The temporary QA executable was removed; capture source and PNGs remain
ignored evidence, not package contents.

| Artifact | SHA-256 |
| --- | --- |
| Native executable | `03bbbf2951285f064c586cab5a9c712d395e0c0f1d484f2c970f9c81a2342218` |
| Rust executable | `8074260079a3e1b267d7f9dc2f6b55a12154325f66d6ccb316d6c7d266dea646` |
| ZIP | `bb124963548c47dc6162d7510d543dc778bc73d2c21f79c0142286b1d47804f2` |
| DMG | `50bb27902a1f42010ea06a8ef4b186fd916f3163e882f18c5477dedf3f8e92dd` |

## Limits

The local app was running but its bridge child was stopped when inspected.
No hardware was opened or started for this work. Notification counts establish
the invalidation bug, not a measured CPU percentage or a verified improvement
in physical scrolling on the DJ laptop. Interactive scrolling and live CPU
comparison remain unverified.

After trying the comparison build, the user reported "Much better!" and asked
to commit it. This confirms a subjective improvement in use, not a measured
CPU reduction or exhaustive hardware verification.

The supplied `IOHIDDeviceSetReport` error is separate: an ordinary tension write
failed, cleanup reported an accepted final neutral write, and the bridge
preserved the original error and exited 1. Later runs exited 0. The cause of
the hardware write failure was not reproduced or fixed. It is not suppressed
or converted into a successful exit by this performance change.
