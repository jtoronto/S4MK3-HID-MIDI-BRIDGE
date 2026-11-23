# Screen/motor diagnostic operator package

These tools prepare and operate a **separately built diagnostic app**. They do
not build it, install a mapping, start or stop NI services, or repair the
controller. The accepted desktop app remains the fallback. Do not use this
package without its matching `BUILD-IDENTITY.json` and verified App directory.

## Packaged use when you are available

For the paired device-output investigation, use **07-usb-pair.cmd** and
[USB CAPTURE START HERE.txt](USB%20CAPTURE%20START%20HERE.txt). It records
whole-device USB traffic alongside fresh A/off and B/acquired bridge traces.
It checks USB stream readiness before opening the stopped GUI, verifies
observed motor/ring USB requests after A, and requires operator confirmation
before B. Keep both wheels untouched for this pair; collect motor/ring notes
after Stop and GUI exit. This is separate from the gesture tests below.
It uses the installed USBPcap and the supplied capture runtime. Recording
requires administrator approval and an explicit watching confirmation.

The paired workflow establishes deck state after its feedback receiver connects:
while paused, confirm Djay Internal mode, eject/reload the agreed Deck B track,
then select Play and right TT. Type CHECK; the console starts observation only
after this worker confirms Loaded, Playing, Internal and an open motor gate.
Unknown feedback remains unready. A stopped, failed or mismatched worker receipt
cannot authorize observation. The first repaired-launcher check ends after A.

The capture host receives a binary PCAP header after USBPcap starts filtering.
It stops USBPcap through an isolated console and validates the saved file.
Only binary stdout is redirected: stock USBPcapCMD needs a console stream to
attach for keyboard shutdown. The capture console's bounded tail is saved in
usb-stderr.txt after shutdown.
Those checks establish file integrity and observed report coverage, not zero
USB drops, a complete kernel-buffer tail, process attribution or a hardware fix.

Use the separate diagnostic desktop folder supplied with this build. Deployment
leaves it stopped. The working **S4 MK3 Bridge BLOOP** folder stays intact.
The corrected-build reproduction is already confirmed; no full recheck is
required.

1. Close the ordinary bridge app normally. Keep Djay and the existing matching
   BLOOP mapping; do not reinstall a mapping or restart NI services.
2. Open this diagnostic folder. Double-click **03-off.cmd** for A or
   **04-acquired.cmd** for B. Each launcher creates a fresh `captures` run,
   validates/copies the diagnostic app and snapshots current preferences.
   Preparation does not open the app or controller.
3. Follow the console. It asks for the exact motor mode, side/deck and setup.
   Confirm watching only after setup is ready. It opens the GUI stopped;
   you click **Start bridge** yourself.
4. In Diagnostics, wait for **Bridge: ready** and the case's exact log line:
   A `S4_SCREEN_DIAG_READY off`; B `... acquired`; C `... static`;
   D `... updates`; N `... normal`. B intentionally has no active frame:
   its ordinary Screens status may remain pending. Do not enable/retry
   screens or change cases inside a run.
5. Type READY in the console only after that evidence. Observe 20 seconds,
   entering gesture/symptom notes. Keep the reproducing motor settings and
   physical procedure the same; report motor and ring response separately.
6. Click **Stop bridge**, wait for stopped status and endpoint cleanup, then
   quit the diagnostic GUI normally. Type STOPPED in the console. Keep the
   resulting capture directory and `result.json`; incomplete results must
   not be treated as clean evidence.
7. Return to a fresh A run before advancing. B bad: stop after checking A
   recovery. B normal: try C, then A. C normal: try D, then A. Repeat only the
   first bad/good boundary once. Stop if OFF fails to recover.

**01-normal-capture-off.cmd** and **02-normal-capture-on.cmd** are only an
observer-overhead comparison, not another prerequisite reproduction test.
N-off has no recorded case-ready line; use the normal ready/active statuses
and retain its manual observations separately.

The diagnostic host overrides screen routing, raw/debug logging and duration
only in its launch copy, without changing saved settings. Captures have a
fixed 64 MiB memory budget and are exported after Stop; lengthy setup or
waiting can exhaust the budget. Any loss or missing footer invalidates a
complete-timeline claim. Manual UTC markers are not hardware synchronization.

Capture preparation finishes before the host makes the diagnostic MIDI
connection visible to Djay. The app then continues its normal startup.
`S4_SCREEN_DIAG_PREPARED 1` only confirms that preparation step; continue to
wait for **Bridge: ready** and the case's **S4_SCREEN_DIAG_READY** line.
This ordering does not supply missing Djay feedback or authorize a motor
without actual Loaded, Playing and Internal feedback.

## Advanced manual preparation (optional)

1. Obtain the exact diagnostic app directory, its source/build identity file,
   and the existing installed Djay mapping file. The source identity should
   identify the exact source revision and any uncommitted changes used for the
   build; the script hashes it but cannot verify its truth. The app must include
   the screen helpers and `screen-access.json`.
2. Close the accepted app and any bridge/worker processes normally. Do not
   change services. Keep the installed mapping in place; do not reinstall it.
3. Open PowerShell in this directory. Enter these commands and supply the
   requested absolute paths. `App` is the diagnostic build, not the accepted
   app; the evidence parent should be a task-owned temporary directory.

   ```powershell
   $App = Read-Host 'Diagnostic app directory'
   $Prefs = Join-Path $env:LOCALAPPDATA 'S4 MK3 Bridge'
   $Mapping = Read-Host 'Existing installed Djay mapping file'
   $Source = Read-Host 'Diagnostic build source identity file'
   $EvidenceParent = Read-Host 'Existing task-owned evidence parent directory'
   ```

4. For each run below, choose its case, make a **new** evidence path, and
   prepare it. This copies and hashes the app, all preference files, mapping
   and source identity, verifies the app copy, records both mapping hashes, and
   creates a case-specific configuration. Different mapping hashes are retained
   and flagged as an evidence limitation, not overwritten or reinstalled. It
   does not start the app.

   ```powershell
   $Case = Read-Host 'Case: N-off, N, A, B, C or D'
   $Run = Join-Path $EvidenceParent ([guid]::NewGuid().ToString())
   .\Prepare.ps1 -AppDirectory $App -PreferenceDirectory $Prefs -InstalledMapping $Mapping -SourceIdentity $Source -EvidenceDirectory $Run -Case $Case
   ```

   If preparation fails, stop. Do not reuse its partial directory. The script
   prints the exact `Run-Case.ps1` command for this run. Alternatively run the
   printed `Run-Case.ps1 -Prepared ... -Case ...` command directly. Numbered
   CMD launchers prepare their own new run and take no arguments.

## Attended sequence

Use the same track, side, deck, playback, authorization and motor settings for
each comparison. Prepare a fresh run for **every** entry, including repeated A.

| Order | Case | Command after preparation | Expected condition |
| --- | --- | --- | --- |
| Optional | N-off | `.\01-normal-capture-off.cmd` | Normal production screens, capture disabled |
| Optional | N | `.\02-normal-capture-on.cmd` | Normal production screens, capture enabled |
| 1 | A | `.\03-off.cmd` | No owned screen session or submitted pairs |
| 2 | B | `.\04-acquired.cmd` | Acquired screen session, zero submitted pairs |
| 3 | A | `.\03-off.cmd` | Screen session released; check motor/ring recovery |
| 4 | C | `.\05-static.cmd` | One acknowledged pair, then no further pairs |
| 5 | A | `.\03-off.cmd` | Released; check recovery |
| 6 | D | `.\06-updates.cmd` | Continuing diagnostic frame pairs |
| 7 | A | `.\03-off.cmd` | Released; check recovery |

Compare N-off and N for observer effects. If capture changes the symptom, stop
and report it. Proceed from B to C only if B is normal, and C to D only if C is
normal. After the first bad condition, return to A, then repeat that first
bad/good boundary once with the same setup. Stop if A fails to recover or
release is unconfirmed; do not reset the controller or NI service to force the
sequence onward.

For each numbered command:

1. Enter setup and exact motor-mode notes. The console warns that either
   platter can move. Confirm only when setup is finished and you are watching.
2. The staged GUI opens **stopped**. Confirm this, then press Enter in the
   console and immediately click **Start** in the GUI. Start itself saves
   host preferences; preparation does not. Do not change tuning or mapping.
3. Wait for a genuinely observable host state or receipt appropriate to the
   case. Type `READY` only if it establishes the requested state; otherwise
   type `FAIL`. The script marks UTC and records elapsed time from the manual
   Start prompt; over 60 seconds is incomplete. A visible screen image alone
   does not prove release, acquisition without frames, or exact frame counts.
   If the host provides no usable readiness indication, type `FAIL`.
4. After READY, observe at least 20 seconds. For TT steady playback keep
   hands off. For Assist/Brake repeat the same demonstrated manual gesture;
   enter each touch or movement in the console as it occurs. Type `END` after
   the observation. Give separate motor and ring descriptions.
5. Click **Stop**; wait for neutral, release and stopped status. Quit the GUI
   normally. Only then type `STOPPED`. If any part cannot be confirmed, type
   `FAIL`. Do not force-kill and call it complete. The capture is exported by
   the diagnostic worker after teardown; the script inspects it only after the
   GUI and worker have exited and hashes it. Save the whole run directory.

`observations.jsonl` contains manually entered UTC markers, including Start
prompt, readiness, motor mode, observation boundaries and gesture notes.
`capture.jsonl` uses its `utc_unix_us` header plus each record's `us` offset;
manual timestamps are not synchronized hardware measurements. `result.json`
is incomplete for unconfirmed readiness/Stop, a short observation, startup
over 60 seconds, capture failure, missing receipts, runtime failure or lost
records. Capture-off is necessarily marked incomplete for instrumented
evidence; its manual observations remain useful for overhead comparison.

The post-stop traffic checks cover the whole capture, not just the marked
window. They are conservative preliminary checks, not proof that readiness or
release occurred at the manual marker. Analyze event times against the UTC
markers, verify releases and compare real motor/ring observations separately
before drawing conclusions. A successful software frame receipt is not a
device-side display acknowledgement, and a successful motor write is not
measured torque.

The automated checks assume the current diagnostic worker JSONL contract:
`header`/`footer`, `utc_unix_us`, `runtime_ok`, `incomplete`, `first_stop_us`,
three role loss summaries, and the screen event names and publish-result field
currently defined in `src/screen_motor_diag` and the NI screen worker. The
traffic expectations are whole-run checks: off has no connect or publish,
acquired has a connect and no publish, static has exactly one publish, and
updates has at least two. A changed worker format or legitimate extra screen
transition can make those checks incomplete; inspect the retained capture
rather than treating the checker as a complete protocol verifier.
