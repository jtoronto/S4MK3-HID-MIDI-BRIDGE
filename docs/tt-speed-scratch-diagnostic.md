# TT speed-to-Scratch diagnostic

This is an unpublished diagnostic, not a finished Traktor emulation.
Software replay does not establish audible continuity or physical torque.
This document describes the current-source response-stability behavior. Its
[validation record](tt-response-stability-validation.md) separates planned
morning checks from performed evidence. Earlier validation records remain
historical, including their upper-speed handoff behavior.

## What this build changes

- Normal mode starts with Vinyl on; TT starts with Vinyl off. JOG changes the
  current mode's choice, and switching modes restores their separate choices.
  Both startup defaults are saved independently in Jog settings.
- With TT authorized and Vinyl off, steady powered rotation produces no jog
  MIDI. Slowing below the configured floor holds Scratch Note 7 and sends raw
  relative movement through CC5 instead of sending CC4 pitch bend. A stopped
  wheel keeps Scratch held without invented movement. Reverse movement stays
  signed. Acceleration through the exit threshold releases Scratch.
- The initial floor is 50% of target speed: 16.5 RPM at 33 RPM. Exit is 18.5 RPM
  at those settings. Entry uses the existing 100 ms speed window after normal
  speed qualification; it is not an instantaneous touch detector.
- Only the lower floor triggers automatic Scratch. Forward nudges remain on
  CC4 regardless of upper speed; explicit Vinyl-on top touch still selects
  Scratch. Exit is `floor + min((target - floor) * 0.25, 2)` RPM.
- Bend has a true +/-2.25 RPM deadband with no active hysteresis. Neutral
  clears fractional residual. Across the next 1 RPM, smoothstep
  `x*x*(3 - 2*x)`, where `x = clamp((abs(error) - 2.25) / 1, 0, 1)`,
  multiplies the existing full-error law. The full response is restored at
  3.25 RPM deviation. Braking remains 2x and forward nudge 1x, multiplied by
  saved gain, range 0.1-10 and default 1.
- Normal rings advance with playback, follow Scratch movement and freeze where
  paused. Scratch release resumes from that position. TT rings stay physical.
  Firmware loop flashing is retained and restores the mode's phase on exit.
- TT retains forward assistance during top touch, braking, holding and reverse
  movement. Full assistance `F0 = min(max_force, base_force + force_per_rpm *
  target_rpm)` is 5280 force units with the supplied settings. Multiply it by
  `clamp((target_rpm - measured_rpm) / min(2, target_rpm), 0, 1)`: full stopped/reverse
  assistance, a linear taper over the last 2 RPM below target, and zero at or
  above measured target, including the touch-release fallback. This is a
  requested command, not measured torque. It neither sums normal tension/backspin
  assistance nor accumulates speed-controller integral against the user's hand.
- Untouched startup retains speed regulation. After qualified rotation, braking
  below target minus 2 RPM selects assistance; recovery within 0.75 RPM of target
  restores regulation. This motor-policy threshold is not the Scratch floor.
- Top contact near target must not cause acceleration or a release-time kick.
  At/above measured target, the hold-assist and touch-release fallback requests
  are zero. Holding and lower-speed braking still retain bounded assistance.
- Playback permission, Shift/GRID, deck-owner mismatch, Stop and stale input
  still remove drive. Fresh recovery needs a new baseline and a full 20 ms
  sensor window. Physical touch does not bypass that window.
- With TT Vinyl on, top touch still enters conventional Scratch immediately,
  but it does not inherently cut TT assistance. Ordinary TT-off behavior remains
  separate.

## Evidence for the provisional floor

`examples/1-khz-pitch bend held.wav` contains a 1000 Hz tone descending to
500 Hz and holding there. Positive-going zero crossings in 250 ms windows
show an observed 0.5 frequency ratio. The knob recording reaches approximately
1500 Hz, an upward ratio of 1.5, not a downward floor.

Neither recording establishes saturation of the selected relative CC4 mapping,
nor a wheel RPM that guarantees a smooth Scratch handoff. The default 50% is
therefore an adjustable diagnostic starting point, not a confirmed Djay limit.
The integer preference `tt_pitch_bend_floor_percent` accepts 1 through 95.
Its default remains 50%; existing saved floor and gain values are preserved.
Changing either requires Save and restart, not a new mapping. Don't reset
preferences for this test: start at gain 1x with the current floor, then compare
floor settings separately.

## Prepare the test

Follow the [ordered morning test](tt-response-stability-validation.md#ordered-morning-test)
from the beginning. It names the separate build, preserves saved preferences,
sets the 1x baseline, verifies the actual launch receipt, and records the
physical and audio comparison. The response update reuses existing CC4, CC5
and Scratch Note 7; no mapping export or reinstall is needed if the current
Internal-feedback mapping is already selected.

## First attended comparison

The morning sequence checks hands-free RPM and zero bend, rim braking and
forward nudging, light top contact/release without acceleration, lower-floor
stop/reverse Scratch, pause/recovery and Stop. It also checks explicit Vinyl-on
Scratch and both physical sides. There is no upper entry to provoke.

Normal rings continue to advance with playback, follow Scratch and hold on
pause; TT rings remain tied to physical movement. Their earlier evidence is
not evidence that this response update passed a new physical test.

## Calibrate against the actual CC4 path

1. For a separate recorded comparison, select TT with JOG red (Vinyl on) and
   manipulate only the rim, avoiding top touch. This disables the automatic
   speed-floor Scratch handoff so it cannot mask CC4's response.
2. Record gradual braking and a sustained slow/held interval, then recovery.
   Retain the full motor diagnostic log, audio, selected mapping and saved Jog
   configuration together. Confirm the log contains CC4 corrections and no
   Scratch ownership during the interval being measured.
3. Measure the reference frequency, lowest sustained output frequency, and
   whether stronger/longer braking changes it further. Correlate the audio with
   logged wheel RPM. A frequency ratio alone does not establish the matching
   wheel-speed threshold.
4. Return JOG blue and compare the automatic handoff. Adjust only the floor
   preference between trials, saving/restarting each time. Keep mapping
   sensitivity, reaction and motor forces unchanged while assessing the
   audible transition.
5. Retain both sides of the handoff and release in each recording. Report a
   jump, discontinuity, delayed stop or wrong direction explicitly; passing
   MIDI/replay checks does not rule them out.

The diagnostic contract includes the actual loaded `S4_MOTOR_CONFIG` receipt
per launch, bounded `MOTOR_TT_STATE` touch/authorization/owner/requested-force
reports, existing `S4_PLATTER_STATUS` RPM/CC4 status, `MOTOR_SCRATCH_HANDOFF`,
`MOTOR_JOG_BEND`, `MOTOR_JOG_POSITION` and playback-gate changes.
`MOTOR_SCRATCH_MODE` has no actual runtime emitter. Requested force is not
applied torque. Physical acceptance, force feel and audio-floor calibration
remain to be established.
