# Fresh A: normal motors, complete bridge trace, USB stop timeout

The user-offered fresh A completed on November 21 local time / November 22,
2026 UTC. Actual motor authorization and the observation interval succeeded.
The operator reported normal motors and rings. Stock USBPcap still failed to
stop normally, so the combined recording remains incomplete and B did not run.

Run directory:
`C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 3\usb-captures\20261121-192709-cb34ac98`.

## Actual procedure and readiness

The original audio driver remained selected. The user ignored the inappropriate
Thru/Internal mode-change instruction. They could not find an eject control;
instead, they loaded a different track and then reloaded 303. That is the
reported procedure, not an ejection.

The current worker's Deck B/right-TT gate confirmed Loaded, Playing, Internal,
Thru off and authorization. CHECK was not bypassed. Readiness took
34.042 seconds from the manual Start prompt, within the 90-second limit.
The formal hands-off observation lasted 29.660 seconds. The user entered
"normal operation" separately for motor and ring behavior.

This establishes the reported screens-off A response in this bounded attempt.
It does not establish every control, long-term stability, a reliable initial
feedback snapshot, or screen-acquired B behavior.

## Bridge and USB evidence

The bridge footer reports runtime_ok=true, incomplete=false, and zero losses:
63,096 output entries, 36,432 input entries, and one screen entry. Stop was
recorded at 64,060,151 microseconds. The stopped readiness receipt revoked ready.
The trace SHA-256 is
`f507912fce4e8b59138915e7e9de7f1012e922d6490fa9d44e3d8307844b44c0`.

USB ran from 00:28:06.698 UTC to 00:29:53.494. Stop was confirmed at
00:29:43.438; approximately ten seconds later the host reported
"Capture did not stop" and forced termination. This is the ordinary stop-path
timeout, unlike the first attempt's 180-second duration expiration.

The current host emits this timeout only after its console-q injection returned
success. That does not prove stock USBPcap consumed q or identify whether its
remaining stall is in input handling or shutdown. The retained console tail is
empty. Native fixture success did not establish real USB shutdown acceptance;
the remaining cause is unproven.

The PCAP is structurally complete, with 144,936 target packets, 4,431 motor
requests and 1,448 ring requests. Size is 10,412,729 bytes; SHA-256 is
`0678682632f52e864918a12b523029cc31f8792028ab5228320362f88ba52943`.
Forced termination prevents a complete-tail claim; zero USB loss and request
origin attribution are not established.

## Preservation and disposition

Only A exists in this pair. Post-run inspection found no bridge, worker or
USBPcap process. All 13 Djay mapping hashes remained unchanged. Of the 24
pre-run preference/log files, only the normal last-session log changed;
the original is preserved in preferences-before.

The full run remains in its original desktop directory. Small receipts and
observations are retained under
`.omo/evidence/s4-usb-a-20261122-01a1dbd8/fresh-20261121-192709-cb34ac98/`.
No temporary build workspace was created; the frozen per-run App copy is
intentionally retained with the recording and its identity.

No further capture, B/C/D run, service/driver/registry change, source repair,
commit or push was performed during this attended follow-up. The next repair
must address the unsupported mode/eject instructions and the real USB stop
failure before another requested diagnostic. This record does not claim those
repairs have been made.
