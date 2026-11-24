# Serato supported-controller identity discovery probe

**Status: deferred, November 3, 2026.** Retained as a diagnostic, not a
production backend. The [investigation results](serato-investigation-results-20261103.md)
record the subsequently recovered native Starlight profile and USB-triggered
assembly path. Do not rerun the live sequence as part of this closeout.

This is a bounded MIDI discovery experiment, not a working hardware emulator
or a Serato backend. It presents temporary CoreMIDI source and destination
endpoints and records everything sent to the destination, including SysEx.
Capture mode sends no MIDI, including no Play, jog, touch or authentication
response. It does not open the S4, route audio, install a driver, change a
mapping or activate a license.

The initial candidate is Hercules **DJControl Starlight**. The endpoint name
comes from the upstream Mixxx mapping's controller ID. Its manufacturer and
model properties are set to `Hercules` and `DJControl Starlight`. This is a
candidate identity, not proof of the complete identity of the real controller
on macOS. A later native-profile recovery confirmed this MIDI name and
VID/PID; it did not establish the real USB interface descriptors.

The program is native Swift because CoreMIDI exposes endpoint manufacturer/
model properties directly. The existing Rust jog experiment is left intact.
No dependency or application build configuration is changed.

## What this can distinguish

- A Serato log entry establishes discovery of the temporary connection.
- Received bytes establish traffic to this destination. A complete SysEx
  request may expose a verification protocol, but an initialization or
  state-dump request is not automatically authentication.
- A change in Serato's supported-hardware state would be separate evidence of
  recognition. Discovery alone is not that evidence.
- Silence cannot establish that emulation is impossible. The missing
  prerequisite could be a different name, driver-owned MIDI entities, USB
  identity, an audio device or another native-profile condition.

Virtual endpoints do not have owning MIDI entities or USB device descriptors.
The diagnostic reports entity lookup status as well as the resulting entity
reference; on the tested macOS, no entity returns `kMIDIObjectNotFound`
(`-10842`). It does not manufacture an owning device or assign USB IDs.

## Build and transport verification

Run these commands in order:

```sh
cd /Users/joey/Development/S4MK3-HID-MIDI-BRIDGE/.slim/worktrees/serato-mode
swiftc -swift-version 6 -warnings-as-errors \
  examples/serato-identity-probe.swift \
  -o target/debug/examples/serato-identity-probe
./target/debug/examples/serato-identity-probe --self-test
```

The existing worktree's Cargo build creates `target/debug/examples`. If it is
absent in another checkout, create that directory before compiling.

Expected: `SERATO_IDENTITY_LOOPBACK_PASS`, `bytes_per_direction="609"`,
`directions="2"`, then `SERATO_IDENTITY_STOPPED` with
`client_dispose_status="0"` and exit status 0.

The self-test uses uniquely named endpoints, not the candidate name. It
subscribes both real CoreMIDI receive paths before sending a two-packet list:
a 603-byte noncommercial-manufacturer SysEx and six channel-message bytes.
The exact received byte streams must match in both directions within a
three-second failure deadline. This exercises packet lists and SysEx larger
than the C structure's nominal 256-byte data member, without sleeps or mock
delivery. It establishes transport only, not Serato acceptance.

## Live discovery test in execution order

1. Leave physical controllers and the normal S4 bridge out of this test.
   An existing synthetic jog probe can remain idle; do not send its commands.
2. Open Serato normally if it is not already running. Do not restart it,
   activate another license or change audio/mapping settings for this probe.
   Both decks should be unloaded or paused.
3. Note the current Serato log filename in `~/Music/_Serato_/Logs`. Preserve
   the existing log. The application adds its own discovery records there.
4. Start the default candidate:

   ```sh
   ./target/debug/examples/serato-identity-probe --seconds 45
   ```

   Expected: two `IDENTITY_ENDPOINT` records and `SERATO_IDENTITY_READY`.
   Both endpoints should report the selected name, `Hercules` manufacturer
   and `DJControl Starlight` model. An existing endpoint with the same name
   makes the command fail rather than shadowing it.
5. Watch the existing Serato window during the bounded capture. Do not play
   tracks or change settings. Record whether its MIDI/hardware state changes.
   Any `MIDI_RECEIVED` record contains the raw packet bytes and timestamp;
   retain the entire packet, including `f0`/`f7` when present.
6. Let the timer finish. Expected: `SERATO_IDENTITY_STOPPED` with disposal
   status 0 and received packet/byte totals. Ctrl-C also requests graceful
   cleanup. Client disposal closes both temporary endpoints.
7. Inspect the newly appended Serato log lines for the candidate name,
   connection URL, VID/PID, verification attempts, errors and disconnection.
   Correlate them with the probe's timestamps. Do not copy account/license
   records or the entire application log into a shared report.
8. If the first name produces no verification request, one explicitly
   labeled alias comparison can use the same candidate metadata:

   ```sh
   ./target/debug/examples/serato-identity-probe \
     --name "Hercules DJControl Starlight" --seconds 45
   ```

   This is a name hypothesis, not a second recovered profile. Repeat the
   observation, cleanup and log checks; keep the two results separate.

Do not fabricate a handshake reply from the control mapping. Do not interpret
an empty capture as a failed scratch command: this experiment sends no
scratch command and does not load the earlier jog XML.

## Sources and boundaries

- [Upstream Mixxx Starlight mapping, pinned revision](https://github.com/mixxxdj/mixxx/blob/bc49961fd3e5a46cd46588090c652a0d7ec44e18/res/controllers/Hercules%20DJControl%20Starlight.midi.xml):
  attributed to Hercules Technical Support and contributors; establishes the
  candidate name/control protocol, not Serato authentication.
- Apple SDK `MIDIServices.h`: `MIDIEndpointGetEntity`, virtual endpoint
  creation and manufacturer/model property contracts.
- Apple SDK CoreMIDI Swift interface: pointer-backed `unsafeSequence()` and
  `bytes()` packet iteration and `MIDIPacketList.Builder`.
- Installed Serato 5.0.0/build 5.0.0.1278: the later
  [native-profile and assembly findings](serato-investigation-results-20261103.md)
  recover this candidate's values and explain the examined USB trigger.

The MIDI callback does not identify the sending process. Without matching
application-log evidence, traffic cannot be attributed conclusively to Serato.
This probe does not establish a usable native jog protocol, audio routing,
four-deck operation, paid feature access or a complete production backend.

## Recorded result, November 3, 2026

Tested with the already-running Serato DJ Pro **5.0.0/build 5.0.0.1278**,
process 36809. No restart, setup-page interaction, account action, preset
installation or physical-controller access was performed. The existing idle
`S4 Serato Jog Test` process and endpoint were left running.

Times below are the local timestamps printed in Serato's log:
`~/Music/_Serato_/Logs/20261103-082940.36809.log`. Probe output uses Unix epoch
timestamps; its start and stop events agree with these connection windows.

| Endpoint name | Capture interval | Serato connection VID:PID | Received traffic | Outcome |
| --- | --- | --- | --- | --- |
| `DJControl Starlight` | 09:12:23-09:13:08, 45 seconds | `06f8:b128` | 0 packets, 0 bytes | Discovered as a duplex connection; no observed verification request or native-device-added event |
| `DJControl Starlight` | 09:13:15-09:13:16, additional one-second CLI run | `06f8:b128` | 0 packets, 0 bytes | Same classification; clean timeout cleanup |
| `Hercules DJControl Starlight` | 09:13:46-09:14:31, 45 seconds | `0000:0000` | 0 packets, 0 bytes | Generic device aggregation and `Device Added`/`New MIDI connection` records; no observed hardware unlock |

**The useful positive finding is name-sensitive identity resolution.** The
exact unprefixed name produced `vid=06f8&pid=b128` in Serato's connection URL
even though this program created no USB device and assigned no USB IDs. With
identical manufacturer/model metadata, the prefixed alias produced zero IDs.
This supports a name-based identity-normalization path; it does not establish
that Serato accepted the native controller profile or verified supported
primary hardware.

The unprefixed connection was logged at line 689 and removed at line 697.
The alias's generic aggregation is at lines 719-725; removal and RtMidi
connection cleanup are at lines 734-740. No candidate-associated SysEx
verification attempt appears in the captured windows, and neither destination
callback received any MIDI. The alias's generic `Device Added` message is not
an unlock: its URL retained zero VID/PID and the visible MIDI control remained
grey. Screenshots before the probe and during the alias capture showed no
observed hardware-state transition. Both decks remained unloaded.

Every completed capture reported `client_dispose_status="0"`; Serato separately
logged removal of the temporary connections. A second invocation while the
alias was live was rejected with exit status 1 and `an endpoint already uses
Hercules DJControl Starlight`, leaving the active capture untouched.

### Verification and limits

- Swift 6 compilation with warnings treated as errors passed. The real
  CoreMIDI self-test passed exact 609-byte delivery in both directions,
  including the 603-byte SysEx and two-packet iteration. CLI help and invalid
  duration rejection passed.
- Repository formatting, all-target/all-feature Clippy, all 510 Rust tests,
  all-target Rust build, all 80 Swift tests, build-script syntax and Info.plist
  validation passed. SwiftPM retained its pre-existing two-unhandled-files
  warning. The PTY Git whitespace check hung at the final command and was
  killed by the 180-second watchdog; rerunning only
  `git --no-pager diff --check` passed with exit status 0.
- The first transport check treated a virtual endpoint's absent entity as a
  failure. The diagnostic was corrected to report that specific
  `kMIDIObjectNotFound` result; its client closed cleanly on that failure, and
  the corrected transport check passed.
- This was hot-plug discovery only, not a cold-start test. No native audio
  identity, driver-owned MIDI device/entity, USB interface or valid
  authentication response was supplied. The native profile had not yet been
  recovered during these captures; the later static findings are separate
  evidence, not additional live acceptance tests.
- The existing application log contained authorization phone-home error 1000
  before this probe. It was not diagnosed or changed; trial/license validity
  was not independently established by this experiment.

**Capture conclusion:** ordinary virtual MIDI can reach Serato's Starlight-like
identity classification, but these identities alone did not produce an
observable handshake or usable primary-controller connection. The next
technical question is which prerequisite blocks native device assembly after
that classification: actual USB presence, an associated audio device,
driver/entity information or another profile condition. This run does not
select one of those explanations and does not rule out software emulation.
Subsequent static inspection identified the USB-triggered normal assembly
path for this kind-2 native profile; see the results record. USB-only
sufficiency, associated audio and later verification remain untested.
