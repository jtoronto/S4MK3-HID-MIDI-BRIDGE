# TT speed-to-Scratch diagnostic

This is an unpublished diagnostic, not a finished Traktor emulation.
Native UI verification is intentionally excluded at the user's request.
Software replay does not establish audible continuity or physical torque.
The current corrected build keeps TT physical-ring tracking, restores normal
playback/Scratch/pause ring behavior, and retains both speed handoffs.

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
- The ceiling is `200% - floor`: with the 50% setting, 49.5 RPM enters the
  upper Scratch handoff and 47.5 RPM exits it. Scratch replaces CC4 at either
  bound. The inferred symmetry also needs actual mapping/audio calibration.
- Normal rings advance with playback, follow Scratch movement and freeze where
  paused. Scratch release resumes from that position. TT rings stay physical.
  Firmware loop flashing is retained and restores the mode's phase on exit.
- TT retains forward assistance during top touch, braking, holding and reverse
  movement. Assistance uses `min(max_force, base_force + force_per_rpm *
  target_rpm)`: 5280 force units with the supplied settings. This is a command,
  not measured torque. It neither sums normal tension/backspin assistance nor
  accumulates speed-controller integral against the user's hand.
- Untouched startup retains speed regulation. After qualified rotation, braking
  below target minus 2 RPM selects assistance; recovery within 0.75 RPM of target
  restores regulation. This motor-policy threshold is not the Scratch floor.
- Upper Scratch MIDI ownership alone no longer selects constant forward force.
  Once physical touch is released above target, the motor continues regulating
  toward 33 RPM instead of sustaining the higher speed. Holding and lower-speed
  braking still retain bounded assistance.
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
The new integer preference `tt_pitch_bend_floor_percent` accepts 1 through 95.
Changing it requires Save and restart, not a new mapping.

## Prepare the test

1. Finish the current session and choose **Stop bridge** in the existing app.
   Quit that app before opening the separate diagnostic app. Do not run two
   bridges against the controller.
2. Open
   `dist/tt-normal-ring-recovery-diagnostic-20261030/S4 MK3 Bridge.app`.
   Existing accepted quiet-CC4 and stronger-braking packages are preserved.
3. In Jog > Testing, check Normal Vinyl startup is on, TT Vinyl startup is off,
   target is 33 RPM and TT pitch-bend floor is 50%. Leave the other motor values
   at the supplied settings for the first comparison. Save before starting.
4. Use the matching generated mapping under a new filename and select it in
   Djay; preserve your previous mapping. The handoff reuses existing CC4, CC5
   and Scratch Note 7 rather than introducing a new MIDI address.
5. Select Full profile and Debug logging. Start the bridge with TT off. Load
   an Internal deck and play the original steady 1 kHz reference track used
   for the recordings, not a recording that already contains a pitch sweep.
6. Start motor diagnostic capture and prepare an audio recording of Djay's
   output. Clear the other platter. If an agent will trigger anything, finish
   setup and explicitly confirm watching before that trigger. This document
   does not authorize an unattended motor start.

## First attended comparison

1. Select TT yourself. Confirm JOG changes from red to blue and nominal
   rotation produces neither Scratch mode nor repeated CC4 corrections.
2. Apply a light top touch while allowing normal rotation. With Vinyl off,
   touch alone must not enter Scratch or cut drive.
3. Slow the wheel progressively. Confirm CC4 braking first, then
   `MOTOR_SCRATCH_HANDOFF active=true` at the configured measured-speed floor.
   Listen for the transition rather than treating the log as an audio pass.
4. Hold the wheel stopped, then make a small controlled backward movement.
   Expect held Scratch, signed CC5 movement and continuing forward resistance.
   No artificial Scratch movement should be sent while stationary.
5. Allow forward acceleration. Confirm one Scratch release above the exit
   threshold, return to CC4 handling and quiet nominal rotation.
6. Select JOG red while TT remains selected and repeat a brief top-touch
   Scratch. It should enter Scratch immediately while retaining assistance.
7. Switch TT off. Confirm normal mode restores its own Vinyl choice. Re-enter
   TT and confirm its separately selected choice is remembered.
8. Repeat on the other side, then verify pause, TT-off and Stop bridge each
   remove drive. Turn TT off or Stop bridge before quitting Djay; feedback
   silence is not a disconnect signal in this diagnostic.
9. In normal mode, move and scratch each wheel while paused and playing.
   Confirm normal playback advances the spot, Scratch follows the wheel, pause
   freezes it and release resumes without a jump. In TT, confirm physical
   movement/holding still controls it. Check loop flash and exit.
10. In an attended TT/Vinyl-off test, apply a controlled forward nudge toward
    the ceiling. Confirm upper Scratch entry, raw CC5 movement, and one release
    as speed falls through its hysteresis exit. Judge the audible transition
    separately; symmetry is not a measured playback-rate guarantee.
11. Release the forward nudge without touching the top and leave playback
    running. Confirm the platter returns to the configured target rather than
    sustaining the higher RPM. Repeat twice, retaining the diagnostic capture.
    Then recheck that a stopped, held platter still provides forward resistance.

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

Logs include `MOTOR_SCRATCH_HANDOFF`, `MOTOR_JOG_BEND`,
`MOTOR_JOG_POSITION` and playback-gate changes. New physical acceptance,
force feel and floor calibration remain to be established by these checks.
