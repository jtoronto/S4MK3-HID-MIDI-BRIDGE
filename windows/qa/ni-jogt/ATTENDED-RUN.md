# KENSUKE: NI Decks JOG tension trial

**Do not launch unattended.** This separate full bridge build compares the
stock global JOG damping values **25** and **244**, then restores **107** (the
operator's original Traktor setting). It changes both wheels together. Do not
use this runbook on a controller with an unknown/different original value.
There is no slider, motor timeout experiment, TT test, or stock IPC observer
in this run. The Direct HID settings are not involved.

1. Close Traktor, Frida observers, old bridge hosts and helpers normally. Leave
   NIHardwareService running. Use the existing working Djay BLOOP setup; do not
   reinstall mappings or change drivers. Load an Internal track on each
   selected deck; pause both and leave both in touch-sensitive JOG. Lower audio,
   clear the platters and keep the S4 power switch accessible. Start only when
   watching. If a platter moves unexpectedly or stop cannot be confirmed, use
   the power switch; do not hold it still.
2. Open `Start-JOGT-Trial.cmd`, enter `WATCHING`, and click **Start bridge** in
   the trial host. Leave the operator console open. Check Diagnostics for
   `--ni-decks`, `NI_JOGT_TRIAL_IDLE_ARMED`, real deck feedback and no errors. Verify
   both rings and both screens. With wheels at rest, change the left and right
   pad pages separately and confirm that only the intended screen changes.
   If either screen or the NI session is missing, stop; do not enable Direct
   HID or open a second session. Do **not** play, enable TT or touch the top
   plate during the comparison.
3. In the console enter **A**. Wait for the **worker**, not the console, to
   report `NI_JOGT_TRIAL_SUBMITTED 25` in Diagnostics. On the same rim make one
   light nudge from rest. Record continuous resistance separately from any
   clicks, ring flicker, or chatter. Enter **I** and wait for the worker's
   idle status. Enter **B**; after the worker reports **244**, repeat the same
   direction, speed and travel. Do not push harder just to produce a result.
   Confirm both screens still update without cross-over.
4. Enter **I**, wait for idle, then **R**. Confirm submission of **107**.
   Click **Stop bridge**, wait for stopped status, verify both wheels stopped,
   and **Quit** the host normally. Return to the operator console and press
   Enter if it is still waiting. `Q` alone does not stop a bridge. If the
   bridge loses control or restore is unconfirmed, mark it **unconfirmed**;
   do not start a second writer. With the bridge closed, the known recovery
   reference is the stock Traktor preference at 107, not a direct HID write.
5. Keep `runs\<run-id>\operator.log`, host Diagnostics / last-session log, and
   the operator's observations for each value, both screens and stop. A file
   edit is **intent**; the worker receipt means **pipe submission**, not a
   service reply, NI `0x31` output or applied torque. Record any rejected
   request; do not count it as a negative feel comparison. This trial does
   not automatically start a writer trace. A repeat on the other side, or
   separate NI-side output observation, is a follow-up, not a reason to mix
   output writers.
