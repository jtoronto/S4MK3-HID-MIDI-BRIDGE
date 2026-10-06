# S4 MK3 factory behavior and Djay bridge comparison

This report compares the TRAKTOR KONTROL S4 MK3 Manual (software version 3.2,
July 2019) with the current full-profile bridge catalog and public Djay Pro
support documentation. It describes the bridge's intended MIDI actions, not a
claim that every control in the expanded profile has passed a live hardware
test. Printed page numbers below refer to the page number in the manual footer;
the supplied PDF has three preceding cover/front-matter pages.

## Evidence and naming

The manual is the source for factory Traktor behavior. The bridge's `input`
suffixes and Djay `target` strings are software identifiers in
`src/full/catalog.rs`; they do not prove which physical control produced an
event. Subsequent [physical capture](encoder-capture.md) verified all four
rotation fields and all four presses, both with and without Shift. All rotation
names are reversed; left press names are reversed, but right press names are
correct. These findings supersede the earlier provisional correspondence.

Inspection of the pinned Encdr descriptor identifies these encoder rotations:

| HID payload field | Decoded event name | Verified physical control |
| --- | --- | --- |
| Byte 19, low nibble | `left_loop_encoder` | Left MOVE |
| Byte 19, high nibble | `left_move_encoder` | Left LOOP |
| Byte 20, high nibble | `right_loop_encoder` | Right MOVE |
| Byte 21, low nibble | `right_move_encoder` | Right LOOP |

The same descriptor labels the press bits as loop/move: left payload byte 6,
masks `0x04` / `0x20`; right payload byte 15, masks `0x04` / `0x20`. The
physical capture established left MOVE=`0x04`, left LOOP=`0x20`,
right MOVE=`0x20`, right LOOP=`0x04`. Do not swap right press names along with
the rotation names. Clockwise increments and counterclockwise decrements were
verified on each knob; Shift did not change those hardware fields.

Mixxx's public S4 MK3 controller page explicitly calls the left encoder MOVE and
the right encoder LOOP. Its current mapping source is a second implementation
reference, not Native Instruments' factory mapping and not evidence of how this
bridge's pinned descriptor labels its fields:

- [Mixxx S4 MK3 controller documentation](https://manual.mixxx.org/2.7/en/hardware/controllers/native_instruments_traktor_kontrol_s4_mk3)
- [Mixxx S4 MK3 JavaScript mapping](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js)

## Controls compared, historical snapshot

The table retains the original comparison. Its encoder mapping is superseded
by the current overlay below; source identifiers in this snapshot aren't a
description of the corrected full-profile physical naming.

| Control | Traktor factory behavior and manual evidence | Current bridge / Djay mapping | Equivalence and remaining evidence |
| --- | --- | --- | --- |
| **MOVE and LOOP encoders: press, turn, and Shift** | The manual identifies MOVE as the left encoder and LOOP as the right one (Overview, §5.1, p. 22). Turning LOOP selects loop size when no loop is active and changes its size when active; pressing LOOP enables/disables the loop (§7.10, pp. 61–62). Turning MOVE jumps within the track when no loop is active; with a loop, it moves that loop by the selected size (Overview, p. 22; §7.10, p. 62). The manual's shifted uses are context-specific: Shift+LOOP press/turn locks or changes track key (§7.8.1–7.8.3, pp. 57–59), or toggles/adjusts Remix Deck quantize (§7.17, pp. 73–74); Shift+MOVE adjusts Sample or STEM-part volume (Overview, p. 22; §§7.17–7.18, pp. 71, 77). The manual does not describe a general Shift+MOVE beat-jump action. | Catalog rotations bind `loop_encoder` to `autoLoopDurationRotary` (Shift: `autoLoopMoveRotary`) and `move_encoder` to `autoLoopMoveRotary` (Shift: `skipRotary`). `loop_encoder_press` maps to `autoLoopOnOff` (Shift: `loopInOut`); `move_encoder_press` maps to `loopInOut` (`src/full/catalog.rs`, deck bindings). | These are custom Djay equivalents, not factory parity. Subsequent physical capture verifies reversed rotation names on both sides and reversed press names only on the left; see encoder-capture.md. The bridge's Shift actions also differ from the manual's key-lock/quantize and Sample/STEM functions. Physical pairing and polarity were verified with and without Shift. |
| **Reverse hold, Slip, and jog / haptics** | JOG, TT, and GRID are the three Haptic Drive modes (§3.2, p. 5). JOG supports rim nudge/tempo bend, top-plate scratch, seek and, with Flux, backspin-and-release return to the virtual playhead (§7.2.2, pp. 41–44; §7.11, pp. 63–64). TT rotates during playback and supports scratch (§3.2, p. 5; §7.2.3, pp. 45–47). GRID is for beatgrid correction (Overview, p. 22; §7.19, pp. 80–81). Preferences expose nudge ticks, wheel tension, haptic hotcues, and TT speed (§8, p. 83). REV reverses only while held and automatically enables Flux for the hold (§7.11, pp. 63–64). | Jog touch routes to Djay scratch or jog-seek behavior; untouched motion bends/nudges, Shift seeks, and GRID+jog emits beatgrid-shift actions. JOG/TT buttons choose the bridge's local wheel behavior. `reverse` maps to `turntable{deck}.reverse`; `flux` separately maps to `deckSlipToggle` (`src/full/catalog.rs`, `src/full/mod.rs`). | Djay actions can approximate scratch, seek, bend, beatgrid movement, reverse, and Slip. The catalog has no haptic motor/tension/tick or haptic-cue output, and the full profile deliberately sends no motor command. REV's Traktor hold-and-Flux behavior is not guaranteed by the current separate Reverse and Flux bindings; verify Djay's button mode and reverse/slip behavior in the actual mapping. Jog feel is a live-test question, not inferred from MIDI delivery. |
| **EXT: live input and MIC/LINE selection** | EXT inserts the channel's external signal as Live Input. Shift+EXT changes the input type/preamplifier: MIC C/D, PHONO, or LINE as applicable (§9.1–9.3, pp. 84–86). In standalone mode, EXT enables a mixer channel and Shift+EXT cycles its input type (§9.5, p. 88). | `EXT` is explicitly listed as unsupported in `src/full/catalog.rs`: no verified Djay live-input-selector action. | Djay's current public docs expose Microphone and Recording Input settings and a Microphone MIDI target, but do not establish the S4 channel's four-way MIC/LINE/PHONO preamp selection or a deck Live Input replacement. Audio-device routing is not the same action as selecting an S4 channel's external input. Keep the equivalence unresolved pending a version-specific live-input test. |
| **Preview button and encoder scrubbing** | Hold Preview to load/play the selected browser track into the Preview Player; while still holding Preview, turn Browse to seek; release Preview to stop/unload (§7.15, p. 68). | `library_play` maps to `musicLibrary.togglePreview`; `browse_encoder` maps to `musicLibrary.libraryRotary` (Shift: `sectionRotary`) (`src/full/catalog.rs`). There is no held-preview-dependent encoder route in the catalog. | Djay publicly documents browser preview through pre-cue/headphones and custom MIDI targets for Music Library, but those pages do not establish encoder seeking within a playing preview or the Traktor hold-to-stop interaction. Preview start is a plausible equivalent; scrub/hold semantics remain unverified and are not implemented as a conditional route here. |
| **MUTE, STEMS, and REC pads** | MUTE is a modifier used with a pad to mute/unmute a Remix sample or STEM part; STEMS selects STEM controls on a STEM Deck. STEM pads expose part volume/filter functions (§5.1.2, p. 24; §§7.17–7.18, pp. 71–78). REC enables the Remix Deck Pattern Recorder; pad taps record a quantized repeating sequence (§5.1.2, p. 24; §7.17.1, p. 74). | STEMS and MUTE both select the bridge's Stems pad layer. Four pads map to Neural Mix mute, four to solo, with Shift+STEMS pads 1–4 mapped to exclusive solo; REC maps to `turntable{deck}.recordSample` (`docs/full-midi-test.md`; `src/full/catalog.rs`). | Neural Mix mute/solo is a useful custom performance layout, but not Traktor's modifier-plus-volume/filter behavior. The public Djay sample-recording workflow is performed in Sampler > Edit > pad menu > Record new sample; it records a selected Deck/Microphone source while held and saves to My Samples. That does not prove `recordSample` starts that UI workflow, picks a particular sample slot, or reproduces Traktor's quantized Pattern Recorder. Those native-action semantics require direct testing. |
| **Shift+Sync / tempo range** | SYNC synchronizes the deck; Shift+SYNC locks/unlocks that deck's tempo fader (§5.1, p. 22; §7.7, pp. 54–56). This is a tempo-fader lock, not a tempo-range selector. | `sync` maps only to `turntable{deck}.bpmSync` and declares no shifted target (`src/full/catalog.rs`). | Djay Sync is mapped, but the factory Shift+Sync tempo lock is not. Do not describe the bridge as changing tempo range with this combination. A separate lock action/equivalent has not been established. |
| **Crossfader startup and channel assignments** | The quick-start procedure explicitly sets the crossfader to the left-most position before mixing (§6.1, p. 32). Front selectors assign each channel left/right/center-through and the front curve switch selects constant/smooth/sharp (§7.3, pp. 48–49; Overview, p. 30). This is a setup recommendation, not a statement that the controller boots at the left edge. | The crossfader sends its position to `mixer.crossfade`. `front_panel` starts selector caches as unknown and emits a selector pulse when it first observes each decoded selector state (`src/full/mod.rs`, `Translator::default` and `front_panel`, around lines 90–105 and 623–650). | Crossfader position is controllable, and selector state is not simply ignored at bridge startup: the first observed snapshot causes a pulse. Because that pulse is momentary, it can still be lost if Djay has not attached its mapping listener yet. The evidence supports a startup-listener race as a possibility, not a missing initialization pulse. Reapplication after the MIDI mapping/listener is ready or on reconnect is a proposal requiring a lifecycle test; no extra initialization or device I/O was done here. |
| **Browse direction** | The Browse encoder scrolls/selects tracks and its press loads the selected track (§5.1.1, p. 23; §7.1, pp. 39–40). While Preview is held, turning Browse seeks within the preview (§7.15, p. 68); the manual does not state clockwise/forward polarity or which direction moves down/up the browser list. | The relative encoder routes to `musicLibrary.libraryRotary` (Shift: `sectionRotary`) (`src/full/catalog.rs`). | Browser navigation is mapped, but direction polarity in Djay is not proven by the action name or the manual. Verify clockwise/down-list and counterclockwise/up-list on the running target; do not silently flip the relative sign based on labels. |
| **FX bank assignment and enable** | FX Unit 1/2 are the left/right banks. Each channel's FX Assign buttons route either/both units to that channel (§5.2.1, p. 28; §7.13, pp. 66–67). FX ON buttons 1–4 enable/disable individual effect parameters in the selected unit (§5.3, p. 29; §7.13, p. 70). | The two physical strip selectors choose the deck targeted by each Djay FX strip. FX ON maps to `fxActive`; buttons 1–3 map to `fx1Enabled`–`fx3Enabled` (Shift selects next effect). Mixer FX controls map to per-deck instant FX (`src/full/catalog.rs`). | Djay can approximate effect-bank control on a selected deck, but this does not reproduce Traktor's channel-to-shared-FX-unit audio routing. The current bridge intentionally selects a MIDI control target rather than connecting mixer channels to a common FX bus; it also exposes three manual effects, not Traktor's four per-unit buttons. A fourth button and true assignment semantics are not established equivalents. |

## Historical shared loop-length and beat-jump proposal

The user clarified the factory encoder coupling: turning LOOP with no active
loop selects the length for both subsequent loop activation and MOVE beat
jumps. Selecting 8 beats makes LOOP press create an 8-beat loop, or MOVE turn
jump 8 beats without activating a loop. Selecting 32 beats makes those actions
use 32 beats instead. With a loop active, MOVE shifts that loop by the selected
length.

That factory coupling was the earlier proposal, not a requirement of the
implemented increment. The current custom Djay layout deliberately keeps
native loop size independent from native jump size.

## Current encoder overlay

Full-profile physical naming now corrects all rotations and only left
presses. LOOP turn uses `autoLoopDurationRotary`, and press uses
`autoLoopOnOff`; shifted LOOP uses `autoLoopMoveRotary` and `reloop`.
Manual loop in/out is unmapped. The user observed Shift+LOOP moving inactive
loop regions too, apparently by loop length; Reloop restores a stored range.
Normal MOVE sends CC 2 on channels 1-4 (`skipRotary`) for jumping with or
without an active loop. MOVE press emits no MIDI and toggles per-logical-deck
size-select mode without a timeout. In selection mode, unshifted MOVE sends
CC 7 on channels 1-4 (`skipDurationRotary`), changing native jump size
immediately. A second press exits, rather than saving a deferred value.

Shift+MOVE always jumps one beat per detent in either direction, even during
selection, preserving chosen jump size. Runtime sends note 5 forward or
note 14 backward only on shifted channels 9-12; the native catalog also has
base-channel aliases. A/B/C/D sizes remain independent native Djay values,
not bridge counters or shared-size resets. Local mode flags are independent
and start off on bridge restart.

The selected deck selector pulses on a 1.2-second palette-brightness cycle
with a dark trough while selecting. Hidden decks retain their mode without
flashing inactive selectors. Screen and motor traffic remain excluded;
the minimal profile is unchanged.

The user mouse-verified beat jump moving an active loop, but fresh MIDI/native
and physical LED checks remain pending. Reinstall a newly generated mapping
under a new filename, preserving custom maps. See the focused
[live checks](full-midi-test.md#encoder-layout-live-check).

## Djay public support evidence

The following current Algoriddim support pages were retrieved successfully on
October 6, 2026. They establish general Djay capabilities, not every
version-specific MIDI action name or the behavior of the bridge's generated
native key paths:

- [Mapping a MIDI device](https://help.algoriddim.com/user-manual/djay-pro-mac/midi/mapping):
  custom mapping targets include Decks 1–4, Mixer, Music Library, Sampler,
  Microphone, and General; rotary formats, button modes including Hold, pickup,
  jog reaction, and MIDI Out are available. MIDI mapping requires PRO.
- [Audio devices](https://help.algoriddim.com/user-manual/djay-pro-mac/settings/audio-devices):
  internal mode lists Main, Pre-cue, Booth, Microphone, and Recording Input;
  external mode lists separate Deck 1–4, Sampler, and Looper outputs plus
  Recording Input. This is audio routing documentation, not proof of a hardware
  EXT Live Input selector.
- [Recording your own samples](https://help.algoriddim.com/user-manual/djay-pro-mac/dj-tools/performance-tools/recording-samples):
  Sampler > Edit > pad menu offers Deck 1, Microphone, or Deck 2 as recording
  sources; hold to record, release to stop, then save to My Samples. The UI
  workflow does not establish what a mapped `recordSample` action does.
- [Sampler overview](https://help.algoriddim.com/user-manual/djay-pro-mac/dj-tools/performance-tools/sampler):
  Sampler plays samples and its editor can record short samples from Deck 1 or
  Deck 2.
- [Previewing tracks](https://help.algoriddim.com/user-manual/djay-pro-mac/music-library/advanced/preview):
  browser preview plays through the pre-cue/headphone path. It does not document
  a MIDI seek action for a held preview.

## Explicitly deferred

- **On-device pad legends/screens:** deferred. The full bridge does not implement
  screen feedback or dynamically printed pad legends; the current pad meanings
  are software-selected layers.
- **A+C+B+D settings chord:** deferred. A and C are ordinary left-deck focus
  controls and B and D are ordinary right-deck focus controls (manual §7.16,
  p. 69; bridge selectors in `src/full/mod.rs`). Consuming all four as a settings
  chord conflicts with their normal deck-selection role unless a distinct,
  tested gesture is designed.
- **Encoder and button physical pairing:** subsequently verified by the
  isolated [physical capture](encoder-capture.md), including clockwise polarity
  and Shift. The full-profile naming correction is implemented; fresh musical
  routing and selector-LED validation remain pending.
- **EXT routing, preview scrubbing, REC action semantics, Reverse+Slip hold
  behavior, tempo lock, Djay FX assignment semantics, and startup selector
  reapplication:** none is promoted from a catalog key or generic support-page
  description to verified end-to-end behavior without the corresponding
  Djay-version test.

No bridge code, mapping, hardware state, or device I/O was changed for this
comparison. The only artifact produced is this report.
