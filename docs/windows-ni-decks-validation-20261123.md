# Windows NI decks and direct HID validation

November 23, 2026. This records the accepted behavior of the attended
development trials, not a general Windows release qualification.

## From diagnosis to two output modes

Acquiring the Windows NI screen session while the bridge still wrote
native motor and ring reports produced struggling platters and flickering
rings. The [writer trace](windows-writer-attribution-attended-validation-20261122.md)
attributed additional motor and ring submissions to NIHardwareService.
Those submissions are evidence of a second writer, not USB completion
or measured torque. Stock Traktor evidence then showed the two screens
and jog control coexisting in one NI session. The standalone left-jog
diagnostic confirmed the approach on hardware; a ring position alone
was not enough, but a lit color and needle mode restored illumination.
The diagnostic remains a separate tool.

The opt-in full bridge trial uses one acquired NI session for both
screens, motor control and jog rings, and disables its own native
motor and ring writes in that mode. It still reads the S4 over HID and
uses the existing BLOOP mapping for Djay. Play and motor authorization
depend on real selected-deck feedback and fresh controller input.
The ordinary bridge retains its direct-HID motor and ring path.

| Trial | Attended evidence |
| --- | --- |
| NI r3 to r5, KENSUKE | Independent screen updates and JOG playback ring animation were physically confirmed after correcting display framing and restoring NI's stock JOG playback signal. The operator separately confirmed that r5 resumes TT drive after a released backspin settles. Two Djay-first r5 logs showed both selected decks loaded and Internal, with one paused and the other playing, before controller input. |
| Direct HID r5, ASUKA | Windows installed NI device drivers automatically, but NIHardwareService was not installed. With screens off, HID input and MIDI translation worked. Enabling Djay PRO resolved the initial lack of MIDI control and feedback, and the operator confirmed that direct-HID motors worked. This was not a no-NI-driver test and did not test displays or S4 audio. Djay-first startup feedback could still be missed. |
| Direct HID r6, ASUKA | The operator reported two successful Djay-first starts. One lossless capture independently contained both decks loaded, Internal, not THRU, and paused before any bridge MIDI output attempt. Both decks' state arrived by 99.361 ms after capture start; the first output attempt was at 432.718 ms. The second pass is operator-confirmed; the other exported MIDI file was from r5. |

The TT recovery fix lets NI resume forward drive once fresh platter
motion is near rest or forward and no touch or wheel action owns the
platter. The scratch gesture keeps ownership until the normal forward
handoff. This changes the drive gate, not TT force or the interpretation
of Djay state. The intermittent crossfader motion was not reproduced;
code inspection found startup assignment/curve sync but no programmed
position sweep. No crossfader fix was made.

## Remaining Djay-first startup race

The Windows host creates and publishes the BLOOP endpoint before the
worker can open that endpoint's receive connection. Djay may send its
initial state as soon as it discovers the endpoint. If it sends that
refresh before the listener is attached, the bridge has no verified way
to replay it or request another one. Endpoint visibility, MIDI
connection readiness and actual deck feedback are separate milestones.
Loopback echoes of bridge commands do not establish Djay feedback.

r5 moved the receive-only connection ahead of controller discovery,
ownership checks and HID setup in NI mode. r6 applies that same ordering
to **both** full-profile BLOOP modes. The worker passes the open
connection and its queued feedback to the full bridge instead of
reopening it. MIDI output still opens at the original point. The
mapping, other profiles and backends, and motor authorization did not
change. This removes controller setup from the vulnerable window, not
the gap between endpoint publication and listener attachment.

If startup state is missing, leave TT off, save the bridge log and raw
MIDI capture, and fully quit and reopen Djay with the bridge ready.
Do not infer deck state from a successful MIDI open. For a repeatable
comparison, use two loaded Internal decks with opposite Play states,
keep screens and TT off in direct mode, and record two starts without
touching any controls. See the [direct-HID runbook](../windows/qa/ni-bridge/ASUKA-STARTUP-TEST.md).

## Verification and next boundary

The listener-policy regression failed with the former NI-only gate
and passed with the full-BLOOP gate. Rust tests, strict Clippy, the Swift
suite and the Windows x64 cross-build passed. The r6 archive reused the
r5 host and the other 238 runtime files; all 239 manifest entries were
verified and the regenerated mapping matched byte for byte. Full
receipts, hashes and packet evidence are retained locally under
`.omo/evidence/s4-ni-decks-trial-20261123/`. The archives and raw captures
are not source artifacts.

In NI mode, NI renders force and JOG pitch-bend detents; the bridge's
native tension and TT brake settings do not tune either. The trial sends
the stock nudge tick value of 12 in JOG and 0 in TT; the exact angular
origin and physical count are unverified. The separate
[NI service investigation](windows-ni-motor-service-investigation-20261123.md)
identifies candidate service controls but no verified TT braking-force
setter. NI force tuning is separate work and must keep NI as the sole
motor and ring writer.
