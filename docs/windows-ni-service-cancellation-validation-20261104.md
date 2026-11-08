# Windows NI service cancellation validation

This record extends the recovery evidence for the engine-managed NI screen
route. It is not full Windows product, motor, audio or endurance acceptance.

## Scoped transport correction

The exact Mio 1.2.4 source is retained under `vendor/mio`, including its MIT
license and provenance. Its nondefault `s4-cancel-pipe-writes-on-drop` feature
cancels an OS-pending pipe write when its public owner is dropped. The Windows
application enables that feature; the main crate still forbids unsafe code.
Existing completion-owned buffers, `OVERLAPPED` storage and reference counts
remain unchanged. Completed registration replies still establish success.

Cancellation abandons upstream's graceful flushing of pending output. It does
not recall delivered bytes or guarantee cancellation wins a completion race.
Late completions can retain their owner after selector teardown; the private
tests' extra completion-port drainer is not production cleanup machinery.
General synchronous leak-free destruction is not claimed.

## Native proof

Required repository checks passed. Windows native application suites passed
96 shared, 256 engine and 2 CLI tests: 354 total, with no failures, ignored or
filtered tests. This includes cancellation, original-deadline and dropped-
future tests with an oversized backpressured request. An additional matrix
destroys the actual current-thread Tokio runtime after each interruption and
requires peer closure without an extra test drainer; all three paths passed.

The direct Mio suites passed five feature-on and two feature-off cases under
an external process watchdog. They prove actual OS-pending submission,
completion winning cancellation, safe storage retention through forced late
completion, ordinary pipe-then-selector teardown, and feature-off graceful
delivery. Ordinary teardown reported no retained late owner in these runs.

A copied-source negative control removed only the new write-cancellation
branch. Its pending-write reclamation test failed normally with exit 101.
All three backpressured application regressions also failed normally with
exit 101. Neither negative control merely reached the watchdog deadline.
The working source was never mutated.

Two test-harness defects were corrected before accepting those results:

- The original completion-race fixture blocked after its peer read completed,
  at `GetOverlappedResult(TRUE)` on a handle also carrying a pending read.
  The corrected fixture dequeues the actual write packet while withholding
  its callback, preserving the intended completion-wins race.
- The original PowerShell `Start-Process` wrapper returned zero despite a
  failing native test. The watchdog now owns a direct .NET `Process` and
  reports its real exit code. Both native passes and exit-101 rejections
  were re-established with that corrected oracle.

Evidence is under `.omo/evidence/windows-recovery/20261104/`:
`ni-runtime-drop-build/native-tests`, `mio-native-build/accepted-native`, and
`ni-cancel-mutation/native-rejections`. Earlier failed fixtures and watchdog
results remain preserved and are not substituted for the corrected receipts.

Extra Windows-target Clippy remains blocked by the previously recorded
`needless_return` at `src/jog_diagnostic/ownership.rs:14`. It was not suppressed
or changed. Native heap-verifier tools were not found on PATH or at the checked
standard SDK/System32 locations; no heap-verifier run or installation occurred.

## Integrated startup and both displays

Frozen worker:
`a9dfbb3b05af134d9eb7f2435200d93487cd38d426b056a4f1efb09c60d84b36`.
Run `6082bd61bf724325ba04266da10c0aba` used the existing minimum-force
configuration, with the webcam recording before output. The bridge opened
before Djay. Djay's startup then supplied genuine initial MIDI state,
including tempo-range CC 8 value 20. The webcam showed populated right-screen
state, and the user reported, "The left screen works too."

The 90-second bound ended with exit 0 at November 5, 2026,
00:01:31.2160238 UTC, still November 4 on the Windows laptop. Final neutral
native output was accepted. The camera recording completed with exit 0.
The PowerShell transcript did not retain native stdout status lines, so it
does not prove the screen-command acknowledgements or frame counters.
The later packaged host trial below adds bounded Off/On UI observations and
retained manual Stop and Quit lifecycle records.

This establishes the selected engine-managed route and physical output on
both displays under these conditions. It does not establish every screen
field, jog or normal motor behavior, audio coexistence, sustained load,
all fault paths or complete package acceptance.

## Successor packaged host lifecycle

The successor ZIP, `app-cancel-package/portable-final.zip` under the evidence
root above, has SHA-256
`86cacacafe8a0a953e7ec34a60d833322cdb59359010548c4dcb0272b48f30c6`.
All 591 manifest payloads were verified locally and after native extraction;
the self-contained native host suite passed 88/88, exit 0.

In the actual app, Screens changed from active to off and back to active while
the bridge stayed ready. Worker 7720 then reached its timer limit and exited
0; a later Stop click on its disabled button wasn't a manual Stop test.
The checkbox transition lines had fallen out of the bounded debug log.
Those transitions are native UI observations, not retained protocol counters
or renewed physical acceptance of both displays.

Separate launches exercised the actual Stop button, worker 13108 exiting 0,
and running tray Quit, worker 10164 and host 13780 both exiting 0.
`app-cancel-package/native-last-session.log` retains those launches, screen
startup and active acknowledgments, managed stopping request 1 and worker
exit 0. All runs used maximum force 1, base force and regulator gains zero,
backspin and assistance disabled, and tension zero. They don't establish
normal motor, audio or sustained-load behavior.

After exit, all five original preferences were restored against their SHA-256
inventory, with the bounded trial copies preserved on Windows.
`app-cancel-package/profile-restoration-final.json` records the restored
hashes and absence of the host and worker. No driver or service restart was
used.

The tested package exposed two UI defects: successful NI access information
in the error banner and a blank rendered log despite available log text.
Both were source-fixed with 92/92 source host tests passing. The subsequent
r2 package checks below establish their targeted native rendering. Full Windows readiness,
all screen fields, both jogs with normal motor settings, remaining recovery
paths, audio and the 60-minute attended run remain open. See the
[successor package record](windows-recovery-validation-20261104.md#successor-package-native-screen-and-lifecycle-checks)
for the detailed evidence boundaries.

## Corrected r2 package and generation acknowledgments

The later `app-ui-corrected-package/portable-r2.zip` has SHA-256
`ec243eba9708f6612d7700a52f77ecc1fc05d35781b5a201c52f6467d3561efc`.
The parent verified all 591 payloads locally/native and the corrected native
host suite passed 92/92, exit 0. Host 10020 visibly rendered `--list`
enumeration, no interfaces opened and `EXIT: 0`. Stopped Screens Off/On
prepared healthy access detail without a false error banner; this was pending
access, not active screens. Normal Quit exited 0 and all five original
preferences were restored exactly. See
[native UI evidence](../.omo/evidence/windows-recovery/20261104/app-ui-corrected-package/parent-native-ui.md).

The separate [r2 worker protocol trial](../.omo/evidence/windows-recovery/20261104/app-ui-corrected-package/protocol-trial.md)
retains full stdout with OFF generation 1 `inactive`, followed by ON generation
2 `pending` and `active`, in one worker launch. This closes the missing retained
release/reacquisition acknowledgment evidence for that CLI trial. It doesn't
replace the earlier UI-only checkbox record with an app protocol transcript.
The same minimum-force configuration was used. Timer exit was 0 after 90
seconds, with accepted neutral debug writes; no additional manual Stop
occurred. Cleanup records no host/worker, five unchanged original preferences
and NIHardwareService Running, PID 4968.

The camera ended before ON and its dark, overexposed final image cannot prove
physical ON recovery or field appearance. The later
[UI/package gate](../.omo/evidence/windows-recovery/20261104/app-ui-corrected-package/integration-ui-gate.md)
and [integration milestone gate](../.omo/evidence/windows-recovery/20261104/app-ui-corrected-package/integration-milestone-gate.md)
approve their bounded software/native scopes. This supersedes the earlier
pending-review statement. The recovery plan is now 8/11 milestones complete;
normal motors, both jogs, audio, all fields, full recovery and endurance
acceptance remain open.

## Later stopped workflows and preview package

The [focused r2 workflows](windows-recovery-validation-20261104.md#focused-r2-native-workflows)
add native validation/error detail, LED export/import/reset, Palette save
and fresh-process reload, profile/capture gating, editor-only pending capture
and exact Explorer reveal, tray hide/restore and normal Quit evidence.
All six tabs were reachable. All five original preferences were restored.
These stopped-host observations don't establish running restart, genuine
motor capture or the remaining provider/fault paths.

The [successor preview correction](windows-recovery-validation-20261104.md#successor-package-launch-preview-correction)
has ZIP SHA-256
`264be03947d43903b00a369a1c4edc8c0f424bb0746f4592553144c812e24d62`,
591 verified native payloads and 93/93 native host tests. It keeps the frozen
a9df worker, mapping and corrected 161-package Rust inventory. Actual stopped
screenshots establish draft Minimal, Probe and requested Full-screen preview
behavior; owned-child retention uses a supervised native fixture, not another
physical run. Its separate independent preview gate remains pending.

Full native workflows and all original 133 parity obligations remain open.
The legacy NI workflow's applicability is still unanswered; no service action
or N/A waiver is approved. These later results don't authorize publication
or establish normal motor, both-jog, audio or 60-minute acceptance. Earlier
failed records, camera limits and transport ownership limits above still apply.
