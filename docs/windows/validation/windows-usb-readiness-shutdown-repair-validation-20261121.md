# USB diagnostic readiness and shutdown repair

This record covers the November 21, 2026 software repair and stopped delivery.
It does not establish normal physical motors, successful real USBPcap shutdown,
or a cause or fix for the screen-acquisition motor/ring failure.

## Scope and causes

The previous paired workflow loaded the track before the new feedback receiver
connected. BLOOP publication precedes the prepared worker's diagnostic BEGIN
handoff, so controller/case readiness did not establish fresh Djay Loaded or
Internal feedback. The manual READY answer could start observation while those
values were unknown.

The repaired paired workflow establishes state after connection: keep Deck B
paused and right TT off; confirm Djay software Internal mode, eject/reload the
agreed track, then select Play and right TT. CHECK waits for this worker's
existing motor-gate verdict. Unknown feedback never becomes authorization.
This is a diagnostic workflow correction, not an automatic Djay state query.
Its real Djay behavior remains subject to the later attended A check.

USBPcapCMD 1.5.4.0 is a GUI-subsystem executable. Its console attachment code
skips AttachConsole when both stdout and stderr are redirected. The old capture
host redirected both; the earlier console-subsystem fixture masked that branch.
The corrected GUI-subsystem fixture reproduced missing console input and failed
graceful shutdown. The host now redirects binary stdout alone, retains console
stderr, sends q through that isolated console, drains the stream, and saves a
bounded console tail in usb-stderr.txt.

The pinned upstream implementation is USBPcap 1.5.4.0,
`USBPcapCMD/cmd.c`, `attach_parent_console` and its call from `main`:
<https://github.com/desowin/usbpcap/blob/1.5.4.0/USBPcapCMD/cmd.c>.

## Implementation boundaries

- The opt-in host receipt binds launch identity and PID to lifecycle MIDI
  readiness, the exact screen-case receipt, and existing right-wheel gate logs.
- Only Deck B, TT on, confirmed Loaded/Playing/Internal, no confirmed Thru, and
  the worker's open playback gate can make the receipt ready.
- Stop revokes readiness. The PowerShell reader checks the worker against
  capture metadata, rejects failed/stopped receipts, and registers its file
  watcher before waiting. Readers permit atomic file replacement.
- A failed readiness check skips observation. USB stop errors remain recorded
  while the script continues to preserve available observations and trace checks.
- The Rust worker, motor control law, screen helpers, mapping and saved tuning
  were not changed. All baseline Rust source hashes remained unchanged.

## Software verification

The final native Windows host executable passed **126/126** tests. The final
GUI-subsystem capture suite passed all eight scenarios, including the real
PowerShell wrapper, GUI gating, CHECK with unknown authorization, shutdown before
notes, and GUI EOF preservation. The PCAP reader, pair gates, bridge-capture
validator and readiness-file suites passed on Windows PowerShell 5.1.

Two causal proofs are retained: the pre-fix GUI-subsystem fixture failed for
missing console attachment; removing the real script's authorization wait
started observation and failed the CHECK regression with expected zero
observation starts versus one. The unmutated script was restored and passed.
The restored-source hash was checked before its verification run.

The corrected host and capture runtime compiled successfully. Source whitespace
checks passed. The package-manager lockfile was restored to its original hash.
Startup/output handling were separated from the central lifecycle partial; its
remaining shared-lock lifecycle has a documented size exception.

Evidence is under the repository root:
`.omo/evidence/s4-usb-repair-20261121-01a1dbd8/`.
Key records include `native-host-exact-final.log`,
`capture-final-import-fixed.log`, `native-powershell-final.log`,
`gui-subsystem-red2.log`, `gui-subsystem-green.log`,
`authorization-mutant.log`, `deployment-receipt.json`, and the stopped preflight
manifest. The source-only snapshot and deployment inputs are retained there.
No whole SDK, runtime or build tree is retained as evidence.

## Delivery and preservation

The stopped kit is `C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 3`.
Use its `07-usb-pair.cmd` and `USB CAPTURE START HERE.txt` when an attended run is
authorized. The next check is A only; end at the NEXT prompt and report A.
Ending there leaves the pair incomplete even when A's own recording is valid.
B requires a working physical A baseline and a later attended comparison.

The kit reuses the complete prior app's frozen worker and dependencies, replacing
only the host DLL/PDB and updating the runtime identity. Its 514 runtime entries
were validated, and real Prepare.ps1 created a stopped A preflight with matching
mappings. Neither the GUI nor a controller worker was started.

All **4,901** inventoried existing files matched their pre-delivery hashes,
including the accepted BLOOP app, both older diagnostic folders and captures,
24 preference files, and 13 Djay mappings. Older deliveries remain available.
The new build identity distinguishes the inherited engine source/validation from
the current host source and tests. No commits, service changes, SDK installation,
real USB capture, MIDI endpoint publication, or motor/screen operation occurred.

Task-owned Windows and macOS staging is removed after receipts are retained.
The new desktop kit is intentionally retained as the runnable user deliverable.
Physical testing requires a fresh confirmation that the user is watching.

## Attended follow-up

The later attended A checks are recorded in
[the first, timed-out attempt](windows-usb-a-first-repaired-attempt-20261122.md)
and [the fresh A result](windows-usb-a-normal-motors-stop-timeout-20261122.md).
Fresh A confirmed authorization, a loss-free bridge trace and reported normal
motors/rings, but actual USBPcap still timed out on normal Stop. The packaged
mode-change/eject instructions also proved inappropriate. Those later findings
limit this earlier software-only repair record; they do not establish complete
USB shutdown or launcher workflow acceptance.
