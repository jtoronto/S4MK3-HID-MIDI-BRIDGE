# Mixer ON / Neural Mix EQ live check

This checks the new full-profile bridge and its matching generated mapping.
Software tests and factory preset metadata do not establish these physical or
live Djay results. Preserve all existing custom mapping names.

## Recorded MIDI-only result

On October 20, 2026, the separately named S4 MK3 Neural EQ mapping was imported
and selected in Djay without replacing existing configurations. Djay returned
initial normal-EQ state on Note 20 for all four decks. The user then changed
each deck independently with the mouse: A, B, C and D each returned positive
state for Neural Mix and zero for EQ on its own channel. This verifies live mode
feedback, not the physical Filter/ON gestures or hardware lamp colors.
Local capture evidence is `.omo/evidence/neural-eq/djay-feedback.log`.

The user subsequently reported "Works good" for the physical workflow and
"Lookin good" after the dim-cyan idle Filter refinement on October 20, 2026.
On October 25, 2026, the user confirmed working LED/FX behavior in the
[DJ-laptop test set](dj-laptop-set-validation.md), with no LED/FX fault reported.
The scenarios below remain optional exhaustive regression checks, not known
failures or outstanding normal-use acceptance requirements. No instrumented
pass of every simultaneous-button scenario is claimed.

## MIDI feedback first (no HID)

Run from the repository root. This sequence creates MIDI ports, not a hardware
bridge. Do not run a full bridge or another capture with the same port name.

1. In the running menu-bar app, click **Stop bridge** and wait for **Stopped**.
2. Export the new mapping through the rebuilt app's **Install Djay mapping...**,
   saving it as **S4 MK3 Neural EQ.djayMidiMapping**, not over an existing file.
3. In Terminal, run:

   ```sh
   cargo run --locked --example feedback-capture -- --seconds 300
   ```

   Wait for `FEEDBACK_CAPTURE_READY: no HID interface opened`.
4. In Djay, select the newly imported **S4 MK3 Neural EQ** configuration for
   **S4 MK3 MIDI Full**, with MIDI Out enabled. If only a blank S4 MK3 device is
   available, restart Djay while the capture remains running, then select it.
5. Check the initial capture for Note 20 (hex `14`) on channels 1-4. For A-D,
   positive state packets begin `90 14`, `91 14`, `92 14`, `93 14`; values
   should distinguish normal EQ (zero) from Neural Mix (positive).
   Note Off `80..83 14 00` also means normal EQ. Do not infer zero from silence.
6. Using the mouse, change only deck A to Neural Mix, back to EQ, and then change
   B, C and D independently. Confirm the corresponding channel's Note 20 changes
   each time, including the transition back to zero. Record if Djay changes
   multiple decks together rather than assuming independence.
7. Change the mapping away and back. Confirm a fresh initial Note 20 state for
   every deck. Stop with Ctrl-C before starting a real bridge. If state is absent
   or inconsistent, stop this check and retain the capture; cyan indication is
   not verified and must not be replaced by a local toggle counter.

## Physical buttons and indicators

1. Quit the older menu-bar app and open `dist/S4 MK3 Bridge.app`. Select the new
   mapping in Djay, then **Start bridge**. Keep the existing audio routing and
   saved hardware tension unchanged.
2. Put all decks in normal EQ with the mouse. Confirm ON is bright white for
   the armed Instant FX deck and dim white for the others. An unknown mode is
   deliberately dark until actual feedback arrives.
3. Confirm **Filter** is dim cyan at idle, including with inactive lamps set dark.
   Press and release it. It must continue flashing cyan after release;
   no filter reset or EQ toggle should happen yet. Press Filter again: it goes
   dim cyan with no software action.
4. Arm Instant FX on A. Press Filter, then C's ON. Filter returns to dim cyan; only C
   changes to Neural Mix and its ON is dim cyan. A remains the bright white
   Instant FX target. Repeat Filter then C ON to restore C to dim white.
5. Select C ON normally, without Filter. C becomes bright white. Repeat Filter
   then C ON: it becomes bright cyan and remains the Instant FX target.
6. Repeat mode changes on A, B and D, and use the mouse to change each mode.
   Confirm ON follows actual Djay state. Turn each deck's high/mid/low EQ knobs
   and confirm Djay gives the intended EQ or Neural Mix control without changing
   the selected FX target or resetting the filter.
7. Hold each central Instant FX button in turn. Its target deck's ON must show
   the corresponding button color (red, green, blue, yellow), restoring its
   white/cyan mode and armed brightness on release.
8. Hold two Instant FX buttons. The latest active one wins; releasing it reveals
   the earlier still-active effect. Change FX target while holding one: its old
   deck must retain the active color and release correctly. Filter/ON EQ
   selection must neither release that held effect nor move its FX target.
9. Hold Filter or ON without repeated tapping. There must be no repeated mode
   toggles. If multiple ON buttons are pressed in one report during selection,
   the lowest A-D deck wins; none of them moves the FX target.
10. Enter Filter selection and stop/restart the bridge. Filter starts dim cyan.
    Djay's real modes must return through feedback, not assumed defaults.
    Check normal playback, jog, mixer, LEDs and saved tension still work.

Report the mapping name, deck, physical sequence, expected versus actual
indicator/action, and the corresponding raw Note 20 packets for any mismatch.
