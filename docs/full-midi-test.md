# Automatically generated full input mapping

The full profile uses a separate `S4 MK3 MIDI Full` virtual source. The verified
nine-control profile remains available with `--midi` and its original source
name. This profile does not overwrite that saved Djay configuration.

## Generate and install

```sh
cargo run --locked -- --generate-mapping "S4 MK3 MIDI Full.djayMidiMapping"
cargo run --locked -- --midi full --seconds 3600
```

The exporter creates a new file and refuses to overwrite an existing path.
Use a new filename when regenerating. It needs neither the S4 nor a running
MIDI source. Start the full bridge before configuring Djay.

Open the exported `.djayMidiMapping` in Djay (normally by double-clicking it),
then select it for `S4 MK3 MIDI Full`. The mapping entries are generated from
the same catalog used by the bridge, rather than individually learned.
Use Djay's four-deck layout and keep S4 audio selected.

The encoder layout below requires a newly generated mapping to be reinstalled
and selected, not just a bridge restart. Save it under a new filename and keep
user-edited mappings under their existing names. An older app bundle must also
be replaced with a build containing this layout; this document doesn't establish
that an existing bundle has been rebuilt.

## Layout

| Area | Default behavior |
| --- | --- |
| A/C and B/D | Select the left/right deck and route subsequent controls to it |
| Play, Cue, Sync, Master, Reverse, Flux | Native transport and sync/master; REV uses `reverseHold` only while held, Flux controls Slip independently |
| Tempo | Absolute speed with pickup; Shift uses relative-speed action |
| Touch jog | Touch scratches; untouched rotation nudges; Shift seeks |
| Jog / Turntable buttons | Choose CD/nudge or vinyl/touch-scratch behavior; no motor command |
| GRID + jog | Move the selected deck's beatgrid left/right |
| LOOP encoder | Independent native loop size (`autoLoopDurationRotary`); Shift retains `autoLoopMoveRotary` |
| LOOP encoder press | `autoLoopOnOff`; Shift uses `reloop` to reactivate the stored loop range; manual loop in/out is unmapped |
| MOVE encoder | Native jump by chosen jump size (`skipRotary`), with or without an active loop; selection mode changes jump size immediately (`skipDurationRotary`) |
| MOVE encoder press | Local per-logical-deck size-select toggle, no MIDI, no timeout; second press exits |
| Shift+MOVE turn | Fixed one-beat jump per detent in either direction, even while selecting; preserves chosen jump size |
| Browse encoder / press | Browse / load active deck; Shift selects a section / goes back; both rotation layers use one native mapping inversion to correct reported direction |
| Library buttons | Show library, queue, mark selection, and preview |
| HOTCUE pads | Hotcues 1-8; Shift clears them |
| SAMPLES pads | Play samples; Shift stops them |
| STEMS / MUTE pads | Four Neural Mix mute pads plus four solo pads |
| Shift STEMS pads 1-4 | Exclusive stem solo |
| REC | Native Record Sample action |
| Channel strips | Fixed deck 1-4 gain, EQ, volume, filter, and headphone cue |
| Channel FX1 / FX2 | Select the left/right hardware FX section's deck target |
| Deck FX buttons | Enable three effects; Shift selects the next effect |
| FX ON | Enable/disable that target deck's manual FX |
| FX knobs | Parameters 1-3; dry/wet changes all three effect mixes |
| Channel FX SELECT | Choose the deck targeted by the mixer instant-FX buttons |
| Mixer FX 1-4 / Filter | Instant FX 1-4 / reset that deck's filter |
| Quantize | Toggle quantize on all four decks |
| Front channel switches | Native Left / Through / Right crossfader assignment |
| Front curve switch | Three positions of Djay's crossfader curve control |

These are custom Djay equivalents, not identical Traktor behavior. In
particular, FX assignment selects a control target rather than routing channels
to a shared audio FX bus. Each section initially targets deck 1 or 2.

Djay has two sampler banks: A/C use bank 1 and B/D use bank 2. Stem availability
depends on the track and Djay's Neural Mix support. The eight physical pad masks
are corrected to the S4 order 5, 4, 7, 6, 3, 2, 1, 0.

## Routing guarantees

Encoder naming is corrected for all rotations and only left presses in the
full profile. Right presses and the minimal profile are unchanged.
Normal MOVE uses CC 2 on channels 1-4; unshifted size selection uses CC 7 on
those channels. Shift+MOVE emits note 5 forward or note 14 backward on
channels 9-12 only. The native catalog also provides base-channel aliases
for the fixed-jump actions, but runtime Shift routing doesn't use them.

Each logical deck A/B/C/D retains independent native Djay jump and loop
values. There are no bridge-owned size counters or shared-size resets.
Selection changes the native jump value immediately, not on exit.
Local selection-mode flags persist across deck switches without a timeout,
and all initialize off on bridge restart. While selecting, only the selected
deck selector pulses through a 1.2-second palette-brightness cycle with a
dark trough. Hidden decks retain mode without flashing inactive buttons.

Notes retain the deck, modifier, and pad/FX destination that owned their press.
Releasing a held control after changing decks or modes still releases that
original destination. Report releases are handled before selector changes and
new presses, so descriptor item ordering cannot choose a simultaneous
Shift-plus-pad route. Shutdown releases owned notes.

Touched jog movement stays with its touch owner until release. Integer HID
counts are recovered from Encdr's normalized float before MIDI conversion,
so rounding error cannot defer a count or cancel it on reversal. Fractional
grid steps are accumulated, and larger relative deltas are split rather than
discarded. Tempo and FX pickup are handled by Djay; changing a target does not
replay a cached knob position.

On the tested macOS/S4 combination, interrupt jog reports arrive about
256 ms apart, causing MIDI movement to be emitted in bursts. Full mode now
requests native input snapshots for buttons/touch and jogs, paced by a
2 ms bounded interrupt read. Older interrupt copies of those reports are
discarded so they cannot replay stale motion or touch state. Slider interrupts
and the minimal profile keep their existing transport. This requests input
only; it does not claim audio interfaces or enable motors.

High-rate jog and MIDI-message logs use debug level to avoid terminal output
on every movement in normal use. Use `RUST_LOG=debug` for a short diagnostic
capture; `--raw` remains an explicit full-report diagnostic.

Deck MIDI channels are 1-4, shifted variants 9-12, mixer 5, and global selection
6. The underlying native mapping file uses zero-based channel numbers.
Continuous MIDI controls use 7-bit values; physical feel and tempo precision
need real testing rather than inference from successful message delivery.

## Deliberate boundaries

- Master, booth, headphone level, and headphone mix remain hardware-local when
  using the S4's outputs, avoiding duplicate software attenuation.
- EXT presses are decoded, but no equivalent Djay live-input-selector action
  has been established. They are not given an arbitrary software assignment.
- Screens, motor rotation, controller emulation, and audio forwarding are not
  implemented by this input mapping.
- Regenerated full mappings include software-state LED feedback and configurable
  presentation. See [led-feedback.md](led-feedback.md) for setup and verification
  limits. Older input-only mappings do not emit those states. Minimal/probe Play
  mirroring remains separate and is not a playback-state indicator.

## Verify

### REV, Browse, and crossfader startup live check

Install this build's mapping under a new name, preserving existing custom
mappings. The crossfader gate requires its raw playback outputs on channel 7,
CC 4-7; an older mapping without those outputs cannot initialize selectors.

- Hold and release REV on every deck. Reverse must end on release and on
  bridge Stop, even if deck or Shift changes while held. Slip remains
  independent.
- Check clockwise/down-list and counterclockwise/up-list Browse movement
  on both sides, including Shift's section navigation. The mapping alone
  inverts direction; physical counts and transmitted CC values are unchanged.
- Preset all physical assignments and curve without moving them. Check
  Djay-first and bridge-first startup, including Through positions and
  empty/paused decks. Diagnostics should show `CROSSFADER_FEEDBACK_RECEIVED`;
  the latest switch snapshot should apply without touching switches.
- After initial synchronization, change a Djay UI assignment and leave the
  physical switch still. Ordinary playback/LED updates must not overwrite
  the UI choice. Moving the physical switch should apply its new position.
- After mapping reselection or Djay reconnect, Stop/Start the bridge to
  initialize a fresh feedback session. Automatic same-process reconnect
  detection is not implemented. These checks remain live-test requirements,
  not results established by software regressions.

Run the independent receiver alongside the full bridge:

```sh
cargo run --locked --example midi-monitor -- --port "S4 MK3 MIDI Full" --seconds 300
```

Check deck changes, held Cue/pad releases across changes, every pad mode,
left/right touch jog direction and feel, tempo center/endpoints, EQ/PFL,
independent FX targeting, front selectors, library controls, and direct S4 audio.
Use a test track when checking recording, hotcue clearing, and beatgrid editing.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
```

Native actions and encoding conventions were checked against the installed
Djay Pro 5.6.9 MIDI menu/metadata and its NI S3 and DDJ-SX2 presets. The latter
are references for file/action syntax, not controller emulation.

### Encoder layout live check

These checks are pending. The user verified with the mouse that beat jump
moves an active loop; this isn't evidence of fresh MIDI/native routing or
physical selector-LED behavior.

1. Install and select the new mapping, then monitor MIDI while using both
   physical encoder pairs. Confirm corrected LOOP/MOVE pairing and both
   directions, with and without Shift.
2. On A, press MOVE once, hold, then release. Expect one local mode toggle
   and no MIDI from press/release. Check repeated identical pressed and
   released input reports don't toggle again. Turn unshifted MOVE: expect
   CC 7 on channel 1 and an immediate native jump-size change. The next
   distinct press exits; subsequent rotation sends CC 2, not CC 7.
3. Repeat on C, B, and D, using channels 3, 2, and 4 respectively. Give
   decks different native jump values and mode states. Switch A/C and B/D
   away and back: values and flags must remain independent, without a
   timeout or shared reset.
4. While selecting, hold Shift and turn both directions. Expect one-beat
   jumps per detent via note 5 forward / note 14 backward on channels
   9/10/11/12 for A/B/C/D, never their base-channel aliases. Release Shift:
   selection resumes with the chosen jump size intact. Repeat outside
   selection mode.
5. Exit selection and check normal MOVE in both directions without a loop,
   then with an active loop. The jump and active-loop movement must use
   the chosen jump size. Change LOOP size and toggle LOOP on/off: its
   native size must remain independent of MOVE's value. Exit a loop, move its
   inactive region with Shift+LOOP turn, then press Shift+LOOP to reloop.
   Check restoration of its position and length on each deck. Reloop requires
   a stored loop range; no fallback creates a new loop when none exists.
6. Observe each selected selector's 1.2-second palette-brightness pulse
   and dark trough. Hide a selecting deck: its inactive button must not
   flash. Return to it and confirm the pulse resumes; exit selection and
   confirm ordinary selector lighting is restored.
7. Stop and restart the bridge with some modes on. All four mode flags
   must be off, normal MOVE must jump, and selector lighting must be
   restored. Confirm the bridge hasn't reset native Djay sizes. No screen
   or motor traffic belongs in this check.

# Validation record

The records below describe earlier revisions, not a validation pass for the
current encoder layout or its selector pulse.

The full profile generated a native mapping on October 5, 2026. Djay Pro
accepted its installation and saved the separate configuration at
`~/Music/djay/MIDI Mappings/S4 MK3 MIDI Full.djayMidiMapping`. Native HID
opened successfully and the `S4 MK3 MIDI Full` source is available.

Formatting, strict Clippy, build checks, and 28 deterministic tests passed;
language-server diagnostics were clean. The expanded profile's physical
control behavior and jog sensitivity still require live confirmation. The
earlier minimal profile's successful physical tests do not establish that
the expanded mapping works.

The live physical-test request received no response within its 30-minute
window. The bridge was then stopped with Ctrl-C. Installation is confirmed,
but no expanded-profile physical pass is claimed; resume with `--midi full`
and the checklist above.

That idle session recorded zero decoded input events and zero MIDI messages.
It exited with `no decoded S4 input received during the probe` (exit 1),
so it proves source creation and HID opening, not input translation in live use.

Later partial user testing found most controls working but reported notchy
jogs. A regression reproduced a five-count normalized HID delta becoming
four MIDI steps. The translator now rounds back to the original integer
counter before scaling. The MIDI addresses and sensitivity are unchanged,
so this correction needs a bridge restart, not a mapping reimport. Its effect
on physical feel remains unconfirmed. All 29 tests pass after the correction,
along with formatting, strict Clippy, and build checks.

Follow-up live captures reproduced the main discontinuity: right-jog deltas
arrived approximately every 256 ms, with matching bursts at the independent
MIDI receiver. Direct input-report requests returned in about 0.1-0.9 ms and
showed the device timer updating within about 2 ms. The snapshot transport
passes all 30 tests, including rejection of stale jog/touch interrupts, and
formatting, strict Clippy, diagnostics, and build checks.

With the corrected bridge, an independent one-second right-scratch sample
contained 110 MIDI messages: median spacing 8.09 ms, range 1.44-45.18 ms
(movement-dependent, not a guaranteed polling or latency bound). The user
confirmed "Smooth now" when retesting slow movement and touch scratching.
This verifies the reported jog discontinuity on this device; it does not
replace a complete controller test.
