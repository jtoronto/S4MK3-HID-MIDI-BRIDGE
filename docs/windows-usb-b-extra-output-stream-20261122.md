# B reproduces the problem with an additional USB output stream

Fresh B completed on November 21 local time / November 22, 2026 UTC. The user
reported motors "struggling to move", flickering rings and blank screens.
The bridge trace confirms one NI screen-session acquisition and zero image
publications. Blank screens are expected in this case.

The USB recording reveals an additional motor-command stream absent from the
bridge's loss-free write trace. Interference from these commands is now a
concrete explanation to investigate. The owning process and exact onset remain
unproven; this is diagnostic evidence, not a product fix.

## Runs and procedure

- Reference A: kit 3 `usb-captures\20261121-192709-cb34ac98\A`. This is the
  fresh, physically normal A, not the earlier duration-expired attempt.
- Fresh B: kit 3 `usb-captures\20261121-201419-B-7ebe06e2\B`.
- Mistimed B: `usb-captures\20261121-200120-B-c9b336cc\B`, preserved and excluded.
  The user reported clicking Start before pressing Enter; no authorized
  observation interval was established in that attempt.

The user requested the retry. It used the same 515-file app as A and unchanged
motor-tuning preferences. The user had enabled screens during mistimed B.
The initial retry preflight incorrectly rejected that toggle. Only the
temporary B caller was corrected: it allows the screen-enabled flag to differ,
while checking every other screen field and all other saved preference hashes.
Native checks exercised the actual guard against the live configuration and
isolated enabled-only/other-field changes. User preferences were not rewritten.

The original audio driver and agreed 303 reload procedure were retained.
Mode switching and an eject control were not required. The real current-worker
Loaded/Playing/Internal/right-TT authorization gate passed in 28.985 seconds.
The hands-off observation lasted 30.130 seconds; Stop and Quit were confirmed.
No A repeat or C/D run occurred.

## Observation-window USB comparison

Both exact-hash PCAPs span their entire observation intervals. Rates below use
the UTC observation markers; their spans differ slightly from the stopwatch
durations because writing the markers adds overhead.

| Measurement | Normal A | Problematic B |
| --- | ---: | ---: |
| Operator's observation, seconds | 29.660 | 30.130 |
| Target USB packets in window | 48,534 | 83,356 |
| Motor requests in window | 2,174 | 17,016 |
| Motor requests per second | 73.27 | 564.41 |
| Ring requests in window | 948 | 3,189 |
| Ring requests per second | 31.95 | 105.78 |
| Largest target-packet gap, milliseconds | 2.912 | 2.405 |

Within B's window, 1,952 motor requests match the bridge's recorded motor
writes. The other 15,064 carry this 11-byte report:

```text
31 01 e0 fe 00 00 01 e0 fe 00 00
```

At approximately 499.67 requests per second, this commands zero force for both
wheels according to the existing `src/jog/model.rs` layout. That payload is
absent from the bridge's write trace during the window and from A's observation.
The bridge's own two motor payloads account for the remaining 1,952 requests.
B also contains 2,842 ring requests whose payloads are absent from the bridge's
347 recorded ring writes during the observation.

Across the full B PCAP, all 43,636 submissions of the additional motor payload
have matched successful USB completions; none failed or remained unmatched.
This establishes actual USB-stack transactions, not physical motor compliance
or process identity.

The stream is already present at 01:21:25.801 UTC, near the start of USB
recording and 23.475 seconds before the fresh worker's screen-ready event.
Its last request is at 01:22:53.131, during screen release; none follow the
successful release end at 01:22:53.160. Therefore this recording cannot establish
that the fresh acquisition initiated it. Earlier-attempt state, stopped-GUI
initialization and the underlying NI ownership/lifecycle remain possibilities.
USBPcap does not identify the writer PID here.

## Completion, preservation and limits

The bridge footer is complete, runtime_ok=true, with zero losses: 61,474 output,
37,377 input and five screen entries. Acquisition and release succeeded; no
bridge image publication occurred. This reproduces the problem without bridge
image publication.

USB normal Stop again timed out after ten seconds and forced termination.
The PCAP is structurally valid, but complete-tail and zero-USB-loss claims
remain unsupported. This stop defect is separate from the additional output
stream observed throughout the working interval.

All 13 Djay mappings remained unchanged. Among the 24 files snapshotted before
fresh B, only the normal last-session log changed. The user's enabled-screen
preference remains preserved. Full recordings, originals and frozen App copies
remain in their desktop run directories. Small receipts, native guard proof,
the exact caller and the comparison are retained under
`.omo/evidence/s4-usb-b-20261122-01a1dbd8/`.

The owned Windows temporary caller/fixture directory and macOS analysis staging
were removed after evidence retention. Post-run inspection found no bridge,
USBPcap, screen-supervisor or screen-worker process. Desktop recordings and
their frozen App copies are intentionally retained for further offline work.

PCAP SHA-256:

- A: `0678682632f52e864918a12b523029cc31f8792028ab5228320362f88ba52943`
- B: `054f3c89be32995c223f8932a8db181dcac718c6a907bccfc7bc4bf9cf11a1e6`

No product source, service, driver or registry repair was made. The next
investigation should identify the additional output's owner and lifecycle
before choosing a correction; this result does not authorize another run.
