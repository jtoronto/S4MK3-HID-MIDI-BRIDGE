# Serato investigation results and deferred work

November 3, 2026. Investigated against Serato DJ Pro **5.0.0, build
5.0.0.1278**, installed at `/Applications/Serato DJ Pro.app`.

## Decision

**Serato support is deferred.** No Serato backend, software selector or
production mapping has been implemented. The existing Djay product is
unchanged. Keep the research and isolated diagnostics for a future attempt;
the implementation plan is an archived proposal, not active work.

The attempt established MIDI transport and name-sensitive native identity
classification, but not a usable Serato controller. Read-only inspection then
identified a USB-triggered native assembly path for the Starlight candidate.
USB-level emulation is a possible future hardware experiment, not a working
solution or a committed product direction.

## What was attempted

1. Investigated generic Serato XML and drafted a separate-backend plan:
   [investigation and archived plan](serato-mode-investigation-and-plan.md).
2. Prepared a minimal synthetic Play/touch/jog probe and legacy XML:
   [jog diagnostic and runbook](serato-jog-test.md).
   Real CoreMIDI loopback delivered the expected 260 messages. Installed-Serato
   jog acceptance was not established: MIDI assignment was unavailable in the
   tested configuration without supported primary hardware. This is a setup
   gate, not evidence that the legacy jog action was rejected.
3. Presented temporary `DJControl Starlight` MIDI endpoints and a prefixed
   alias, without creating a USB/audio device or sending authentication:
   [identity probe, captures and cleanup evidence](serato-identity-probe.md).
4. Inspected the installed application's resource archive and arm64 assembly
   without patching, launching under a debugger or attaching to the process.
5. Assessed USB-level emulation options; no USB emulator was built or tested.

## MIDI identity results

The receive-only identity probe created both source and destination with
manufacturer `Hercules` and model `DJControl Starlight`.

| Name | Local capture window | Serato VID:PID | Result |
| --- | --- | --- | --- |
| `DJControl Starlight` | 09:12:23-09:13:08 | `06f8:b128` | Native-looking identity in discovery URL; no observed native assembly or unlock; zero received bytes |
| `DJControl Starlight` | 09:13:15-09:13:16 | `06f8:b128` | Same classification; zero received bytes |
| `Hercules DJControl Starlight` | 09:13:46-09:14:31 | `0000:0000` | Generic aggregation and device-added records; no observed unlock; zero received bytes |

Evidence is in `~/Music/_Serato_/Logs/20261103-082940.36809.log`, lines
689/697 for the initial native-name add/remove and 719-740 for the alias.
The program assigned no USB IDs, so these IDs are Serato's identity
classification, not proof of a USB device.

The user's later native-name reruns were also logged: 09:52:13.745237
(line 1043), removed at 09:52:57.744166 (line 1052), and 09:54:18.744897
(line 1066), removed at 09:54:42.743731 (line 1072). They again received
`06f8:b128` discovery identities without observed native assembly. Why the
user's log view did not show these entries was not established.

Every completed agent capture disposed its MIDI client with status 0; Serato
independently logged connection removal. A concurrent duplicate-name probe
was rejected without disturbing the existing endpoints. The probe's transport
self-test delivered exact 609-byte streams in both directions, including a
603-byte SysEx and two-packet iteration.

## Recovered native Starlight profile

The installed resource archive is
`Contents/Resources/userio_resources.bin`. Its decoded index contains
`midi/midi_hercules_dj_control_starlight.xml`.

The archive index consumed all 12,872 decoded bytes and described 229
contiguous payloads ending at the archive's exact 2,290,236-byte length.
The Starlight payload begins at archive offset 824,576, has encrypted length
3,664 and decodes to a 3,660-byte ZIP. The inflated XML is 55,792 bytes;
its CRC32 matched **3137249495**, and `xmllint --nonet --noout -` passed.
Decoded resources were inspected in memory, not installed or committed.

| Field | Recovered value |
| --- | --- |
| Category | `native_intro_device` |
| Native name / MIDI name | `DJControl Starlight` |
| Display name | `Hercules DJ Control Starlight` |
| Vendor / product | `0x06f8` / `0xb128` |
| Supported applications | `Serato DJ`, `Serato DJ Lite` |
| Handshake delay | `3500` |
| macOS audio API | `CoreAudio` |
| Legacy PortAudio | `true` |
| Sample rate | `DEVICE_DEFAULT` |
| Decks / hardware mixer | `2` / `false` |
| Logical audio outputs | `MasterOut`, `CueOut` |
| MIDI ID / mappable | `0xb128` / `true` |

Initialization declares `B0 7F 7F`, `91 0F 7F`, and `92 0F 7F`.
The profile contains native `platter_move` and `platter_touch` bindings, with
relative two's-complement movement. Those native bindings do not validate the
separate generic legacy-jog XML or establish S4 calibration.

There is **no explicit** `requires_usb`, `handshake_type`, `bcd_device` or
`mac_audio_device_name` in this profile. The archive's `midi/MIDI.xsd` was
also decoded with ZIP length/CRC verification: Native Intro does not declare
the `requires_usb` or `handshake_type` fields, while Native Pro documents
`requires_usb` as requiring a matching USB device for completeness.

Consequently, the USB finding below is not an XML `requires_usb=true`
setting. Absence of an explicit SysEx handshake declaration also does not
prove that every later verification or default behavior is absent. Waiting
for SysEx on the MIDI-only probe was not a decisive acceptance test.

The XML's creation date `2018-07-12` and MIDI wrapper `Serato DJ 1.9.9` are
historical resource metadata, not the installed application's version.

## Why the native MIDI identity stalled

These addresses are preferred arm64 addresses in the examined 5.0.0 binary,
not stable APIs or offsets guaranteed across releases.

| Location | Observed behavior |
| --- | --- |
| `0x1004f1b00`-`0x1004f1b48` | Descriptor parsing maps `native_intro_device` to kind 2; `osa_device` maps to kind 3. |
| `0x100e7c30c` | Normal aggregation selector processes separate USB and MIDI connection lists. |
| USB path, including `0x100e7c4f4` and `0x100e7c5c0` | A matching USB connection can start aggregation for kind 2. |
| MIDI path, `0x100e7c9b8`-`0x100e7c9d4` | Native MIDI-triggered aggregation is restricted to kind 3, not Starlight's kind 2. |
| Generic fallback, `0x100e7cb38`-`0x100e7cbe4` | A descriptor found by VID/PID/BCD suppresses generic aggregation; no descriptor allows the third-party path. |

This explains the contrastive runtime observations: the correct MIDI name
resolved to a known kind-2 identity but did not start native assembly, while
the unknown prefixed alias reached generic aggregation.

**The examined normal Starlight assembly path is USB-triggered.** Ordinary
virtual MIDI name/metadata alone cannot start that path. This is not proof
that every Serato device or every possible entry point requires physical
USB hardware.

Further worker code includes a conditional USB-readiness check at
`0x100e7a1b0`, an actual USB-connection-pointer test at `0x100e6c5c0`, and
the diagnostic `Device Aggregation : Device USB connection is not ready [`
at `0x100e7a250`. Those checks are distinct from the initial assembly trigger.

Static disassembly used LLDB file targets without process attachment, for
example:

```sh
/usr/bin/lldb --no-lldbinit --batch --arch arm64 \
  --file "/Applications/Serato DJ Pro.app/Contents/MacOS/Serato DJ Pro" \
  -o 'disassemble --start-address 0x100e7c9b8 --count 80'
```

## USB-level emulation assessment

A normal menu-bar app has no established app-level equivalent of virtual
CoreMIDI endpoints for publishing a USB peripheral to the same Mac.
Apple's documented USBDriverKit surface concerns drivers communicating with
USB devices; this investigation did not establish a supported software-only
virtual USB route.

An external gadget-capable USB device is the more credible prototype route.
Linux USB gadget facilities can configure USB identity and provide MIDI/audio
functions. However, generic gadget descriptors are not evidence of matching
the Starlight's complete interface layout or satisfying Serato.

The largest remaining practical uncertainty is audio. The recovered profile
expects CoreAudio and separate `MasterOut`/`CueOut` logical outputs, but the
exact macOS device association was not recovered. USB identity alone might
start assembly and still fail audio connection or later verification.
Receiving and discarding audio could demonstrate recognition without yielding
a usable S4 setup. Routing actual master/cue audio onward would be additional
engineering and would need its own latency/reliability checks.

References consulted for that feasibility assessment:

- [Apple USBDriverKit](https://developer.apple.com/documentation/usbdriverkit)
- [Linux USB gadget configfs](https://www.kernel.org/doc/html/latest/usb/gadget_configfs.html)
- [Linux gadget audio/MIDI function configuration](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/usb/gadget/Kconfig)
- [Hercules Starlight product and integrated audio interface](https://www.hercules.com/en/product/djcontrol-starlight/)

No USB emulator, hardware descriptor capture, working primary-controller
connection, license unlock, usable audio route or accepted jog behavior was
demonstrated. Starlight recognition must not be described as proof of Pro
licensing, four decks, DVS, feedback coverage, screens or motor support.

## Verification and boundaries

The diagnostic preparation passed repository formatting, locked
all-target/all-feature Clippy, 510 Rust tests, all-target build, 80 Swift
tests, build-script syntax, Info.plist lint, jog XML parsing and CoreMIDI
loopbacks. Identity-probe Swift 6 compilation treated warnings as errors.
The existing SwiftPM two-unhandled-files warning remained. These results
prove source/transport checks, not Serato acceptance.

The installed app passed `codesign --verify --deep --strict` after static
inspection. No app patch, debugger attachment, driver installation, SIP
change, S4 access, audio routing, motor command or screen traffic was used.
Trial/license validity was not independently established; an existing
authorization phone-home error was left undiagnosed.

## If resumed

Do not resume implementation or hardware activity automatically. First choose
whether the next goal is a supported-primary generic MIDI test or USB-level
emulation as a possible bridge backend. For the latter, prepare a bounded USB
identity/MIDI experiment, observe whether native assembly begins, and determine
the audio prerequisites before claiming controller acceptance. Another native
profile might have a different route, but none was demonstrated here.

A future success criterion is actual deck control and usable independent
master/cue audio, followed by jog acceptance and feedback verification.
Recognition logs alone are insufficient. Preserve the user's remaining trial
time and existing Djay setup.
