# BLOOP integration validation - November 15, 2026

## Earlier transport probes

On November 14, Preview 13 Basic Loopback (`1.0.1.8`) passed two independent
WinMM process round trips on the test PC, including handle closure and
transient endpoint removal. Each process also received its own sends, so a
distinct return message was required to prove bidirectional delivery.

A separate Djay test moved the crossfader with a mapped MIDI command and
confirmed the change in Djay's UI; the decoder rejected the command echo.
Receive-only tests with empty and loaded decks both timed out waiting for
Djay's configured crossfader feedback. A synthetic legacy WinRT output reached
WinMM on the same endpoint but did not explain why Djay's output was absent.
These were feasibility tests, not production backend or throughput checks.
The subsequent integrated test below established genuine Djay feedback.

The isolated `omos/bloop-integration` branch starts at
`986a0dbcd0e2fd4f6207fb12ac23c09ba1a271c0`. Local implementation, regression
checks and private packaging are complete. Attended Windows SDK operation,
native software suites, actual app export/readiness, and two Djay startup
connections passed. Physical crossfader delivery, independent GUI feedback
and the physical PLAY light, and actual app recovery/failure/shutdown passed.
Windows QA resources were removed and original preferences restored.
Local delivery and cleanup receipts complete the remaining goal ledger;
this is not full Windows product acceptance.

Evidence is preserved in the original checkout under
`.omo/evidence/bloop-integration-20261115-0mfgZO/`. Its
`local-evidence-index.json` records the SHA-256 and size of 35 retained
files, the package identities, and the restored mutation sources.
Paths below are relative to that evidence directory.

## Local results

These checks ran on macOS against the isolated source. The Windows
cross-build results here are followed by separate attended native results
below.

| Check | Observed result | Artifact |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Exit 0; no output | `local/bloop-rust.0MYfC7/fmt-final.log` |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Exit 0 | `local/bloop-rust.0MYfC7/clippy-final.log` |
| `cargo test --locked` | All registered suites passed | `local/bloop-rust.0MYfC7/test-final2.log` |
| `cargo build --locked --all-targets` | Exit 0 | `local/cargo-build-all-targets.log` |
| Portable Windows host suite | 117/117 passed | `local/host-final-pristine.log` |
| Packager pytest suite | 13/13 passed | `local/packager-final-pristine.log` |
| Actual C# exporter with actual Rust generator | Exact baseline mapping; unsaved scratch and bend preferences applied to all four decks; existing destination preserved | `local/offline-generator-proof/generator-proof.json` |
| `swift test --package-path macos`, with task-owned scratch output | 80 tests, zero failures | `local/swift-tests.log` |
| Windows Rust worker and native test cross-build | Exit 0 | `local/windows-engine-build.log` |
| Native SDK test runner publish | Exit 0 | `local/native-sdk-final-build.log` |
| Native host test runner publish | Exit 0 | `local/native-host-tests-build.log` |
| Self-contained private Windows app build | 240 packaged files; archive entry hashes verified | `local/portable-app-build.log` |

The empty formatter log is accompanied by the recorded successful process
completion; an empty file alone does not establish an exit status.
Shell syntax, `macos/Info.plist` lint, contract JSON parsing and
`git diff --check` also passed during this session. Final review covered the
complete tracked diff and new SDK ownership/test sources, with no blocking
findings. The unchanged error banner repeats the conflict message; this is
a presentation limit, not a worker or endpoint leak.

The temporary C# diagnostic solution initially omitted the packager and
native SDK test projects. Adding the actual projects and restoring their
cached dependencies resolved the packager's seven false unresolved
`PackageValidation` references. After the owned language server reloaded,
both files published empty diagnostic lists and the real class resolved
in `Packaging/PackageValidation.cs`. See `csharp-project-context.json`
and `csharp-project-context-events.log`. No production source was changed
to address that workspace problem.

Python, shell and JSON language servers are unavailable in this environment.
The actual pytest, compiler/build, shell syntax and JSON parsing checks cover
those files; no additional project dependencies were installed for them.

The Swift agent tests create local Observer Fixture applications. Their
passing result does not establish NI service or controller behavior.
The offline exporter proof explicitly records that native Windows
execution did not occur.

## Regression strength and source protection

Eight deliberate mutations failed at the intended boundaries: BLOOP port
identity, mapping domain overlap, echo admission, endpoint disposal,
worker identity recording, Stop during pending creation, exporter plist
identity, and packager plist identity. Logs are retained under `local/`,
including the three Rust mutation logs under `local/bloop-rust.0MYfC7/`.
All mutations were restored before the final passing runs. The evidence
index records matching restored hashes for the supervisor, exporter and
packager.

The LoopBe/BLOOP mapping comparison reported `MAPPING_EQUAL 582 177`:
582 controls and 177 outputs, with endpoint identity as the only expected
difference. See `local/bloop-rust.0MYfC7/mapping-compare.log`.

`original-platter-checkpoint.json` compares all twelve original tracked
and untracked platter files with their pre-implementation byte lengths and
SHA-256 fingerprints. All twelve matched again in
`original-platter-final-checkpoint.json` after attended testing.

## Package and attended-test boundary

The private app ZIP SHA-256 is:

```text
7f59e4fa14d62542f7234b1447d7dc2a45971712b57ec57015ba3570d9e7cddc
```

The SDK dependency is `Windows.Devices.Midi2 0.99.92-preview.11`, with
CsWinRT 2.2.0 and Windows SDK metadata 10.0.26100.57. Preview 13 transports
are the runtime prerequisite; they do not imply a Preview 13 API NuGet
package. The interim Tools DLL is not a build dependency.

The package contains the BLOOP mapping and SDK notices. It has no screen
helper declaration and cannot establish screen acceptance or general
Windows product readiness.

The app and three native test packages were staged under:

```text
C:\Users\toron\AppData\Local\S4BridgeDiagnostics\bloop-integration-20261115-0mfgZO\
```

All four ZIP hashes matched their local values before extraction.
`baseline/before.json` on Windows records six backed-up preference files
and twelve existing mapping-file hashes. `transfer-receipt.json` and
`extraction-receipt.json` record staging. At that preparation checkpoint,
no staged executable had been run.
The temporary archive server was stopped after the downloads.

The initial watching questions expired unanswered, and no diagnostic was
triggered during that wait. After the user confirmed watching, the first
native SDK action was:

```powershell
.\native-sdk\BloopNativeTests.exe --bloop-native .\sdk-native-evidence
```

It ran from the staging directory with a fresh evidence directory and
exited 0. It created and removed owned temporary MIDI endpoints and exchanged
test notes between processes without S4 HID, NI, screen or motor access.

| Goal criterion | Current evidence and limit |
| --- | --- |
| C1: actual Windows app and export | Passed actual UI export, byte-identical generator baseline, packaged worker and GUI ready state; `windows/c1/`, `windows/ui/` |
| C2: initial state and reconnect | Passed two untouched measured connections with A playing/B paused and opposite cue states; `c2-startup-reconnect-validation.json` |
| C3: control, feedback and echo | Physical crossfader, independent GUI playback/crossfader and physical PLAY light passed; returned command addresses remain rejected; `c3-control-feedback-validation.json` |
| C4: lifecycle and owned recovery | Actual Stop/Quit, stale recovery, visible conflict error, foreign preservation and successful retry passed; `c4-lifecycle-validation.json` |
| C5: adjacent regression and preservation | Local checks, native host 117/117, Rust 110+270+2 and all twelve final platter hashes pass; complete diff reviewed |
| C6: cleanup and delivery | Windows staging/QA removed, six preferences restored and twelve mappings retained; local cleanup and branch delivery use separate final receipts |

## Attended native results

The SDK run at 13:14 UTC exited 0 and proved exact distinct messages in
both directions between separate WinMM processes, live-worker ownership
protection, exact removal, stale recovery, preservation of a foreign
same-name endpoint, and malformed receipt rejection. The post-run
`midi.exe basic-loopback list` reported no endpoints of that kind, and
no test/bridge process remained. See `windows/sdk-native-exit.json`,
`windows/sdk-native-evidence/native-events.jsonl` and
`windows/sdk-post-endpoints.stdout.txt`.

The actual Windows host suite passed 117/117. The unchanged Windows Rust
runner passed all 110 shared, 270 engine and two CLI tests, with no failures
or ignored tests. Its current supervisor fixture ran only explicit self-test
modes. See `windows/native-host-*`, `windows/engine-native-evidence/` and
`native-component-validation.json`.

After the separate controller-run confirmation, the actual UI export was
installed under the unique task name `S4 BLOOP Acceptance 0mfgZO`. All twelve
original mappings still matched their baseline hashes. The packaged app
reported `Bridge: ready`, screens off, with a real owned Rust worker.
Wheel tension was 0, backspin assistance disabled and TT modes off; this
does not test motorized playback. The full copied worker log and observed
zero-force receipts are in `windows/c1/`.

For C2, Djay was already running with A playing/B paused and cue A on/B off.
No Djay control changed between the two measured connections. Both raw
captures received channel 7 CC4/5 values 127/0 and channel 8 notes 0/1
values 127/0 before any follow-up control action. The four states arrived
within 0.85 seconds on the first connection and 0.86 seconds on the second.
The finalized files contain 2,963 and 6,521 written events respectively,
with zero drops, truncation or unwritten events.

`windows/c1-c2-midi-captures.zip` preserves those two raw captures and the
initial C1 capture. Its SHA-256 is
`6a97331f48e437aec38b3c92fb41111c40fe028a1508af465afa3252337cfe65`.
The decoded values, times, file hashes and final counters are in
`c2-startup-reconnect-validation.json`; actual before/after GUI captures
and invocation evidence are under `windows/ui/`.

For C3, moving the physical crossfader fully left moved Djay fully left.
The capture contains five channel 5 CC20 output attempts with values
47, 44, 15, 3 and 0, and five matching returned input packets. Those command
addresses are rejected before the feedback cache by `midi_layout::normalize`;
the kept real decoder/cache regressions cover command rejection and genuine
feedback admission. Shared-bus input alone does not identify a sender.

Independent GUI actions then moved the crossfader right and played/paused A.
Dedicated channel 7 CC4 input changed 0 to 127 to 0 without any outgoing A
PLAY command. The user observed the physical PLAY light change to playing.
The final capture contains 19,224 written events with zero loss.
`windows/c3-midi-capture.zip` has SHA-256
`ac9e3faa05c49e22f028c5a94799515bf3531822762fae5f249b6ea4aa7d0571`.
The values and action receipts are in `c3-control-feedback-validation.json`
and `windows/ui/actual-ui-action-log.json`.

For C4, actual running Quit left no host, worker, endpoint or ownership
receipt. A separately prepared dead-owner endpoint
`bfcd8b4e-0032-4607-b2df-b7404d3f440c` was recovered by actual app Start,
which reached ready with a new association and worker. Actual Stop removed
that worker, endpoint and receipt. A task-created same-name endpoint with a
different identity caused a visible start error while preserving its exact
association and creating no worker or ownership receipt. After removing only
that task association, actual Start reached ready again. Final Quit cleaned
the new run without restarting any service.

## Cleanup and delivery

`windows/windows-restoration.json` records six exact preference-file restores,
including the pre-existing instance lock; all twelve original mappings match.
Only the unique task mapping, four hash-matched archived captures and new
ownership lock were removed. `windows/baseline-preferences.zip` preserves the
original bytes. The complete Windows package/test staging directory was
removed; `windows/staging-absence.json` is its read-back absence receipt.
The diagnostic Djay track/fader/library state was left as requested.

All five completed child workspaces were removed with read-back absence
receipts in `child-workspace-cleanup.json`. The two implementation roots
occupied 2,568,628 KiB before removal. Thirty-nine small historical logs,
generated mappings and research files were copied with matching hashes
before cleanup. These historical attempts include failures and an early
incorrect SDK-version assumption; they do not replace the final evidence.

The task-owned C# wrapper, local tool installation, user LSP override,
original-checkout search allowlist and diagnostic Wine prefix were also
removed. Their setup and receipts are preserved in `tooling-setup/` and
`local-tooling-cleanup.json`. The global files were unchanged from their
task-owned versions and returned to their pre-task absence. No owned C#
server remained. The prefix-specific Wine stop returned 1 without output;
a subsequent process inventory found no Wine or projection processes
before the owned prefix was deleted. The retained shared Wine prefix
was left in place.

The isolated committed source worktree is the retained deliverable; the
private QA package is not a published Windows release. Final local run-root,
notepad and branch-retention receipts are kept with the evidence rather than
in public setup instructions. Screens, motorized playback, complete feedback
coverage, controller audio and sustained combined-load acceptance are outside
this BLOOP milestone.
