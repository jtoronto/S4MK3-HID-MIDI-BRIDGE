# Direct-HID Djay-first startup check

Use the Windows full-profile BLOOP build. This checks startup feedback
without the NI screen session or powered TT playback.

1. Extract the whole package into a fresh local folder. Stop and quit
   any other bridge, including its tray app. Launch `app\S4Bridge.exe`
   directly, not the NI opt-in launcher. Select **full**, turn off
   **Enable S4 screens**, and keep both TT modes off. Use **Info** log
   detail and leave **Raw HID reports** off.
2. Leave Djay open with active PRO and the existing **S4 MK3 Bridge
   Windows** mapping for **S4 MK3 Bridge BLOOP**. Load both decks in
   Internal mode. Leave Deck A paused and Deck B playing, with audio
   turned down.
3. Arm **Diagnostics → Capture MIDI on the next full-profile launch**,
   then click **Start bridge**. Do not touch controls on the S4 or in
   Djay. After a few seconds stop the bridge, save **Diagnostics → Copy
   log**, and keep the new MIDI `.jsonl`.
4. Re-arm the one-shot capture and repeat without closing Djay or
   changing deck states. Save the two runs under distinct filenames.

Expect `BLOOP_CREATED`, then `MIDI_FEEDBACK_READY` before `S4_HID_OPEN`
and `MIDI_FULL_READY`. The log must also show actual loaded, Internal
and Play states for both decks, not merely **Ready**. Preserve failing
logs and traces. For normal use after a missed refresh, leave the
bridge running and fully quit and reopen Djay; do not change drivers
or remap controls as part of this test.
