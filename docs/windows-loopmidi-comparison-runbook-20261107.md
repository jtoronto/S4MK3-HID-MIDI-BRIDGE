# Attended Windows loopMIDI comparison runbook

This is a plan for the separately staged development build, not evidence that
loopMIDI or the controller has passed the comparison. The user must be ready
and watching before the bridge or any test sends MIDI or controller output.

## Before starting

1. Keep loopMIDI running with its `S4 MK3 loopMIDI Test` port and feedback
   detection enabled. Do not uninstall LoopBe or change its protection.
2. Pause playback. Note the selected Djay mapping and crossfader position.
   Preserve the existing mappings and record the five bridge preference
   files before changing any saved setting.
3. The staged package is
   `C:\Users\toron\AppData\Local\Temp\S4LoopMidiComparisonV2-5c1c7fc433ab44a285aaefec57738959`.
   Its separately named mapping is
   `S4 MK3 Bridge loopMIDI Test.djayMidiMapping` in that folder. The app
   executable is `S4Bridge.exe` in the same folder.
4. The existing diagnostic app must exit before the comparison app can own
   the single-instance lock. Preserve its live incident log before quitting;
   ask the user before closing their open app.
5. Import only the separately named test mapping into Djay and select it for
   `S4 MK3 loopMIDI Test`. Confirm the association in Djay. If import proposes
   replacing an existing mapping, cancel rather than overwrite it.
6. To keep the unrelated screen-recovery problem out of the comparison,
   record the original screen preference, then disable screens for the test.
   Restore the exact original preference afterward. Do not stop or restart
   NIHardwareService as part of this comparison.
7. In the comparison app, select the full profile and enable **Capture MIDI
   on the next full-profile launch** in Diagnostics. Confirm the app is stopped.
   Warn that starting initializes controller LEDs and applies saved jog/tension
   settings; ensure playback is paused and hands are clear.

## Attended run

1. Ask whether the user is watching and ready. Start the bridge only after
   explicit confirmation. Check that the app reports bridge ready and MIDI
   capture recording. If either fails, stop and preserve the details.
2. With the user's guidance, first check an ordinary control and feedback,
   then play a track and reproduce their normal workflow. Do not assume that
   successful MIDI sends prove Djay responded.
3. If the reported stutter, lost controls, or port issue occurs, click
   **Mark issue** promptly and note what action and deck preceded it. Stop
   playback if control is lost, then stop the bridge and preserve the JSONL
   file and its final written/drop counts. A queued marker is not proof it
   was persisted; inspect the file.
4. Only if the user is still ready and wants to, compare the previously
   observed backward left-wheel scratch. Stop immediately if the controller
   becomes unresponsive or a platter behaves unexpectedly.
5. Record whether the loopMIDI port stays active and whether controls and
   independent Djay feedback actually work. A test without a reproduced
   fault is evidence only for the actions and duration observed.

## Restore

Stop the bridge and quit the owned comparison host normally. Restore the
recorded screen preference and original Djay mapping selection without
overwriting mappings; confirm the saved preference files against their
baseline. Keep the captured file and do not delete the separate test mapping
or port without confirming their ownership and the user's wishes. Report
anything that could not be restored or verified.
