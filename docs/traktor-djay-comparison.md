# S4 MK3 factory behavior and Djay bridge comparison

This report compares the TRAKTOR KONTROL S4 MK3 Manual (software version 3.2,
July 2019) with the current full-profile bridge catalog and public Djay Pro
support documentation. It describes the bridge's intended MIDI actions, not a
claim that every control in the expanded profile has passed a live hardware
test. Printed page numbers below refer to the page number in the manual footer;
the supplied PDF has three preceding cover/front-matter pages.

For the current workflow rather than the historical comparison table, use the
[user guide](user-guide.md). Current overlays below supersede the older rows.

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

The EXT row below predates current-source support. The
[EXT overlay](#current-source-ext-overlay) supersedes its unassigned status,
not its historical evidence.

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

Current-source FX browsing adds a custom Djay gesture: Shift + a strip's
parameter knob selects previous/next effects in its slot using relative
movement. Release Shift for parameter pickup; dry/wet and the old Shift +
button next-effect shortcut remain. This requires the matching new mapping.
It is an ergonomic adaptation for finite-travel knobs, not a claim of Traktor
parity. See [manual FX operation](user-guide.md#manual-fx-strips).

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

Shift+MOVE selects one beat, sends the full signed detent count through normal
deck-channel CC 2, then restores the last confirmed jump size using normal-channel
notes 64-73. Notes 5/14 are retained mapping aliases, not the runtime shortcut.
If feedback is missing or conflicting, no previous size is guessed: the one-beat
jump still occurs but a prior size cannot be restored. A/B/C/D sizes remain
independent native Djay values, not bridge counters or shared-size resets.
Local selection flags are independent and start off on bridge restart.

The selected deck selector pulses on a 1.2-second palette-brightness cycle
with a dark trough while selecting. Hidden decks retain their mode without
flashing inactive selectors. This encoder increment adds no screen or motor
traffic; the separate hardware-tension increment below does add motor output.
The minimal profile is unchanged.

The user mouse-verified beat jump moving an active loop. Native MIDI/GUI
jumping, confirmed-size restoration and LED-report checks are recorded in
[control validation](controls-state-sync-validation.md); a complete physical
gesture matrix is a separate check. Reinstall a newly generated mapping under
a new filename when bindings change, preserving custom maps. See the focused
[live checks](full-midi-test.md#encoder-layout-live-check).

## Current hardware tension and mixer EQ overlay

The normal full-profile bridge now supports saved hardware wheel tension:
one Assist / Neutral / Brake setting, with independent per-wheel movement
and touch response. The user confirmed the focused integration checks on
October 20, 2026. This is not playback-driven TT rotation, haptic cue points,
or full Traktor jog parity. See [the integration record](jog-tension-integration.md).

Mixer Filter now arms/cancels a one-shot deck selection; the next channel ON
toggles only that deck's native Neural Mix EQ without changing the Instant FX
target. Outside selection, ON still targets Instant FX. ON reports EQ mode
in white/cyan, brightens for the target deck, and temporarily shows an active
effect's color. Idle Filter is dim cyan. This custom workflow replaces filter
reset, not Traktor's mixer-FX selection model. Live MIDI feedback and general
user acceptance are recorded in [the focused check](neural-mix-eq-check.md).

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

## Current-source EXT overlay

The full profile maps physical mixer EXT1-4 to fixed Djay decks 1-4,
independent of deck focus and FX selection. Each new unshifted press selects
Internal if actual THRU feedback is true, or Thru if false. With unknown
feedback there is no switch and the lamp stays dark. Either Shift suppresses
software switching, preserving hardware Shift+EXT input-type selection.
A held press is consumed until release, even if Shift is released or feedback
arrives later. The lamp lights only from true per-deck THRU feedback;
false/unknown stay dark regardless of dim-idle settings.

This uses Djay's deck DVS THRU route, not bridge-owned audio forwarding.
Enable DVS manually and leave it on; input assignments remain GUI setup.
The confirmed USB pairs are A 1-2 PHONO/LINE, B 3-4 PHONO/LINE,
C 5-6 LINE/MIC C and D 7-8 LINE/MIC D. THRU feedback doesn't establish
global DVS enable or audible input.

Install a newly generated matching mapping for this current-source addition;
it is not a newly published release. [EXT validation](ext-input-validation.md)
confirms all four indicator addresses, physical switching, mouse-driven
feedback and Shift suppression. Channel 4's manually selected
route is confirmed, but remaining mixer-control coverage, phono-stage behavior
and four simultaneous sources still need validation. See the
[external-audio workflow](user-guide.md#external-audio-through-a-deck).

## Explicitly deferred

- **Additional on-device pages and settings:** current source implements Info
  screens and reactive STEMS/MUTE legends, with optional FX pickup warnings.
  Hotcue/sample pad grids, artwork, waveforms, playback-position tracking and
  an on-controller settings menu are not implemented. See [screens](screens.md).
- **A+C+B+D settings chord:** deferred. A and C are ordinary left-deck focus
  controls and B and D are ordinary right-deck focus controls (manual §7.16,
  p. 69; bridge selectors in `src/full/mod.rs`). Consuming all four as a settings
  chord conflicts with their normal deck-selection role unless a distinct,
  tested gesture is designed.
- **Encoder and button physical pairing:** subsequently verified by the
  isolated [physical capture](encoder-capture.md), including clockwise polarity
  and Shift. The full-profile naming correction is implemented; fresh musical
  routing and selector-LED validation remain pending.
- **Preview and navigation:** held Preview+BROWSER scrubbing, Shift+BROWSER
  source selection and Star's Switch Library Table route are implemented.
  Preview toggle state can still need mouse-close recovery; there is no
  latched-preview mode. See [navigation](user-guide.md#browse-preview-and-load).
- **Startup and feedback:** physical volume/EQ application, FX pickup and
  confirmed-size restoration are implemented. Musical-key feedback ordering
  is fixed and physically verified for replacement/eject. Reconnection still
  requires an explicit bridge restart.
- **Remaining differences:** broader external-audio validation remains pending
  as above; EXT switching and indicator acceptance are complete. REC's end-to-end recording semantics,
  automatic Reverse+Slip linking, Traktor-style tempo lock and true shared
  FX-bus routing are not established equivalents. Shift+Sync changes the
  global range; channel FX1/FX2 choose control targets rather than audio buses.

The original comparison was documentation-only. Current overlays are maintained
alongside the implementation; dated validation records identify what was
actually exercised.
