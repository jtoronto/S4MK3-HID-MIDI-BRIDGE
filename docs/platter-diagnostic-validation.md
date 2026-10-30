# Platter diagnostic software readiness

October 27, 2026. Software preparation passed; attended passive measurements,
bounded powered trials and ring-following checks are recorded below.
Use [the ordered runbook](platter-diagnostic.md).

## October 30 source checkpoint

The standalone tools are retained for future sensor calibration, isolated
motor tuning, ring checks and input-timing investigations. They remain separate
from Djay playback and do not replace the production bridge.

Before committing this checkpoint, formatting, strict all-target/all-feature
Clippy, the full Rust test suite, all-target build, 62 Swift tests, shell syntax,
plist lint and whitespace checks passed. Rust unit counts were 51 library,
216 bridge, 175 jog diagnostic and two screen tests (444 total); integration
targets also passed. Top-level help and all four platter subcommand help pages
exited zero without entering hardware modes. Two existing Swift unhandled-file
warnings remain. No UI validation or new physical trial was run.

The older evidence and counts below describe their named revisions rather than
this checkpoint.

## Delivered

- `platter-measure`: no HID output, relative raw-count integration, travel,
  counts/sec and optional calibrated RPM.
- `platter-drive`: one selected physical wheel, explicit raw polarity,
  MOVE release/press/release arming, fixed force capped at 2,000 and each pulse
  capped at 2,000 ms. Touch cancels; release alone does not restart.
- `platter-ring`: independently supplied encoder calibration, relative physical
  following, touch hold and release/resume without replaying held movement.
- JSONL raw frames, decoded motion, state transitions, requested output,
  transport results, totals and stop reasons.

Normal bridge runtime, app settings and mappings were not changed for this
diagnostic. Legacy diagnostic helpers gained sibling-module visibility; their
behavior was not changed.

## Software evidence

Passed:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
CARGO_PROFILE_RELEASE_STRIP=none cargo build --locked --release --bin s4mk3-jog-diagnostic
swift test --package-path macos
bash -n macos/build-app.sh
plutil -lint macos/Info.plist
git diff --check
```

The full Rust suite includes 135 tests in the standalone jog diagnostic.
Swift passed 55 tests. New deterministic regressions exercise real decoder
wrapping, calibrated speed, selected-wheel arming and isolation, touch/re-arm,
ring wrapping and hold/resume, pulse/overall deadlines, saturated input,
duplicate-sensor freshness, Ctrl-C, reader failure, short/failed writes and
neutral cleanup with failing logging. These establish software behavior, not
motor response.

Three test-only Clippy findings were corrected before the passing run.
The first ordinary release build failed loading proc-macro dylibs, including
`mis-aligned LINKEDIT string pool`. Rebuilding with stripping disabled passed,
using the existing packaging workaround; no dependency or toolchain was changed.

The release executable's top-level help and all three new subcommand help
screens ran successfully without entering a hardware mode. `file` identified
Mach-O arm64; `otool -L` listed only `/usr/lib` and `/System/Library` dependencies.

Original attended executable: `target/release/s4mk3-jog-diagnostic`

Original attended build SHA-256 (before the instrumentation increment):
`1a1a6769e1f9bd11fd27819429c62b33ef7fb6c1c0070bfc08c91001769b3978`

## Physical boundaries

During initial software preparation, no HID device was opened and no motor,
ring, screen, MIDI or audio traffic was triggered. Physical motor sign, starting force, speed
response, selected-side initialization, ring alignment and actual release on
stop remain for the attended session.

The watchdog uses a 50 ms selected-sensor freshness deadline; duplicates cannot
refresh it. The initial interval includes initialization and reader startup.
Deadlines remain best-effort software checks during native or logging I/O,
not hard-real-time guarantees. A neutral attempt cannot prove physical release
when the output path fails.

This is open-loop response measurement, not playback-driven constant-speed
control. It does not assume Djay track-position feedback, an absolute platter
angle, a shared left/right polarity, or 2,880 encoder counts per revolution.

## Attended passive measurement: left clockwise

Direction correction: the initial two left captures retain their original
action labels below, but those labels are superseded by the explicit clockwise
recheck at the end of this record. Current evidence establishes clockwise
positive on both wheels.

On October 27, 2026, the user confirmed completing exactly five clockwise
revolutions and finishing at the starting reference point.

- Capture: `/tmp/s4-platter-measure-left-20261027T172959Z.jsonl`.
- Verified executable hash matches the software-readiness build above.
- Exit 0; 30-second duration completed; 5,124 decoded samples.
- Net displacement: -14,414 raw counts; absolute travel: 14,490 counts.
- Directional totals: -14,452 and +38 counts.
- No `platter_output` records; passive mode sent no output reports.
- Provisional measured scale: 2,882.8 counts/revolution. The original clockwise
  label was not reliable; see the direction recheck below.

This is a manual reference-point measurement, not proof of an exact integer
encoder scale or motor polarity.

## Attended passive measurement: left counterclockwise

The user confirmed exactly five counterclockwise revolutions ending at the
starting reference point on October 27, 2026.

- Capture: `/tmp/s4-platter-measure-left-ccw-20261027T173230Z.jsonl`.
- Exit 0; 30-second duration completed; 4,881 decoded samples.
- Net displacement: +14,376 raw counts; absolute travel: 14,400 counts.
- Directional totals: +14,388 and -12 counts.
- No output records.
- Provisional scale: 2,875.2 counts/revolution.

The clockwise and counterclockwise estimates differ by approximately 0.26%;
their mean is 2,879 counts/revolution. This supports approximately 2,880 as a
working left-wheel scale, not an exact calibration.

## Attended passive measurement: right clockwise

The user confirmed exactly five clockwise revolutions ending at the starting
reference point on October 27, 2026.

- Capture: `/tmp/s4-platter-measure-right-cw-20261027T173420Z.jsonl`.
- Exit 0; 30-second duration completed; 4,897 decoded samples.
- Net displacement: +14,384 raw counts; absolute travel: 14,400 counts.
- Directional totals: +14,392 and -8 counts.
- No output records.
- Provisional scale: 2,876.8 counts/revolution.

Clockwise raw motion is positive on the right, matching the corrected left-wheel
sign. This does not establish right motor-command polarity.

## Attended passive measurement: right counterclockwise

The first attempt, `/tmp/s4-platter-measure-right-ccw-20261027T173554Z.jsonl`,
exited 1 after 1.92 seconds with `selected sensor freshness watchdog`.
It logged 296 samples, zero travel and no output records. The last decoded
sample was approximately 82 ms old at termination. The cause of that gap is
unconfirmed; the watchdog was not changed.

The repeat completed, and the user confirmed exactly five counterclockwise
revolutions ending at the starting reference point.

- Capture: `/tmp/s4-platter-measure-right-ccw-20261027T173648Z.jsonl`.
- Exit 0; 30-second duration completed; 4,908 decoded samples.
- Net displacement: -14,386 raw counts; absolute travel: 14,466 counts.
- Directional totals: -14,426 and +40 counts.
- No output records.
- Provisional scale: 2,877.2 counts/revolution.

The right-wheel direction estimates average 2,877 counts/revolution and differ
by approximately 0.014%. Both wheels support approximately 2,880 as a working
scale, within manual reference-point error. The corrected clockwise raw sign
is positive on both wheels.
No motor polarity, powered response or ring behavior is established by these
passive measurements.

## Attended powered trial: left positive, 500 units

Capture: `/tmp/s4-platter-drive-left-positive-20261027T173918Z.jsonl`.
The 15-second trial used selected-left ring initialization and one MOVE-armed
positive 500-unit pulse bounded to 500 ms. It exited 0 on duration.

The log contains 103 nonzero writes, all requesting `[500, 0]`, spanning
host timestamps 11,812,778 through 12,309,637 us. All output writes completed
without error, including final neutral. Left absolute travel was four counts
(+2/-2), net zero; right travel was zero.

The user reported no visible movement and confirmed they were not touching
the wheel. This does not establish physical motor polarity or useful torque.
Insufficient starting force and incomplete motor activation remain possible;
accepted native writes alone cannot distinguish them.

## Attended powered trial: left positive, 1,000 units

Capture: `/tmp/s4-platter-drive-left-positive-1000-20261027T174117Z.jsonl`.
The same 15-second, selected-left initialized trial used a single positive
1,000-unit pulse bounded to 500 ms. Exit 0 on duration; 113 nonzero writes
requested `[1000, 0]` from host timestamps 8,196,914 through 8,694,566 us.
All output writes, including final neutral, completed without error.

Left travel was 20 counts (+10/-10), net zero; right travel was zero. The user
again reported no visible movement. Useful powered rotation and motor polarity
remain unestablished.

## Attended powered trials: left positive, 2,000 units

`/tmp/s4-platter-drive-left-positive-2000-20261027T174247Z.jsonl` completed
without a MOVE press or nonzero output. It is not a powered-response trial.

The repeat, `/tmp/s4-platter-drive-left-positive-2000-20261027T174340Z.jsonl`,
completed with two MOVE-armed 500 ms pulses at `[2000, 0]`. Exit 0; 217
nonzero writes; no output errors; successful final neutral. Left travel was
130 counts (+104/-26), net +78; right travel was zero.

The user saw slight clockwise movement and explained that the second press
was deliberate to confirm it. This establishes visible motor response at
2,000 units with selected-left initialization, not sustained rotation.

The physical clockwise observation conflicts with the earlier passive
left-clockwise negative raw-count interpretation. During the first pulse the
counter increased approximately 54 counts, settling near +42 relative to its
pre-pulse baseline; the second increased approximately 44, settling near +34.
Do not resolve this discrepancy by silently relabeling raw signs or assuming
motor polarity. Opposite-polarity physical comparison remains pending.

## Direction recheck: left explicitly clockwise

The user raised the possibility that the first diagnostic had been performed
backwards. A fresh passive capture explicitly requested clockwise viewed from
above. The user confirmed these were definitely five clockwise turns ending
at the starting reference point.

- Capture: `/tmp/s4-platter-measure-left-cw-recheck-20261027T174652Z.jsonl`.
- Exit 0; full 30-second duration; 4,847 decoded samples; no output records.
- Net +14,416 counts; travel 14,472; directional totals +14,444/-28.
- Measured scale: 2,883.2 counts/revolution for the five manual turns.

This supersedes the initial left-direction labels: left clockwise raw motion
is positive, consistent with the user-observed clockwise response to positive
2,000-unit motor pulses. The count-scale estimates remain approximately
2,880 counts/revolution. Negative motor polarity remains to be compared.

## Attended powered trial: left negative, 2,000 units

Capture: `/tmp/s4-platter-drive-left-negative-2000-20261027T174917Z.jsonl`.
One MOVE-armed 500 ms pulse requested `[-2000, 0]`, with selected-left ring
initialization. Exit 0 on duration; 104 nonzero writes spanned host timestamps
9,384,838 through 9,878,994 us. All output writes, including final neutral,
completed successfully.

Left net displacement was -100 counts, total travel 128 (+14/-114); right
travel was zero. The user confirmed counterclockwise movement and settling
after the pulse. Left raw motor polarity is now physically established:
positive clockwise, negative counterclockwise. This establishes short-pulse
response and settling, not constant-speed playback or touch interruption.

## Attended left touch cancellation

Capture: `/tmp/s4-platter-drive-left-touch-2000-20261027T175057Z.jsonl`.
Positive 2,000-unit force was armed with a two-second maximum pulse, within
a 15-second session. Exit 0; no output errors; successful final neutral.

Arming occurred at host 11,998,491 us, with a deadline of 13,998,491 us.
Touch was sampled at 12,570,868 us; a successful `sample_gate_neutral` write
followed at 12,570,881 us. The pulse therefore ended after approximately
0.57 seconds, before its normal deadline. Touch release was observed at
13,614,196 us. No nonzero output followed touch or release.

The user reported that initial force was slight, felt no force while touching,
and saw no restart on release. This supports physical no-restart behavior;
the weak initial force limits how distinctly physical cutoff could be felt.
It is not a high-force touch-interruption test.

## Attended powered trial: right positive, 2,000 units

Capture: `/tmp/s4-platter-drive-right-positive-2000-20261027T175313Z.jsonl`.
One MOVE-armed 500 ms pulse requested `[0, 2000]`, with selected-right ring
initialization. Exit 0 on duration; 103 nonzero writes spanned host timestamps
8,340,731 through 8,836,555 us. All output writes, including final neutral,
completed successfully.

Right net displacement was +34 counts, total travel 66 (+50/-16); left travel
was zero. The user confirmed slight clockwise right-wheel movement, settling,
and the left wheel staying still. Positive right motor polarity is physically
confirmed clockwise; negative right polarity remains untested.

## Attended powered trial: right negative, 2,000 units

Capture: `/tmp/s4-platter-drive-right-negative-2000-20261027T175432Z.jsonl`.
One MOVE-armed 500 ms pulse requested `[0, -2000]`, with selected-right ring
initialization. Exit 0 on duration; 99 nonzero writes spanned host timestamps
10,200,697 through 10,688,652 us. All output writes, including final neutral,
completed successfully.

Right net displacement was -88 counts, total travel 120 (+16/-104); left travel
was zero. The user confirmed counterclockwise movement, settling and the left
wheel staying still. Both motor polarities are now physically confirmed on
both sides: positive clockwise, negative counterclockwise.

## Attended right touch cancellation

Capture: `/tmp/s4-platter-drive-right-touch-2000-20261027T175556Z.jsonl`.
Positive 2,000-unit force was armed with a two-second maximum pulse in a
15-second session. Exit 0; no output errors; successful final neutral.

Arming occurred at host 7,636,805 us, with a deadline of 9,636,805 us.
Touch was sampled at 8,043,945 us; a successful `sample_gate_neutral` write
followed at 8,044,043 us. Force was canceled after approximately 0.41 seconds,
before the normal deadline. Touch release occurred at 9,185,291 us.
No nonzero output followed touch or release.

The user felt no force while touching and saw no restart after lifting off.
As with the left, this is low-force physical acceptance, not a high-force test.

## Attended left-ring following

Capture: `/tmp/s4-platter-ring-left-2880-20261027T175812Z.jsonl`.
Used the measured approximate scale of 2,880 counts/revolution, default ring
direction and a requested 60-second session. The user confirmed correct
rim-following direction, hold during top-touch movement, smooth release/resume
and stationary hold: "Yes everything worked perfectly."

The capture ended early after 30.07 seconds with exit 1 on
`selected sensor freshness watchdog`. Its last decoded sample was approximately
61 ms old. This is the second freshness abort in this attended series;
the underlying delay remains unconfirmed and the cutoff was not relaxed.
The physical function confirmation covers the observed behavior before abort,
not uninterrupted 60-second operation.

The log contains 4,864 decoded samples, 1,316 ring ticks, touch hold/release
transitions, zero nonzero-force writes and no output errors. Final neutral
and selected-left ring clear both completed successfully.

## Attended right-ring following

Capture: `/tmp/s4-platter-ring-right-2880-20261027T180006Z.jsonl`.
Used 2,880 counts/revolution and default ring direction. The full 60-second
session completed with exit 0: 9,727 decoded samples, 2,626 ring ticks,
touch hold/release transitions, no nonzero-force writes and no output errors.
Final neutral and selected-right ring clear completed successfully.

The user confirmed correct direction/travel, hold during top-touch movement,
release/resume and stationary hold. Both rings now have physical acceptance
for these relative-following behaviors. This does not establish integration
with powered playback or uninterrupted left-ring operation beyond its
freshness-aborted capture.

## Attended Ctrl-C / SIGINT shutdown checks

The first capture, `/tmp/s4-platter-drive-left-ctrlc-2000-20261027T180405Z.jsonl`,
exited 0 with `ctrl_c` and successful final neutral. However, its two-second
pulse had already ended at host 9,077,632 us; shutdown occurred at 13,276,803 us.
This is a neutral-state Ctrl-C check, not active-force interruption.

For the repeat, a local log watcher was subscribed before launch and sent
SIGINT after observing the first nonzero force record. Capture:
`/tmp/s4-platter-drive-left-ctrlc-local-1793124366317.jsonl`.

- Left positive 2,000 units, two-second pulse maximum, selected-left
  initialization; MOVE armed at host 7,050,115 us.
- First nonzero write at 7,050,183 us.
- A force write at 7,134,133 us failed with
  `IOHIDDeviceSetReport failed: (0xE00002BC) (iokit/common) general error`.
- Final neutral at 7,134,746 us succeeded, 11/11 bytes; exit 1 retained the
  HID error.
- Net travel +8 counts; the user confirmed the wheel stopped.

This establishes physical stopping through the error-cleanup path during
an active pulse, approximately 85 ms after arming. It is not a clean
active-drive Ctrl-C pass. The HID error coincided with the interrupt, but
causation is unconfirmed. No error was suppressed and no cutoff was relaxed.

## Attended session outcome

Approximately 2,880 counts/revolution and clockwise-positive motion are
established on both wheels, with the initial left labels corrected above.
Both motor polarities, short-pulse response, wheel isolation, low-force touch
cancel/no-restart, and both rings' relative following/hold/resume are physically
confirmed within the recorded limits.

Remaining findings are two input-freshness aborts and the signal-time HID write
error. Continuous rotation, speed regulation, powered-playback MIDI separation,
ring/motor playback integration and a clean active-drive Ctrl-C result remain
unverified. No normal-app behavior was changed by this attended session.

## Instrumentation increment

The software-only follow-up adds JSONL session version 2 to distinguish actual
native call timing from scheduling timestamps. Existing attended captures above
remain version 1 evidence; they must not be interpreted as containing the new
measurements.

New output records contain native write begin/end/duration and an independently
latched first stop-callback timestamp. Periodic and final health records include
the reader's current native operation, latest and maximum per-operation
durations/failures, batch/publication timing, sampled queue high-water,
poll lateness, consumer lag and log record/flush timings.

Reader telemetry is opt-in for the three platter modes. Shared snapshot
observation hooks default to no observer, retaining the normal input framing,
report ordering and interrupt behavior. No log I/O runs on the reader thread
or under its telemetry lock.

The deterministic stop-during-write regression blocks a fake native writer on
a channel, requests Stop at a known injected-clock timestamp, then explicitly
releases the call. Both successful and failed in-flight returns are covered.
Assertions check actual call timing, persisted stop time, preservation of the
original transport error, exactly one final-neutral attempt after startup,
and no further non-neutral output. No fixed sleep or real HID device is used.

This increment retains the 50 ms cutoff, force/pulse limits and error exits.
It adds evidence collection, not a claimed remedy for either native issue.
Physical verification of this instrumented build remains pending.

The increment passed the same required source checks listed above: formatting,
strict all-target/all-feature Clippy, the full locked Rust test suite
(including 146 standalone diagnostic tests), locked all-target build,
55 Swift tests, shell syntax, plist lint and diff checks. The release executable
was rebuilt with stripping disabled. All four CLI help screens ran
successfully without opening HID; architecture is arm64 and linked libraries
remain system-only.

Instrumented executable: `target/release/s4mk3-jog-diagnostic`

Instrumented build SHA-256:
`d8366e3148f6835b50994598fe983d323907e31da516049e93778cd82c666c5b`

No attended hardware run was performed for this increment. The original
captures, physical observations and original executable hash above are
preserved separately from this software-only evidence.

## Bounded speed-probe preparation

Current standalone source adds `platter-rotate`, using the existing 11-byte
motor reports and the same reader, selected-wheel arming, neutral guard and
timing/Stop telemetry. It does not change the normal bridge, mapping or app.
There is no new attended acceptance or replay of Traktor packets.

The Traktor motor-driven recording includes the user's deliberate finger
slowdowns and hand speedups. It is disturbance-response evidence, not an
identified native control law or a proven hands-off stability test. The new
controller is an independently chosen conservative experiment.

The first run targets 5 RPM with explicit 2,880-count calibration and positive
motor/counter signs. The existing 2,000-unit force limit, two-second armed
window and 50 ms selected-sensor freshness cutoff remain. A fresh MOVE gesture
is required for every run. Touch cancels and clears the controller; release
alone cannot restart it. The proposed baseline, touch and Stop runs are
listed in execution order in `platter-diagnostic.md`, Stage 4.

The regulator measures a 20 ms count window and ramps its target at 10 RPM/s.
Its proportional/integral gains are 100 and 200 force units per RPM and
RPM-second respectively. Integral headroom is clamped against the force limit;
force increases are limited to approximately 2,000 units/s with integer
quantization. Reductions are immediate. It never requests reverse braking.
Wrong-direction speed below -1 RPM or speed above `2 * target + 5` RPM exits
through neutral/error cleanup. At this force ceiling, reaching the requested
speed is unproven; saturation or insufficient movement is retained evidence.

Deterministic tests use the real decoder/model and native-report/session
builders with injected frames, clocks and writers. They cover warm-up,
target/force ramps, force ceilings 1/500/2,000, integral saturation, measured
overspeed response, independent signs/wheel selection, counter wrap,
touch/rearm/history reset, pulse expiry, duplicate freshness rejection,
direction/overspeed neutralization, control telemetry, latched Stop and
Stop/deadline priority over queued frames in the production event loop.
The existing in-flight success/error regression and transport/cleanup
regressions remain. No fixed sleeps or hardware are used.

This preparation does not establish physical regulation, attainable RPM,
final tuning, native Traktor touch behavior, sustained playback, or separation
of motor motion from Djay jog MIDI. The earlier freshness aborts and
signal-time native error remain unresolved physical findings.

Software verification passed formatting, strict all-target/all-feature Clippy,
the full locked Rust suite (including 155 diagnostic tests), the locked
all-target build, 55 Swift tests, shell syntax, plist lint and diff checks.
Swift retained its pre-existing unhandled-file warnings for
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`; both warnings
were present in the preceding instrumentation verification.

The arm64 release executable was rebuilt with
`CARGO_PROFILE_RELEASE_STRIP=none`. Top-level help and all four platter
subcommand help screens passed without opening hardware. Its linked
libraries remain system-only.

Speed-probe executable: `target/release/s4mk3-jog-diagnostic`

Speed-probe build SHA-256:
`da27a25ce97f2d046566c3422e55672e3685b4663b66132152d3e14cd3d51986`

This hash identifies the software-prepared speed-probe build, not the older
attended or instrumentation-only executables recorded above. No hardware
command was launched for this preparation.

## First target-speed baseline

The prepared speed-probe executable above was used for an initial unarmed
attempt and one explicitly confirmed repeat. No controller tuning, force
limit or powered-window limit was changed.

The first 15-second attempt,
`/tmp/s4-platter-rotate-left-baseline-20261027T215119Z.jsonl`,
exited 0 with reason `duration`, no non-neutral requests, no recorded travel
and successful final neutral. No verified left MOVE press bit appeared.
The user subsequently clarified they were not watching and may have pressed
before the diagnostic started. This is not physical acceptance. Later trials
must wait for the explicit instruction to press MOVE after the diagnostic
reports Ready.

For the repeat, the user confirmed watching and gave a fresh go-ahead.
The overall session was 60 seconds to allow time for arming; the powered
window remained two seconds. LEFT, positive motor and counter direction,
2,880 counts/revolution, 5 RPM target and 2,000-unit ceiling were retained.
Capture:
`/tmp/s4-platter-rotate-left-baseline-repeat-20261027T215437Z.jsonl`.

- MOVE press/release registered. Arming occurred at host 12,989,972 us,
  with the fixed deadline at 14,989,972 us.
- There were 422 non-neutral writes, all left-only, with peak requested
  force 1,900 units. The force cap was never reached.
- First non-neutral native call began at 13,016,862 us. The last completed
  at 14,986,834 us; no non-neutral call began at or after the deadline.
- The first post-deadline neutral began at 14,991,159 us and completed at
  14,991,710 us, respectively 1,187 and 1,738 us after the deadline. This
  illustrates the software deadline's scheduling/native-call limits.
- The driving interval recorded +30 counts and 30 counts of travel,
  approximately 3.75 degrees at the supplied calibration. Maximum windowed
  speed estimate was 2.035 RPM; the 5 RPM target was not reached. Low-speed
  count quantization limits interpretation of individual speed estimates.
- The user confirmed slight clockwise left-wheel movement and a stationary
  right wheel. Right requested force and right counter displacement were
  zero throughout. No left top-touch samples were recorded.
- The full session ended normally at 60,000,455 us with +20 net counts and
  52 counts of travel. These totals include movement after the powered
  interval; its cause is not established by this capture.
- All output writes and logged reader operations succeeded. No freshness,
  direction or overspeed exit occurred. Final neutral succeeded, 11/11 bytes.
  Maximum native write duration was 7,552 us; maximum consumer lag 4,046 us
  and sampled queue high-water 2.

This establishes slight powered clockwise movement under the experimental
regulator and clean deadline/neutral behavior in this run, not useful steady
rotation or target-speed regulation. Because the controller did not reach
the force ceiling, it does not establish that 2,000 units cannot sustain the
target speed. No touch/Stop/right-wheel repeat, tuning change or force
increase was performed after this baseline.

## Reference-based rotation preparation

Following the low-force baseline and Mixxx comparison, the user requested
preparation of a revised standalone experiment. This revision changes only
`platter-rotate`, not the app, bridge, mappings, fixed-force drive or other
diagnostic modes. Preparation does not launch hardware.

Reference: official Mixxx
[`Traktor-Kontrol-S4-MK3.js`](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js),
SHA-256 `c058a168d537ccaf0374701004755af344c62a395d55e7d9f5aa668e9a8597c8`,
particularly `S4Mk3MotorManager`; defaults are also in its
[`Traktor Kontrol S4 MK3.hid.xml`](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor%20Kontrol%20S4%20MK3.hid.xml).
Mixxx's default ceiling is 25,000 raw units, with a 1,650 baseline and
110 units per base RPM before nonlinear correction. At matched 33.33 RPM
its formula calculates 5,316 units. The provided Traktor recording likewise
contains substantially higher commands than the first probe, but includes
deliberate finger slowdowns and hand speedups. Neither source proves our
required starting torque, final gains or hands-off regulation.

The revised first configuration targets integer 33 RPM, with an 8,000-unit
ceiling. The CLI allows 1–45 integer RPM and 1–25,000 raw force units. This
raises the rotation-only ceiling from the old 2,000-unit limit deliberately;
no other mode inherits that change. The proposed first run uses 60 seconds
overall for communication/arming, not 60 seconds of drive. The armed window
remains at most two seconds; the freshness cutoff remains 50 ms.

The old PI loop is replaced by Mixxx-derived feed-forward/cubic correction,
with a 120 RPM/second target ramp and approximately 50,000 raw units/second
force-rise limit. Both use the control-loop clock, not the age of queued
snapshots. Speed still uses fresh selected-wheel counts over at least 20 ms.
At matched 33 RPM the computed feed-forward command is 5,280 units, reflecting
the integer target rather than Mixxx's 33.33 RPM preset.

This is not a full Mixxx TT implementation. Negative requests coast rather
than reverse-brake; observed touch cancels drive instead of emulating
blocked-wheel force or scratching. The existing selected-wheel arming,
opposite-wheel neutral, wrong-direction/overspeed exits, independently
latched Stop, native-error propagation and final-neutral path are retained.
No retry or timeout increase is introduced.

Tuning records identify `mixxx_derived_unidirectional_v1`. Rotation state
reports `feed_forward_force` and signed `unclamped_force` instead of
`integral_force`; earlier version-2 JSONL files remain historical evidence
for the older PI algorithm. The session version alone is not an algorithm
identifier. Raw/native I/O, queue, consumer-lag and log timing remain.

The ordered Stage 4 runbook now requires observing Ready before explicitly
telling the user to press/release LEFT MOVE. It proposes one hands-off
33 RPM/8,000-unit baseline before separately reviewed touch and Stop trials.
This preparation does not establish its physical speed response, stopping
behavior or sustained rotation, and does not resolve the older native
freshness/SIGINT findings.

Final software verification passed formatting, strict locked all-target/
all-feature Clippy, the full locked Rust suite (including 158 diagnostic
tests), the locked all-target build, 55 Swift tests, shell syntax, plist lint
and diff checks. The two existing Swift unhandled-file warnings for
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh` remain unchanged.
LSP found no diagnostics in the diagnostic directory or the final modified
test file. An initial regression fixture assumed sub-count speed precision;
it was corrected to an exactly representable 30 RPM input and an explicit
5,621-unit expected response, rather than loosening its numeric tolerance.

The release diagnostic was rebuilt with `CARGO_PROFILE_RELEASE_STRIP=none`.
Top-level and all four platter help screens passed without opening hardware;
rotation help reports defaults 33 RPM, 8,000 units and 2,000 ms. The executable
is arm64 and links only system libraries.

Executable: `target/release/s4mk3-jog-diagnostic`

Reference-based build SHA-256:
`36dd805b7d093caba8e576eb3c685ab3e9a96ece66d7c262a132f6f911c52986`

This identifies the revised software-prepared build, not the earlier
low-force baseline executable. No motor trial, app packaging or release
publication was performed for this revision.

## First reference-based attended baseline

Used the reference-based executable above, after checking its hash and
controller-process ownership, then obtaining fresh watching confirmation.
The agent observed Ready before explicitly instructing one LEFT MOVE
press/release. Settings: positive motor/counter signs, 2,880 counts/revolution,
33 RPM target, 8,000-unit ceiling, 2,000 ms powered window, 60-second overall
session and selected-ring initialization. No touch, Stop or right-wheel
trial, automatic retry or settings change followed.

Capture:
`/tmp/s4-platter-rotate-left-33rpm-8000-20261027T224511Z.jsonl`.

- Arming registered at host 12,737,094 us; deadline 14,737,094 us.
  There were 413 non-neutral output writes, all left-only, with peak
  requested force 8,000. Eleven of 325 driving samples reported the cap.
- Driving displacement/travel was +2,988 counts, approximately 1.04 turns.
  The final approximately half-second averaged 34.815 RPM from displacement
  over snapshot time. Windowed estimates in that interval ranged from
  29.520 to 39.675 RPM; the last driving estimate was 31.096 RPM.
  These are short-window estimates, not proof of sustained regulation.
- The user reported clockwise but uneven rotation. Right force and counter
  displacement remained zero. This supports useful rotation relative to
  the earlier creep, but does not pass a smooth target-speed criterion.
- The last non-neutral call began at 14,736,870 us and completed at
  14,737,417 us. No non-neutral output began at or after the deadline.
  First post-deadline neutral began at 14,737,419 us and completed at
  14,737,800 us, 325/706 us after the deadline.
- The process subsequently exited **1** with
  `selected sensor freshness watchdog` at 57,425,908 us, approximately
  42.688 seconds after the post-pulse neutral completed. Force had already
  been neutral throughout that idle interval. This was not a normal
  60-second completion and must not be reported as a clean trial.
- The last consumed snapshot was at 57,375,761 us: age 50,147 us at the
  failure. Reader telemetry reported 48,216 us poll lateness, empty queue,
  sampled queue high-water 2 and maximum consumer lag 5,827 us.
  The last completed batch took 473 us (maximum 4,685 us). Completed GET1
  and GET3 maxima were 3,957 and 1,933 us; a new GET1 had begun at
  57,425,503 us and was in flight at the health snapshot.
  These measurements identify late reader polling/publication rather
  than a recorded long completed HID read or a queued-frame backlog.
  They do not establish the underlying scheduling or wake-up cause.
- All recorded output writes succeeded, including final neutral
  11/11 bytes at 57,426,339 us. Reader operation and log error counts were
  zero. Full-session net/travel was +3,182/3,206 counts, including motion
  after the powered interval.

Outcome: the stronger probe approached normal platter speed but was visibly
uneven, and the later idle freshness failure remains unresolved. Keep the
50 ms cutoff and preserve the failure. This capture does not justify an
automatic ceiling increase, a smooth-rotation claim or further hardware runs.

## Sensor-clock feedback and reader-wait preparation

Prepared after the user requested the next revision. No hardware launch or
normal bridge/app change is part of this preparation. The first configuration
remains LEFT, 33 RPM, 8,000 raw units, a two-second armed window, 60 seconds
overall and a 50 ms freshness cutoff. No OS scheduling-policy change,
native retry or timeout relaxation is introduced.

The capture above distinguishes several hypotheses:

- **Arrival-time measurement noise:** driving snapshot intervals were
  469–9,647 us, median 6,376 us. Sensor ticks/host microsecond varied between
  individual intervals, while the complete capture accumulated
  5,737,330,159 sensor ticks over 57,374,112 host microseconds: 99.9985875
  ticks/us. Using a 100 ticks/us sensor calibration reduced the final
  half-second's windowed estimate range from 29.520–39.675 to
  31.245–37.665 RPM. Host-time jitter therefore contributed to the estimated
  variation, but did not explain all of the motion reported by the user.
- **Control response:** the cubic correction has zero small-error slope at
  the target. The new experiment retains the reference-derived feed-forward
  but uses linear proportional feedback and a signed integral to provide
  small-error correction and trim baseline bias. This is experimental tuning,
  not proof of the physical cause or of smoother response.
- **Slow HID versus late polling:** the earlier batch/read measurements and
  empty queue do not support a long completed HID call or consumer backlog
  as the failure cause. Inspection found cancellation-aware, actual-start
  relative polling with no catch-up burst. Timer wait, resumption and OS
  scheduling remain possible; no pacing-logic fix is claimed.

The deterministic arrival-jitter regression first failed against the old
estimator with 32.03883495145631 RPM for constant 33 RPM motion. The revised
rotation-only estimator unwraps the selected sensor timer and measures over
at least 20 ms of sensor time. It uses 100 ticks/us, handles timer rollover
and rejects a sensor-time interval over 50 ms. Host time still controls
freshness, target/force ramps and all deadlines; a sensor timer cannot refresh
an old or duplicate host snapshot. The shared decoder remains frequency-free.

The new tuning identifier is `feedforward_pi_sensor_clock_v2`: feed-forward
is unchanged, proportional gain is 450 raw units/RPM, integral gain
150 raw units/(RPM second), target rise 120 RPM/second and force rise
approximately 50,000 units/second. The integral is bounded to the signed force
cap and frozen when its error would push further into either saturation.
Opposite error may unwind it. Reductions/coasting, touch, rearm, expiry,
wrong-way/overspeed and Stop/error cleanup retain their existing paths.

Reader-wait instrumentation is opt-in through the already instrumented
platter reader. It records requested wait and return timing so another
freshness failure can distinguish wait overshoot from later poll-admission
delay. It changes neither the wait policy nor cancellation behavior.

This preparation does not establish a macOS scheduling cause, guarantee
50 ms availability, or accept the revised physical speed response. The next
attended run must preserve an idle watchdog failure even if the two-second
powered response is improved.

Verification passed: formatting, strict locked all-target/all-feature Clippy,
the full locked Rust suite (164 diagnostic tests), locked all-target build,
55 Swift tests, shell syntax, plist lint and diff checks. The two existing
Swift unhandled-file warnings for `macos/Assets/AppIcon.png` and
`macos/build-jog-diagnostic.sh` remain. Integrated LSP diagnostics were clean.
The arrival-jitter regression now reports constant 33 RPM and unchanged
settled force despite three-phase host arrival jitter. Additional coverage
includes sensor rollover, excessive sensor-time gaps, numeric PI correction,
saturation/no-windup, immediate coasting, existing gate/Stop/error paths,
wait overshoot, post-wait admission delay and cancellation.

The arm64 release diagnostic was rebuilt with
`CARGO_PROFILE_RELEASE_STRIP=none`. Top-level and all four platter help
screens passed without hardware; system-only linkage was confirmed. Defaults
remain 33 RPM, 8,000 raw units and 2,000 ms per arm.

Executable: `target/release/s4mk3-jog-diagnostic`

Sensor-clock/PI/wait-telemetry build SHA-256:
`0b25eabce670e714708c0f695d8204199aa95c158aef45fa9496d171c7cf77e8`

No hardware trial or app package was produced for this revision. Smooth
physical regulation remains unverified; reader-delay instrumentation is not
a fix for the underlying late wake/resumption.

## First sensor-clock PI attended baseline

Used the sensor-clock/PI executable above after verifying its hash and
controller ownership. The user confirmed watching, then pressed/released
LEFT MOVE only after the diagnostic reported Ready and the agent gave the
explicit cue. Settings remained LEFT, positive signs, 2,880 counts/revolution,
33 RPM, 8,000 raw units, 2,000 ms per arm, 60 seconds overall and selected-ring
initialization. No extra arm, touch/Stop/right-wheel trial or limit change
was performed.

Capture:
`/tmp/s4-platter-rotate-left-pi-33rpm-8000-20261027T231504Z.jsonl`.

- Arming occurred at host 13,994,953 us; deadline 15,994,953 us.
  There were 411 non-neutral writes, all left-only, with peak force 8,000.
  Driving displacement/travel was +2,964 counts, approximately 1.03 turns.
- The final approximately half-second averaged 34.005 RPM by displacement
  over sensor time; controller estimates ranged 32.192–35.555 RPM.
  Recalculating both captures using the same 20 ms sensor-window estimator
  gives the preceding cubic run 31.245–37.665 RPM, versus this run's
  32.192–35.555 RPM. This comparison does not mix the old host-clock
  estimates with the new sensor-clock estimates.
- The user reported noticeably smoother clockwise rotation, with the right
  wheel still, but also said startup needed more oomph. Right force and
  counter displacement were zero. This supports a short-run improvement,
  not acceptance of sustained regulation or the startup response.
- First nonzero force began 22,613 us after arming. The first and only
  8,000-unit output began 210,889 us after arming. The ramp requested
  approximately 2,141 units at 56 ms, 4,328 at 104 ms and 6,601 at 145 ms.
  A faster startup ramp within the same ceiling is a candidate follow-up,
  not an authorized or tested change.
- Last non-neutral native call began at 15,991,817 us and completed at
  15,992,272 us. No non-neutral call began at/after the deadline.
  First post-deadline neutral began at 15,999,012 us and completed at
  15,999,267 us, 4,059/4,314 us after the software deadline.
- The process exited 0 with reason `duration` at 60,001,556 us. Final
  neutral succeeded, 11/11 bytes, at 60,002,083 us. All recorded output,
  reader-operation and log error counts were zero; no freshness failure
  occurred in this run. Full-session net/travel was +3,166/3,190 counts,
  including post-pulse movement.
- Wait telemetry recorded maximum elapsed wait 36,569 us, maximum wait
  overshoot 35,000 us, maximum poll lateness 35,004 us and maximum
  post-return-to-poll interval 183 us. GET3 duration reached 26,113 us;
  maximum batch duration was 29,241 us. Sampled queue high-water was 2,
  ending empty, and maximum consumer lag was 4,817 us.

This run did not reproduce the earlier idle abort, but substantial wait
overshoot and native-call duration remain observable. Wait-boundary timing
does not identify kernel timer expiry versus thread resumption. Do not
declare the polling fault fixed or relax the unchanged 50 ms cutoff.

## Startup-only faster-ramp preparation

Prepared after the user accepted a faster startup ramp within the existing
ceiling, following the smoother PI trial and weak-start feedback. No hardware
is launched during this preparation. The diagnostic's force-rise allowance
is now 100,000 raw units/second only during the first 200 ms after a fresh arm,
including the unchanged initial 20 ms measurement warm-up. It returns to the
existing 50,000-unit/second rate afterward. An update straddling 200 ms uses
each rate only for the corresponding part of its elapsed time.

This is not a fixed-force starting kick: PI demand and the selected ceiling
still bound the command, and reductions/coasting remain immediate. Sensor
calibration, feed-forward, PI gains, 120 RPM/second target ramp, 33 RPM default,
8,000 initial/25,000 maximum ceiling, two-second arm limit, 50 ms cutoff,
touch/rearm/isolation and Stop/error behavior are unchanged. Reader wait
instrumentation and its unresolved physical scheduling limits are unchanged.

Tuning records identify `feedforward_pi_sensor_clock_startup_v3`, adding
`startup_seconds` and `startup_force_rise_per_second` alongside the existing
steady rate. The preceding v2 capture remains evidence for the slower-start
revision, not physical verification of this change.

The new deterministic startup-only regression failed before the change:
the old code produced 100 units where the initial 2 ms interval should now
allow 200. After implementation, all 17 targeted rotation tests pass.
Boundary cases prove 200 units during startup, 100 afterward and 350 for a
4 ms interval containing 3 ms of startup plus 1 ms of the steady rate.
The real decoder/model force-cap regression retains its tighter 100-unit
per-2-ms bound after startup. Existing gate, watchdog, Stop and error tests
remain, and a new startup overspeed case verifies immediate coasting.

The next attended command remains the Stage 4 LEFT 33 RPM/8,000-unit baseline,
with one explicit post-Ready MOVE cue and a 60-second overall session. Announce
the quicker initial force buildup before obtaining watching confirmation.
Startup feel and any effect on the subsequent speed response remain untested;
no shorter physical time-to-speed is claimed from the software checks.

Final software verification passed formatting, strict locked all-target/
all-feature Clippy, the full locked Rust suite (166 diagnostic tests), locked
all-target build, 55 Swift tests, shell syntax, plist lint and diff checks.
The two pre-existing Swift unhandled-file warnings remain unchanged.
Integrated LSP diagnostics were clean.

The arm64 standalone executable was rebuilt with
`CARGO_PROFILE_RELEASE_STRIP=none`. Top-level and all four platter help
screens passed without hardware; linked libraries are system-only. CLI
defaults remain 33 RPM, 8,000 raw units and 2,000 ms.

Executable: `target/release/s4mk3-jog-diagnostic`

Startup-ramp build SHA-256:
`327317e72868998a51bbd7047198ebf09362b1a0d0e55c67e56ff23036258f85`

No hardware run, app package or release publication was performed for this
revision. The earlier clean PI session is evidence for the preceding build;
the quicker startup still requires an attended comparison.

## First faster-force-rise attended comparison

Verified the startup-v3 executable hash above and controller ownership,
obtained watching confirmation, then cued one LEFT MOVE gesture after Ready.
Settings remained 33 RPM, 8,000 units, 2,000 ms per arm, 60 seconds overall,
positive signs and 2,880 counts/revolution. No retry or further adjustment.
Capture:
`/tmp/s4-platter-rotate-left-startup-v3-33rpm-8000-20261027T235913Z.jsonl`.

The user reported **startup still too weak**. Driving displacement was
2,988 counts (approximately 1.04 turns). The final approximately half-second
averaged 33.899 sensor-timed RPM, with windowed estimates 32.192–35.411 RPM.
The user did not separately accept steady smoothness in this comparison.

The first nonzero write began 24,729 us after arming. Peak native force was
7,621; peak controller force was 7,635, and early unclamped demand peaked at
7,773. The ceiling was not reached. At 84 ms, commanded/measured speed was
10.08/4.17 RPM and requested force 5,474; at 152 ms, it was 18.24/13.89 RPM
and force 5,733. The target first reached 33 RPM at 278,041 us after arming;
the first windowed speed at or above 90% of target was at 294,516 us.
This supports investigating the unchanged 120 RPM/second target ramp rather
than raising the force ceiling or only increasing the force-rise allowance.
It does not establish an accepted physical startup response.

Arming/deadline were 14,157,070/16,157,070 us. No non-neutral call began at
or after the deadline. Post-deadline neutral began at 16,160,533 us and
completed at 16,161,169 us, 3,463/4,099 us after the software deadline.
The process exited 0 with `duration` at 60,000,156 us; final neutral
succeeded, 11/11 bytes, at 60,000,511 us. Right force/counter displacement
and all recorded output, reader and log error counts were zero.

Wait overshoot reached 30,079 us, poll lateness 30,082 us and post-wait
admission delay 214 us; maximum batch duration was 14,142 us. The queue
high-water was 2 and maximum consumer lag 10,243 us. No freshness fault
occurred, but the existing scheduling uncertainty remains.

## Faster target and rim-interaction preparation

The approved change doubles only the target ramp from 120 to 240 RPM/second:
the requested speed reaches 33 RPM at 137.5 ms instead of 275 ms.
Force ceilings, startup/steady force-rise rates, sensor calibration, PI gains,
two-second arm window, 50 ms cutoff and all gate/error behavior remain.
The new tuning identifier is `feedforward_pi_fast_target_v4`.
No hardware is launched during preparation.

The target-ramp regression failed against the prior code, then checks 24 RPM
at 100 ms, 33 RPM at 137.5 ms and the same 33 RPM afterward. A real
decoder/model regression applies a 24 RPM rim-brake or 45 RPM forward-nudge
trace during a 33 RPM run, then returns to 33 RPM without another arming
gesture. Brake demand reaches the existing ceiling; nudge demand coasts;
drive returns to a below-cap value after release, with the opposite wheel
neutral throughout. This is software response evidence, not a simulated
motor plant or physical recovery proof.

Stage 4 now contains exact separate commands/actions/result checks for
hands-off baseline, top-touch cancellation, light rim brake, forward rim
nudge and early Stop, with fresh watching confirmation and post-Ready MOVE
cue for each. Rim trials use one brief contact early in the existing
two-second window; expiry during contact makes recovery inconclusive.
Top touch still cancels force without automatic restart. Rim contact that
also asserts touch is a cancellation trial, not successful recovery.
No scratching, reverse braking, Djay MIDI/tempo behavior or new touch mode
has been introduced. Physical startup, resistance and recovery remain untested.

All required checks passed: formatting, strict locked all-target/all-feature
Clippy, full locked Rust tests (168 diagnostic tests), locked all-target
build, 55 Swift tests, shell syntax, plist lint and diff checks. The two
existing Swift unhandled-file warnings remain. Integrated LSP diagnostics
were clean.

The release diagnostic was rebuilt with `CARGO_PROFILE_RELEASE_STRIP=none`.
Top-level and all four platter help screens passed without hardware.
Architecture is arm64 and linked libraries are system-only.

Executable: `target/release/s4mk3-jog-diagnostic`

Fast-target build SHA-256:
`2fa6c3fbc5b3d3ffa6fe6d6e1101275097e9f09af6a2b69105a952165b96350a`

No hardware run or app packaging was performed for this revision. Startup
feel, rim recovery and the underlying scheduling cause remain unverified.

## Fast-target v4 attended baseline: startup still too weak

Verified the current build hash above and no competing bridge/Traktor
process. NIHardwareAgent and Djay remained running. The first 60-second
session never armed: the user confirmed missing the MOVE window. It sent
no nonzero force, recorded zero displacement and exited 0 with successful
final neutral. It is not startup evidence.
Log: `/tmp/s4-platter-rotate-left-fast-target-v4-33rpm-8000-20261028T013326Z.jsonl`.

After fresh watching confirmation, the unchanged LEFT 33 RPM/8,000-unit/
2,000 ms baseline was retried. Ready was observed before the explicit MOVE
cue; one gesture armed it and no additional trial was launched.
Log: `/tmp/s4-platter-rotate-left-fast-target-v4-retry-33rpm-8000-20261028T013558Z.jsonl`.
The filename was staged before launch; actual Ready was at 01:36:22 UTC.

The user again reported **startup still too weak**. The first force began
22,434 us after arming; native output reached 8,000 at 98,332 us and stayed
at that ceiling for approximately 118,761 us in total. Early unclamped
demand peaked at 13,768.7. The target reached 33 RPM at 139,891 us;
windowed measured speed first reached 90% of target at 228,409 us,
versus 294,516 us in the prior v3 trial. This single comparison demonstrates
earlier response, not accepted startup feel or a general timing guarantee.
Driving displacement was 3,016 counts; late sensor-timed average was
33.800 RPM, windowed range 32.014–35.411. Steady smoothness was not
separately accepted in this response.

Arming/deadline were 10,055,284/12,055,284 us. No nonzero call began at
or after deadline; neutral began/completed at 12,056,807/12,057,104 us
(1,523/1,820 us late). Exit 0 recorded `duration` at 60,000,394 us;
final neutral succeeded, 11/11 bytes, completed at 60,000,681 us.
Right force and counter displacement were zero. All output, reader and
log error counts were zero; no freshness fault occurred. Wait overshoot
peaked at 36,689 us, poll lateness 36,690 us, post-wait delay 5,672 us,
batch duration 6,064 us and consumer lag 4,311 us. Queue high-water was 2;
maximum log flush took 13,404 us. The scheduling cause remains unresolved.

The initial force ceiling now demonstrably binds; another ramp increase
alone cannot increase force during that interval. No ceiling or source
change follows automatically from this result. Rim braking, forward nudge,
top-touch cancellation and early Stop remain separate attended checks;
no physical recovery claim is made by this baseline.

## Startup-step v5 preparation

The user approved removing both startup ramps rather than raising the force
ceiling. The target now steps to its full value at the first valid speed
update. During the first 200 ms after a fresh arm, capped PI demand is applied
without force-rise slew. At rest the first eligible update can request
8,000 units; the existing 20 ms speed warm-up remains neutral beforehand.
After startup, increases retain the 50,000-unit/second limit. An update
crossing the boundary uses only its post-boundary time for that allowance,
so it cannot apply a late unlimited step. Reductions remain immediate.

The default/absolute ceilings, PI/feed-forward, sensor clock and guards,
two-second window, 50 ms cutoff, touch/rearm, opposite-wheel isolation and
Stop/error cleanup are unchanged. The tuning record now identifies
`feedforward_pi_startup_step_v5`, with `target_step` and `startup_force_step`
true instead of obsolete target/startup force-rise rates.

The updated startup regression failed against v4 before implementation.
The real decoder/model cap test now checks neutral during warm-up and an
immediate capped first update at 1, 8,000 and 25,000-unit settings. Existing
startup overspeed, steady slew, touch/rearm, Stop and error checks remain;
the brake/nudge disturbance traces remain software-only evidence.

The attended commands retain their previous limits. The prelaunch explanation
now explicitly announces sharper startup engagement before watching
confirmation. No hardware is run during preparation; startup feel and
physical rim recovery are not established by these changes.

Final verification passed formatting, strict locked all-target/all-feature
Clippy, the full locked Rust suite (168 diagnostic tests), locked all-target
build, 55 Swift tests, shell syntax, plist lint and diff checks.
The two pre-existing Swift unhandled-file warnings remain; integrated LSP
diagnostics were clean. The standalone arm64 release executable was rebuilt
with `CARGO_PROFILE_RELEASE_STRIP=none`; top-level and all four platter
help screens passed without hardware, with system-only linked libraries.

Executable: `target/release/s4mk3-jog-diagnostic`

Startup-step build SHA-256:
`15a5a8a50841e00fe3f3302f9380ff00018439c4e618afd6d947400e351b0924`

This build has not been run on hardware. No app bundle, publication or
scheduling fix is part of this preparation.

## Startup-step v5 attended baseline accepted

Verified the v5 hash above and no competing bridge/Traktor. NIHardwareAgent
and Djay remained running. After setup and the sharper-start explanation,
the user confirmed watching. One LEFT MOVE gesture was explicitly cued
after Ready, arming one hands-off 33 RPM/8,000-unit/two-second window.
No extra trial was launched.
Log: `/tmp/s4-platter-rotate-left-startup-step-v5-33rpm-8000-20261028T014931Z.jsonl`.
The filename was staged earlier; actual Ready was 01:50:40 UTC.

The user accepted startup as **sufficient, still smooth**. Full target
appeared at 19,981 us after arming; the first force write was already
`[8000, 0]` at 21,725 us. Windowed measured speed first reached 90% of
target at 175,824 us, versus 228,409 us in v4. Driving displacement was
3,072 counts; the late approximately half-second sensor average was
33.613 RPM, windowed range 32.046–35.411. These are single-capture results,
not timing guarantees or proof of interaction recovery.

Arming/deadline were 13,030,753/15,030,753 us. No nonzero call began at
or after deadline. Neutral began/completed at 15,030,893/15,031,167 us,
140/414 us after the software deadline. Exit 0 recorded `duration` at
60,001,832 us; final neutral succeeded, 11/11 bytes, at 60,002,216 us.
Right force/counter displacement and all output, reader and log error
counts were zero. No freshness fault occurred.

Wait overshoot peaked at 28,117 us, poll lateness 28,118 us, post-wait
delay 42 us, batch duration 21,787 us and consumer lag 7,467 us.
Queue high-water was 2. The scheduling cause remains unresolved.
Top-touch cancellation, rim braking, forward nudging and early Stop
remain separate attended checks; startup acceptance does not cover them.

## Touch-trial attempt: rim contacted, direction guard stopped output

The user later clarified that they touched the outer rim, having misread the
instruction, rather than the flat top plate. The capture contains no asserted
left or right touch samples, so this is **not** evidence of top-touch
cancellation. The user's physical impression was inconclusive.

Log: `/tmp/s4-platter-rotate-left-touch-cancel-v5-33rpm-8000-20261028T015433Z.jsonl`.
The single arm began at 11,895,728 us with a deadline of 13,895,728 us.
The first `[8000, 0]` output began at 11,919,229 us. During the later rim
contact, the reported left speed fell to 0 RPM while force remained 8,000;
counts plateaued at 1,856 and then briefly fell to 1,854. The measured
counter-direction guard tripped at 13,448,885 us, before the pulse deadline.
Because the log has no touch input, it cannot establish the precise physical
contact interval or distinguish an intentional brake from another rim
interaction.

The error path sent `decode_error_neutral` successfully (11/11 bytes,
13,448,338–13,448,820 us), then final neutral also succeeded (11/11,
13,448,886–13,449,171 us). No output, reader, or log write errors were
recorded. This was neither the planned light quarter-second brake-and-release
trial nor a valid touch test; no recovery claim follows. Repeat top-touch
cancellation separately with explicit instruction to touch only the flat
top surface, away from the outer rim. Do not combine it with braking.

## Optional 15-second repeatable rim-feel window

After the early rim contact left no clear physical impression, the user asked
for a considerably longer powered window with room to repeat contacts. The user
chose 15 seconds rather than the proposed 30 seconds. Only `platter-rotate`'s
optional per-arm `--drive-ms` maximum increases to 15,000 ms; its default
remains 2,000 ms, and fixed-force `platter-drive` remains capped at 2,000 ms.
The overall capture stays bounded separately by `--seconds` (maximum 120).

This lets the user repeat light outer-rim contacts of one kind during a single
braking or nudge capture. Brake and nudge remain separate logs. All speed,
force, sensor freshness, top-touch cancellation, wrong-direction, overspeed,
and Stop behavior stays the same. There is no stall detector: if rim pressure
holds speed near zero without asserting top touch or reversing the counter,
PI output can remain at the 8,000-unit ceiling until the 15-second pulse ends.
Any safety-guard exit is an interrupted result, not successful recovery.
Top-plate touch still neutralizes and requires a fresh MOVE gesture; it cannot
be used for repeated contacts in a single powered pulse.

The boundary regression accepts 15,000 ms and rejects 15,001 ms, confirms the
default remains 2,000 ms, simulates expiry neutral and a fresh MOVE re-arm.
The physical brake and nudge trials are recorded below; braking ended early
on the freshness watchdog, while the separate nudge window completed.

## 15-second rim-brake attempt interrupted by freshness watchdog

The rebuilt arm64 v5 executable (SHA-256
`e805bf4ebc5a226dedffa29e339131435bbba4fb030015b9a11fc1d3184321af`) was
verified before launch. NIHardwareAgent and Djay were running; no bridge or
Traktor process was found. The user confirmed watching. The run used the
unchanged 33 RPM/8,000-unit settings, a 15,000 ms arm window and a 60-second
overall capture.

Log: `/tmp/s4-platter-rotate-left-rim-brake-v5-15sec-33rpm-8000-20261028T124326Z.jsonl`.
The drive armed at 20,492,517 us with a 35,492,517 us deadline. It exited at
22,015,288 us with `selected sensor freshness watchdog`, about 1.52 seconds
after arming and 13.48 seconds before the pulse deadline. The last selected
sensor sample was 54,945 us before exit, exceeding the unchanged 50 ms cutoff.
The capture had no top-touch samples; this neither proves the user did not
touch the rim nor verifies top-touch cancellation.

The user reported that the platter stopped as soon as they applied light
braking force to the rim. The last motion sample showed 2,072 relative counts,
0 RPM and `[8000, 0]` requested force. The last powered report completed
before the watchdog error.
Final neutral succeeded with 11/11 bytes at 22,015,288–22,017,478 us; no
powered report followed it. Output, reader-operation and log error counts
were zero. Maximum reader wait overshoot was 15,872 us and poll lateness
15,873 us. The exact cause of the 54,945 us selected-sensor gap is unknown.
Because the pulse ended early, this is not a completed 15-second feel trial;
no sustained recovery conclusion follows. No touch samples are expected from
this rim contact and do not timestamp its exact onset.

## Repeated rim-nudge capture completed

The same verified v5 build was used after a fresh watching confirmation.
One LEFT MOVE gesture armed the 33 RPM/8,000-unit setting for 15 seconds.
Log: `/tmp/s4-platter-rotate-left-rim-nudge-v5-15sec-33rpm-8000-20261028T125609Z.jsonl`.

The arm began at 21,187,278 us with a deadline at 36,187,278 us. The gate
returned to `NeedRelease` at 36,189,970 us and `Ready` at 36,194,299 us.
The overall session exited 0 with `duration` at 60,000,183 us. It recorded
2,472 driving samples; maximum windowed speed was 35.979 RPM. There were no
touch samples, output/reader/log errors, freshness fault, or right-wheel
force. Final neutral completed 11/11 bytes at 60,000,707 us.

Reader wait overshoot peaked at 40,090 us and poll lateness at 40,090 us;
maximum consumer lag was 10,585 us. The log does not mark individual rim
nudges, so physical recovery timing and feel come from the user's report,
not inferred sample timing.

The user reported that it was **way too easy to brake the wheel compared
with Traktor** and **took too long to pick back up**. This is the physical
result of the nudge-feel trial; it does not give a specific nudge timestamp
in telemetry. The longer powered window provided time to assess the
interaction but did not make its braking resistance or recovery feel
acceptable.

## Corrected flat-top touch cancellation completed

After the earlier rim misread, the physical action was clarified as a brief
touch to the flat platter face well inside the outer rim, then lift. The v5
arm64 build was unchanged. The user confirmed watching; the trial used the
33 RPM target, 8,000-unit cap, two-second arm window and 60-second overall
capture.

Log: `/tmp/s4-platter-rotate-left-touch-cancel-v5-33rpm-8000-20261028T130230Z.jsonl`.
The single arm began at 26,831,935 us with a deadline of 28,831,935 us.
The first selected-wheel touch sample was at 28,171,057 us; its gate was
`NeedRelease` and requested force `[0, 0]`. `sample_gate_neutral` began
28,171,084 us and completed 28,171,552 us, with 11/11 bytes written and no
error. The top-touch state continued through 28,356,430 us. After lifting,
the gate transitioned to `Ready` at 28,363,927 us; no new MOVE gesture or
nonzero output followed.

The 60-second session exited 0 with `duration` at 60,001,206 us. Final
neutral succeeded, 11/11 bytes, through 60,001,637 us. Right-wheel force and
all output/reader errors were zero; no freshness fault occurred. The log
establishes software-detected top touch, neutral output and no automatic
restart. The user reported that drive stopped on touch and stayed off after
lifting.

## Self-service `platter-rotate` tuning parameters (v6)

The user requested to run the standalone diagnostic and adjust controller
parameters directly; bridge-app controls and saved preferences remain out of
scope. New CLI flags accept per-run feed-forward base/slope, proportional and
integral gains, post-startup force-rise limit, startup duration and optional
startup force-rise limit. Defaults preserve the current feel-test behavior:
1,650 base, 110 per RPM, Kp 450, Ki 150, 50,000 force units/second after
startup, 200 ms startup duration and zero startup slew (an immediate capped
step after the 20 ms speed warm-up). `--max-force` remains the hard ceiling,
and all values are copied into both the session settings and
`platter_rotation_tuning` record (`feedforward_pi_tunable_v6`).

The sensor-timed 20 ms speed window, 100 ticks/microsecond calibration,
50 ms freshness guard, direction/overspeed checks, touch cancellation,
one-selected-wheel output, STOP and neutral cleanup remain fixed. This
provides a repeatable command-line tuning loop, not live parameter changes:
edit values for the next attended invocation and use a fresh `--out` path.
The sample command and flag ranges are in `docs/platter-diagnostic.md`.

Parameter tests cover CLI defaults and custom overrides, upper-bound
rejection, custom PI/feed-forward math, startup-step/slew behavior, and the
effective JSONL tuning record. The focused rotation suite passed 23 tests.
The full required checks passed: formatting, strict Clippy, locked Rust tests,
all-target build, Swift tests, shell syntax, plist lint, diff check, and the
release diagnostic build. The pre-existing two Swift unhandled-file warnings
remain. The standalone release executable is arm64; top-level and all
platter-mode help screens pass without opening HID. No app packaging or
hardware run was part of this tuning-interface change.

Executable: `target/release/s4mk3-jog-diagnostic`

Self-service tuning build SHA-256:
`be8266f0bb2d4f86cd5634d2d855d33102ce1686828dc0cf6fd1e34ac93b377d`

## Per-run overspeed limit (v7)

The user chose a configurable guard rather than removing overspeed protection
by default. If `--overspeed-limit-rpm` is omitted, the current
`2 * target_rpm + 5` threshold remains (71 RPM at a 33 RPM target). A positive
value sets an explicit limit up to 250 RPM; zero disables only this high-speed
abort. Reverse-counter, sensor-freshness, force-cap, pulse, touch, Stop and
neutral behavior remain active.

The v7 tuning record logs the effective `overspeed_limit_rpm` (`null` when
disabled). On an overspeed exit, `platter_end.reason` now includes both the
rejected measured speed and the limit, making them available to a jq summary.
For example:

```sh
/usr/bin/jq -s 'map(select(.kind == "platter_rotation_tuning"))[0].overspeed_limit_rpm' "$rotate_tune"
/usr/bin/jq -sr 'map(select(.kind == "platter_end"))[-1].reason' "$rotate_tune"
```

The two reviewed v6 overspeed logs had a 71 RPM limit. Reconstructing the raw
rejected sensor windows gives 72.905 RPM (70 counts over 20.0032 ms); the last
accepted sample in each was 70.822 RPM. This is slightly above the threshold,
not a trigger near the 33 RPM setpoint.

### Overspeed v7 verification

The focused regression confirms a 33 RPM target defaults to 71 RPM, explicit
limits replace that threshold, and zero disables only the high-speed abort.
The parser accepts 0–250 and rejects 251. Overspeed errors include both the
rejected measured RPM and effective limit in `platter_end.reason`.

Formatting, strict all-target/all-feature Clippy, the full locked Rust suite
(175 diagnostic tests), all-target build, 55 Swift tests, shell syntax, plist
lint and diff checks passed. The two pre-existing Swift unhandled-file
warnings remain. The standalone executable was rebuilt with
`CARGO_PROFILE_RELEASE_STRIP=none`; top-level and all platter-mode help checks
passed without HID access. The arm64 artifact hash is recorded below. No app
packaging or physical hardware run was performed for this change.

Overspeed-configurable executable SHA-256:
`fcec609f8bac87263dc43a08eaef64c356ec6fd17caebe28a08776b386beeac6`
