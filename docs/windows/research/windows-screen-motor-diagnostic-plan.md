# Windows screen/motor diagnostic plan

November 15, 2026 proposal; implementation authorized November 16.
The user confirmed that the issue persists on the corrected analog build
and explicitly skipped another full attended reproduction check. Implement
the diagnostic package now; attended controller testing waits for the user.

## Objective and starting point

Determine why enabling S4 screens can disrupt motor response or ring
animation. Separate screen-session acquisition, the first display write,
continuing frame traffic, sensor freshness and native output timing.
Do not change the motor controller to compensate for an unidentified cause.

Use the isolated BLOOP worktree and preserve the original checkout's
unfinished platter changes. The currently delivered desktop app contains
the regular analog-snapshot correction. The user reported "Smooth as butter"
after testing that correction. The user subsequently confirmed that the
screen/motor failure still occurs on this corrected build.

The [saved-trace analysis](windows-input-screen-latency-analysis-20261115.md)
established a screen-dependent motor/ring effect, but did not establish a
large new scheduler stall, USB saturation or competing NI motor writes.
Its generic WPR traces lack the force and sensor timeline needed here.
Do not repeat that broad capture as the first diagnostic.

## Reproduction prerequisite: satisfied

The November 16 user confirmation satisfies the prerequisite. Do not repeat
the full check below before implementation. The retained sequence describes
the original proposal and the conditions to record in later diagnostic runs.

1. Identify the exact motor behavior to reproduce: normal Assist/Brake or
   TT playback, physical side, logical deck, selected mode and relevant
   saved settings. "Motors on" alone is not sufficient. Keep the other
   wheel's state recorded too.
2. Use the current desktop BLOOP app and existing matching Djay mapping.
   Keep Raw HID logging off. Do not change tuning or the analog correction.
3. Set up the same track, playback, touch/Vinyl and motor conditions used
   for the earlier smooth-off/failing-on comparison. Do not substitute a
   standalone fixed-force diagnostic for the production motor behavior.
4. Finish setup, state exactly which wheel can move, and wait for the user
   to confirm watching. Trigger the prepared action immediately afterward.
5. Observe screens OFF, normal screens ON, then OFF again, for 20 seconds
   each after the requested state is reached. Keep motor mode and track
   state unchanged across screen transitions. The user describes motor
   response and ring behavior separately; a smooth fader is not their proxy.
6. If the failure no longer reproduces, stop. Record that result and its
   limits instead of building speculative instrumentation.
7. If the failure persists, record whether OFF restores normal behavior.
   If it does not, preserve that persistent-state finding and stop the
   sequence; do not silently reset the controller or NI service.

Screen OFF means the owned session is released, not that the panels turn
black. Existing shutdown deliberately leaves the last image in place.
Pending, failed or unconfirmed release is not a successful OFF comparison.

## Evidence needed if the failure persists

Add an opt-in capture to the actual production worker. The normal Windows
host must retain BLOOP endpoint ownership and managed Stop. Do not run a
second HID reader, MIDI owner or motor experiment beside it.

Use one shared monotonic epoch inside the engine, per-producer sequence
numbers and operation IDs. Include the runtime receipt before initialization:

- Worker, host and helper hashes; controller identity; configuration copies
  and hashes; command line; build/source identity; logging/capture settings.
- Loaded/playing/Internal/THRU feedback, physical side/deck selection,
  TT/Vinyl/touch state, tension and motor settings.
- Actual requested screen case, case-ready event, measured window and
  operator observations. Record readiness and failure, not just intent.

Required timeline:

| Boundary | Record |
| --- | --- |
| Input acquisition | Batch ID; begin/end/result of GET1, GET3, interrupt read and GET2; returned lengths/IDs; batch publication time |
| Consumption | Batch ID, dequeue time, acquisition interval, queue depth and queue age |
| Sensor state | Both wheels' raw counters/timers, deltas, decoder status, last nonduplicate host timestamp and freshness deadline |
| Motor decisions | Requested per-wheel force, mode/authorization/ownership, immediate-zero reason, timeout/recovery and reset reason |
| Native output | Operation ID, logical report ID/length/payload, native write begin/end/result, and returned native byte count before Windows padding normalization |
| Other HID output | Ring initialization, ring, button and meter write spans, with side/report identity and ordering relative to motor writes |
| Screen route | Acquisition begin/READY-observed, generation, render begin/end, frame-pair sequence/hash, publish begin/end, FRAME_OK-observed, release and errors |
| Stop | First Stop request, neutral-write begin/end/result, reader/helper teardown and capture completion |

Record actual force selection, not a recalculated estimate. Timestamp the
same native call that sends the report; never issue a second write to
measure it. An accepted write is not measured torque.

Both wheels matter even when only one is being tested: the production
deadline uses the older wheel's nonduplicate sensor timestamp. A stationary
counter is not stale if the sensor timer advances. Distinguish Rest from
Duplicate and host publication time from sensor freshness.

### Keep the measurement from becoming the problem

- Allocate bounded, thread-owned binary event storage before the measured
  run: a 64 MiB total ceiling, partitioned 32 MiB input / 24 MiB output /
  8 MiB screen. Records must fit a documented fixed maximum size.
- No formatting, JSON serialization, disk writes or waiting on a capture
  queue in the reader, motor owner or screen send path during measurement.
  Do not add a shared hot-path mutex or a new lock-free algorithm.
- Never grow storage while running. Count overflow explicitly; any loss
  invalidates absence-based conclusions. Capture failure must not alter
  force selection or block normal Stop.
- Flush after motor neutral and normal resource teardown. Include declared
  capacity, record counts, losses and completion state in the exported file.
  A crash/forced kill before export is an incomplete capture, not a clean
  run. This first instrument is not crash-resilient tracing.
- Compare the diagnostic build with capture disabled and enabled in the
  reproducing normal screen condition. If capture changes the symptom,
  reduce its cost before drawing causal conclusions.
- Disable Raw HID printing and sampled Debug force printing for these runs.
  Do not enable generic WPR or MIDI packet logging concurrently by default.

Existing standalone diagnostic timing provides a useful pattern, not a
replacement runtime. Reuse its monotonic span and first-Stop semantics where
appropriate; do not import its separate platter model or unfinished tuning.

## Controlled screen comparisons

The user has confirmed current-build reproduction; implement these controls.
They are temporary diagnostic choices, not saved user preferences or new
normal screen modes.

| Case | Required actual condition | Question |
| --- | --- | --- |
| A: off | No bridge-owned screen session; no frame submissions | Is motor/ring behavior normal without this screen route? |
| B: acquired, no frame | NI READY observed; continue existing control polling; submit zero frame pairs | Does acquisition alone introduce the failure? |
| C: one static pair | Same route as B; acknowledge exactly one pair, then submit none during the measurement window | Does the first display transaction introduce or retain the failure? |
| D: continuing pairs | Same held page content, with a diagnostic-only screen phase changing every 500 ms; acknowledge and count actual pairs | Does continuing display work introduce a failure absent in C? |

Also capture N, the unchanged normal production screen route, to establish
that the diagnostic cases explain the actual reported behavior.

For D, alter only the renderer's visual phase. Do not press MOVE, change
pad mode, change Djay playback or toggle LEDs to manufacture screen updates.
The 500 ms period matches existing screen selection flashing. It is a
bounded comparison, not a maximum-throughput test. Record actual cadence:
never claim 2 Hz if calls or acknowledgements miss their slots, and never
catch up missed slots in a burst.

Freeze the actual rendered pair for C. A page that merely looks static can
still change; count submissions to prove the condition. B needs an explicit
barrier after acquisition and before publication: the existing Active
status cannot represent it because Active follows the first successful pair.
Keep the current status protocol truthful; export a separate diagnostic
case-ready receipt for B instead of inventing a successful frame.

The existing helpers can be reused for these first comparisons. Gate frame
publication in the engine's screen worker, and observe its supervisor
transactions. Do not rebuild or change the helper protocol just to add
initial telemetry. Verify retained helper/source-archive identities before
any later helper modification.

READY and FRAME_OK are software observations. FRAME_OK follows encoding,
both NI send calls and control checks. It is not a device-side display
acknowledgement. With unchanged helpers, individual NI calls and hidden
service/controller actions remain outside the initial capture.

### Run order and operator steps

The implementation supplies `windows/qa/screen-motor/Prepare.ps1` and
numbered case launchers. Follow their [ordered runbook](../../../windows/qa/screen-motor/README.md)
when the operator is available. Preparation leaves the app stopped.

For each case, the delivered runbook must list its exact command, expected
state and next physical action together, in execution order:

1. Prepare a fresh capture directory and stopped diagnostic app copy;
   verify hashes and unchanged saved settings. Leave the accepted desktop
   app available as the fallback.
2. Select the reproducing motor mode and physical side, keeping the same
   track and authorization state. Record all controls that had to change.
3. Finish setup, explain the impending motion and wait for confirmation
   that the user is watching before triggering the case.
4. Await the case's actual readiness event. Bound acquisition using the
   existing setup timeout; a timeout is a failed setup, not a measured case.
5. Mark 20 seconds of observation. Keep hands off for TT steady playback;
   for normal Assist/Brake, repeat the same previously demonstrated manual
   gesture and annotate each touch/movement. Do not call those equivalent
   force inputs or combine their results.
6. Stop through the owned host, verify neutral/exit receipts, then export
   and hash the completed capture. Record the user's motor and ring
   observations independently.
7. Check return to A before advancing to a stronger screen condition.
   An unconfirmed release or failure to recover ends the sequence.

Do not require a separate A/N/A reproduction pass. During later attended
diagnostics, use normal capture disabled/enabled only to assess observer
overhead, then A/B/A. If B is normal, proceed to C; if C is normal, proceed to D. Repeat
the first bad/good boundary once with the same setup before treating it as
reproducible. Stop at that boundary rather than running a broad matrix of
decks, screen sides, speeds and forces.

## Interpretation and next action

Compare aligned measured windows and event transitions, not whole-process
CPU totals. Report interval distributions, maximums, error/loss counts and
the exact timeline around each physical symptom.

| Observation | Supported next direction |
| --- | --- |
| B fails before any frame is sent | Acquisition/control-state interaction; repeated image traffic cannot explain this run |
| B is normal, C fails | First display transaction or persistent resulting controller state; not a continuing-traffic requirement |
| C is normal, D fails | Continuing render/encode/send work is necessary in this comparison; still distinguish CPU work from NI/device traffic |
| Sensor timers repeat while GET3 and publication continue | Sensor freshness path; identify the affected side and ensuing gate/neutral events |
| Queue age rises or operation spans lengthen before the failure | Local acquisition, consumption or native I/O timing; identify the operation rather than inferring it from thread wakes |
| Force becomes neutral or changes mode with feedback/ownership | Investigate that specific authorization, timeout or state transition |
| Correct regular force writes persist while physical response fails | Software submission is insufficient; NI/device state or another writer remains possible |
| Ring writes stay regular while the visible ring stutters | Presentation/device interaction remains possible; do not call this a stalled motor loop |
| N fails but D does not | The synthetic display cadence did not reproduce the normal workload; compare normal frame content, rate and transitions |

If only D fails, add one conditional render-only control: do the same engine
render work at the same cadence with no NI session or frame submission.
If that is normal, the result narrows toward encoding/IPC/NI/device work;
it does not yet distinguish those layers. Add helper encode/send spans only
if that distinction is needed next.

If bridge force/sensor/output records remain correct, the next escalation
is an attributed USB/HID request capture or a targeted NI-service trace.
Specify its providers/tool, driver changes, attribution and expected report
evidence in a separate proposal first. No service restart, driver install,
USB capture setup or direct guessed report is included here. Do not claim
that NI overwrote motor report 0x31 without observing such a write.

## Implementation and verification boundaries

Authorized implementation sites:

- `src/input.rs`, `src/input/batch.rs`, `src/bridge_runtime/input.rs`:
  native input spans and frame identity.
- `src/bridge_runtime.rs`, `src/bridge_runtime/paced.rs`, `src/platform.rs`,
  `src/full/output.rs`: decisions, gating and actual shared HID write spans.
- `src/tension.rs`, `src/tension/diagnostics.rs`: passive state snapshots,
  including both sensor freshness timestamps and reset reasons.
- `src/screens/worker/ni/lifecycle.rs`, `src/screens/worker/ni/ipc.rs`:
  acquisition/publication barriers, truthful case receipts and transaction
  spans. Keep rendering/I/O outside the mailbox lock.
- `windows/bridge-app/Core/LaunchSpec.cs` and focused QA tooling: opt-in,
  process-scoped diagnostic configuration using the existing host-owned
  worker. No saved diagnostic preference or second endpoint owner.

Keep default-off production behavior, force laws, gains, limits, freshness
policy, analog cadence, MIDI addresses and normal screen scheduling unchanged.
Do not disable safety gates to obtain a smoother trace.

Before hardware use, deterministic tests must prove:

1. Capture disabled/enabled yields identical decisions, MIDI and HID reports
   for the same input, clock and feedback sequence.
2. A sends no frames/acquires no session; B acquires without publishing;
   C sends one pair; D has bounded admission without catch-up bursts.
3. Failed grant/send/release cannot produce a false case-ready receipt,
   advance the sequence or reuse a poisoned IPC transaction.
4. Delayed writes, duplicate sensors, queue aging, resets and neutral paths
   produce the correct correlated records, including errors and first Stop.
5. Buffer overflow and capture-export failure preserve normal motor/Stop
   behavior and mark the evidence incomplete.

Use the existing paced-output and fake NI transport test boundaries.
Run the repository's required source checks, then the native Windows
software suites against the exact packaged worker. Verify package/helper
hashes, mapping identity and saved preferences. Software passes authorize
an attended test package, not a physical-fix claim.

Retain manifests, small completed captures, observations, decoded timeline
tables and the selected hypothesis/remaining limits under useful evidence.
Use unique temporary build/staging paths. Preserve the accepted desktop
app, previous builds, mappings and original WPR captures; remove task-owned
tools/packages only when the attended investigation no longer needs them.

## Planning evidence

Source inspected for this proposal:

- `src/bridge_runtime/input.rs`: 2 ms admission, publication timestamp and
  bounded snapshot queue.
- `src/tension.rs:246-286,436-446` and `src/jog/decode.rs`: per-wheel
  duplicate handling, gap resets and the 50 ms deadline.
- `src/bridge_runtime/paced.rs`: neutral/recovery priority and shared ticks.
- `src/platform.rs` and `src/full/output.rs`: Windows output padding and
  shared native motor/LED write path.
- `src/jog_diagnostic/platter_telemetry.rs`,
  `src/jog_diagnostic/platter_session.rs`,
  `src/jog_diagnostic/reader_telemetry.rs`: existing diagnostic timing
  patterns and their standalone scope.
- `src/screens/worker/ni/lifecycle.rs`: acquire before publish and
  unchanged-render suppression.
- `src/screens/worker/ni/ipc.rs`,
  `windows/encdr-screen-worker/src/session.rs`: READY/FRAME_OK meaning and
  bounded parent transactions.

During the original planning pass, LSP symbol queries failed with a tool
error; the flow above was verified by reading these sources. That planning
pass ran no new tests, builds or live diagnostics. Implementation evidence
is recorded separately in
[the diagnostic build validation](../validation/windows-screen-motor-diagnostic-validation-20261116.md).
