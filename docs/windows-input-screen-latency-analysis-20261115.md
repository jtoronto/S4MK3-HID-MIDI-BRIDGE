# Windows input and screen interaction analysis - November 15, 2026

The evidence separates an analog-input acquisition problem from a screen-dependent
motor/ring problem. It does not establish physical USB bandwidth saturation.
No product fix or new hardware experiment was performed during this analysis.

The strongest specific input-path finding is that full-profile analog state is
still gated by interrupt notifications, whereas buttons and jogs receive regular
snapshot queries. The supplied Raw HID log already contains long gaps and large
crossfader-value steps before MIDI conversion. This is a concrete mechanism to
correct and validate.

The motor failure with screens enabled was reproduced, but the saved WPR profile
does not contain the requested/written motor forces, sensor values, or attributed
HID report payloads needed to identify its exact cause. In the matched motor-on
pair, the reconstructed control-thread scheduling and aggregate I/O waits do not
show a broad CPU overload or a new long blocking interval that explains the motor
struggling to move. NI/controller ownership interaction remains a candidate, not
a proven diagnosis.

## Scope and identity

The delivered Rust/host source is the isolated `omos/bloop-integration` worktree
at `60df4353335f597d644ccb2aba82e2f743407c3c`, based on `986a0db`.
The original checkout's unfinished platter work was not edited or merged.
The retained Windows screen helpers came from the November 7 package; their
exact source-commit identity was not independently established by this analysis.
The inspected helper source pins Encdr `a5414658a69c4330d10b95f2359371dfc8550326`;
the main engine pins `5d7a689ca5789161b3efd1b2049d7e54504fe881`.

The BLOOP feature commit did not change `src/input.rs`, `src/bridge_runtime*`,
`src/full/output.rs`, `src/screens/`, or the two screen-helper source trees
relative to its base. That does not establish an unchanged runtime workload.
The exact previously good complete Windows build has not been identified.

Only saved ETLs, prior captures, source and read-only analysis tools were used.
There were no new recordings, HID opens, MIDI endpoints, motor/screen commands,
bridge restarts, preference changes, driver installations or service changes.

## Captures and physical observations

All runs used the existing WPR `GeneralProfile.Verbose` in file mode, with a
20-second test portion. The operator configured the app state; WPR itself did
not verify or change the motor/screen settings.

| Run | Screens | Motor mode | User observation |
| --- | --- | --- | --- |
| `173933-ScreensOff-b2931fad` | Off, explicitly confirmed | Off | Ring normal; crossfader not separately reported |
| `180540-ScreensOn-6a48ce1c` | On | Off requested | Ring and crossfader jerky/stuttering |
| `181046-ScreensOffRepeat-8c71e75f` | Off | Off | Ring smooth; crossfader jerky |
| `181517-ScreensOff-cfbd345e` | Off | On | Motor smooth; crossfader jerky; ring not separately described |
| `181814-ScreensOn-8d7edb13` | On | On | Crossfader jerky as usual; motor struggling to move; ring not separately described |

The original ETLs and operator annotations remain under
`C:\Users\toron\Desktop\S4 WPR Diagnostics\captures`.
Their exact paths, sizes and SHA-256 values are recorded in
`.omo/evidence/wpr-setup-20261115-60df435/capture-matrix.json`.

The ring's off/on/off contrast is useful physical evidence of a screen-linked
effect. The crossfader remains jerky with screens off and with motors off or on,
so it must not be presented as a symptom caused solely by screens. The matched
motor-on pair reproduces a screen-dependent motor problem.

## Method and data quality

The analysis used portable ETWAnalyzer `v4.5.0.7`, release archive SHA-256
`5371df2cb34bb30b7ffe18437e7ebe6ddfa1f7cfff25356f021ffc6d2405ec0a`.
It was run one extraction at a time with concurrency 1 and below-normal process
priority. Microsoft symbols were cached only in the task's temporary directory.
No analyzer was globally installed.

Each test region was aligned from the ETL session start to the capture helper's
recorded UTC start, then limited to 20 seconds before the
`Wpr_Initiated_Rundown` mark. Save/rundown and merge activity were excluded.
Offsets were approximately 0.20-0.35 seconds after ETL session start; full
details are in `active-windows.json`.

A focused reader using Microsoft .NET TraceProcessing `1.12.10` independently
exported per-thread scheduling distributions. Its accepted version computes
off-CPU intervals directly from consecutive activities of the same thread,
including ready-only preemptions. It reports missing previous activity,
censored boundaries and inversions explicitly. Null SDK waiting duration is
not silently interpreted as a measured zero wait. Build/publish completed with
zero warnings/errors, and all five offline queries completed successfully.

There are important limits:

- All recorded worker command lines include `--raw`. Raw HID printing can
  perturb absolute timing. Both members of the strongest motor comparison
  have it enabled.
- The first motors-off/off worker starts about 10.07 seconds into the nominal
  test window, and the off-repeat worker about 6.49 seconds into its window.
  Their whole-window CPU totals are not full steady-state baselines.
  Both matched motor-on workers already exist at trace start.
- Configuration file paths are identical in the matched motor pair, and the
  command lines differ by `--screens`; historical configuration file contents
  and physical/controller modes were not independently snapshotted.
- Pre-stop WPR loss counters were zero, but the final ETL headers report lost
  events: 65,803; 172,146; 112,293; 7,029; and 92,912 in capture order.
  The analysis did not localize every loss to the active or rundown period.
  Zero pre-stop counters are not proof of a complete final trace or of no
  controller/HID report loss.
- The focused reader allowed known event loss, but not time inversion.
  Reconstructed control threads in the matched pair have no missing previous
  activity and no inverted gaps. This is useful consistency evidence, not
  proof that every physical operation was observed.
- The generic profile has scheduling stacks, not complete HID operation
  spans, report IDs/payloads, requested force, or USB wire utilization.
  A context switch or wakeup is not an HID write count.

## Measured motor-on comparison

The following CPU values are sampled CPU milliseconds in the aligned 20-second
windows. For example, 750 ms is 3.75% of one core, not 3.75% of the six-core
machine.

| Process | Screens off | Screens on |
| --- | ---: | ---: |
| Rust bridge | 750 ms | 636 ms |
| App host | 606 ms | 541 ms |
| NIHardwareService | 485 ms | 1,032 ms |
| Djay | 12,274 ms | 11,069 ms |
| Encdr screen worker | Not present | 63 ms |
| Screen supervisor | Not present | 4 ms |

The screen worker is not consuming enough observed CPU to support a CPU-render
overload explanation on its own. The NI service does more work with screens
enabled, as expected for an additional service path, but this increase is not
proof of contention or competing writes.

Inclusive bridge method waits are also similar:

| Method | Screens off | Screens on |
| --- | ---: | ---: |
| `KernelBase!DeviceIoControl` | 575 ms | 564 ms |
| `KernelBase!WaitForSingleObjectEx` | 629 ms | 531 ms |

These are inclusive aggregated waits. They must not be summed across a call
stack or treated as the latency of one I/O request. A `WriteFile` call can return
pending and wait elsewhere; its own zero aggregated wait does not prove that
every write completed immediately or reached the hardware.

The complete reconstructed off-CPU distributions are:

| Control thread | Screens | p95 off-CPU | Maximum off-CPU | Maximum ready delay |
| --- | --- | ---: | ---: | ---: |
| Main/output | Off | 15.030 ms | 26.157 ms | 0.746 ms |
| Main/output | On | 15.313 ms | 26.714 ms | 0.628 ms |
| Snapshot reader | Off | 15.259 ms | 26.402 ms | 1.069 ms |
| Snapshot reader | On | 15.334 ms | 26.559 ms | 1.068 ms |

No reconstructed off-CPU interval exceeds 50 ms in these control threads, in
any of the five runs. Across the five conditions, p95 off-CPU intervals are
around 15 ms and maxima around 26-27 ms. The nominal 2 ms loop interval is not
an assurance of 2 ms servicing.

This does not support a newly introduced, large scheduler freeze when screens
are enabled. It does not prove the absence of all short deadline misses,
accumulated processing delay, stale sensor data, or a different controller
state.

## Crossfader: specific acquisition mechanism

`src/input.rs:67-119` queries buttons/report 1 and jogs/report 3 every full
batch, then performs one interrupt read. A fresh analog/report-2 GET is made
only when that interrupt returns report ID 2. Continuing report-1/3 activity
therefore does not imply regular analog snapshots.

The separate Raw HID capture already showed:

- 24 analog snapshots in the available 1.73-second rolling tail.
- A raw crossfader step from 2695 to 3006 after 268.043 ms.
- 34 other button/jog raw reports processed during that gap.

The changing field is report-2 payload bytes 0/1, normalized to 4095 by
`src/main.rs:572-638`. MIDI quantization occurs later in
`src/full/mod.rs:700-748`. The roughness is already present at the raw analog
processing boundary, before rounding to MIDI CC values.

The new scheduling results are consistent with that distinction: control
threads continue running rather than pausing for the whole analog gap.
Notification-dependent analog refresh is the strongest supported input
mechanism. The captures do not establish whether the notification was delayed,
queued, lost, or not generated at the expected cadence.

The smallest input correction to validate is a regular, bounded full-profile
analog snapshot cadence independent of receiving an interrupt-2 notification.
Preserve minimal/probe interrupt behavior, stale-report protections, MIDI
addresses and held-control ownership. Do not interpolate MIDI values to hide
the acquisition gaps. Native response and overhead still require attended
before/after verification; no such fix was made here.

## Screens and motors: boundary isolated, exact mechanism unresolved

Motor/LED writes are synchronous operations serviced by the shared output
owner (`src/bridge_runtime.rs:54-108,155-165`,
`src/bridge_runtime/paced.rs:111-123`, `src/platform.rs:6-22`).
The pinned Windows HID backend waits on native operations. Source capability
alone does not establish an observed long write.

Actual screen rendering and transfer occur on another thread and helper,
outside the screen mailbox lock
(`src/screens/worker.rs:253-306`,
`src/screens/worker/ni/lifecycle.rs:105-128`). Thus the evidence does not support
the simple claim that full image rendering runs directly in the motor loop.

Screen enablement also connects to the NI service and waits for a control
grant. Display sends are then synchronous NI-service transactions
(`windows/encdr-screen-worker/src/session.rs:36-50,104-142`).
The inspected helper does not explicitly send motor/LED HID reports.
That does not establish what the closed NI service, driver or firmware does.
Its full-frame encoder and separate service/controller ownership make these
different mechanisms worth distinguishing:

1. An NI/controller state or output-ownership interaction after control grant.
2. Sensor samples remaining duplicate/stale, or motor-mode/feedback state
   changing, while the reader still runs.
3. Frame-transfer-dependent pacing or lower-level output interference not
   represented as a large user-thread wait.

These are candidates. Generic WPR data cannot select among them or establish
that the NI service overwrites a particular motor report.

The motor freshness logic also prevents an overly strong exclusion:
`src/tension.rs:436-446` uses the older per-wheel sensor freshness timestamp
when playback regulation is active. It is not simply the time since the
reader thread last ran. `src/bridge_runtime/paced.rs:54-95` can clear force on
expiry and requires fresh input for recovery. Duplicate sensor samples,
queue age, or accumulated delays can matter without a single 50 ms
descheduling interval. Neither motor force values nor those transitions were
captured by this WPR profile.

## Next corrective work

First address and verify the independent analog snapshot cadence. That is the
most specific code-level input finding.

For the screen/motor issue, the next attended diagnostic should record a
bounded in-memory timeline of:

- GET/read begin/end, report identity and input-frame queue age.
- Jog sensor counters/timers, duplicate/freshness decisions and motor mode.
- Desired force, submitted force, neutral/recovery transitions and write
  begin/end.
- Screen connection/control state, frame submission and acknowledgement.

Avoid the current continuously printed Raw HID stream for timing measurement.
Keep normal motor tuning unchanged. Compare screens off, an acquired NI
session with a held static image, and ongoing screen updates. This separates
session/control effects from repeated frame traffic.

If force submissions remain correct and regular while physical response is
wrong, attributed USB/HID request capture is needed to check competing
outputs or device interpretation. WPR scheduling alone cannot supply that
proof. No unattended hardware test is authorized by this report.

## Retained evidence and tools

Bounded analysis data are under
`.omo/evidence/wpr-offline-analysis-20261115-a0afc289/` in the original checkout:

- Five verified compressed CPU/method extracts.
- Per-thread query output in `thread-results-v2.zip`.
- `active-windows.json`, `process-cpu-summary.csv`,
  `control-thread-summary.csv`, `startup-window-limits.json`.
- Trace metadata and the explicit query-model correction note.
- Analyzer source/configuration, tool release identity and cleanup receipts.

Original capture manifests and operator observations remain under
`.omo/evidence/wpr-setup-20261115-60df435/` and beside the Windows ETLs.
Large ETLs remain on Kensuke; they were not copied into the repository.
Temporary analysis binaries and dependency copies were removed from both
machines during closeout, and the transfer server was stopped. The cleanup
receipt confirms that the controller app and all five original ETLs remain.
Capture launchers and unfinished platter files were retained.

Primary tool/API references:

- [ETWAnalyzer pinned extraction documentation](https://github.com/Siemens-Healthineers/ETWAnalyzer/blob/v4.5.0.7/ETWAnalyzer/Documentation/ExtractCommand.md)
- [Pinned inclusive method-cost serialization](https://github.com/Siemens-Healthineers/ETWAnalyzer/blob/v4.5.0.7/ETWAnalyzer.Reader/Extract/CPU/MethodCost.cs)
- [.NET TraceProcessing documentation](https://learn.microsoft.com/windows/apps/trace-processing/)
- [WPR command-line and named-instance behavior](https://learn.microsoft.com/windows-hardware/test/wpt/wpr-command-line-options)
