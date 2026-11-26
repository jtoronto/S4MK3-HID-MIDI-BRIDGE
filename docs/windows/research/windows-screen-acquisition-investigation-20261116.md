# Screen acquisition and motor interaction investigation

## Finding

The strongest remaining explanation is an NI service/driver/device interaction
introduced by registering and acquiring an active controller instance. The
existing evidence does not identify the exact NI operation or prove a second
writer.

There is a useful new discriminator: during B, the bridge repeatedly submitted
one unchanged positive-direction motor report for **22.184 seconds**. The
returned encoder data showed little movement during that interval. The bridge
was not rapidly reversing or chopping its requested motor force in that
interval.

B also requested a steady ring mode and color rather than firmware flashing
or darkness. Position changes could still move or jitter the spot. These
observations focus the investigation beyond the bridge's recorded output
selection; successful native calls do not establish what reached the device
or what torque it applied.

The investigation made no product changes, new controller runs, Windows or
SSH calls, service changes, or application starts/stops. It used existing
captures, retained helper/source evidence and the pinned upstream source.

## Evidence and comparability

| | A: off | B: acquired |
| --- | --- | --- |
| Run | `20261116-111932-A-abbfb4a5` | `20261116-113213-B-6609cf3b` |
| Parsed JSONL rows | 166,177 | 218,630 |
| Runtime footer | Successful, complete | Successful, complete |
| Producer losses / sequence gaps | 0 / 0 | 0 / 0 |
| Screen connections | 0 | 1, successful |
| Bridge-rendered/submitted image pairs | 0 | 0 |
| Physical operator report | Normal motors, assist/brake, lights/rings | Buzzing/struggling motor, rotation not established, rings flickering |

The first A launcher marked its own result incomplete because the GUI had
not exited. The worker had already exported a complete trace. That saved
launcher result remains unchanged; it does not negate the trace's footer or
establish GUI exit. B's launcher result is complete.

The user corrected A's initial intended untouched-wheels note: the requested
assist/brake gestures were performed. A is a mixed playback/manual-gesture
run. Neither its full-run motion totals nor the A/B difference is a controlled
hands-off speed-stability measurement. Notes entered later do not establish
the exact time a gesture occurred.

The captured LED, jog and tension configuration contents match exactly.
Both prepared-run manifests identify the same engine, supervisor and screen
worker hashes. In particular, the engine is
`75bf26f55c1cfbe8bde7e25bf7e8e325922f6d044e8905235b4a5bdf8747618c`.
The full identities, field counts and analysis are retained in the evidence
directories listed below.

## What the bridge actually submitted

Attribution below uses the latest preceding right-side `feedback` record:
Loaded, Playing and Internal true, with the local motor/TT flag enabled.
It describes recorded eligibility, not an independent physical authorization
measurement.

| Recorded right-side output | A | B |
| --- | ---: | ---: |
| Native motor submissions in those intervals | 4,017 | 2,737 |
| Zero-force submissions | 1,906 | 56 |
| Median / p95 requested force, command units | 1,928 / 18,068 | 10,630 / 20,000 |
| Negative-direction submissions | 0 | 0 |
| Suspended/recovery output decisions, whole run | 0 | 0 |
| Right-ring mode/color in those intervals | Only 2 / 46 | Only 2 / 46 |

Mode 2 is the stationary-spot command used by this renderer; mode 3 is its
firmware loop-flash command. Color remained 46 in these recorded ring writes.
The renderer did not request a flashing/dark pattern in those intervals.
Counter-driven position changes remain a possible contributor to perceived
spot jitter; the ring observation alone does not prove a competing LED writer.
See `src/full/output.rs`, `render_ring`, and `src/full/ring_phase.rs`.

### Sustained command with little returned motion

From capture time **22.408095 seconds until the next changed report at
44.592026 seconds**, B's requested right motor force stayed at 10,630.
There were 1,430 motor submissions within that
22.183931-second interval, all with the identical logical payload:

```text
31 01 20 01 00 00 01 20 01 86 29
```

The maximum gap between these submissions was 16.889 ms. At 44.592026 seconds
the next motor submission changes the payload; that endpoint is excluded
from the identical-report count. No intervening bridge motor command changes
the payload.

In that interval, 1,420 returned right-sensor samples had a net delta sum of
156 counts and absolute delta sum of 556 counts. Their counter range was
102,054-102,212. Sensor time advanced 2,216,704,740 ticks, approximately
22.167 seconds at the code's calibrated 100 ticks per microsecond.

This is evidence against an explanation based on the bridge's rapid command
reversal or force toggling throughout the buzzing condition. It does not
prove that the motor received uninterrupted force, establish the absence of
manual rim interaction, or distinguish an external writer from changed
driver/device behavior. Command units are not measured physical torque.

The payload encoding is `src/jog/model.rs:58-75`. Continuous assistance
selection is in `src/tension/playback.rs:245-265`; passive sensor/force
recording is in `src/tension/diagnostics.rs`.

## Input and timing checks

Every captured right-side sensor counter/timer pair matched the corresponding
raw numbered GET3 report **in sample order**: 6,834/6,834 in A and
9,251/9,251 in B. Recomputed modular counter deltas matched every subsequent
sensor delta. Neither run had reversed or duplicate sensor timers.

GET1, GET3 and GET2 retained their expected 23-, 60- and 79-byte shapes.
Windows interrupt padding is a separate path; stale report-1/3 interrupts
are excluded from full-profile state. See `src/input.rs`,
`src/input/batch.rs` and `src/jog/decode.rs:89-145`.

| Measured boundary | A p99 / maximum | B p99 / maximum |
| --- | --- | --- |
| Native write duration | 0.715 / 1.126 ms | 0.771 / 1.270 ms |
| Output decision gap | 16.491 / 20.937 ms | 16.398 / 18.693 ms |
| Published-batch consumption age | 0.400 / 5.102 ms | 0.378 / 4.624 ms |
| Sensor host-time gap | 16.640 / 17.481 ms | 16.582 / 18.149 ms |

All 22,522 native write calls across the two runs returned success and 95
bytes. No captured native input operation failed. Both stopped with a
successful neutral motor submission.

These measurements do not show a broad new bridge scheduling or blocking
stall, missing playback feedback, expired input recovery, or stale/reversed
sensor time explaining B. They do not exclude downstream buffering,
driver-generated/cached data, another output path or device-side effects.

`src/platform.rs:6-54` measures the call to `hid.write` and normalizes the
selected Windows backend's padded 95-byte result. The capture contains the
bridge's logical payload and API result, not an independent USB/device
acknowledgement.

## Acquisition changes NI client state before frames

The actual B trace records acquisition beginning at 1.077840 seconds and
READY at 1.893685 seconds, then zero bridge render/publication events.
READY timestamps the bridge's observation after helper connection; it is not
a direct timestamp of the private NI control notification.
`src/screens/worker/ni/lifecycle/presentation.rs:43-50` returns before
rendering or sending a pair for `Case::Acquired`.

The candidate helper source explains what precedes READY:

1. Rust starts the sibling supervisor with the controller serial.
2. The supervisor contains the native worker, sends ARM byte `a5`, and waits.
3. The worker calls `ServiceSession::connect(0x1720, Some(serial), ...)`.
4. Encdr registers a product client and a device instance with NI, retaining
   request/notification pipes.
5. It sends eleven instance setup messages.
6. The worker waits for the active-control notification before emitting READY.
   The registrations and notification handling remain alive without images.

This is a device/instance registration sequence, not merely sending pixels.
The upstream control event describes this client becoming the active
instance. However, upstream also explicitly intends screens to use NI while
inputs/LEDs continue through OS HID. An active-instance notification therefore
does **not** by itself prove exclusive raw-HID ownership or a global motor/LED
takeover on S4.

References:

- `src/screens/worker/ni/ipc/supervisor.rs:12-48`
- `windows/screen-supervisor/Session.cs:84-108`
- `windows/encdr-screen-worker/src/session.rs:36-52,97-127`
- Frozen source ZIP's vendored Encdr
  `windows/encdr-screen-worker/vendor/Encdr-a5414658/encdr/src/usb/ni_service/`:
  `mod.rs:1-11,84-100` and `protocol.rs:25-43,57-107,167-175`.

The source track contains the eleven exact message payloads and archive
line references. Their private S4 semantics have not been established.
The traced no-frame helper path has no demonstrated periodic HID, motor,
LED or display setter; notification acknowledgements still occur. A service
side effect remains a hypothesis, not an identified background writer.

### Source identity limit

The retained helper binaries and source ZIP hashes match the supplied
candidate manifest, and both run manifests identify those helper hashes.
The manifest explicitly labels source-to-binary binding as candidate and
not independently verified. The source path is therefore a constrained
explanation of the candidate implementation, not a reproduced-build proof
of the exact helper instructions executed.
The candidate also treats the requested serial as a preference and permits
fallback; strict NI-side device selection was not separately captured. No
wrong-device routing was observed or established here.

The pinned upstream source was consulted directly. Context7 had no relevant
Encdr entry; unrelated returned libraries were not used. The public protocol
notes connection/screen verification on Windows Maschine Mk3, which does
not establish all private S4 setup semantics:

```text
https://raw.githubusercontent.com/RufusIbiza/Encdr/a5414658a69c4330d10b95f2359371dfc8550326/encdr/src/usb/ni_service/mod.rs
https://raw.githubusercontent.com/RufusIbiza/Encdr/a5414658a69c4330d10b95f2359371dfc8550326/encdr/src/usb/ni_service/protocol.rs
```

## Competing explanations

| Explanation | Result of this investigation |
| --- | --- |
| Missing Loaded/Internal feedback blocks motor force | Does not explain B's recorded active intervals and positive submissions. |
| Bridge force direction/chopping causes the sustained symptom | The 22.184-second identical-payload interval argues against that mechanism for that interval. |
| Bridge requests ring flash or darkness | Not present in the attributed ring reports; positional jitter is still possible. |
| Bridge image rendering/transfer is necessary | Contradicted by reproduction with zero bridge frame publications. This does not exclude service-internal startup traffic. |
| Broad bridge input/output stall or stale parser state | Not supported by measured gaps, successful reads/writes, raw-to-sensor correspondence or recovery state. |
| NI acquisition changes controller mode, output arbitration or activates another writer | Best remaining family of explanations; no particular operation/writer is proved. |
| Hand motion, differing physical conditions or run order accounts for A/B contrast | Not completely controlled; the user's observations support the contrast, but A included gestures and no post-B recovery A is captured here. |

The earlier November 15 WPR report independently lacked attributed HID
payloads and did not show a broad motor-thread scheduling freeze. It cannot
fill the remaining device-output gap; generic CPU/wait activity is not a
motor-write count. See
`docs/windows/research/windows-input-screen-latency-analysis-20261115.md`.

## Smallest useful next verification

The missing NI detail cannot simply be recovered from these bridge traces.
The supervisor drains native-worker stderr without retaining it
(`windows/screen-supervisor/Session.cs:94-99`), and the Rust adapter also drains
supervisor stderr to a sink (`src/screens/worker/ni/ipc/pipes.rs:91-94`).
This avoids blocked log pipes but leaves no retained helper diagnostic stream
for this investigation. The complete JSONL timelines cover the instrumented
bridge boundaries, not every actor's NI/service/device operations.

Do not change force gains, add retries, or treat a guessed NI message omission
as a fix. C/D image cases are not needed to establish that ongoing bridge
image updates are unnecessary for this reproduction.

The missing discriminator is one short, passive, time-correlated observation
of **all device-bound S4 output**, including NI/service-originated traffic or
mode changes if observable, across the acquired-session boundary. Retain the
bridge trace, identify the source/path of additional writes where possible,
and compare full motor/LED payloads and other control transfers around the NI
control grant. The same Djay playback and motor settings should remain in use;
separate a documented hands-off interval from any assist/brake gestures.

An extra motor/LED writer would support output competition. A changed
service/driver/firmware control state without extra reports requires a
different correction. Failure to observe another write is meaningful only
if the capture covers the real output paths. Existing bridge-only records
cannot distinguish those cases.

The current GUI launcher forces screen acquisition at startup for case B
(`windows/bridge-app/Core/LaunchSpec.cs:33`), and the runbook forbids changing
cases mid-run. It is not already a prepared same-process off/acquired/off
experiment. The runtime does have a live screen-control channel: the startup
override is not a technical prohibition on later screen-state changes.
A later targeted procedure must deliberately cover that sequence and verify
capture coverage before requesting an attended run. A post-B A recovery would
also check reversibility, but would not identify the private NI mechanism.
No such test was started or requested from the operator during this analysis.

There is no verified product fix yet.

## Retained evidence

All paths below are relative to the repository root:

- `.omo/evidence/s4-off2-read-20261116-76ed4699/`: original copied A records,
  independent completeness inspection and operator correction.
- `.omo/evidence/s4-acquired-read-20261116-6548e144/`: copied B records and
  initial inspection.
- `.omo/evidence/s4-screen-acquisition-investigation-20261116-55224869/`:
  this investigation's hypotheses and computed `analysis.json`.
- `.omo/evidence/s4-acquisition-source-track-20261116.md`: independent
  helper call path, setup payloads, provenance and private-boundary limits.

No temporary build trees, diagnostic processes or remote staging were created.
