# Proposed S4 MK3 LED mapping

Status: **approved for implementation on October 5, 2026**.
Prepared October 5, 2026 against bridge commit `fc19dc0`, Encdr revision
`5d7a689ca5789161b3efd1b2049d7e54504fe881`, and installed Djay Pro 5.6.9.
This document records the approved design; runtime and physical evidence are
required before claiming its implementation complete.
See [led-feedback.md](led-feedback.md) for the current implementation, editable
configuration, acceptance evidence, and unresolved live checks. The remaining
text is the approved design, not a claim that every proposed field is available.

## What you would see

- Play, Cue, Sync, Master, Reverse, Flux, headphone cue, and FX lights follow
  Djay, including changes made with the mouse.
- Deck selectors, pad-mode buttons, Shift, GRID, jog mode, and FX target
  selectors follow the bridge's actual routing.
- Switching A/C or B/D immediately redraws the selected deck's cached state.
  FX lights follow their separately selected FX target, not necessarily that deck.
- Hotcue pads match their assigned colors in Djay, using the closest supported
  S4 palette color. Sample pads indicate playing samples. Stem
  pads indicate mute and solo states.
- Inactive controls are dim; active controls are bright. Unknown software
  state is dark, not guessed from the last button press.
- Channel meters follow their four fixed decks. Master meters already work
  through the hardware, as confirmed by the user; leave them alone.
- Jog rings show the selected deck color. Where software feedback permits,
  flash green in an active loop and red near the track end.
- Presentation settings are editable after implementation without rebuilding
  the bridge or remapping controller inputs. No playback-position animation,
  motors, or screen work is proposed.

The presentation choices below are proposals you can change before approval.
In particular: colors, inactive brightness, stem mute meaning, quantize's
mixed-state display, and ring warning timing. Channel meters and deck-colored
rings are now requested scope; loop/end indications depend on available feedback.

## Ownership and evidence

| Label | Meaning |
| --- | --- |
| Djay / example | A bundled Djay mapping has an `output` entry for this action; actual emitted messages still need capture |
| Djay / metadata | Installed action metadata declares a corresponding state; generic custom-mapping output needs verification |
| Local | The bridge already owns this state; no MIDI feedback is needed |
| Momentary | A physical held-button indication only, explicitly not software-state feedback |
| Unresolved | Neither a dependable generic output binding nor complete hardware behavior is established |

An input action can have output semantics: for example, Djay's metadata maps
`playPause` to `isPlayingButtonState`. We should use the action's native output
binding, not invent a second public API for its internal `modelState` string.
Cue uses Djay's traditional-cue button state, not an invented "Cue is held" or
"a hotcue exists" indicator.

Only left Play's existing bridge-controlled HID light has been physically
verified so far. The user separately confirms the master meters already work
and appear hardware-driven; channel meters do not yet work.
The remaining hardware offsets and palette values are descriptor/reference
evidence, not a claim that their lights have been tested on this controller.

## MIDI feedback contract

Keep controller-to-Djay input messages and `S4 MK3 MIDI Full` unchanged.
Propose a bridge-owned virtual MIDI **destination with the same name** for
Djay-to-controller output. Source and destination are opposite directions,
not an echo loop. Verify that Djay associates this destination with the
existing source; if it does not, resolve endpoint association before shipping.

All numbers below are decimal. Channels are human-facing 1-16; native plist
`midiChannel` values are zero-based. `D` means deck 1-4. Feedback is always
deck-addressed, never addressed to "whichever deck is currently on the left."

Binary feedback: MIDI Note, inactive value 0, active value 127. Accept Note Off
and Note On velocity 0 as inactive; interpret nonzero Note On as active.
Configure explicit output minimum/maximum, not the S3 preset's dim value 20.
Physical dimming is a bridge rendering decision. Preserve any on/off timing
Djay actually emits; do not synthesize a beat clock.
Hotcue color messages are the exception: their values must be decoded as
color tokens, not reduced to a nonzero boolean (see Pads).

| Output channel | Note | Logical state | Proposed native action binding |
| --- | --- | --- | --- |
| D | 0 | Play button state | `turntableD.playPause` |
| D | 1 | Traditional Cue button state | `turntableD.cuePositionOrJumpConsideringPlayState1` |
| D | 2 | Sync button state | `turntableD.bpmSync` |
| D | 3 | Reverse | `turntableD.reverse` |
| D | 4 | Slip/Flux | `turntableD.deckSlipToggle` |
| D | 8 | Sync master | `turntableD.turntableIsSyncMaster` |
| D | 6 | Loop active | `turntableD.autoLoopOnOff` (existing loop-press action; shared loop-state metadata) |
| D | 15 | Quantize | `turntableD.toggleQuantize` |
| D | 16-19 | Instant FX 1-4 active | `turntableD.instantFx1` through `instantFx4` |
| D | 32 | Manual FX active | `turntableD.fxActive` |
| D | 33-35 | Manual FX 1-3 enabled | `turntableD.fx1Enabled` through `fx3Enabled` |
| D | 40-47 | Hotcue 1-8 set and color | `turntableD.cueOrJumpIfAlreadySet1` through `8`; native pad-color encoding to verify |
| D | 48-55 | Sample 1-8 playing | `sampler.turntableB.playerN.playingConsideringHoldSetting` |
| D | 56-59 | Stem 1-4 muted | `turntableD.unmixerFourTrackChannelNMuted` |
| D | 60-63 | Stem 1-4 solo | `turntableD.unmixerFourTrackChannelNSolo` |
| 5 | 0-3 | Headphone cue, decks 1-4 | `mixer.monitorActive1` through `4` |

For samples, bank `B=1` for decks A/C and `B=2` for B/D. These deliberately
duplicate bank feedback on two deck channels; both must update the same
logical bank cache. They must not create four independent sampler banks.

Attach one canonical output stream per logical state. Specifically, enable
manual FX output on the left-strip bindings (32-35), not also on right-strip
bindings (36-39). Both physical strips read the same per-deck FX cache.
Shifted actions keep their input mappings but have no duplicate LED outputs:
clearing a cue or selecting an effect updates the canonical state.

Native examples establish Play/Cue/Sync/Master/Reverse/Slip, PFL, hotcue
presence, loop active, manual FX, and sample-playing output entries. Quantize, Instant FX,
and stem state have metadata support; verify their generic output at runtime.
Do not copy preset-specific `customClassName`, modifiers, or USB identity.

No feedback address is assigned to local selectors. Meter and end-warning
addresses remain unassigned until their generic output is established.
Hotcue colors should use the existing pad addresses if the native encoding
works; do not allocate speculative extra messages.

## Visual policy

Suggested deck theme: **A red, B blue, C yellow, D purple**. This follows the
Mixxx reference defaults, not necessarily the currently displayed Djay theme.
Fixed-color lamps retain their physical factory color.

Palette-capable lamps/pads use `base + intensity`: dim intensity 0, active 2,
and raw 0 for genuinely dark/unknown. Reference bases: red 4, orange 12,
yellow 20, green 28, cyan 36, blue 44, purple 48, white 68.
These are S4 palette bytes, not outgoing MIDI velocities.

For fixed-color Play/Reverse/Flux/Shift lamps, propose raw brightness 16 dim,
127 active, 0 dark. Active 127 is already tested for left Play; dim intensity
and each other lamp need hardware verification. Do not send palette color
bytes to fixed-color lamps or assume every descriptor `single` means grayscale.

### Changeable presentation settings

Use one optional human-editable LED preferences file, selected with
`--led-config PATH`. Built-in defaults
remain usable without a file. Editing it and restarting the bridge should be
sufficient; live reload is not required. No Rust edit, rebuild, or Djay input
remapping is needed for presentation changes.

Keep the wire protocol separate from these settings: MIDI addresses, HID
offsets, and decoding belong to the device mapping, not user preferences.
Validate preferences at startup and report invalid values clearly.

| Preference | Proposed default / editable behavior |
| --- | --- |
| Deck colors | A red, B blue, C yellow, D purple; shared by selectors and rings |
| Hotcue color source | Djay; optional fixed-slot mode only if explicitly selected |
| Djay-to-S4 palette lookup | Editable color correspondence calibrated to Djay's palette indices |
| Brightness and inactive display | Dim/active levels; inactive dim or dark |
| Stem display | Mute/solo colors and whether bright means muted or unmuted |
| Quantize presentation | All-enabled and mixed-state display choices |
| Ring indications | Individually enable/disable loop and end warnings; colors and timing |
| End warning threshold | Provisional 30 seconds remaining, adjustable |
| Ring flashing | Provisional 2 Hz, 50% duty cycle; alternate warning color and deck color |
| Meter presentation | Segment curve/brightness, calibrated to returned levels |

Only palette-supported colors and physically supported brightness are valid.
Changing the visual policy must not change note ownership, deck routing, or
MIDI input behavior. Djay theme changes are not automatically reflected in
the deck-color setting unless a deck-color feedback source is established.

## Complete button/pad report inventory

Offsets below exclude report prefix `0x80`. The complete report is 95 bytes:
one prefix plus 94 payload bytes. Left/right rows expand to two physical lights
except where a channel range is listed.

### Deck areas

| Physical light | Left / right offset | Owner and meaning | Proposed display |
| --- | --- | --- | --- |
| Play | 55 / 66 | Djay / example: Note 0 of selected deck | Factory color, dim stopped, bright playing |
| Cue | 8 / 31 | Djay / example: Note 1 of selected deck | Orange, dim inactive, bright according to Djay Cue state |
| Sync | 14 / 37 | Djay / example: Note 2 | Selected deck color, bright when active |
| Master | 15 / 38 | Djay / example: Note 8 | Selected deck color, bright when sync master |
| Reverse | 60 / 71 | Djay / example: Note 3 | Factory color, bright reversed |
| Flux | 61 / 72 | Djay / example: Note 4 | Factory color, bright Slip enabled |
| A/C and B/D selectors | 12,13 / 35,36 | Local: actual routed deck | Each deck's color; selected bright, other dim |
| HOTCUE | 9 / 32 | Local: Hotcue layer selected | Selected deck color, bright only in Hotcue layer |
| SAMPLES | 57 / 68 | Local: Samples layer selected | Cyan, bright only in Samples layer |
| STEMS | 10 / 33 | Local: Stems layer selected | Green, bright in Stems layer |
| MUTE | 58 / 69 | Local: alias of Stems layer in current input mapping | Green; both STEMS and MUTE bright in that shared layer |
| REC | 56 / 67 | Momentary: existing Record Sample button held | White while held; no claim of ongoing recording |
| Shift | 59 / 70 | Local: that side's Shift held | Factory color, bright while held |
| GRID | 18 / 41 | Local: grid-edit modifier held | White while held |
| JOG | 16 / 39 | Local: CD/nudge mode selected | Deck color, bright in CD mode |
| TURNTABLE | 17 / 40 | Local: vinyl/touch-scratch mode selected | Deck color, bright in vinyl mode; does not indicate motors |
| Library view | 19 / 42 | Momentary; generic visibility feedback unresolved | White while held, otherwise dim |
| Playlist/queue | 20 / 43 | Momentary; focus feedback unresolved | White while held, otherwise dim |
| Star | 21 / 44 | Momentary; selected-song marked feedback unresolved | White while held, not a persistent favorite-state light |
| Library preview | 22 / 45 | Momentary; preview-playing feedback unresolved | White while held, not a persistent preview-state light |
| Tempo indicator | 11 / 34 | Local proposal: physical fader center only | White at physical center, otherwise dark; never claim software pickup/Sync match |
| Pads 1-8 | 0-7 / 23-30 | Selected deck and pad layer; see next section | Mode-specific state colors |

Tempo offsets come from Mixxx and are missing in the pinned descriptor.
Center tolerance is an open calibration point; do not pick a threshold without
checking the actual fader center/detent. A feedback-capable software pickup
indicator would be a separate proposal.

There are no mapped LED outputs for jog touch itself, browse/loop/move encoder
presses, or the front-panel crossfader switches in the button descriptor.

### Mixer and FX areas

| Physical light | Payload offset(s) | Owner and meaning | Proposed display |
| --- | --- | --- | --- |
| Left FX ON, 1, 2, 3 | 62-65 | Djay / example: Notes 32-35 for left FX target | Target deck color; enabled bright |
| Right FX ON, 1, 2, 3 | 73-76 | Djay / example: same canonical state for right FX target | Target deck color; enabled bright |
| Channel 1-4 PFL | 77,81,85,89 | Djay / example: channel 5 Notes 0-3 | White, bright when headphone cue enabled |
| Channel 1-4 FX1 | 78,82,86,90 | Local: left FX target selector | White, exactly one selected bright |
| Channel 1-4 FX2 | 79,83,87,91 | Local: right FX target selector | White, exactly one selected bright |
| Channel 1-4 FX SELECT | 46-49 | Local: instant-FX target selector | Target deck theme, exactly one selected bright |
| Mixer FX 1-4 | 50-53 | Djay / metadata: Notes 16-19 for instant-FX target | Red, green, blue, yellow; active bright |
| Mixer Filter | 54 | Momentary: reset-filter action held | White while held; does not claim Filter FX enabled |
| Quantize | 93 | Djay / metadata: all four Note 15 states | White bright only if all four on; dim if known mixed/off; dark if any unknown |

Mixer FX offsets 50-54 come from Mixxx and are missing in the pinned descriptor.
FX1/FX2 selector lights mean **control targeting**, not shared audio-FX routing.

Payload bytes **80,84,88,92** have no named mapping in the inspected descriptor.
Do not assign these to EXT lights by pattern alone. Keep them zero as in the
current output buffer until their physical purpose is established. EXT has no
verified Djay live-input action, so no truthful external-input status is planned.

This accounts for all 94 payload positions: 83 named descriptor items,
two reference tempo indicators, five reference mixer FX lights, and four
unresolved bytes.

## Pads

Use physical pad order 1-8 as already corrected by the input mapping. LED byte
order is sequential; do not apply the input bit permutation to LED offsets.

| Layer | Feedback and meaning | Inactive / active |
| --- | --- | --- |
| Hotcue | Selected deck's Notes 40-47: cue slot set and Djay color | Dark if unset/unknown; bright closest S4 match to assigned Djay color if set |
| Samples | Bank 1 on A/C, bank 2 on B/D; Notes 48-55: playing | Dim cyan if known stopped; bright cyan if playing; dark if unknown |
| Stems, pads 1-4 | Notes 56-59: muted | Dim assigned stem color if unmuted; bright red if muted |
| Stems, pads 5-8 | Notes 60-63: solo | Dim assigned stem color if not solo; bright assigned color if solo |

Fixed-slot colors are no longer the default. Keep that mode as an explicit
user preference only. Suggested stem channel colors: red, yellow, green, blue for channels
1-4. Channel numbering follows Djay's four-track Neural Mix configuration;
do not hard-code vocal/drum/bass/instrument labels without verifying that mode.

The mute display deliberately means "this mute is engaged," not "this stem is
audible." It does not infer effective audibility from another channel's solo.
Shift does not replace these status colors: clearing cues/stopping samples/
exclusive solo still displays the resulting canonical software state.

Djay metadata exposes `cuePointPadPresentationColorIndex1..8`. The bundled
DDJ-SX2 mapping also defines `userInfo.padColorsActive`, `padColorOff`, and
`padColorWhiteActive`. This is concrete color-encoding evidence, not yet proof
that the same mechanism works for a generic virtual controller.

Proposed verification: use a pad-color lookup with distinct nonzero MIDI
tokens for Djay palette indices and a separate white token, with off=0.
Do not copy Pioneer velocities as S4 palette bytes. Capture the returned
messages while assigning every available Djay cue color, changing a color
without retriggering the cue, clearing a cue, and switching/loading decks.
Then decode those tokens into semantic color plus presence and render through
the editable S4 palette lookup. Confirm index order and white handling rather
than guessing them from the native array order.

If this mechanism cannot return generic color output, matching Djay colors is
an unresolved requirement, not permission to silently ship fixed-slot colors.
Preserve actual per-deck cue color in the cache independently of layer selection.
An active cue with unknown color stays dark until synchronized under the default
policy; do not display a stale color from the previous track.

Sample `loadingSuccess` has no verified generic loaded-state wire encoding.
Dim stopped samples do not promise the sample is loaded. Loaded/empty and
Neural Mix availability still need separate observable fields.

## Channel meters and jog rings

| Area | Hardware evidence | Proposed behavior / unresolved prerequisite |
| --- | --- | --- |
| Four channel meters | Report `0x81`, 78-byte payload; column bases 0,15,30,45; clip offsets 14,29,44,59 | Fixed deck 1-4 levels, independent of A/C or B/D; need generic Djay channel-level/clip output and segment calibration |
| Master meters | User confirms these already work and appear hardware-driven | No bridge-driven master level or clip feedback; preserve existing hardware behavior |
| Left/right jog rings | Report `0x32`, 40-byte payload and reference initialization via `0x30` | Selected deck color, plus conditional loop/end indications below; verify mode/position/deck-index and initialization in isolation |

The meter descriptor names only each column's first level byte and clip byte;
writing a level into that first byte will not render a complete segmented bar.
The reference uses a segment array, with ambiguity around its top segment.
Verify the segment count and brightness on the hardware before defining a curve.
Suggested eventual meter refresh cap is 20 Hz, latest value only; stale levels
must go dark, and input/jog processing must take priority.
Because this is a complete 78-byte output report, verify that writing channel
segments does not override the hardware-driven master meters. Do not assume
zeroing the remaining bytes is harmless. Channel-meter acceptance includes
master bars/clip behavior continuing unchanged with direct S4 audio.
Define staleness only after confirming meter update cadence; ordinary silence
on a latched boolean does not make that boolean stale.

S3 native `mixer.masterLeftMeter`/`masterRightMeter` rows are a research lead,
not proof of generic MIDI output: they lack an `output` dictionary and the
preset has a custom implementation. No channel-meter addresses are invented
here. Channel meters are requested, with generic channel-level/clip feedback
still an implementation prerequisite, not grounds to substitute master levels.

Ring initialization is not currently allowed by the bridge's restricted
descriptor. Implementing rings requires an isolated protocol test and a
separate allowed output group. Never enable motor report `0x31`, torque,
playback-position chasing, or screens as part of ring lighting.

### Ring state and priority

Each physical ring follows that side's currently selected deck, including when
an FX strip targets a different deck. Proposed priority:

1. **End warning:** flash red if a loaded, playing, non-looping track is within
   the configured remaining-time threshold and the required feedback is known.
2. **Active loop:** flash green while Djay reports that a loop is actually
   enabled. This takes precedence over elapsed-track proximity when looping.
3. **Normal:** steady configured deck color.

Disabling an indication restores the next applicable state. Flashing alternates
the warning color with deck color rather than turning the ring dark. Restore
deck color immediately after leaving a loop, seeking away from the end, or
switching decks. Paused tracks do not flash red; a loop indication can persist
while paused because it describes enabled loop state.

`turntable.loopingActive` has both metadata state
`loopingEnabledInView` and a DDJ-SX2 output example. Verify that the emitted state
means enabled looping, not merely a saved loop region. The existing Note 6
action is `autoLoopOnOff`, whose metadata reports the same
`loopingEnabledInView` state. Attach feedback to that existing action and verify
it responds to manual as well as automatic loops; do not replace the input
action with `loopingActive`. Never infer a loop from pressing the encoder.

No end-warning or remaining-time field was found in the inspected metadata.
Prefer a native per-deck end-warning output if one is discovered and has suitable
semantics; otherwise require actual remaining-time feedback plus play/load/loop
state. A native boolean may have a fixed threshold: expose the 30-second setting
only if the returned data permits calculating it. Do not estimate remaining
time from wall-clock playback, MIDI jog steps, or the last load-button press.
If unavailable, leave red warning disabled and report that limitation.

Flashing is visual status, not a beat-synchronization claim. Use monotonic
deadlines in the existing event loop, render only at phase changes, and never
sleep/block jog processing. Cache hidden-deck state so selecting it redraws
the correct ring immediately. Hardware packet encoding and LED-write cost
remain to be measured with both rings active.

## Feedback flow and lifecycle

1. Create the MIDI feedback destination before Djay can send output. Receive
   messages into a bounded queue; never write HID from the MIDI callback.
2. Cache logical state for all four decks and both sampler banks, including
   hidden decks, retaining cue colors and loop/warning state separately from
   display preferences. Only local deck/mode/FX selection determines which LEDs render.
3. Keep local routing and physical held-button state authoritative. Mouse
   selection in Djay does not silently switch A/C or B/D on the hardware.
4. At each input-loop boundary, apply pending feedback, compose one complete
   button report, and write only if bytes changed. Coalesce duplicate updates;
   a full-buffer write must not clear another control's unrelated state.
5. Cap button-report writes at 100 Hz and collapse intermediate updates to
   the latest image. Do not add a blocking wait to the existing jog loop.
   Do not send identical images repeatedly; Encdr marks every `set` dirty.
6. Remove the probe's held-Play LED override in full-feedback mode. Keep the
   verified minimal/probe behavior separate.
7. On startup, show local A/B, Hotcue, vinyl, and default FX targets. Software
   state begins unknown/dark until actual feedback arrives. Test whether Djay
   sends initial state when connecting/importing; do not assume it does.
8. Changing deck/mode/FX target redraws from cache without toggling any Djay
   action to solicit feedback. Track loads must invalidate cue/sample-related
   state if a verified output can signal the transition; otherwise resolve
   snapshot/refresh behavior before claiming correct load handling.
9. Feedback silence is not a disconnect: paused states may emit no messages.
   MIDI source disappearance/application exit and reconnect behavior need
   verification. Never declare an arbitrary silence timeout as state false.
10. On graceful bridge stop, release owned MIDI notes and clear bridge-managed
    button lights while HID remains open. USB removal cannot guarantee a final
    clear write; terminate visibly rather than claiming success.

No saved user mapping is overwritten automatically. A feedback-enabled
generated mapping is a reviewed revision of the full profile; installation
must preserve the existing working file and the minimal configuration.

## Proposed implementation and acceptance order

The user approved this sequence and the revised scope before implementation.

| Step | Deliverable | Acceptance |
| --- | --- | --- |
| 1 | Capture native Play/Cue/PFL feedback and startup/reconnect behavior through a virtual destination | Exact state packets observed from mouse actions, with no HID input needed; destination association proved |
| 2 | Shared feedback catalog, per-deck/bank cache, complete button renderer | Deterministic tests for addresses, off encodings, unknown state, deck/mode changes, FX target mismatch, and duplicate updates |
| 3 | Button/pad feedback, Djay-matched hotcue colors, and local selectors | All available Djay cue colors and live color edits captured and physically matched; hidden-deck and track-load state redraws correctly |
| 4 | Metadata-only bindings and unresolved indicators | Quantize/Instant FX/stems captured; every unresolved field either proved or explicitly left out of the approved revision |
| 5 | Channel meters and deck-colored rings; loop/end warnings where feedback permits | Channel segments calibrated, master meters unaffected, loop output captured; seek/load/pause/loop tests for end warning if supported; no motor/audio/screen outputs |
| 6 | Regression and live coexistence | Existing mapping/input tests remain green; smooth jogs and direct S4 audio continue with feedback traffic |

Tests belong beside existing Rust translation/mapping tests, with actual
CoreMIDI receiver subscriptions before sending and bounded event waits, not
fixed sleeps. Pure documentation gets no new tests now. Later implementation
must run formatting, strict Clippy, all tests, build, and LSP diagnostics.

Physical evidence should include Play/Cue/Sync/Master/PFL, one mouse-triggered
FX change, hotcue creation/clear, sample start/stop, stem mute/solo, every local
selector, held controls across deck changes, startup/reconnect, and shutdown.
Also check changing preferences without rebuilding/remapping, Djay hotcue-color
changes while another pad layer is selected, channel-meter routing on all four
decks, unaffected master meters, ring priority, loop start/exit while paused,
near-end seeking, unknown warning state, and disabling either ring indication.
Automated flashing tests use an injected clock and explicit phase advancement,
not timing luck.
Capture both the actual returned MIDI and the visible S4 lights; a successful
HID write alone is insufficient. Measure jog MIDI cadence with feedback active
to guard against reintroducing the 256 ms burst problem.

## Review decisions and open facts

**Proposed defaults to inspect:** A/B/C/D colors; dim inactive lights;
Djay-matched hotcue colors; bright red means muted; STEMS and MUTE both indicate their
shared layer; quantize bright means all four enabled; REC/library/Filter lights
are momentary where software-state output is unresolved; configurable 2 Hz
green loop flashing and red end warning, with a provisional 30-second threshold
if remaining-time output permits it.

**User-requested scope:** Djay cue-color matching, easy presentation changes
afterward, channel meters, deck-colored rings, and loop/end indications where
possible. Master meters already work and must remain hardware-driven.

**Facts to resolve during approved implementation:** virtual destination
association; initial-state and track-load refresh; native output blinking and
enabled/disabled behavior; palette/intensity on each physical lamp; the seven
descriptor additions; tempo center tolerance; unknown output bytes; generic
cue-color encoding, loaded-sample, Neural Mix availability, channel-meter
feedback, ring initialization, and end-warning/remaining-time support.
No fake state or guessed address should substitute for a failed verification.

## Sources and reproducible references

- [Djay Mac mapping guide: MIDI Out](https://help.algoriddim.com/user-manual/djay-pro-mac/midi/mapping#advanced-options):
  official channel/type/control/min/max/blend/invert configuration, not a
  complete native file schema.
- Installed `/Applications/djay Pro.app/Contents/Resources/MIDI Mappings/NI Traktor Kontrol S3.djayMidiMapping`:
  Play/Cue output at lines 455-503, Reverse/Slip at 563-610, Sync/Master at
  833-859, hotcue at 1042-1188, samples at 1649-1799, master-meter leads at
  7706-7738. These are examples, not runtime proof for our virtual device.
- Installed `Pioneer DDJ-SX2.djayMidiMapping` in the same directory:
  FX enabled/active output around 874-1020; sample output around 3094-3244;
  loop-active output at 1997-2008 and pad-color lookup at 13560-13575.
- Installed `MidiModelMetadata.plist` in the parent Resources directory:
  Cue state at 2082-2087; hotcue set/color metadata at 1818-1904;
  Sync/Master at 1760-1788; Play at 4054-4061; Reverse at 4142-4149;
  Slip at 3547-3552; stem solo/mute at 4619-4725; FX at 5626-5757 and
  6501-6558; Quantize at 6777-6782; sample playing/loading at 7835-7855;
  loop-enabled state at 3821-3829 and 1358-1365. The focused end-warning/remaining-time
  search returned no matching metadata field; availability is unresolved.
- Pinned Encdr `encdr/descriptors/ni_kontrol_s4_mk3.json`, LED groups at
  lines 217-360; `encdr/src/device/led_builder.rs` and `core/led.rs`:
  whole-buffer writes, raw palette bytes, and generic NI palette support.
- [Mixxx S4 MK3 mapping](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js):
  inspected reference palette/deck colors, brightness packing, tempo outputs
  11/34, mixer FX outputs 50-54, segmented meters, and ring initialization.
  Physical verification takes precedence over this moving upstream reference.
- Project `src/full/catalog.rs`, `src/full/mod.rs`, and
  [full-midi-test.md](full-midi-test.md): existing addresses, local routing,
  pad semantics, and the verified low-latency jog transport.
