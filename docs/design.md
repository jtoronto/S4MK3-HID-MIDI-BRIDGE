# Design discussion

Recorded October 4, 2026. This document captures decisions and research from the
initial discussion, not a completed engineering specification.

Update October 5: the user authorized the first Encdr connectivity stage.
See [connectivity-test.md](connectivity-test.md) for its bounded scope and checks.
The minimal macOS virtual MIDI stage was subsequently authorized; see
[midi-test.md](midi-test.md). Broader MIDI mappings and standalone audio remain
future stages.

## 1. Problem and priorities

The Traktor Kontrol S4 MK3 provides its physical controls through HID rather than
a user-selectable MIDI control mode. Support outside Traktor is limited.

The primary goal is personal use of the S4 MK3 with Djay Pro. The intended product
is a standalone hardware bridge, with controller translation running on the
bridge rather than the laptop.

The original guest-DJ use case was to plug in any laptop and start playing without
installing bridge software. That remains desirable, but it is not the overriding
requirement. Custom application mappings are acceptable.

Broader compatibility should remain possible, including VirtualDJ, Serato, and
other DJ software. A generic MIDI device and custom mappings do not automatically
provide compatibility with every application, particularly applications that
require supported primary hardware. Future controller emulation would need to
reproduce whatever identity, interfaces, initialization, and protocol behavior
the application actually expects.

## 2. Decision ledger

| Topic | Agreed direction |
| --- | --- |
| First application | Djay Pro |
| First laptop platform | macOS |
| First control interface | Custom MIDI mapping |
| Later emulation candidate | Pioneer DDJ-SX2 |
| Initial jog behavior | Normal touch scratching and pitch bending; no powered rotation requirement |
| Audio | S4 master and headphone outputs required for the first usable standalone version |
| Screens | Working screens in alternate software are not required |
| FX assignments | Exact behavior deferred; not a blocker for architecture discussion |
| EXT and front-panel switches | HID fields recorded in [connectivity-test.md](connectivity-test.md#deferred-controls-verified-hid-fields); decoding and Djay assignments deferred |
| Hardware | Not selected; Pi Zero was an initial suggestion, not a requirement |
| Current work | Stage 1 Encdr input/LED probe and stage 2 minimal macOS MIDI output authorized October 5 |

Powered rotation, richer haptics, Windows support, controller emulation, and
guest plug-and-play are outside the initial agreed scope. Their ordering and
eventual requirements remain undecided.

## 3. Controller translation

The bridge needs a bidirectional control path:

```text
S4 HID reports -> control translation -> USB MIDI -> Djay custom mapping
S4 LEDs       <- feedback translation <- USB MIDI <- Djay MIDI output
```

The initial mapping should preserve the S4's control set rather than force it
into the layout of another controller:

- Four-channel mixer controls and deck selection.
- Transport, cue, sync, pads, and shift combinations.
- Tempo faders and loop, move, and browse encoders.
- Touch-sensitive jogs and pitch bending.
- Both deck FX sections and per-channel FX1/FX2 assignment controls.

This is a mapping inventory, not a claim that every Traktor function has an
equivalent in Djay. Per-control assignments, pad modes, and shifted actions still
need definition and testing.

A proposed MIDI profile would use distinct messages for relevant decks and
layers. The bridge can track physical deck selection and modifiers locally.
Software state should come from return feedback where available, rather than
being guessed from button presses.

### Jog wheels

Initial behavior separates platter-touch scratching from ordinary pitch bending.
Translation must handle position-counter wraparound, direction, movement speed,
touch transitions, and release behavior.

Sensitivity, low-speed precision, fast spins, and the end of a scratch need
physical testing with Djay. Successful message delivery is not enough to establish
acceptable feel.

Powered rotation is a separate future feature. Expected motor rotation must not
be interpreted as a continuous user jog command. Motor behavior may need software
playback state and speed information unavailable from the tempo fader alone.

### Tempo and encoders

A single seven-bit MIDI CC has 128 values. Higher-resolution encodings exist, but
Djay's accepted encoding for each mapped action needs verification. Preserve the
S4's available resolution internally rather than prematurely reducing it.

Djay documents relative encoder formats, pickup, jog speed, and reaction settings.
The proposed starting point is software pickup for tempo changes caused by Sync
or deck switching. Bridge-side takeover logic is not yet selected.

### Feedback

Local state includes deck selection, Shift, and bridge-defined modes. Playback,
Sync, software FX state, and cue availability should follow application feedback
where possible.

Djay documents MIDI output for LEDs. That does not establish support for all
desired feedback, such as meters, cue colors, precise playback position, or
playback rate. These require an explicit feedback inventory.

Screens could remain blank or show local bridge status. No waveform, title,
artwork, or other track-metadata display is required.

## 4. FX behavior: deferred

The S4 has two physical FX sections and per-channel FX1/FX2 assignments. The S3
lacks the corresponding full deck FX control set, making it a poor initial fit.

The DDJ-SX2 has similar assignment buttons. In its intended Serato behavior,
each channel can be assigned to FX unit 1, unit 2, or both. Multiple channels can
use the same unit. Those effects are processed by software, not by the controller.

Button labels do not guarantee equivalent behavior across applications:

- Djay documents three manual effects per deck. Algoriddim support states that
  one deck's manual effects cannot be assigned to another deck.
- VirtualDJ's documented SX2 mapping uses FX assignment buttons to copy effects
  between decks.

Possible custom Djay behavior includes having each FX section follow the selected
deck or using assignment buttons to select the decks that receive MIDI parameter
changes. Neither creates shared FX audio buses. Sending changes to multiple decks
also does not automatically synchronize their effect types or complete state.

No option has been selected. The user explicitly deferred this question.

## 5. Audio and USB topology

The user reports that the S4 appears as a regular audio interface on macOS and
Windows without additional setup. Mixxx documents it as class-compliant audio.
Djay accepts compatible audio interfaces without requiring a specific vendor.

This simplifies laptop driver requirements, but not the bridge topology. The S4
has one USB connection to one host. If the bridge hosts it, an ordinary hub or
splitter cannot give the laptop ownership of audio while giving the bridge
ownership of HID.

### Proposed standalone topology

```text
Guest Mac / Djay
       |
       | USB device presented by bridge: audio + MIDI
       |
Hardware bridge
       |
       | USB host connection: S4 audio + HID + optional screen interface
       |
Traktor Kontrol S4 MK3
       |
       +-- master / booth / headphones
```

The initial audio requirement is stereo master and stereo headphones. Mixxx
documents S4 playback channels 1-2 as main and 3-4 as headphones; verify that
routing on the actual device.

Line/microphone input forwarding, recording, and other audio paths have not been
specified. Hardware-linked master, booth, and headphone controls must not be
mapped to duplicate software attenuation without checking their behavior.

### Option A: transparent USB protocol proxy

Reproduce the relevant S4 USB interfaces on the laptop-facing connection and
forward requests and transfers to the real controller, while intercepting
controls for translation.

This is active emulation, not passive wiring. Real-time isochronous audio,
endpoint allocation, control requests, and timing make it more involved than an
HID-only proxy. It is not the selected implementation.

### Option B: class-compliant audio forwarding

Present a new standard USB audio interface alongside the bridge's MIDI interface.
Receive laptop audio and forward PCM streams to the S4 through the Linux audio
stack. The S4 still performs digital-to-analog conversion.

Linux provides a USB Audio Class 2 gadget function and documents routing its
audio to another sound card. This is the leading proposed approach, not a tested
S4 solution or final architecture decision.

The gadget alone does not forward audio automatically. A forwarding path must
handle formats, channels, buffering, start/stop transitions, and clock differences.
Clock drift may require feedback-aware pacing or rate adaptation. Bit-perfect
operation, latency, and dropout-free performance are not established.

### Hardware implications

A stock Pi Zero has one USB data controller, usable as host or device, not both
simultaneously. Its power connector is not a second USB data connection. An
ordinary hub does not solve this limitation.

A Pi 4-class board with separate USB host ports and gadget-capable connectivity
is a candidate for experiments. The final board is not selected. Device-controller
capabilities, combined audio/MIDI endpoints, power, and sustained performance
need validation before selecting hardware.

## 6. Emulation candidates considered

The S4 MK2 and S3 were early suggestions, not selected targets. Official support
of the S3 across several applications made it interesting, but its FX controls
do not match the S4's requirements.

Non-Traktor candidates discussed:

| Candidate | Relevant similarities | Mismatches / investigation needs |
| --- | --- | --- |
| Pioneer DDJ-SX2 | Four mixer channels, four-deck selection, eight pads per side, two three-knob FX sections, FX1/FX2 assignments | Loop buttons versus S4 encoders; different FX knob arrangement; USB recognition untested |
| Denon MC7000 | Four-channel/deck layout, eight pads per side, two FX sections, FX assignments | Loop-button and FX beat-encoder translation; identity and feedback untested |
| Numark NS6II | Four-channel/deck layout, eight pads per side, two FX sections, FX A/B assignments | Pad-based looping; touch knobs and displays add differences |

These are listed in Djay's hardware catalog. Application and platform limitations
must be checked for the actual target version. Catalog support does not prove
that a bridge can successfully impersonate the device.

DDJ-SX2 was chosen as a possible later emulation target, after the custom mapping.
Its published MIDI specification is useful, but is not a complete specification
of USB identity, audio behavior, or application initialization.

## 7. Proposed validation stages

These stages were suggested during discussion. They have not been implemented
or approved as an execution plan.

### Desktop prototype with the real S4

```text
S4 HID <-> temporary Mac bridge application <-> virtual MIDI <-> Djay
Djay audio -----------------------------------------------> S4 directly
```

Use a temporary laptop application to test translation, custom mapping, jog feel,
tempo, and LEDs while leaving audio directly connected to the S4.

Selective control-interface access must coexist with the audio driver; this
coexistence is an assumption to verify. Do not claim the whole USB device or
detach its audio interfaces indiscriminately.

The desktop application is a proposed development tool, not the intended final
deployment. Direct S4 audio here does not satisfy the standalone forwarding test.

### Recorded input and simulated feedback

Replay captured HID reports and feed known MIDI feedback into the same translation
logic. Check wraparound, direction, deck/layer transitions, held controls, and
reconnection state deterministically.

Replay supports functional regression checks but cannot establish physical jog
feel or hardware timing. Captures need timestamps to preserve the events relevant
to the behavior being tested.

### Linux USB simulation

Virtual USB host/device facilities such as Linux's dummy host controller can help
exercise gadget descriptors and protocol behavior inside Linux.

A Linux VM alone does not make macOS see a physical USB gadget through a normal
Mac host port. This can support internal protocol tests, not end-to-end Djay USB
enumeration or physical-link latency validation.

### Physical standalone bridge

Only the real host-to-bridge-to-S4 path can validate:

- Composite USB audio/MIDI recognition by macOS and Djay.
- Master/headphone separation and hardware volume-control behavior.
- End-to-end latency, jitter, clock drift, and dropouts.
- Concurrent jog traffic, LED updates, and audio load.
- Device disconnect/reconnect, laptop handover, boot, and power behavior.

Numeric latency targets, acceptable buffer sizes, and soak-test duration have not
been chosen. A documented Linux example proves the mechanism exists, not that
this application meets DJ performance requirements.

## 8. Reference findings

### Encdr

Encdr describes named S4 controls, LEDs, jog rings, motors, and dual screen
transfers. It is a candidate protocol layer, not an existing complete hardware
bridge or Djay mapping.

Its current core includes a GPU-based screen pipeline; the optional WebView
renderer is separate. Headless operation and suitability for a small board need
inspection rather than assuming screen-related dependencies are optional.

The repository states GPL-3.0-or-later. Check licensing before reusing or
distributing code. No library, implementation language, or dependency has been
selected.

### Mixxx

The S4 mapping provides examples of input decoding, deck/mode state, LEDs, jog
handling, and experimental motors. It accesses Mixxx's internal engine values.
Those values are not automatically available over Djay MIDI feedback.

The screen implementation uses Mixxx-specific QML and track data. It is a
hardware-protocol reference, not a portable source of Djay track information.
Mixxx documentation describes motor support as partial/beta.

### NIME 2026 paper

Zeph Thibodeau's paper documents reverse-engineering and implementing S4 MK3 motor
control with the Mixxx community. The full ten-page paper was read.

Useful author-reported findings:

- Wheel reports arrive at 500 Hz and motor output is expected at that cadence.
  Responding to incoming reports avoided software-timer limitations (section 6.1).
- One position stream showed discontinuities while another remained continuous
  (section 5.3).
- Coupling playback directly to motor movement introduced flutter; steady
  playback, scratching, and post-release motion need different handling
  (section 6.2).
- Crown nudging, motor calibration, and cue-and-stop behavior introduce further
  state and timing requirements (sections 6.3-6.5).
- The author describes early general-purpose HID-to-MIDI/OSC middleware
  experiments (sections 8-9).

These are references to verify on the actual controller, not measured contracts
for this project. The paper states both 2,880 counts per revolution and 0.0125
degrees per count; those are inconsistent, since 360 / 2,880 is 0.125 degrees.
Do not copy constants without checking.

The linked experimental Mixxx branch has not been inspected. The paper does not
validate USB audio forwarding or Djay compatibility.

## 9. Open and deferred questions

Before implementation:

- Agree on the desktop-prototype scope and when implementation should start.
- Choose a candidate board/topology for standalone audio testing.
- Establish audio formats, channel exposure, clock strategy, and performance
  acceptance criteria.
- Verify control-interface access alongside direct macOS audio.
- Inspect Djay's actual mappable actions, high-resolution tempo support, and
  available return feedback.

Deferred:

- FX assignment semantics.
- Powered rotation and richer haptics.
- Screens and track metadata.
- Windows and broader application compatibility.
- DDJ-SX2 emulation and guest plug-and-play.

## 10. Sources

Links were consulted during this discussion unless explicitly noted. Published
support and community observations are not project test results.

- [Encdr repository](https://github.com/RufusIbiza/Encdr)
- [Encdr S4 MK3 protocol notes](https://github.com/RufusIbiza/Encdr/blob/main/docs/hardware/ni_kontrol_s4_mk3.md)
- [Encdr core dependencies](https://github.com/RufusIbiza/Encdr/blob/main/encdr/Cargo.toml)
- [Mixxx S4 MK3 HID mapping](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js)
- [Mixxx screen implementation](https://github.com/mixxxdj/mixxx/tree/main/res/controllers/TraktorKontrolS4MK3Screens)
- [Mixxx S4 MK3 manual](https://manual.mixxx.org/2.7/en/hardware/controllers/native_instruments_traktor_kontrol_s4_mk3)
- [NIME 2026 paper](https://nime.org/proceedings/2026/nime2026_123.pdf)
- [Paper-linked experimental Mixxx branch: not yet inspected](https://github.com/jtMUMT/mixxx/tree/fix/s4-motors-chattering-and-cue)
- [Djay hardware catalog](https://www.algoriddim.com/hardware)
- [Djay custom mapping guide](https://help.algoriddim.com/user-manual/djay-pro-mac/midi/mapping)
- [Djay compatible audio interfaces](https://help.algoriddim.com/topic/hardware/using-audio-interfaces-with-djay)
- [Djay manual FX model: Windows documentation](https://help.algoriddim.com/user-manual/djay-pro-windows/dj-tools/effects/effects-section)
- [Algoriddim SX2 FX discussion](https://community.algoriddim.com/t/sx2-mapping-effect/35969)
- [Serato DDJ-SX2 guide: assignment behavior referenced through search results](https://support.serato.com/hc/en-us/articles/10173704069135-Pioneer-DJ-DDJ-SX2-Quickstart-Guide)
- [VirtualDJ DDJ-SX2 effects mapping](https://virtualdj.com/manuals/hardware/pioneer/ddjsx2/layout/effects.html)
- [DDJ-SX2 MIDI specification: located, not fully reviewed](https://downloads.support.alphatheta.com/software_info/dj-controllers/DDJ-SX2/DDJ-SX2_List_of_MIDI_Message_E.pdf)
- [VirtualDJ MC7000 effects mapping](https://virtualdj.com/manuals/hardware/denon/mc7000/layout/effects.html)
- [VirtualDJ NS6II effects mapping](https://virtualdj.com/manuals/hardware/numark/ns6ii/layout/effects.html)
- [Linux USB gadget testing: UAC2 and MIDI functions](https://docs.kernel.org/usb/gadget-testing.html)
- [Raspberry Pi gadget connectivity and port roles](https://www.raspberrypi.com/news/usb-gadget-mode-in-raspberry-pi-os-ssh-over-usb/)

The October 4 discussion contained no project software or hardware tests.
Stage 1 connectivity work is tracked separately in
[connectivity-test.md](connectivity-test.md). No Djay MIDI mapping or standalone
audio-forwarding path has been implemented.
