# S4 MK3 left platter and both NI screens

This Windows x64 diagnostic opens one S4 instance using the bridge's existing
NIHardwareService `strt` acquisition. It sends only NI jogwheel requests and
screen frames. NI owns physical motor and ring writes throughout the run.
Traktor's reference used `inst`. This run tests `strt` with actual hardware.

## Get ready

1. Extract the entire ZIP to its own folder. Close Traktor, Djay and all
   bridge instances normally. Leave NIHardwareService running and connect
   just the target S4. The launcher detects it automatically.
2. Clear both platters. Keep the S4 power switch in reach. Press Ctrl+C
   for an early stop. If the controller keeps moving after Pause, or either
   platter moves unexpectedly, switch the S4 off. Pipe closure alone does
   not confirm a physical stop.
## One attended run

Open **Windows PowerShell as administrator** in the extracted folder. When
you are ready to watch, run the launcher. It starts the NI output observer
before the diagnostic makes any NI requests.

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\Start-Attended.ps1
```

Type `WATCHING` when you are ready to start the observer. Wait for
`NI observer ready`, then type `START` to run one cycle. The observer has
a three-minute deadline for detaching its hooks.

| Phase | What to watch |
| --- | --- |
| Blue baseline, about 2 seconds | Both screens show a moving white bar. The left top band is red, the right cyan. Both wheels should be still. |
| Purple TT while paused, about 2 seconds | Both wheels should stay still and both bars should move. The diagnostic requests a blue marker on the left ring only. |
| Green Play, up to 3 seconds before the Pause attempt | Only the left wheel should spin. The request uses the captured 32.28 RPM value. Watch whether the left marker moves and both bars keep advancing. |
| After Pause and return to JOG, about 2 seconds | The left wheel should stop. Both bars should still advance. Check whether the left marker stays lit. |
| Close | The diagnostic requests ring off before closing. Check that both wheels are stopped and the left ring is dark. |

Report whether the left wheel moved and stopped on schedule, whether the
right wheel stayed still, and whether the two rings and screen bars stayed
steady or changed during the run. Note the new `captures` folder printed by
the launcher. Preserve any unexpected result before considering another run.

The console reports request submission and acknowledged screen frame pairs.
The observer attributes matching NI native write submissions. Compare these
with the physical observations before accepting motor stop or screen output.
The ring commands are unverified on hardware. The expected NI output is
left-side report `0x32`, mode `2`, color `46` (blue), followed by mode `0`,
color `0` at cleanup. The diagnostic does not write raw HID reports.
