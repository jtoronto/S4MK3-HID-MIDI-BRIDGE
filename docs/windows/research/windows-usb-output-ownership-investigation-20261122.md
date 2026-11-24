# USB output ownership investigation

## Finding

The evidence strongly supports a command-conflict mechanism associated with NI
controller acquisition and release. The bridge sends positive motor force, but
an additional stream sends zero force for both wheels at approximately 500 Hz.
In fresh B, the next zero request follows each bridge motor request within
3.184 ms, with a median of 0.962 ms. Additional ring commands provide a matching
explanation for the observed flicker.

The earlier mistimed B supplies the onset missing from the fresh B recording:
the extra stream begins during its NI connection handshake, before READY.
It then continues after that attempt's successful release receipt and bridge
exit. Fresh B starts with the stream already active and finally loses it during
release. This makes NI-related output ownership/lifecycle the investigation
target, rather than image publication or changing the motor gains.

The exact originating process, driver, and triggering NI request remain
unidentified. This report does not establish that a named NI executable is the
writer, that one particular setup message is defective, or that a proposed
protocol change fixes it.

No product behavior was changed and no new controller test ran.

## Recordings and provenance

All three original runs remain under
`C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 3\usb-captures`.

| Evidence | Run | PCAP SHA-256 |
| --- | --- | --- |
| Normal A | `20261121-192709-cb34ac98\A` | `0678682632f52e864918a12b523029cc31f8792028ab5228320362f88ba52943` |
| Mistimed B | `20261121-200120-B-c9b336cc\B` | `fa5b07234f8f367d428336657e267c81cf0863f6a1b10958b14004ee29d02ced` |
| Fresh B | `20261121-201419-B-7ebe06e2\B` | `054f3c89be32995c223f8932a8db181dcac718c6a907bccfc7bc4bf9cf11a1e6` |

The mistimed B remains excluded from the hands-off physical comparison. Its
complete 16,950-row worker trace and USB recording are useful for acquisition
and release timing. Its footer reports runtime_ok=true, incomplete=false and
zero losses; the host log records the same run and EXIT: 0. Fresh B's 98,858-row
trace is likewise complete and loss-free. Saved metadata identifies worker PIDs
14936 and 6572 respectively and identical LED/jog/tension configuration contents.

Original files were retrieved read-only after the user made the laptop
reachable. No Windows diagnostic command, application launch, capture,
controller operation, service/driver/registry change, commit or push occurred.

## The first acquisition reveals onset and incomplete output recovery

Times below are November 22, 2026 UTC:

| Mistimed B event | Time |
| --- | --- |
| USB recording already receiving packets | 01:05:21.243 |
| Worker screen connection begins | 01:05:48.461338 |
| First additional zero-force motor request | 01:05:49.150812 |
| Worker reports screen READY | 01:05:49.356384 |
| Worker reports successful release | 01:05:59.828176 |
| Worker lifecycle ends successfully | approximately 01:05:59.829 |
| Last observed additional motor request | 01:06:32.840646 |

There are no copies of that additional payload in normal A. In the first B,
it first appears 689.474 ms after connection begins and 205.572 ms before READY.
All 21,831 such submissions have matched successful USB completions.

After the first B's successful release receipt, another 16,496 requests occur,
continuing for 33.012 seconds to the recording's end. Thus a successful client
release/bridge exit does not, in this recording, establish that additional
device output has stopped. The failed capture shutdown does not explain away
these successfully completed device-bound requests.

Fresh B already contains the stream at 01:21:25.801497, before its worker
connection begins. Its last request occurs at 01:22:53.131958 during release;
none follow release completion at 01:22:53.160750. The interval between the
recordings was not captured: continuity across that gap is consistent with
the evidence, but is not proved.

## The bridge and USB clocks agree

Every bridge motor write was paired, in order, with the identical USB payload
after excluding the distinct additional zero-force payload:

| | Paired writes | Median USB minus trace time | Maximum |
| --- | ---: | ---: | ---: |
| Mistimed B | 734 / 734 | 10 microseconds | 99 microseconds |
| Fresh B | 4,196 / 4,196 | 15 microseconds | 330 microseconds |

This independently checks payload attribution and the UTC conversion. The
23-second pre-READY interval is not explained by a comparable clock offset.
The trace anchors its monotonic epoch to SystemTime at capture construction
(`src/screen_motor_diag/capture.rs:55-68`).

Fresh B's authorized observation contains 1,952 bridge motor submissions and
15,064 additional zero-force submissions. For each bridge request, the next
additional zero request follows at:

| Minimum | Median | 95th percentile | Maximum |
| ---: | ---: | ---: | ---: |
| 0 | 962 | 2,169 | 3,184 microseconds |

The additional payload is:

```text
31 01 e0 fe 00 00 01 e0 fe 00 00
```

The existing motor encoder identifies its two force magnitudes as zero
(`src/jog/model.rs:58-75`). The bridge's window contains only its positive
right-force payloads for 10,630 and 20,000. The additional zero payload is
absent from that write trace. Fresh B also has 2,842 ring requests absent from
the recorded bridge ring payloads.

This is evidence of rapidly interleaved conflicting commands. Submission order
and USB success do not directly measure the force applied by firmware; the
mechanical explanation remains an inference supported by the operator's
struggling-motor and flickering-ring observations.

## Source investigation

The inspected GUI source does not acquire NI merely because the stopped GUI
opens or the saved screen flag is enabled:

- `windows/bridge-app/Core/HostController.cs:40-65` constructs stores and
  managers. Start performs launch preparation at lines 67-95.
- `Platform/NIServiceControl.cs:12-32` reads a packaged access declaration.
- `Core/ScreenCoordinator.cs:75-96` saves the screen preference; an ON command
  requires a bridge child.
- `Core/LaunchSpec.cs:25-35` forces acquisition for diagnostic B, independently
  of the saved flag.

The runtime releases an existing transport before processing a new screen
request (`src/screens/worker/ni/lifecycle.rs:174-196`). Acquired-only
presentation returns before image publication
(`lifecycle/presentation.rs:43-50`). The worker cannot register with NI before
ARM; STOP drops its service session
(`windows/encdr-screen-worker/src/session.rs:36-50,69-78`). The supervisor waits
for worker exit and terminates/reaps its job before acknowledging completion
(`windows/screen-supervisor/Session.cs:120-159`).

The pinned Encdr client registers device and instance request/notification
pipes, sends eleven setup messages, and exposes active-instance notifications.
Without image frames, the inspected path retains those registrations and
acknowledges notifications. It does not demonstrate a periodic HID motor/ring
writer. Request-pipe handles close on drop, but the private service/driver
behavior is outside this client source.

No documented display-only registration or selective motor/LED relinquish
operation was found in that inspected protocol. The setup words' exact S4
semantics remain unresolved. Source references:

- `https://raw.githubusercontent.com/RufusIbiza/Encdr/a5414658a69c4330d10b95f2359371dfc8550326/encdr/src/usb/ni_service/windows.rs`
- `https://raw.githubusercontent.com/RufusIbiza/Encdr/a5414658a69c4330d10b95f2359371dfc8550326/encdr/src/usb/ni_service/protocol.rs`

The retained native helper source-to-binary binding was explicitly candidate,
not independently verified; this limit remains. See the earlier source
analysis at `.omo/evidence/s4-acquisition-source-track-20261116.md`.
A source-level cleanup path is not proof of the NI service's internal recovery.

## Remaining discriminator

The next missing evidence is the origin of the exact zero-force and extra-ring
requests: owning PID/thread and module or driver call stack, bound to the
payload/IRP rather than inferred from general CPU activity.

One later authorized registration/release observation should correlate that
writer identity with client ARM, registration/control grant, worker exit and
post-release traffic. It must distinguish a user-mode NI writer, a driver-side
output loop, and a surviving helper/client. Existing PCAPs do not carry this
attribution, and discarded helper stderr cannot supply it retrospectively.
Bounded read-only searches found no `.log` files in the inspected NI
ProgramData, LocalAppData and installed Hardware directories, nor `*NI*.log`
in the user's temporary directory. Logging was not enabled or changed.
This is an evidence requirement, not a prepared runbook or authorization for
another test.

Identify the owner of the additional requests before choosing a correction;
the NI registration/release path is the leading candidate. Do not compensate
by increasing force or write rate, guess which private
setup message to omit, or treat another image case as the missing proof.

## Evidence retained

Compact data, exact capture metadata, the two host logs and method descriptions
are under `.omo/evidence/s4-usb-output-investigation-20261122-9acf7e36/`.
`timeline-and-interleaving.json` records the timeline, hashes, complete trace
footers and timing distributions; `hypotheses-and-cross-checks.json` records
the competing explanations and successful USB completion checks.

All original recordings remain unchanged on the laptop. The known USBPcap
ten-second normal-stop failure remains separate and unrepaired; forced
termination still prevents complete-tail or zero-USB-loss claims.
The owned local raw-file staging and root debug journal were removed after
retaining the useful results and a journal copy. No remote staging was created.
