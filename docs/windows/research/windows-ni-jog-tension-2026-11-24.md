# JOG rim tension through NI: stock path and Windows evidence gap

November 24, 2026. **There is a specific stock path worth observing on
Windows: the global unsigned property `JOGT`, not a per-wheel motor command.**
In the installed macOS binaries the caller can serialize it and the service
maps it to both JOG damping renderers. This is static evidence of how stock
tension can work, not a verified Windows bridge setter. No tension command
was sent and no running NI software, Windows machine or hardware was accessed.

**Subsequent Windows result:** the [KENSUKE stock capture](#kensuke-stock-capture)
confirmed the predicted request and values on the service pipe. The offline
findings and comparison plan below describe the state before that run.

## Stock setting → service message

The [S4 MK3 manual](../../../reference/TRAKTOR_KONTROL_S4_MK3_Manual_English_0719.pdf),
printed pp. 45 and 83, distinguishes resistance while turning (Jogwheel
Tension, 0–255; higher means more resistance) from discrete Ticks when
Nudging. Holding JOG and turning the wheel is a second way to change tension;
it is **not** the gesture to use to isolate a Preferences edit. Disabling top
touch sensitivity makes the JOG haptic preferences inapplicable according to
the manual. Leave sensitivity enabled and the top plate untouched.

The local stock app is `Traktor Pro 4.app`, 4.4.1 build 22, executable SHA-256
`cdc051995be78f6cfa88e3da1ccc7eee0bd6a982e27351177140bc184fe3ecb6`.
These are static, unslid virtual addresses; architecture matters:

| Link | Reproducible anchor |
| --- | --- |
| Preference → surface pin | `Contents/Resources/qml/CSI/S4MK3/S4MK3.qml:26–27` declares `mapping.settings.haptic.tension` as integer, default 50, range 0–255. A `Wire` from `s4mk3.jogwheel.tension` uses a `ValuePropertyAdapter` on that path and ignores `PinEvent.WireEnabled`. It is global, not `left` or `right`. The arm64 preferences constructor references the same path for the integer selector/value edit at `0x100872b14` and `0x100872c08`. The QML alone does not specify a packet or a startup write. |
| Pin → hardware property ID | x86_64 `S4MK3PortDescriptor` loads the literal `jogwheel.tension` at `0x1002a1fdc`, then passes `0x4a4f4754` (`JOGT`) and access argument 2 to `NHLPortDescriptor::intProperty` at `0x1002a2005`. This is a property **key**, not the IPC command ID. |
| Changed integer → pending output | arm64: the property registration connects the surface and port property (`0x100bd3274`, call at `0x100bd33a4`). `NHLProperty<int>::setImpl` (`0x1002be57c`) writes the ID/value to the port cache. `handleOutputUpdate` calls `sendPendingUIntProperties` (`0x100bb71b4`); its callback `0x100bc49e8` dispatches through slot `+0xd8`. A binding or cache update need not transmit immediately or at all if unchanged. |
| S4 output → serialized request | x86_64 S4 port vtable `0x1058af120`, address point +16, slot `+0xd8` resolves to `writeUIntProperty` at `0x1002c1ba0`. It jumps to `Controller::setUIntProperty` (`0x100de6240`). The constant at `0x104360330` contains wrapper words `12, 16, 0` and command `0x03567375`; the function appends the property key and unsigned value. Its `+0xf0` dispatch resolves to `sendServerMessage` on the S4 controller vtable `0x10598d760`. `IPCConnection::sendMessage` takes the body at wrapper +12 (`0x100dd0ef9`) and the request length from the wrapper. |

Thus the **stock macOS request body** for this property is three little-endian
32-bit words: **`0x03567375`, `0x4a4f4754`, setting**. No deck index, float
bits, or pair of force values. The 12-byte wrapper prefix is not transmitted
as part of that body. This identifies what to look for in stock traffic; it
is not a request to replay it or a claim to have captured a Windows write.

## Service message → renderer contribution

The local macOS service-side image is `NIHardwareAgent.app/Contents/MacOS/NIHardwareAgent`
under `/Library/Application Support/Native Instruments/Hardware`, version
2.8.2 (R0), SHA-256 `4374d2aceeb68ee210c5f25f87661e3e6f9d825b159eb30c315ac2edcc44b6fa`.
In its x86_64 `SERVER::ControllerBase::onClientRequest`, the branch at
`0x100029b1b` recognizes low selector `0x567375`, takes the two words after
the command and invokes the property handler. The following anchors are from
the **arm64** slice of the same image:

1. `KontrolS4MK3::Device::setUIntProperty` compares against `JOGT` at
   `0x1001c3fd4`. Its matching branch stores the unsigned value, maps the
   0–255 UI range linearly to approximately −0.09…1.10 and calls
   `MotorJogwheelControl::setJogwheelTension` at `0x1001c408c`.
2. The latter clamps the float and stores it to motor-control `+0xf7c` and
   `+0x1ecc` (`0x1001ae96c`, `0x1001ae980`). These are not just adjacent
   anonymous scalars: the two wheel bases are `+0x40` and `+0xf90`; each
   contains `JogRenderer` at `+0x898`, containing `DampingRenderer` at
   `+0x660`. In each case the store lands at damping target `+0x44`.
3. `DampingRenderer::processImpl` (`0x1001aff98`) moves its smoothed value
   2% of the remaining distance toward that target per update, multiplies
   by the negated kinematic input at `+0xc` and clamps to [−1, 1]. This is
   a motion-dependent contribution, not a fixed force corresponding to the
   slider number. Physical units are not established.
4. `JogRenderer::processImpl` invokes the damping renderer and adds its
   result to the selected contribution (`0x1001af69c–0x1001af6d4`).
   Another branch can replace the result before clamping. The parent
   selects JOG versus TT (`0x1001aec88–0x1001aeca4`); `processMotorJogwheels`
   obtains both outputs and calls `sendDirectControlReport` at `0x1001ada5c`.
   This establishes a route to an NI output, not its magnitude on a particular
   nudge. The separate JOG `TicksRenderer` is at JOG `+0x380`; the TT renderer
   is at wheel `+0x278`. The identified damping stores do not target either.

## What the Windows evidence does and does not establish

- The [Windows service investigation](windows-ni-motor-service-investigation-20261123.md)
  identifies the Report 3 → per-side renderer → `0x31` encoder path in
  `NIHardwareService.exe`, hash `88f90c4b378f547508b09f190b4bcb30a35cbc5f5d16a7a0d533267a2c1956d9`.
  Historical follow-up analysis of that image recorded a `JOGT` change
  notification from `0x140248b40` and a getter at `0x140240e10` mapping
  motor-state `+0x1ff0` back to 0–255. **Readback is the opposite direction
  from a setter.** This corroborates the key and range but is not a fresh
  disassembly of the Windows write branch. Do not port Mac offsets to the PE.
- The old `.omo/evidence` service exports, caller snapshots and IPC run
  directories are absent from this checkout (clean `windows` at `029eabe`
  before this note). The historical four-run review (441 selected requests)
  identified no numeric tension write. The surviving `dist/S4 Traktor IPC
  Trace-20261123-178b94.zip` is **observer software**, not those recordings.
  Its `traktor-ipc.js:54–56` accepts `0x036a…`, display, and enumerated
  lifecycle IDs. **It drops `0x03567375`** and does not log incoming property
  payloads. Re-running it unchanged would not test the proposed message.
  `wireshark_captures/traktor_usbcap_tension.pcapng` survives (SHA-256
  `0816488b42a33fd382aa60719077eda0e4954fbbeafdb390ff8ca54dbe65089f`);
  it is a separate Windows USBPcap stream. The October 8 session notes
  describe stock settings 0, 127 and 255 and interpret successive `0x31`
  patterns as assistance, braking and stronger braking. The edits were not
  timestamped: that setting-to-segment assignment is retrospective, not a
  synchronized record of preference edits, pipe messages, writer identity
  and physical feel.
- `S4MK3Deck.qml:119–121` wires `.play` to `.jogwheel.motor_on`; in JOG this
  informs NI of playback and can change detent force as well as ring motion.
  The [stock reference procedure](../../../windows/qa/ni-bridge/STOCK-TRAKTOR-CHECK.md)
  is unexecuted and varies playback, not tension. `0x036a7341` is a Boolean
  adjustment-mode lead, not a resistance value. `0x036a7378` configures nudge
  ticks; `fb2694b` changes JOG ticks from 12 to 0, not damping. Neither the
  unresolved per-side floats nor the speed/RPM setters establish TT braking.
- The [attended writer attribution](../validation/windows-writer-attribution-attended-validation-20261122.md)
  found **both** NI and bridge motor/ring submissions in an acquired session;
  the operator reported chatter and flicker. NI's pending zero-magnitude
  writes were not torque measurements, and that run did not publish images.
  Separately, [NI Decks validation](../validation/windows-ni-decks-validation-20261123.md)
  confirmed both screens updating and JOG ring animation in the NI-only
  motor/ring configuration. The [worker](../../../src/screens/worker/ni_decks.rs)
  publishes pairs and applies deck changes through one acquired session.
  The [deck commands](../../../src/screens/ni_service/decks.rs) include mode,
  ticks, timeline, ring, speed and playback, but not `JOGT`. The Direct HID
  tension slider tunes a different writer. Do not re-enable it in NI mode.

**Decision:** retain the existing one-session ownership model. The stock
property is a concrete candidate compatible with that design. Neither stock
Windows traffic for a controlled tension edit, nor acceptance and physical
effect from the bridge's acquired instance with continuous paired screens,
has been demonstrated. No product change follows from the static result.

## Smallest discriminating Windows comparison (not run)

Only with **separate authorization and confirmation that the operator is
watching**. Stock Traktor is the only client; leave NI as the motor/ring
writer. No bridge, second screen client, direct HID output or injected
requests. Before any collection, establish that the observation can retain the
generic property request, not just the jog family. Use a common clock and
record Traktor/service identities, initial prefs and both screen states.

1. Use one loaded, paused deck in ordinary touch-sensitive JOG throughout;
   keep the other paused and untouched. Set *Ticks when Nudging* to *None*
   once; keep hotcue/timeline state, track position, focus and other prefs
   fixed. No Play, TT, Shift, JOG-hold, top touch or cue crossing.
2. Select two distinct **stock UI** values A and B (use the original as A
   when practical). Timestamp each edit while hands are off. At each settled
   value make one light rim-only nudge on the same wheel, in the same
   direction with comparable speed and travel from rest; release fully.
   Restore A/original and all prefs. A repeat at restored A is the smallest
   reversal if B differs or the first gestures were mismatched. Do not infer
   resistance from the idle interval. Stop on unexpected motion, chatter,
   screen loss or uncertain ownership; confirm both wheels stopped.
3. Keep three records separate. **IPC:** identify the outgoing instance
   request and key/value at each change; distinguish it from readback,
   submission status and any actual reply. **Output:** attribute `0x31`
   signed-magnitude submissions to NI, comparing matched wheel motion where
   available; record ring and both screen continuity separately. A pending
   write is not USB completion or applied torque. **Feel:** record the
   operator's comparison of continuous resistance, separately from clicks
   or ring movement. Screen headers alone do not prove two visible screens.

A matching edit/request establishes the Windows caller link. An NI output
difference at comparable motion supports the renderer effect; a physical
contrast answers the feel question but is not calibrated torque. Missing
messages with incomplete coverage, unequal nudges or dropped samples are
inconclusive. Even a positive stock comparison is not bridge acceptance.

## KENSUKE stock capture

November 24, 2026 (UTC), stock Windows Traktor Pro 4.4.2 build 158, PID 18320.
Run `d18f1353-40d6-42ca-bd81-413d5e3d1137`, `trace.jsonl` SHA-256
`b7f30aed24c8d26c88c46427eca11dd522d39115a9bead9e1cf51f482d558c9a`.
All 57 tension `NtWriteFile` submissions used handle `0x384c` to the recorded
NIHardwareService PID 4984. All had complete 12-byte bodies: little-endian
command `0x03567375`, key `0x4a4f4754`, unsigned slider value. Every
immediate write return was zero. No service reply payload was captured.

| Operator-reported edit | Last request in drag (UTC) | Seq | Value bytes | Actual trace phase |
| --- | --- | ---: | --- | --- |
| A = 25 | 02:27:02.911 | 81 | `19 00 00 00` | `READY_IDLE` |
| B = 244 | 02:27:20.921 | 90 | `f4 00 00 00` | `TENSION_A` |
| Restore original 107 | 02:28:17.649 | 133 | `6b 00 00 00` | `TENSION_B` |

The drag endpoints match the reported **order and values**, not the phase
names: each endpoint preceded its intended marker. The 107 submission
establishes the restore request, not an independent startup read of 107.
Both display IDs (0 and 1) were sampled on the same handle in each marked
phase. The operator reported timeout at quit; the saved trace lacks `QUIT`
and a stop footer, so no teardown conclusion. No rerun is needed to identify
the stock Windows setter. This records client IPC, not NI `0x31` output,
physical resistance, visible screen continuity or bridge-session acceptance.

## Bridge trial boundary

The opt-in [paused-JOG bridge trial](../../../windows/qa/ni-jogt/ATTENDED-RUN.md)
uses the acquired NI Decks instance for the three stock values, without a
second NI client or bridge-native motor/ring output. The operator controls
when each setting is requested. A successful `NI_JOGT_TRIAL_SUBMITTED`
receipt is local transport evidence only. The unqualified API return of a
Windows pipe write is not an NI reply or proof of the renderer effect.
Do not count the trial as physically accepted without the attended A/B,
screen and stop observations.

**Attended result, KENSUKE, November 24, 2026:** the operator reported a
notable difference in rim resistance between A (25), B (244) and R (107),
with a feel essentially the same as Traktor. This is physical evidence of
adjustable JOG resistance on this rig, not a measured torque curve.
In run `40cb6f36607e4e69a72df21f040759b1`, the operator journal records
all three presets and host exit. The retained part of the bridge log
independently records submissions of all three values and a shutdown restore
submission of 107 after the last manual 25. Those receipts are not NI reply
payloads or a separate motor-output trace. The operator confirmed that both
S4 screens continued updating normally and the rings stayed steady while
tension changed. Physical stop was not separately reported. The operator
could not reliably follow trial status in the scrolling Diagnostics log;
future controls should show the last submitted value at the point of input.
