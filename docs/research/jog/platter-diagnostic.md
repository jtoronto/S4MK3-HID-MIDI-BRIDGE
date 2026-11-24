# Attended platter and ring diagnostic

These standalone developer tools were used to prepare playback-driven platters
and remain useful for sensor calibration, isolated motor tuning, ring checks
and input-timing investigations without Djay or app-state dependencies.
They do not control Djay, emit MIDI,
claim audio interfaces, change saved app settings, or send screen traffic.
The `platter-rotate` mode is a bounded speed-control probe, not a Djay
playback implementation or a reconstruction of Traktor's controller.

Software preparation does not establish physical behavior. Motor direction,
starting force, speed response and ring alignment must be checked in an
attended session. Do not replay captured Traktor motor reports.

## Before the attended session

The agent can build and check the executable without the controller. Do not
start a capture or trial while the user is away.

1. Finish any DJ session. Stop the bridge, then quit its menu-bar app.
2. Quit Traktor, Mixxx and any earlier standalone controller diagnostics.
   Leave the Native Instruments screen driver alone: this experiment does
   not use the screens.
3. Connect one powered S4 MK3. Keep both platters clear.
4. Inspect competing controller processes:

   ```sh
   ps -axo pid=,comm= | grep -E 's4mk3-(hid-midi-bridge|jog-diagnostic|screen-diagnostic)|[T]raktor|[Mm]ixxx'
   ```

   Expected: no bridge, diagnostic, Traktor or Mixxx process. The diagnostic
   also checks ownership before opening HID. Do not kill an unidentified
   process to make this check pass.
5. Agree which physical wheel will be tested. Left/right refer to wheels,
   not logical decks A-D.
6. The agent finishes command setup, explains which wheel can move and how
   to stop it, says it is ready, and waits for the user to confirm they are
   watching. Only then start that trial, without additional setup.

Ctrl-C is the normal early stop. If an output or cleanup write fails, retain
the complete error and do not treat an attempted neutral report as proof that
the motor stopped. Stop testing if motion differs from the announced trial.
The 50 ms sensor watchdog and pulse deadlines are software checks, not
hard-real-time guarantees during a blocked native call or log write. The first
50 ms includes initialization and reader startup; an early watchdog error is
a failed setup to inspect, not a reason to bypass the check.

## Measurement order

1. Passive manual revolutions: establish signed counts per physical revolution
   independently for each wheel. Use a visible reference point and return to
   the same point after a counted number of complete turns. Do not infer this
   scale from the ring's 2,880-position encoding.
2. Bounded single-wheel drive: measure the response to requested force, with
   explicit arming and touch interruption. Establish direction and useful
   drive before considering a speed regulator.
3. Calibrated ring following: use the measured counts per revolution to compare
   the ring against physical rotation, including wraparound and hold/resume.
4. Bounded speed control: use the established calibration and motor polarity
   for a short, hands-off target-speed probe, then separate touch and Stop checks.

Keep the logs from each stage. Requested force and successful writes do not
prove torque, RPM or visible ring behavior.

## Build and select the executable

Build on the development Mac, without starting a hardware mode:

```sh
cd /Users/joey/Development/S4MK3-HID-MIDI-BRIDGE
CARGO_PROFILE_RELEASE_STRIP=none cargo build --locked --release --bin s4mk3-jog-diagnostic
./target/release/s4mk3-jog-diagnostic --help
```

The executable is `target/release/s4mk3-jog-diagnostic`. The commands below run
from this repository root. A copied executable does not need Rust on the test
Mac; change into the folder containing it and substitute
`./s4mk3-jog-diagnostic` for `./target/release/s4mk3-jog-diagnostic`.
The build disables stripping to avoid malformed proc-macro dylibs on this
macOS toolchain; it does not change the diagnostic's runtime behavior.

## Timing and stop telemetry

Current source emits JSONL session version 2 automatically for all four
platter modes. Measure, fixed-force drive and ring modes retain their commands
and motor limits; the revised rotation controller is identified by its tuning
record, not by the session version alone.

- `platter_output` retains the scheduling timestamp `host_us` and adds
  `io_started_us`, `io_completed_us` and `io_elapsed_us` around the actual
  native write. The elapsed field is not inferred from adjacent records.
- `platter_health` is written approximately every 100 ms and during cleanup.
  The cleanup snapshot follows the final neutral attempt. Reader snapshots
  identify an in-flight GET/read and
  its start, per-operation latest/maximum durations and failures, batch and
  publication timing, poll lateness, and sampled queue depth/high-water.
  Consumer lag is measured separately when a queued frame is received.
- The instrumented reader snapshot adds `in_flight_wait` and `waits`, with
  requested duration, scheduled deadline, start/return timestamps, elapsed
  duration, overshoot, cancellation status and bounded latest/max counters.
  `latest_post_wait_us`/`maximum_post_wait_us` measure from the last wait
  return to poll admission; `latest_poll_admitted_us` identifies that poll.
  Compare these with poll lateness, native-call timing and publication time.
  Wait-boundary timing includes observation overhead and cannot distinguish
  kernel timer expiry from thread resumption without an external scheduler
  trace. No such trace or OS-policy change is part of this preparation.
  The in-flight `requested` duration uses `{secs, nanos}`; the completed
  `waits` summary reports integer microseconds.
- `log_timing` records latest/maximum complete JSONL record-write and explicit
  flush durations, counts and errors. Record timing includes serialization
  and any implicit buffered flush. Health records are flushed periodically.
  The final end record reports preceding flushes, not the duration of flushing
  its own timing report.
- `stop_requested_at_us` latches the first Ctrl-C callback independently of the
  control loop. It is present in output, health and end records, even when an
  in-flight write fails before the loop can receive Stop. This timestamp marks
  the callback running, not the exact kernel signal-delivery time.

All timestamps use the session's monotonic clock. Reader snapshots are
observations of a concurrently running producer, not exact queue occupancy
histories. Instrumentation itself has overhead. A missing end record after a
logging failure is not proof that cleanup did not run.

The 50 ms force-freshness cutoff and pulse deadlines are retained. Only
`platter-rotate` has the revised force range described in Stage 4.
Native errors remain failures even when Stop is also pending; this version
does not retry writes, suppress errors or claim to fix the two freshness
aborts or the signal-time HID error. Keep the complete log and terminal error
when investigating either issue.

## Stage 1: passive manual revolutions

Do this first, with no motor or ring writes.

1. Choose the LEFT wheel and a visible platter reference point. Keep it still.
2. Once the user is watching, run:

   ```sh
   measure_left="platter-measure-left-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-measure \
     --side left --seconds 30 --out "$measure_left"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$measure_left"
   grep '"kind":"platter_end"' "$measure_left"
   ```

3. Turn the left platter clockwise through exactly FIVE complete revolutions,
   starting and ending at the same reference point. Avoid reverse corrections;
   pause at the final reference point until the command finishes.
4. Expected: no powered motion or LED changes. The log must show valid motion
   and a normal deadline stop, not a freshness or sensor-time error.
5. Retain the log. The `platter_end` record's `relative_counts` is signed
   displacement from the first accepted snapshot; `travel_counts` includes
   travel in either direction. With exactly five turns and no extra movement,
   divide the absolute `relative_counts` by five for counts per revolution.
   Record the sign separately as the clockwise direction. The agent reviews
   these values before proceeding; do not substitute 2,880.
6. Repeat the same command and five clockwise revolutions for the RIGHT wheel:

   ```sh
   measure_right="platter-measure-right-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-measure \
     --side right --seconds 30 --out "$measure_right"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$measure_right"
   grep '"kind":"platter_end"' "$measure_right"
   ```

7. Repeat each measurement counterclockwise in a separate capture, using the
   corresponding command again. Expected: the opposite sign and comparable
   absolute counts per revolution. Resolve disagreement before calibrated
   RPM or ring tests.

## Stage 2: bounded single-wheel drive

This deliberately applies a fixed requested force, not a requested RPM.
Motor polarity is raw positive/negative, not an assumed clockwise direction.
These commands also send the existing initialization report to the selected
ring before arming; its lights may change. Earlier cold-start motor experiments
needed initialization, but selected-side initialization in this new mode still
needs attended confirmation.
The first comparison uses 500 raw force units for at most 500 ms per arm.
The command exits after 15 seconds even if never armed.

1. Clear both platters. Explain that only the LEFT wheel may receive force.
   Confirm the user is watching before launching:

   ```sh
   drive_left="platter-drive-left-positive-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-drive \
     --side left --polarity positive --force 500 --drive-ms 500 \
     --seconds 15 --enable-motor --initialize-ring --out "$drive_left"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$drive_left"
   ```

2. With the selected MOVE button initially released, press then release
   **left MOVE** while the left platter is untouched. This arms one pulse.
   MOVE is the left of the two small MOVE/LOOP knobs on that deck.
3. Expected: force is requested only for the left wheel, for no longer than
   500 ms. The right wheel must remain unforced. Low force may not overcome
   starting friction; no visible motion is a measurement, not permission to
   raise force automatically.
4. After reviewing that result, repeat a fresh MOVE gesture and touch the left
   platter during the pulse. Expected: touch cancels force; lifting your hand
   does not restart it. Another complete MOVE gesture is required.
5. Stop early with Ctrl-C during a further pulse. Check actual motor release
   as well as the logged final-neutral write and exit status.
6. Review the log before a negative-polarity trial. Announce the change and
   obtain watching confirmation before launching:

   ```sh
   drive_left_negative="platter-drive-left-negative-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-drive \
     --side left --polarity negative --force 500 --drive-ms 500 \
     --seconds 15 --enable-motor --initialize-ring --out "$drive_left_negative"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$drive_left_negative"
   ```

   Use left MOVE to arm. Check direction, touch cancellation and no automatic
   restart exactly as above.
7. Clear the wheels, announce RIGHT-only positive force, and obtain watching
   confirmation. Run:

   ```sh
   drive_right="platter-drive-right-positive-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-drive \
     --side right --polarity positive --force 500 --drive-ms 500 \
     --seconds 15 --enable-motor --initialize-ring --out "$drive_right"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$drive_right"
   ```

   Use right MOVE to arm, check left-wheel isolation, then touch interruption,
   release without restart, and Ctrl-C. Review the log.
8. Announce RIGHT-only negative force and obtain watching confirmation. Run:

   ```sh
   drive_right_negative="platter-drive-right-negative-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-drive \
     --side right --polarity negative --force 500 --drive-ms 500 \
     --seconds 15 --enable-motor --initialize-ring --out "$drive_right_negative"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$drive_right_negative"
   ```

   Use right MOVE to arm and repeat the checks. Do not assume physical motor
   polarity matches the left wheel.

The CLI caps force at 2,000 and pulse length at 2,000 ms. These are limits,
not targets. Choose any later increase together after inspecting the initial
logs. There is no automatic ramp, speed regulator or automatic restart.

## Stage 3: calibrated physical ring following

Run after the Stage 1 logs establish a scale for the selected wheel.
This mode changes the selected ring but requests no motor drive.

1. Enter the measured LEFT counts per revolution when prompted:

   ```sh
   printf 'Measured LEFT counts per revolution: '
   read -r left_counts
   ```

2. Put the left platter at its visible reference point and keep it still.
   Tell the user the ring will initialize and show a position relative to this
   starting point. Confirm they are watching, then run:

   ```sh
   ring_left="platter-ring-left-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-ring \
     --side left --counts-per-revolution "$left_counts" \
     --seconds 30 --out "$ring_left"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$ring_left"
   ```

3. Turn the platter by its rim, leaving top touch inactive. Compare quarter,
   half and full revolutions in both directions. Expected: the ring follows
   relative physical travel and wraps smoothly, without powered movement.
4. Touch the top plate and move it. Expected: the displayed position holds.
   Release the top plate and turn the rim. Expected: following resumes from
   the held display position; movement during the hold is not replayed.
5. Stop turning without touching. Expected: the ring holds, rather than
   continuing a timed chase or resetting to 12 o'clock.
6. If display direction is opposite the intended direction, retain that log
   and repeat with `--reverse-ring` and a new filename.
7. Prepare the independently measured RIGHT scale:

   ```sh
   printf 'Measured RIGHT counts per revolution: '
   read -r right_counts
   ```

8. Align the right reference point, announce right-ring initialization and
   obtain watching confirmation. Run:

   ```sh
   ring_right="platter-ring-right-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-ring \
     --side right --counts-per-revolution "$right_counts" \
     --seconds 30 --out "$ring_right"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$ring_right"
   ```

   Repeat quarter/half/full revolutions, top-touch hold, release/resume and
   stationary hold on the right wheel.

This establishes relative ring motion only. It does not align the ring with
Djay track position, reproduce Traktor playback behavior, or establish how the
eventual playback animation should hand over to physical manipulation.

## Stage 4: bounded target-speed probe

This stage uses `platter-rotate`, separate from the app and Djay. Earlier
attended checks established approximately 2,880 raw counts/revolution and
clockwise-positive counts and motor polarity on both wheels. These commands
use that calibration for the LEFT wheel only. They do not repeat the earlier
Traktor capture or replay any of its commands.

The revised first target is **33 RPM with an 8,000-unit ceiling**, replacing
the low-force 5 RPM probe that produced slight clockwise movement. Each MOVE
gesture defaults to two seconds and can be extended to 15 seconds with
`--drive-ms`. Motor units are raw command magnitudes, not calibrated torque.
The standalone CLI exposes the feed-forward, PI, startup and force-rise values
so you can tune them between trials without changing the bridge app. The
default startup step was accepted in an attended check; current rim resistance
and recovery still do not match the Traktor feel report. The reader-delay cause
remains unresolved.

The reference is Mixxx's S4 MK3 motor manager: a 1,650-unit baseline plus
110 units per base RPM, with cubic speed-error correction and a default
25,000-unit maximum. The Traktor recording also showed ordinary commands
around 5,000 units and much larger starting commands, but included deliberate
hand speed changes. Neither reference establishes our final tuning.
The revised CLI accepts integer targets 1–45 RPM and ceilings 1–25,000;
defaults are 33 RPM and 8,000 units. The ceiling can be adjusted per run;
changing the default remains a separate decision. Neither saturation nor
insufficient movement raises it automatically.

### Adjusting feel parameters

The standalone command accepts tuning values per run; no bridge-app setting or
rebuild is needed between trials. The defaults below preserve the current
`feedforward_pi_tunable_v7` behavior. Each JSONL log records both the requested
settings and the effective rotation-tuning record. Change one or two values
per trial and use a new `--out` filename so results remain comparable.

| Flag | Default | Allowed range | Meaning |
| --- | ---: | ---: | --- |
| `--target-rpm` | 33 | 1–45 | Target speed |
| `--overspeed-limit-rpm` | `2 × target + 5` | 0–250 | Abort above this speed; zero disables this guard |
| `--max-force` | 8000 | 1–25000 | Hard output ceiling |
| `--base-force` | 1650 | 0–25000 | Feed-forward force at zero target RPM |
| `--force-per-rpm` | 110 | 0–25000 | Feed-forward force per target RPM |
| `--kp` | 450 | 0–25000 | Proportional force per RPM error |
| `--ki` | 150 | 0–25000 | Integral force per RPM-second error |
| `--force-rise-per-second` | 50000 | 0–1000000 | Maximum force increase after startup |
| `--startup-ms` | 200 | 0–2000 | Startup behavior interval after arming |
| `--startup-force-rise-per-second` | 0 | 0–1000000 | Startup force-rise limit; zero means step |
| `--drive-ms` | 2000 | 1–15000 | Powered time for each MOVE arm |

`--max-force` remains the hard ceiling regardless of gains or feed-forward.
When omitted, the overspeed abort limit is twice the selected target plus
5 RPM (71 RPM at the default target). Set a positive override to change it,
or `--overspeed-limit-rpm 0` to disable only the overspeed abort for that run.
The counter-reversal, sensor-freshness, force-cap, pulse-expiry and Stop
guards remain active. Without the overspeed abort, the PI controller continues
with the selected gains; those gains may coast or may still request positive
force above target.
Set `--startup-ms 0` to disable the special startup interval. A zero startup
force-rise value allows a step to capped demand during that interval; a
positive value gives startup its own slew limit. After the interval,
`--force-rise-per-second` controls increases. The target remains a step; speed
window, sensor-clock scale, 50 ms freshness watchdog and counter-reversal
guard remain fixed measurement/safety settings. The overspeed cutoff is the
separate per-run setting above.

Start from this copyable baseline and change the numeric values on later runs:

```sh
rotate_tune="platter-rotate-tune-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
./target/release/s4mk3-jog-diagnostic platter-rotate \
  --side left --polarity positive --count-direction positive \
  --counts-per-revolution 2880 --target-rpm 33 --max-force 8000 \
  --base-force 1650 --force-per-rpm 110 --kp 450 --ki 150 \
  --force-rise-per-second 50000 --startup-ms 200 \
  --startup-force-rise-per-second 0 --drive-ms 15000 \
  --seconds 60 --enable-motor --initialize-ring --out "$rotate_tune"
```

Keep each physical comparison attended and change only settings you can
identify in the saved session record. The guide below defines separate
hands-off, top-touch, rim-brake and rim-nudge checks.

### What the controller does

- Wait for a new selected-MOVE release/press/release gesture while top touch
  is inactive. Gather a fresh 20 ms speed window before requesting force.
- Measure speed from integrated raw counts over at least 20 ms of unwrapped
  sensor time, using counts/revolution and counter direction. The rotation-only
  calibration is 100 sensor ticks/microsecond; the full attended capture
  measured 99.9986. Host arrival jitter no longer changes the speed denominator.
  Low-speed count quantization remains. The shared decoder is unchanged.
- Request the full target at the first valid speed update, without a target
  ramp. Feed-forward is `floor(force_per_rpm * target_rpm) + base_force`;
  defaults produce 5,280 units at a matched 33 RPM before integral correction.
  `--kp` and `--ki` set proportional and integral correction in the units
  shown above.
  The signed integral is bounded by the selected cap and does not accumulate
  further into upper or lower saturation; opposite error can unwind it.
  These are experimental gains, not physically accepted tuning.
- By default, during the first 200 ms after each fresh arm, apply capped PI
  demand without a force-rise ramp. At rest this can request the full
  8,000-unit ceiling immediately after the unchanged 20 ms speed warm-up.
  Configure that interval with `--startup-ms` and its optional slew with
  `--startup-force-rise-per-second`. After startup, `--force-rise-per-second`
  limits force increases. An update crossing the startup boundary receives
  only the appropriate portion of each rate, not a delayed startup step.
  The selected ceiling and feedback may still require less force.
  Output is quantized to integer units. Reduce force immediately when required.
  Output stays
  in the selected polarity: overshoot reduces drive or coasts, without
  commanding reverse braking.
- End with an error if the windowed speed is below -1 RPM in the expected
  direction, or above the selected overspeed limit. The default is
  `2 * target_rpm + 5` RPM (71 RPM for this probe); zero disables only this
  high-speed abort. An overspeed failure records measured RPM and limit in
  the end reason.
  Touch cancels force immediately on observation; lifting does not restart it.
  Touch, a new arm and pulse expiry discard the old control history, integral
  and force.
- Keep the 50 ms selected-sensor freshness cutoff and existing transport,
  Stop and cleanup paths. A latched Stop also prevents a new output tick
  after sample logging. An already-blocked native call cannot be interrupted
  by this controller.
- Host time still enforces freshness and all output deadlines. Sensor-clock
  velocity cannot make an old snapshot fresh. Reader wait telemetry records
  requested waits and their returns to investigate polling lateness; the
  cancellation-aware 2 ms pacing is not changed. The earlier 48 ms delay
  is not claimed fixed, and the watchdog is not relaxed.

The right motor remains neutral. `--initialize-ring` initializes only the left
wheel; this mode does not animate its ring. Neutral removes requested force;
it does not actively brake any remaining physical coast.
This is a bounded adaptation, not a copy of Mixxx's full TT behavior: it does
not implement reverse braking, held-wheel force or scratching. Its existing
2 ms output scheduling is unchanged; Mixxx's requested 1 ms callback is not
reproduced or assumed to establish wire cadence.

### First run: hands-off baseline

1. Follow **Before the attended session** above: stop/quit the bridge app,
   close other controller programs, connect the powered S4 and check ownership.
   Prepare this command before announcing readiness. Both platters must be clear.
2. Explain that only the left wheel may move, after left MOVE is pressed and
   released. Announce that the command can step to the full 8,000-unit ceiling
   after speed warm-up, without startup ramps, so engagement may be sharper.
   Say that the two-second window ends drive, not necessarily all
   physical coasting, and Ctrl-C ends the diagnostic.
3. Say the command is ready and wait for confirmation the user is watching.
   Only then launch:

   ```sh
   rotate_baseline="platter-rotate-left-baseline-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-rotate \
     --side left --polarity positive --count-direction positive \
     --counts-per-revolution 2880 --target-rpm 33 --max-force 8000 \
     --drive-ms 2000 --seconds 60 --enable-motor --initialize-ring \
     --out "$rotate_baseline"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$rotate_baseline"
   grep -E '"kind":"platter_(rotation_tuning|transition|end)"' "$rotate_baseline"
   ```

4. While it runs, the agent waits for the `PLATTER_STATE` Ready state with left
   MOVE released, then explicitly says **"Press and release LEFT MOVE now."**
   Do not press at launch or merely on saying you are ready to watch.
   Push the left of that deck's two small MOVE/LOOP knobs until it clicks,
   then release. Keep both hands off the wheels for this baseline.
   Do not add another MOVE gesture; let the 60-second session finish.
5. Expected software behavior: no force before arming/warm-up, then only
   positive left force within the cap. The powered window expires two seconds
   after arming and cannot be extended by motion. The right wheel must remain
   unforced. A normal overall finish records `duration` and successful final
   neutral. Record actual direction, motion, coast and any unexpected behavior.
6. Retain the full log and exit status. Review any freshness, direction,
   overspeed, I/O or cleanup failure before continuing. A successful process
   exit alone does not establish target-speed performance.

### Second run: touch cancellation

1. Review the baseline first. Prepare the following command, announce the same
   left-wheel limits. Explain the exact action: after the MOVE cue and visible
   platter motion, briefly touch the flat upper surface well inside the outer
   rim, then lift. Do not touch the rim in this cancellation check. Wait for
   watching confirmation before launch:

   ```sh
   rotate_touch="platter-rotate-left-touch-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-rotate \
     --side left --polarity positive --count-direction positive \
     --counts-per-revolution 2880 --target-rpm 33 --max-force 8000 \
     --drive-ms 2000 --seconds 60 --enable-motor --initialize-ring \
     --out "$rotate_touch"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$rotate_touch"
   grep -E '"kind":"platter_(transition|end)"' "$rotate_touch"
   ```

2. While it runs, wait for Ready and the agent's explicit press instruction,
   press/release left MOVE, then briefly touch the flat face of the left
   platter, away from its outer rim. Lift your hand without pressing MOVE.
3. Expected: force is cancelled on observed touch and remains neutral after
   release. Do not re-arm in this run. Record whether this happens physically,
   retain the log and inspect it before the next run. This intentionally differs
   from Traktor's recorded touch behavior.

### Third run: light rim brake and release

This tests motor speed recovery only, not Djay pitch bend or vinyl scratching.
The diagnostic emits no MIDI. Top-plate touch still cancels drive; rim contact
that also reports top touch is a cancellation result, not a recovery trial.

1. Review the revised hands-off baseline and touch cancellation first.
   Prepare this separate capture. Announce that the left motor may resist
   repeated light rim brakes during a 15-second per-arm window, under the same
   8,000-unit ceiling. A stalled wheel may remain under capped force until it
   recovers, the timer expires, or a guard stops the run. Confirm watching and
   clear platters before launch:

   ```sh
   rotate_brake="platter-rotate-left-rim-brake-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-rotate \
     --side left --polarity positive --count-direction positive \
     --counts-per-revolution 2880 --target-rpm 33 --max-force 8000 \
     --drive-ms 15000 --seconds 60 --enable-motor --initialize-ring \
     --out "$rotate_brake"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$rotate_brake"
   grep -E '"kind":"platter_(transition|end)"' "$rotate_brake"
   ```

2. Wait for Ready and the explicit LEFT MOVE cue. Press/release once. When
   the wheel is moving, lightly brush only its outer rim, release, and repeat
   as needed during the 15-second powered window. Do not hold it stopped,
   reverse it or touch the top plate. Top-plate touch cancels drive. Agree the
   action before launch; do not wait for another mid-pulse cue.
3. Record resistance, any jerking, whether clockwise speed recovers after each
   release, and whether the right wheel stays still. Do not re-arm during this
   same-type capture. Let the 60-second session finish; retain the complete log
   and exit status.
4. Expected software response: with touch false and the gate still Driving,
   slowing raises requested force up to the existing ceiling; release allows
   feedback to settle again. The controller can stay at its capped command
   while the rim is held and the wheel is stalled; reverse-direction,
   overspeed, touch and freshness guards remain active. If the 15-second pulse
   expires before recovery, label it inconclusive rather than re-arming. If top
   touch or a guard stops the run, record that separately.

### Fourth run: forward rim nudge and release

1. Review the brake capture before proceeding. Prepare this command, explain
   one short clockwise rim nudge with the same limits, and obtain fresh watching
   confirmation before launch:

   ```sh
   rotate_nudge="platter-rotate-left-rim-nudge-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-rotate \
     --side left --polarity positive --count-direction positive \
     --counts-per-revolution 2880 --target-rpm 33 --max-force 8000 \
     --drive-ms 15000 --seconds 60 --enable-motor --initialize-ring \
     --out "$rotate_nudge"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$rotate_nudge"
   grep -E '"kind":"platter_(transition|end)"' "$rotate_nudge"
   ```

2. Wait for Ready and the explicit LEFT MOVE cue, then press/release once.
   Once moving, give the outer rim one small forward clockwise nudge and
   release immediately. Repeat small nudges as needed within the 15-second
   powered window. Do not flick rapidly or touch the top plate. Agree this
   action before launch; braking remains a separate capture.
3. Record whether speed returns toward target without a jerk after each nudge,
   whether drive resumes after any coast, and whether the right wheel stays
   still. Do not re-arm within the capture. Retain the full capture and exit
   status after session end.
4. Expected software response: faster measured rotation reduces positive
   force, potentially to zero; it never commands reverse braking. While still
   Driving, feedback may resume positive drive as speed falls. The same touch,
   wrong-direction, overspeed, freshness and expiry rules remain. A guard
   exit or touch cancellation is not evidence of successful nudge recovery.

### Fifth run: early Stop

1. After reviewing the preceding runs, prepare the command below. Announce the same
   limits and wait for watching confirmation before launch:

   ```sh
   rotate_stop="platter-rotate-left-stop-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
   ./target/release/s4mk3-jog-diagnostic platter-rotate \
     --side left --polarity positive --count-direction positive \
     --counts-per-revolution 2880 --target-rpm 33 --max-force 8000 \
     --drive-ms 2000 --seconds 60 --enable-motor --initialize-ring \
     --out "$rotate_stop"
   printf 'Exit code: %s\nLog: %s\n' "$?" "$rotate_stop"
   grep -E '"kind":"platter_(transition|end)"' "$rotate_stop"
   ```

2. Wait for Ready and the explicit press instruction, press/release left MOVE
   and press Ctrl-C during the powered
   window. Record physical release as well as the final-neutral result.
3. Expected: Stop ends the session, requests neutral, and records the first
   callback timestamp. Preserve any native error even if neutral succeeds.
   An I/O error coinciding with Stop remains a failed trial to investigate.

### Reading the speed-probe evidence

The session records its settings and `platter_rotation_tuning`, with algorithm
`feedforward_pi_tunable_v7`, its clock calibration and effective overspeed
limit (null when disabled), `target_step: true`, the
effective feed-forward and PI values, `startup_ms`, both force-rise rates, and
whether startup uses an immediate step. The preceding
`feedforward_pi_startup_step_v5`
used fixed gains and a 200 ms startup step. The preceding
`feedforward_pi_fast_target_v4`
used a 240 RPM/second target ramp and 100,000-unit/second startup force rise.
The earlier
`feedforward_pi_sensor_clock_startup_v3` used a 120 RPM/second target ramp;
it is historical evidence rather than the current startup behavior. Earlier
`feedforward_pi_sensor_clock_v2` logs used the same feedback with a single
50,000-unit/second force-rise limit. `mixxx_derived_unidirectional_v1` logs
used host-time cubic feedback;
logs without an algorithm field used the low-force PI controller. Both remain
historical evidence, not results for this revision. Each
`platter_motion.sample.rotation` includes the step `commanded_rpm`,
windowed sensor-timed `measured_rpm`, `error_rpm`, `feed_forward_force`,
signed `integral_force`,
`unclamped_force`, unsigned `force`
and `at_force_cap`. The ordinary sample RPM remains an instantaneous estimate;
do not confuse it with the controller's windowed speed. Before warm-up and
after cancellation, a missing measured speed is not a fabricated zero reading.
Existing raw snapshots, transitions, actual write timings, health, stop
timestamps and end records remain available.

Compare the settings in the tuning record with measured speed, time at the
force cap and physical movement. Keep hands-off, top-touch, brake and nudge
captures separate. A false top-touch bit does not prove hands-off operation;
rim contact or hand-speed changes must be noted explicitly. Brake and nudge
capture logs do not timestamp manual contact, so pair them with your
observation of the feel.
This does not yet establish sustained playback, native Traktor
behavior, suitable final gains, or motor/jog separation in Djay.

## Results to retain

For each command, keep the full JSONL log, exit status, complete terminal
error if any, physical direction, whether the other wheel stayed unforced,
and the observed touch/stop/ring behavior. Review final-neutral output
results separately from the reason the run ended. A timeout, sensor reversal,
short write or log error is a failed trial to investigate, not a completed
physical acceptance check.
