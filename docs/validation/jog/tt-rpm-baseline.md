# Hands-free TT RPM baseline

This procedure compares the assistance-admission correction with the earlier
hands-free baseline. It changes when automatic underspeed assistance starts,
not motor tuning, the bend curve, gain or quiet band. Keep the current saved
settings and selected Djay mapping. Do not reset preferences or export a new
mapping for this test.

## Run the baseline

1. In the previous bridge app, choose **Stop bridge**, then **Quit S4 MK3
   Bridge**. Ensure no separate CLI bridge is running. Closing Settings is not
   quitting the menu-bar app.
2. Open the prepared diagnostic app:

   ```sh
   open "/Users/joey/Development/S4MK3-HID-MIDI-BRIDGE/dist/tt-handoff-20261102/S4 MK3 Bridge.app"
   ```

   Do not open the older Applications copy. The version number is unchanged;
   the separate bundle path identifies this diagnostic.
3. In Settings, select **Full controller + LED feedback** and **Debug** log
   detail. Keep Jog motor force, target RPM, gain, floor and Djay mapping
   sensitivity unchanged. Save preferences while stopped, then start the bridge.
   Keep TT off during preparation.
4. In Djay, load the original steady **1 kHz reference track** on the right
   physical deck (normally B), using Internal playback. Turn **Key Lock off**.
   Pause that deck while preparing. Leave the other deck paused with TT off.
   Keep the existing audio routing and selected mapping.
5. In **Diagnostics > Live platter diagnostics**, choose **Start motor
   diagnostic capture**. Choose **Show file** and note the log filename. Check
   the header includes the launch's `S4_MOTOR_CONFIG` receipt and names
   `MOTOR_REGULATOR` among the captured events.
6. Start Djay's audio recording. Start playback on the prepared deck, then
   select its TT mode yourself. TT selection can start the platter. Confirm
   JOG is blue (Vinyl off) and that live Diagnostics eventually shows **Bend**.
7. Once Bend is established, leave the platter completely untouched for
   **30 seconds**. Do not touch its top or rim, nudge, brake, switch decks or
   change settings. Leave the other platter alone too. Note any audible speed
   fluctuation, but let the capture continue.
8. Pause playback, turn TT off, then choose **Stop bridge**. Confirm both motors
   stop. Stop the audio recording and choose **Stop motor diagnostic capture**.
9. Retain the diagnostic log and WAV together. Report the deck/side and whether
   the full 30-second interval was untouched. A later left-side comparison should
   be a separate recording, not mixed into this first baseline.

If an agent is to trigger hardware instead, setup must be complete before the
agent states readiness and waits for confirmation that you are watching.

## What the capture establishes

`MOTOR_REGULATOR` contains JSON after `data=`. It samples each side every 20 ms
and additionally records regulation-mode changes, using the input's `host_us`.
It includes raw counter/timer endpoints, report delta/status, contact and
authorization, exact motor-measurement window counts/ticks, motor RPM, mode and
requested force. Modes are `disabled`, `qualifying`, `regulating` and
`assisting`. Qualifying may use the existing tapered fallback once a separate
motor-speed window is available; it is not necessarily zero force.
`assistance_rpm` is the separate at-least-100 ms encoder estimate used to confirm
new automatic underspeed assistance. It is unavailable before that complete
window. Top touch and reverse retain short-window assistance; existing assistance
still exits using current short-window speed. Neither MIDI sensitivity nor the
bend deadband is widened.

During regulation, the nested `regulator` contains baseline, proportional and
integral terms, the unclamped sum, and the final bounded/rise-limited request.
It is null outside regulation so an old PI state cannot be mistaken for the
active force source. These are force requests, not measured torque or confirmed
USB writes.

`S4_PLATTER_STATUS` supplies the independent approximately 100 ms speed estimate
and successful outgoing bend count rate. Its `host_us` uses the same
bridge-start clock domain, although its sample is produced at a different
processing point. The two streams need not have identical timestamps.

This is sampled telemetry, not a lossless per-report encoder recording.
Sub-20 ms oscillations can require a more focused capture if the first baseline
cannot distinguish measurement quantization from real speed variation.

Compare force changes, physical speed estimates and regulator/assistance
transitions on one timeline. Do not change tuning until that comparison points
to an estimator, regulator or transition problem. After one targeted change,
repeat this same untouched baseline; only then add separate braking and nudge
checks to ensure responsiveness was retained.

For this comparison, expect untouched settled rotation to remain **regulating**
in the detailed log rather than repeatedly cycling through assisting and
qualifying. Compare actual RPM variation and idle bend/audio output as separate
outcomes; eliminating mode transitions alone does not prove stable speed.
The longer confirmation can make a brief rim brake resist more before assistance
begins. Assess that only in a separate attended braking recording after the clean
baseline, including sustained braking, a held stop and normal recovery.
