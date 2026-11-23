# NI service motor and output ownership investigation

Date: November 23, 2026.

The exact NIHardwareService binary from the writer-attribution run contains an
S4-specific jogwheel IPC command handler. It changes the same motor/haptic and
ring state that feeds the observed output writer. This is a stronger lead than
trying to bypass NI while retaining its screen session: configure NI's own
renderer and let NI remain the motor/ring writer.

There is no verified arbitrary force/torque setter or screen-preserving release
procedure from this investigation. The recovered commands are static findings,
not a tested API. Their acceptance through the complete external transport,
parameter meanings and physical effects still need confirmation.

## Scope and evidence

This investigation used local retained artifacts, public upstream source and
offline Ghidra analysis. It made no Windows, service, driver, registry, MIDI,
HID, USB, controller or live-process attachment calls. No product code,
dependency, installed service or controller state was changed.

The retained executable is 14,028,000 bytes, PE32+ x86-64, image base
`0x140000000`, SHA-256:

```text
88f90c4b378f547508b09f190b4bcb30a35cbc5f5d16a7a0d533267a2c1956d9
```

This matches the executable recorded in
[the attended writer-attribution validation](windows-writer-attribution-attended-validation-20261122.md).
That run attributed 32,245 exact zero-force motor submissions and 6,101 ring
submissions to NIHardwareService, including motor-stack RVAs `0x238a13`,
`0x25d6ff`, `0x25add9`, `0x248cc4`, `0x237194` and `0x228d1f`.
The motor submissions span 64.477558 seconds, approximately 500.081 Hz.
Those are API submissions, not established USB completions or applied torque.
The run had operator deviations and incomplete collector validation; the
earlier record preserves those limits.

Ghidra 12.2 analyzed 18,173 functions. A portable ARM64 JDK and a locally built
copy of Ghidra's shipped native decompiler were used. The PE was never executed.
Useful exports, scripts, source snapshots and hashes are retained under:

```text
.omo/evidence/s4-ni-motor-service-20261123-63c9217a/
```

That directory is in the main checkout. Addresses below are preferred-image
virtual addresses; subtract `0x140000000` for module-relative offsets.
`FUN_...` names are generated analysis labels, not recovered NI source names.
Class names explicitly identified as RTTI come from binary metadata.

## What actually produces the competing reports

The retained stack resolves to an input-driven haptic pipeline:

| Address | Static finding |
| --- | --- |
| `0x140236e10` | HID input read loop, including overlapped read completion and callback dispatch. |
| `0x140248b40` | S4 input dispatch. Report ID 3 with sufficient payload length reaches the jogwheel update unless a server-owned predicate returns true. |
| `0x14025aa30` | Updates jogwheel snapshots. When both state fields at `+0x2040` and `+0x2088` are less than 4, invokes each side's force renderer, updates rings and emits the motor report. |
| `0x14025d5b0` | Clamps each force to `[-1, 1]`, converts magnitude using `abs(force) * 60000`, encodes report `0x31` and calls the output callback. |
| `0x140238950` | Generic HID write. Pads to the output-report capacity, calls `WriteFile`, and handles pending completion with `GetOverlappedResult(..., TRUE)`. |

The motor call is:

```text
motor_report(state, left_rendered_force, 1, right_rendered_force, 1)
```

The two renderers are constructed at state offsets `+0x80` and `+0x1040`.
RTTI identifies their implementation as `HapticDriveJogWheelRenderer`.
The constructor also identifies `TurntableRenderer` and other embedded
renderers. The force update runs before the report encoder; the service is
not merely forwarding a prebuilt client motor packet.

The encoder uses its reverse sign words when force is exactly zero because
the positive branch tests `force > 0`. Thus:

```text
31 01 e0 fe 00 00 01 e0 fe 00 00
```

does not by itself establish active reverse torque or braking. Its magnitude
is zero. The native pending returns in the capture also do not establish final
device completion. The generic write routine's completion handling explains
why a lower-level hook can observe pending submission even when the caller
subsequently waits.

The static path supports input-report-driven output. It does not establish an
independent 500 Hz timer, deliberate opposition to bridge commands, or the
controller's arbitration behavior.

## Recovered S4 jogwheel command family

The S4 server vtable at `0x1405857f8` points to `0x140119180` in slot `+0x210`.
The common dispatcher, `0x140116320`, routes the `j` command family there.
The S4 handler masks the message selector to its low 24 bits, reads a side
word, and passes parameters into the underlying device's motor state at
`device + 0x248`.

The table gives candidate full command IDs using the `0x03` prefix employed
by the existing NI protocol. The native handler proves the low-24-bit
selectors and downstream operations. This investigation did not independently
trace the outer transport's validation of every candidate full ID.

Scalar candidate wire bodies follow the existing protocol's little-endian
word convention:

```text
u32 command, u32 side, u32 argument_bits
```

The in-memory message wrapper puts those fields at `+0x0c`, `+0x10`, `+0x14`;
those wrapper offsets are not extra wire headers to send. In the recovered
setters, side zero selects the left state and nonzero selects the right.
This is not a recommendation to use arbitrary nonzero side values.
Boolean arguments compare against `0x74727565` (`true`), rather than integer 1.
Float arguments are IEEE-754 binary32 bits, confirmed by `MOVSS XMM2` at the
dispatch call sites.

Offsets in this table are relative to the motor-state object. Right-side
renderer fields usually add `0xfc0`; ring state has its own layout.
Descriptions distinguish observed assignments from inferred meanings.

| Candidate ID | Argument / target | Proven local operation |
| --- | --- | --- |
| `0x036a734d` | word; `0x14025c8c0` | Writes left `+0xcc` / right `+0x108c`, the renderer's `+0x4c` mode selector. |
| `0x036a736d` | Boolean; `0x14025c8e0` | Changes left `+0xc8` / right `+0x1088` and related renderer flags; resets two subordinate states when changed. Candidate enable/play-state control, not an output-writer release. |
| `0x036a7373` | float; `0x14025c790` | Stores value/60 at left `+0x678`, raw value at `+0x6c8`, and `33.333332/value` at `+0x7c8` (1 when value is zero). Speed-like configuration; units are not proven. |
| `0x036a7353` | float; `0x14025c950` | Stores value/60 at left `+0x67c`, which `0x036a5800` subsequently samples. |
| `0x036a7370` | float; `0x14025cb20` | Updates two renderer scalar values and synchronizes associated cached values when their difference exceeds 0.05. Position-like state is an inference. |
| `0x036a7374` | float register argument; `0x14025cbb0` | Writes the raw 32 bits to left `+0x6cc` / right `+0x168c`. Meaning unresolved. |
| `0x036a7378` | word; `0x14025c800` | Writes left `+0xd08` / right `+0x1cc8`. Meaning unresolved. |
| `0x036a7372` | Boolean; `0x14025caf0` | Writes two subordinate renderer flags. Meaning unresolved. |
| `0x036a7354` | Boolean; `0x14025ca00` | Writes two other subordinate renderer flags. Meaning unresolved. |
| `0x036a7341` | Boolean; `0x14025c730` | Selects/clears one side's `+0xfb8` renderer flag and the shared selected-side field at `+0x20a0`. |
| `0x036a7343` | low argument byte; `0x14025c820` | Changes ring state at left `+0x2048` / right `+0x2090` and can emit a ring update. |
| `0x036a734c` | word; `0x14025c840` | Maps argument 1 to ring value 2, argument 2 to 3, otherwise 0; can emit a ring update. |
| `0x036a7365` | structured; `0x14025ca30` | Copies a vector of 12-byte entries into two subordinate renderer containers. Entry semantics and complete wire schema unresolved. |
| `0x036a5800` | side only; `0x14025d8f0` | Sets a subordinate state to 2 and copies the value configured by `0x036a7353` to another renderer field. |

These commands establish a service-side motor/ring configuration path. They
do not establish a raw signed force setter. None of these recovered handler
branches directly accepts two per-sample force values and calls the motor
encoder. This bounded negative finding is not proof that no other command
exists elsewhere in NI's protocol.

The renderer at `0x140259fa0` selects subordinate results using its mode
field: values 1 and 2 select different renderers; the other recovered cases
return zero. It still feeds the report writer. Changing a renderer mode or
play flag therefore must not be confused with stopping NI's output stream.
No guessed mode or parameter was sent during this investigation.

Evidence: `native/exports-jog/140119180.txt`,
`native/exports-jog-setters/`, and the corrected float-argument export and
assembly in `native/exports-jog-paths/`. The original decompiler did not know
all mixed integer/float signatures; the assembly and corrected export take
precedence over its initial inferred parameter types.

## The output suppression gate and screen ownership

The S4 constructor, `0x14010a830`, binds a Boolean member callback at
`0x140114b80` and installs it with `0x14024cb80`. The device dispatch invokes
this callback before processing Report 3.

Its effective predicate is:

```text
server.active_client != null && server.active_role == 0x536c6570  // Slep
```

The fields are server `+0x178` and `+0x188`. This is not the unrelated button
state byte at `+0xcc2`, which was an early candidate and was ruled out.
`Slep` skips this Report 3 haptic/ring path.

RTTI and `0x140133940` identify a service-created S4 `SleepModeClient`.
The shared constructor `0x1400ed160` gives it `Srvc`, `SlpH`, `Slep`, and
the name `Sleep Mode`. This supplies a concrete internal source for that role.
It does not establish that external clients can register or select `Slep`.

The generic active-client transition, `0x140136740`, copies the selected
client's properties into these server fields. `prmy`, `scnd` and `Slep` map
to the `APP ` mode; `Midi` and `SMap` map to `MIDI`. Those labels alone do
not define selective ownership.

For display selector `0x647344`, the common dispatcher forwards the image
only when the requesting client equals the active client. The S4 display
handler at `0x140117fc0` then forwards to `HIDDisplayImplementation` when
the device is present and the display implementation exists.

Consequently, letting the internal sleep client become active is not an
established way for the bridge's separate client to keep publishing screens:
the bridge would fail that active-client identity test. Registering the
bridge itself with a sleep-like role remains unproven. The gate is useful
evidence about NI's lifecycle, not a ready screen-only ownership switch.

## Public implementations and version boundary

Both checkouts pin Encdr commit
`5d7a689ca5789161b3efd1b2049d7e54504fe881`, package version 0.7.0.
At the time of this investigation, latest release and upstream main were
v0.9.5, commit `86eb735576e2e427d576f981334bbf422e9f1890`, released
November 15, 2026. No dependency was upgraded.

The latest NI service module says screens use the service while inputs and
LEDs use OS HID. Its exposed session operations cover registration, control
events and display frames. It does not expose the recovered S4 jogwheel
commands. Its S4 hardware reference documents direct HID motor report
`0x31`; that does not limit what NI's private binary implements.

DrivenByKate commit `fe15693d7f6b276db37501ef9782bfd1c6ae1ceb`,
June 18, 2023, provides additional public protocol evidence:

- `setLedColors` sends `0x036c7500`, byte length, then raw LED color bytes.
- Acquisition uses `[0x03434300, 0x73747274]`; focus uses
  `[0x03434300, 0x75736572]`.
- Its optional MIDI/no-acquire path explicitly warns that displays break
  without agent changes. Its declared Kontrol IDs are not S4 IDs.

Thus public NI IPC LED updates do exist. The early public-source search's
broader negative impression was superseded by this source and, for S4
jogwheels, by the binary findings. LED colors are not arbitrary raw HID
report forwarding, and this Java implementation does not verify S4 motors.

Primary source locations:

- Encdr release: <https://github.com/RufusIbiza/Encdr/releases/tag/v0.9.5>
- Encdr service source:
  <https://github.com/RufusIbiza/Encdr/blob/86eb735576e2e427d576f981334bbf422e9f1890/encdr/src/usb/ni_service/mod.rs>
- Encdr S4 reference:
  <https://github.com/RufusIbiza/Encdr/blob/86eb735576e2e427d576f981334bbf422e9f1890/docs/hardware/ni_kontrol_s4_mk3.md>
- DrivenByKate:
  <https://github.com/ktemkin/DrivenByKate/blob/fe15693d7f6b276db37501ef9782bfd1c6ae1ceb/src/main/java/com/ktemkin/controller/ni/core/AbstractNIHostInterop.java>

Exact source snapshots are retained in the evidence directory.

## Assessment and bounded next check

The most promising route is to configure NI's existing jogwheel renderers,
including ring state, and make NI the sole producer for those reports while
the same acquired client publishes screens. The command handler and screen
handler coexist in the same S4 server. This avoids relying on an unverified
selective-release mode, but coexistence in code is not runtime acceptance.

Before choosing physical test values, recover a legitimate Traktor caller's
serialization and initialization sequence for the `0x036a...` family from
offline caller analysis or an already retained IPC trace. Resolve the mode
enum, float units/ranges, Boolean sequence, outer framing, and teardown order.
Do not infer these from command letters alone.

After that, a separately authorized attended check can use one acquired
client, NI as the only motor/ring writer, and screen publication. It should
observe command acceptance, emitted motor/ring values, screen continuity and
bounded stop/release behavior. A successful pipe reply alone is insufficient;
the current binary's Boolean success return is not physical verification.

If pursuing selective release instead, the distinguishing question is
whether an external client can become the active `Slep` client and still
pass image dispatch. That remains less supported than the recovered
jogwheel-configuration route. No release recipe or patch to NI is proposed
as a proven solution.

No physical behavior, screen continuity under these commands, arbitrary
torque control, new motor tuning or production integration was tested or
implemented here.
