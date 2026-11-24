# Stock Windows Traktor: JOG tension A/B

Copying this kit does not start or authorize a run. Only run after separate
authorization with the operator present and watching. This launcher starts
stock Traktor with an IPC observer; it does not send service commands or
collect NI motor writes. No bridge, Djay, screen helper or second S4 client.

1. Leave NIHardwareService running. Close Traktor normally first. Clear the
   platters and keep the S4 power switch accessible. In an administrator
   **Command Prompt** in the extracted kit folder, use the actual stock path:

   ```bat
   Start-Traktor-IPC-Capture.cmd "C:\Program Files\Native Instruments\Traktor Pro 4\Traktor.exe"
   ```

2. Wait for `CAPTURE READY` and Traktor. Load Deck A; both decks stay paused
   in touch-sensitive JOG. Record the original Jogwheel Tension and Ticks when
   Nudging settings. Set ticks to None and leave all other conditions fixed.
   Do not press Play, TT, Shift or hold JOG. Use the **Preferences** control.
3. Enter each marker in the console and wait for `MARKED` before acting:

   | Marker | Action |
   | --- | --- |
   | `READY_IDLE` | Note original value, both screen states and paused baseline. |
   | `TENSION_A` | With hands off, set a distinct low UI value (e.g. 64, if different from original). Note the exact settled number. |
   | `TENSION_B` | Change only tension to a higher UI value (e.g. 160). Note the exact settled number. |
   | `RESTORE` | Restore original tension, then original ticks. |
   | `QUIT` | Quit Traktor normally; confirm both wheels are stopped. |

   For IPC identification alone **no wheel movement is required**. If also
   authorized to compare feel, after each value settles use one comparable
   light nudge of the same rim from rest, same direction and speed, without
   touching the top. Record continuous resistance separately from clicks and
   visible screen/ring behavior. Stop on unexpected motion, chatter, screen
   loss, or uncertain ownership; switch off the S4 if a stop is uncertain.
4. Keep `captures\<run-id>`, UI values, nudge times and deviations. Hooks
   expire after 180 seconds; `STOP_CAPTURE` ends capture early. Neither stops
   Traktor or the hardware. Do not treat an expired or late marker as evidence.

Look for `request` rows from Traktor on a pipe to the NI service PID, command
`0x03567375`, property `JOGT`, 12 bytes, and `value` matching each settled
setting. Slider drags can generate intermediate values. `value: null` means
the value word was missing. The paired `return` is immediate API status, not
a service reply, USB completion or physical result. NI-provenanced `0x31`
submissions require separate writer observation; this kit does not attach to
NIHardwareService. Display headers are not proof that both screens stayed lit.
