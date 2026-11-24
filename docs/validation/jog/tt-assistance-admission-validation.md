# TT assistance-admission correction

Date: November 2, 2026.

Source checks, recorded-input replay and packaged executable checks are
complete. Physical RPM stability and
rim-braking feel still require the user's
[ordered hands-free comparison](tt-rpm-baseline.md).

## Evidence and selected correction

The 14:57 hands-free diagnostic recorded 112 automatic assistance entries in a
settled 28-second interval. Its 20 ms estimate dipped below the 31 RPM admission
threshold while its independent 100 ms speed estimate stayed above it. Each
handoff reset PI state and subsequent recovery rebuilt the regulator window.
No top touch or MIDI gesture ownership was recorded during that interval.

A read-only ultrabrain consultation selected admission confirmation rather than
retaining a stale controller across genuine assistance: retention would also
need timing/history rebasing and an output-transfer policy, without preventing
false assistance entry.

The implemented contract in `src/tension/playback.rs` is:

- Keep the existing 20 ms motor estimate `S`, regulator and force response.
- Calculate a separate `L` from signed encoder travel over a sensor-clock
  window of at least 100 ms. Select the newest endpoint old enough for the
  requested window and divide by its actual elapsed sensor time.
- New automatic underspeed assistance requires qualification or MIDI ownership,
  `S < target - 2`, and available `L < target - 2`.
- Qualification remains `S >= target - 0.75`. MIDI ownership does not bypass
  either speed check or the requirement for a complete long window.
- Top touch or negative `S` selects assistance using existing short-window
  readiness, without waiting for `L`.
- An existing assist continues during touch/reverse or `S < target - 0.75`.
  Otherwise it releases using the short estimate, regardless of old low `L`.
- Genuine assistance transitions still reset PI. Rejected short dips do not.
  Existing full resets clear both measurement histories.

At 33 RPM the admission boundary remains strictly below 31 RPM, and release
remains at or above 32.25 RPM. No gain, bend curve, MIDI deadband, floor,
mapping, motor tuning, force taper, startup, overspeed or authorization setting
was changed. `MOTOR_REGULATOR` adds `assistance_rpm` so the admission estimate is
visible separately from short motor RPM.

## Regression evidence

All required checks passed: formatting, strict all-target/all-feature Clippy,
501 Rust tests, all-target build, 68 Swift tests, shell syntax, plist lint and
diff whitespace. Rust LSP checks on the changed runtime/test files were clean.
The previous run's two competing-process CLI failures were absent in this run.

Four added real-Engine regressions cover:

1. A 52-endpoint excerpt from the recorded right-wheel trace, mirrored to both
   sides with the recorded base force. Untouched ripple remains regulating.
2. Sustained untouched rim braking, full stop assistance and short-window
   recovery while the long window is still low.
3. Missing/exact-threshold long measurements even with MIDI ownership.
4. Exact short-threshold rejection after recovery, full-reset history clearing
   and sensor-timer wrap.

The captured excerpt failed against the original code at host time 22,258,557
microseconds: it selected Assisting instead of Regulating. Removing the new
long-window guard reproduced that failure and premature braking assistance
(sample 106 instead of 134). Making either admission comparison inclusive
caused the strict-boundary tests to fail. All mutations were restored.

Existing touch, reverse, held stop, force taper, duplicate/freshness, paused
Scratch, original-deck ownership and output-service regressions remain passing.

## Full recorded-input replay

A local offline helper linked the actual corrected production Engine and read
the captured runtime settings and all 2,440 right-wheel endpoint records.
It reconstructed the idle left wheel and original authorization/contact state.
It did not open HID or MIDI.

For host times 19,000,000 through 47,000,000 microseconds:

| Result | Original capture | Corrected input replay |
| --- | ---: | ---: |
| Assistance entries | 112 | 0 |
| Qualifying samples after startup | 111 | 0 |
| Corrected regulating samples | Not applicable | 1,237 |
| Minimum long admission estimate | Not recorded in original emitter | 32.03909 RPM |

The replay retained PI correction across the interval rather than resetting it
at the rejected short dips. Its integral ranged from approximately -1,951.75 to
-477.91 force units. Sampling rates differ because mode edges cause additional
records; sample counts are not time percentages.

This is an **open-loop replay of the old measured trajectory**, not a simulation
of how the physical platter responds to new force commands. It establishes
admission behavior for captured endpoints, not improved physical RPM or audio.
The source fixture is also sampled, not a lossless original per-report capture.

## Production executable comparison

Development and actual packaged release executable comparisons against the
preceding RPM-diagnostic package passed these settings, with base force 7000
and identical remaining defaults:

| Floor | Gain | Motor reports | Identical non-motor records | Matched short-window samples |
| ---: | ---: | ---: | ---: | ---: |
| 1% | 1 | 12,915 | 4,571 | 2,354 |
| 1% | 5 | 12,915 | 5,081 | 2,354 |
| 50% | 5 | 12,915 | 4,983 | 2,354 |

Non-motor records match exactly: MIDI, configuration, physical RPM/bend status,
routing and local state. Matched short-window RPM values are unchanged.
Motor commands remain bounded and unidirectional; touch/release overspeed is
zero, touched stop/reverse assistance retains 10,630 force units for this
configuration, and final cleanup is neutral. Gain changes do not change motors.

## Portable comparison package

The app is `dist/tt-handoff-20261102/S4 MK3 Bridge.app`, with ZIP and versioned
DMG alongside it. Version stays `0.2.1-beta.1`, numeric `0.2.1`, build `7`.
Use this separate bundle path, not version alone, to identify the comparison.
The build used `CARGO_PROFILE_RELEASE_STRIP=none` and
`S4_BRIDGE_OUTPUT_DIR="$PWD/dist/tt-handoff-20261102"`.

App signature verification, ZIP integrity and DMG checksum verification passed.
The generated mapping and all three bundled default preference files matched
the preceding RPM-diagnostic package byte-for-byte.
The ZIP extraction and read-only mounted DMG app independently passed signature
verification and matched the local app byte-for-byte. The disk image was
detached and the owned extraction directory removed.

| Artifact | SHA-256 |
| --- | --- |
| Bundled Rust bridge | `863c6e5efec4847889ac3f17692add232daf2e6ab8719604c9dff5bd413dd4f8` |
| Native app executable | `2130d2766cc9e10a20e3d197a9648a25ddf0fef54a166ec637c4a474fc1118f5` |
| ZIP | `29c3d68760469ed65b5d177b9c2854eb5e82db9fc2737cbf5bfaf3b18c0479f6` |
| DMG | `dfd8a119daf49e5a80c65ea782d2a896bd5cf4aa3111eca44d63f76f8c02dd35` |

## Physical comparison limits

Brief deliberate rim braking can resemble the rejected speed ripple. For an
ideal sustained 33-to-30 RPM step, a primed 100 ms estimate crosses below 31 RPM
after roughly 67 ms; the quantized regression crosses at 68 ms. PI remains
active until confirmation and can resist the hand more strongly during that
interval. This is not an additional fixed 100 ms delay on MIDI bend, touch or
reverse. Braking feel remains untested.

This admission-only change does not guarantee that every recovery trajectory
avoids repeated transitions, or that PI quantization/ripple itself is solved.
Repeat the clean hands-free recording first, with all settings unchanged.
Assess sustained braking, hold and recovery in a separate attended recording.
No hardware trial, preference write, project commit, push or publication was
performed during preparation. Prior app bundles and the unrelated Serato
research file remain preserved.

Local evidence and helper sources are under
`.omo/evidence/tt-handoff-fix-20261102/`.
