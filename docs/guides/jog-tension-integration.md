# Saved hardware tension in the normal bridge

The standalone cold-start, right-isolation and simultaneous-wheel trials passed
on October 20, 2026. The normal app now uses the shared validated decoder and
per-wheel force law. The user subsequently confirmed all focused normal-app
checks below: assistance/braking, Djay/MIDI/LED coexistence, Stop, Quit,
saved-setting restore while stopped, and USB recovery after explicit Start.
The full UI regression battery remains deferred at the user's request.

## Open this build

1. In the installed old app, click **Stop**, wait for **Stopped**, then choose
   **Quit S4 MK3 Bridge**. Keep Traktor and the standalone diagnostic closed.
   Do not run two bridge processes.
2. Open the newly built app, without replacing the installed copy:

   ```sh
   open "/Users/joey/Development/S4MK3-HID-MIDI-BRIDGE/dist/S4 MK3 Bridge.app"
   ```

3. Select **Full controller** in the Bridge tab. In **Jog > Hardware > Jog Wheel Tension**,
   the first-install setting is **Neutral (0)**. Click **Save preferences**,
   then **Start bridge**. An existing saved tension value is restored instead.
4. Keep the existing S4 mapping selected in Djay. This change does not require
   importing a new mapping or overwriting a custom mapping.

## Use the setting

The slider is shared by both physical wheels; each wheel uses only its own
movement and touch. Negative values select assistance while touching the metal
top, zero selects neutral, and positive values select braking. Full assistance
is -100 (ceiling 7000); full braking is +100 (ceiling 30000).

Current-source **Jog > Backspin > Motor assistance** is a separate default-on
experiment. When enabled with detection, it can override that wheel's ordinary
tension during a detected gesture and through a one-second fade after release.
The independent **Motor assist strength** slider selects 0-100% of the existing
maximum; 0% bypasses the override. It does not write -100 or the selected
percentage into your normal tension preference. Retouching, measured
stop/reversal, cancellation or stale input ends the override, and normal tension
behavior resumes. MIDI scratching may continue through the remaining natural coast.
Change on/off or strength, use **Save and restart**, and repeat the same gesture
to make your own comparisons. The normal tension and software boost stay separate.
See [the separate trial and verification limits](backspin-assist-validation.md).

While the bridge is running, **Save and restart** applies an edited value. Wait
for the bridge to be **Running** again before evaluating the new setting.
Moving a physical mixer fader does not change tension. Scratch/pitch-bend
mapping preferences below the hardware section are separate.

## Focused live integration check

These checks cover the new runtime behavior, not a UI regression sweep. No
timed gesture script or extra power cycle is needed.

1. With tension at 0 and the bridge running, confirm normal Play/Cue, jog input,
   channel-1 fader response in Djay, and the usual Play/pad/ring/meter feedback.
2. Set tension to -100 and **Save and restart**. Freely try either wheel and both
   together. Confirm the known assistance feel while touching each metal top,
   and that the other wheel does not inherit its force. Check the channel-1
   fader still mixes in Djay and lights still update.
3. Set tension to +100 and **Save and restart**. Confirm the known braking feel
   with normal Djay controls and feedback still working. Stop for unexpected
   drive or chatter.
4. Click **Stop** during use. Confirm both wheels become unforced and the app
   reaches **Stopped**. Diagnostics should contain `TENSION_FINAL_NEUTRAL` and
   a successful exit. Start again to check the saved setting is reapplied only
   after startup and fresh qualified movement.
5. Quit the app while the bridge is running. Confirm force clears, then reopen
   this same build. Confirm the chosen setting is still saved and the bridge
   remains stopped until explicitly started.
6. While running, unplug USB. The bridge should exit and show an error, rather
   than continue issuing force. Cleanup may report a native write failure
   because the device is absent; this is not a successful neutral acknowledgement.
   Reconnect USB and explicitly **Start bridge**. Confirm normal fresh-session
   behavior and no force inherited from the old movement history.

Report the feel, MIDI/LED coexistence, and Stop/Quit/reconnect results. The
diagnostics log is the evidence for selected tension, exit status and accepted
cleanup; it cannot establish physical force or audio behavior by itself.
