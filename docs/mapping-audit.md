# Mapping audit and beta reference

October 6, 2026. The portable app worked on the user's DJ laptop. This audit
addresses the subsequent control notes; it does not change the executable or
installed mappings.

The [issues checklist](issues.md) is the current status record. It includes
the user's live-test confirmations and is updated as issues are completed.
Pending notes and proposals below may describe older revisions.

## Master references

- [Djay inputs, outputs, and assigned MIDI addresses](djay-action-reference.md):
  version-scoped software action/state catalog, including unused entries.
- [Bridge-owned functions](bridge-function-reference.md): local modes, routing,
  message conversion, and LED rendering, separated from hardware-local audio.
- [Factory Traktor comparison](traktor-djay-comparison.md): the supplied
  S4 MK3 manual, page references, and compatibility gaps.
- [Verified encoder capture](encoder-capture.md): all MOVE/LOOP fields, press
  masks, directions, and Shift verification. Rotation names are reversed on
  both sides; press names are reversed only on the left.
- [LED settings and limits](led-feedback.md) and
  [macOS installation](../macos/README.md).

Software input means a command sent **to Djay**. Software output means state
sent **from Djay** to the bridge. Neither means an audio input/output.

## Findings and proposed changes

### Calibrated jog mapping preferences

The user's chosen defaults are scratch Speed 2.7% / Reaction 150% and
pitch-bend Speed 2.7% / Reaction 17%. Speed is exported as
`rotarySensitivity`, Reaction as `rotaryAcceleration`, on all four decks.
The app's Jog tab saves a separate jog configuration and generates customized
mapping exports; selecting that new mapping in Djay is required.
Runtime counts, touch ownership, seek, and MIDI addresses are unchanged.
Released-backspin behavior and haptic tension remain open.
See [configurable controls](configurable-controls.md) and
[the measured jog comparison](jog-wheel-investigation.md).

### Implemented LOOP/MOVE layout

The subsequent REV/Browse increment maps REV to native `reverseHold`,
preserving press ownership through deck/Shift changes and shutdown. No
automatic Slip/Flux behavior was added. Browse's normal and shifted rotary
bindings now carry `flipped=true` in the native mapping; the decoder and
wire polarity remain unchanged, avoiding a double inversion. These two
changes pass 72 Rust tests but still require live confirmation.

Crossfader startup now caches the physical switch snapshot until the mapping
has returned a raw playback output, including a paused/zero value. It then
sends the latest four assignments and curve once; later reports send only
physical changes. The combined increment passes 74 Rust tests.
The signal establishes feedback activity, not acknowledged consumption of
input MIDI. Automatic same-process mapping reconnect detection remains
unavailable; restart the bridge after selecting/reconnecting a mapping to
initialize a fresh session. No timed retry or periodic selector replay is used.

This increment applies only to the full profile. Physical naming is corrected
for all four rotations and only the left presses; right presses and the
minimal profile are unchanged.

LOOP turn sends `autoLoopDurationRotary` for Djay's native loop size, and
LOOP press sends `autoLoopOnOff`. Shifted LOOP uses `autoLoopMoveRotary`
on turn and `reloop` on press. Manual loop in/out is unmapped.
The user observed Loop Move affecting inactive loop regions too, apparently
by their loop length. This supersedes the earlier claim that it does nothing
while inactive: an unmoving playhead did not establish unmoving loop bounds.

Normal MOVE sends CC 2 on channels 1-4 to `skipRotary`, for jumping with or
without an active loop. MOVE press emits no MIDI. It toggles a local
size-select mode for the logical deck, with no timeout. In that mode,
unshifted MOVE sends CC 7 on channels 1-4 to `skipDurationRotary`, immediately
changing Djay's native jump value. The second press exits, not saves a
deferred value.

Shift+MOVE always jumps one beat per detent in either direction, including
while selecting: note 5 forward and note 14 backward on channels 9-12.
The native catalog also contains base-channel aliases for these actions;
runtime Shift routing uses only the shifted channels. Fixed jumps preserve
the chosen jump size.

A/B/C/D retain independent native Djay loop and jump values, not bridge
counters or shared-size resets. Only the selection-mode flags belong to the
bridge; all start off after a bridge restart. The selected deck's selector
LED uses a 1.2-second palette-brightness pulse with a dark trough while
selecting. Hidden decks retain their mode without flashing inactive buttons.
Screen and motor traffic remain excluded.

The user verified with the mouse that beat jump moves an active loop.
Fresh MIDI/native-action and physical selector-LED checks are still pending;
that mouse result is not a physical encoder pass. Reinstall a newly generated
mapping under a new filename and preserve custom mappings under their existing
names. See the focused [live checks](full-midi-test.md#encoder-layout-live-check).

### Encdr v0.7.2: jog-ring upgrade candidate

The user flagged [Encdr v0.7.2](https://github.com/RufusIbiza/Encdr/releases#release-v0.7.2).
The release notes and tagged descriptor/helper source were inspected. The
bridge still pins revision `5d7a689ca5789161b3efd1b2049d7e54504fe881`;
this review does not upgrade it.

| Upstream addition | Relevance to this bridge | Remaining check |
| --- | --- | --- |
| Separate left/right ring groups and 32-segment arrays | Could replace custom low-level ring plumbing and enable richer patterns. | New IDs are `left_wheel_leds` / `right_wheel_leds`, not the old `wheel_leds` selector used by our descriptor filtering. Review integration rather than just changing the pin. |
| Absolute physical jog positions | `left_jog_pos` and `right_jog_pos` expose platter angle from Report 3. | Physical angle is not Djay track position or playback phase; stationary unpowered platters do not automatically track a playing song. |
| Needle, flash, addressable-ring and tracker helpers | Candidate for wheel-following indication during manipulation and configurable ring presentation. | Our physical tests established modes 2/3. Mode 5 previously failed to produce the requested result; upstream support is a reason to retest, not a hardware pass on this S4. |
| Live wheel/ring synchronization APIs | Could follow manual motion without software position feedback. | Preserve the existing playback-driven chase unless a different behavior is chosen; physical synchronization does not solve backspin audio sensitivity. |

The tagged descriptor still declares button payload size 25 and jog payload
size 48, while our corrected native framing uses 22 and 59. Its slider maxima
remain 65535 and the original pad masks remain present. Retain our verified
framing, 4095 slider scaling, pad-order corrections, and snapshot transport
until each is independently superseded. Check ring initialization `0x30`,
output-report transport, report-ID offsets, and packed palette handling before
adopting the high-level output path. Keep motor commands disabled.

Recommendation: include a separately verified Encdr upgrade in the implementation
plan, but do not treat it as fixing the mapping bugs, jog-feel/backspin limits,
or missing Djay preview/audio capabilities.

Sources:
[tagged S4 descriptor](https://github.com/RufusIbiza/Encdr/blob/v0.7.2/encdr/descriptors/ni_kontrol_s4_mk3.json),
[hardware reference](https://github.com/RufusIbiza/Encdr/blob/v0.7.2/docs/hardware/ni_kontrol_s4_mk3.md),
and [jog helpers](https://github.com/RufusIbiza/Encdr/blob/v0.7.2/encdr/src/core/jog_ring.rs).

The following table is the historical beta audit, before this encoder increment.
Its MOVE/LOOP row records the earlier failure and proposal; the implemented
layout above supersedes that row. Other proposals remain separate.
User-reported failures are recorded as failures, even where source code suggests
an intended action exists. Existence of a key is not an end-to-end pass.

| Note | Current behavior / evidence | Proposed direction and required check |
| --- | --- | --- |
| FX SELECT Filter / Filter Reset | Channel FX SELECT chooses one deck locally. Central Filter sends `turntableN.resetFilter` to that selected deck; the channel's continuous quick-FX knob is mapped to `turntableN.filter`. The button does not select a new filter mode or reset all decks. | Keep a clearly named reset-to-neutral action unless a different Mixer-FX mode is chosen. Verify neutral filter and knob pickup behavior; button LED is momentary, not persistent filter state. |
| MOVE / LOOP swapped; beat jump fails | The catalog intends Loop rotation to set loop length and Move to move a loop; Shift+Move sends `skipRotary`. The user finds the physical knobs reversed. Factory MOVE should jump the track without a loop, or move the active loop; LOOP should set length and press to engage. Current shifted key/volume behavior also differs from Traktor. | Physical capture now verifies all four rotation names reversed, left press names reversed, and right press names correct; clockwise is +1 on every knob, including Shift. Correct naming at the descriptor boundary; do not also swap action assignments or right press names. Then implement context-dependent MOVE from reliable loop state, with a working beat-jump action when not looping. A plain knob swap alone does not supply that conditional behavior. Test press, turn, Shift, loop inactive/active, and all four decks. |
| Jog too sensitive, backspin too slow | Current mapping sensitivities: bend 7, scratch 25, seek 20. Bridge restores all counts and splits large relative deltas rather than discarding them; no total-speed cap is established in that translator. | Separate slow-touch gain from fast-motion response. Measure slow/fast raw counts, emitted deltas and Djay response before deciding whether native jog reaction or a velocity-dependent bridge curve is needed. Merely lowering the gain can make the maximum-speed complaint worse. Confirm touch/rim/Shift behavior, reversal, and Slip with backspins. |
| REV should be momentary | Bridge emits both press and release, but the generated Reverse assignment uses native `turntableN.reverse` (`button-toggle`) rather than the declared hold action. Factory REV is held and also temporarily uses Flux. | Replace the toggle target with declared `turntableN.reverseHold` (`button-hold`, linked to reverse state). No invented exporter flag is needed. Test press/release and held ownership across deck changes. Treat temporary Slip as a separate design choice requiring real feedback and restoration of the prior state; do not blindly toggle Slip twice. |
| EXT / mic / line inputs | EXT is decoded but has no assignment. Djay supports microphone and recording-input routing; external mixer mode primarily routes deck outputs separately. None of this proves a Traktor-style live input replacing each deck, or S4 preamp/source selection over HID. | Keep EXT unassigned until the S4's CoreAudio input channels and firmware switching are established. Distinguish microphone on/off, external recording source, deck live-input replacement, and hardware MIC/LINE/PHONO choice. Bridge-owned audio forwarding would be a separate audio subsystem, not a MIDI mapping fix. |
| Preview encoder scrubbing | Library Play toggles preview; encoders have no preview-dependent routes. Factory Traktor holds Preview and scrubs with Browse, not LOOP. The requested LOOP scrub is a custom Djay adaptation. | Establish a native preview seek action and playback-state output first. Prefer actual preview state over guessing from button presses, because UI actions can start/stop it too. A held Preview modifier could avoid needing persistent state, but changes the requested interaction and needs agreement. Do not redirect normal deck seek to pretend it scrubs preview. |
| MUTE beside STEMS | Both select the same local Stems layer. Pads 1-4 mute Neural Mix channels 1-4; pads 5-8 solo them; Shift+1-4 uses exclusive solo. MUTE itself does not mute a deck. | Explain the alias now. For closer Traktor behavior, consider MUTE as a held pad modifier instead of a duplicate mode button; decide the normal STEMS actions before replacing the current useful mute/solo layout. |
| REC / Record Sample | REC addresses the selected deck's `recordSample`. It is not mix recording or a Traktor Remix pattern recorder. Public Djay documentation describes hold-to-record from Sampler Edit, then storing in My Samples; that UI flow does not establish every native `recordSample` side effect. | Test the native action with a local test track, inspect My Samples and sampler editing, and identify source/slot/hold requirements. Do not label it as automatically filling the next empty sampler pad without evidence. |
| Shift+Sync tempo-fader range | Sync has no shifted action, so holding Shift currently still sends ordinary Sync. The factory manual specifies tempo-fader lock on Shift+Sync, not range cycling. | Range cycling is a sensible custom enhancement, not literal factory parity. Installed presets use `application.tempoSliderRangeNext` (Switch Tempo Slider Range), so there is a concrete candidate. Confirm whether its scope is global or follows the selected deck before adding a shifted binding. Define the supported ranges and wrap order from Djay's actual choices, not an invented list. |
| Crossfader switches THRU at startup | The bridge already sends each switch's first observed position, then caches it. A pulse sent before Djay attaches can be lost; restarting/reselecting a mapping has no explicit replay. User confirms toggling the hardware fixes it. | Reapply current assignments and curve when the mapping is ready/reconnected, or offer an explicit reapply operation if reliable readiness is unavailable. Test bridge-first, Djay-first, configuration change, reconnect, and pre-set THRU without moving switches. Avoid continuously replaying selectors or overriding subsequent user choices without policy. |
| Browse reversed | Raw relative deltas are sent to `libraryRotary` and shifted `sectionRotary`; neither binding is flipped. The user reports inverted behavior. | Correct polarity once at the decoder or mapping layer. Verify clockwise/down-list and counterclockwise/up-list on both sides, including Shift; avoid two corrections cancelling each other. |
| Leftmost FX button duplicates slot 1 | Leftmost is mapped to `fxActive`; adjacent button maps `fx1Enabled`. They have different keys, but the user observes the same effect. Different key names are not proof of independent bank enable in the selected Djay layout. | A master bypass for all three slots is the best proposed use. First verify native bank enable semantics. If it really aliases slot 1, use explicit multi-slot state/set semantics and restore prior slot choices; three blind toggles can leave a mixed bank mixed. Check both strips and independently selected target decks. |

### Additional open issue: default Djay token colors

The user added this as the ninth remaining issue after confirming the
LOOP/MOVE, Reloop, REV, Browse, and crossfader startup fixes.
Some Djay color-token correspondences were corrected in bridge Settings,
but not all tokens have been checked. Verify the complete correspondence,
then correct the bundled default LED palette in `examples/led-config.json`
so fresh installations use the right colors. Preserve existing saved
preferences; the partially corrected values are evidence to start from,
not a fully calibrated replacement palette. This concerns token-to-LED
color translation, not MIDI control addresses.

### Historical shared-length proposal, superseded for this increment

The earlier factory-behavior clarification proposed one selected length for
LOOP activation and MOVE jumps:

| Selected length, no loop active | Press LOOP | Turn MOVE |
| --- | --- | --- |
| 8 beats | Activate an 8-beat loop | Jump forward/backward 8 beats |
| 32 beats | Activate a 32-beat loop | Jump forward/backward 32 beats |

This table is historical, not the current contract. The implemented layout
keeps native loop size independent from native jump size. MOVE selection
changes the jump value immediately, and normal MOVE uses that value even
when moving an active loop. No bridge-owned shared length is required.

### How Djay handles audio inputs

Normal internal mixing uses the controller/interface as a CoreAudio device:
choose main and headphone outputs, optional booth, microphone input and recording
input in Audio Devices. External mixer mode sends separate deck/sampler/looper
outputs to a hardware mixer. It is not synonymous with switching a software deck
to a live S4 channel.

Sources: [Djay Audio Devices](https://help.algoriddim.com/user-manual/djay-pro-mac/settings/audio-devices)
and [Recording your own samples](https://help.algoriddim.com/user-manual/djay-pro-mac/dj-tools/performance-tools/recording-samples).
The installed-version MIDI catalog and the generic public manual may expose
different levels of detail; unresolved native action semantics stay unresolved.

## Why several things can say C2

A pitch label alone is not a MIDI address. The identity is:

**endpoint + direction + channel + message type + numeric note/CC**.

The mapping file numbers channels from zero; user-facing MIDI channels are
1-16. Decks A/B/C/D use channels 1/2/3/4, shifted actions 9/10/11/12,
mixer controls 5, and deck-selection commands 6. Raw playing/meter/loading
feedback uses channel 7. A Note and a CC with the same number are different
messages; a feedback address can intentionally match an input address.

The same note number is deliberately reused across deck channels. If your
editor calls note 36 "C2", for example, the right FX-strip ON action uses note
36 on whichever deck channel its local target selects. Octave labels vary
between applications; use the decimal number in the master reference instead
of assuming every editor's C2 means 36. If "C2" is shorthand for CC 2 rather
than a pitch label, message type is even more important.

To swap a function safely, identify the physical event first, then change the
normal and shifted assignments for all relevant decks and press/rotation
variants. A manual Djay edit does not change bridge-owned mode/target routing.
The app's installer exports the generated catalog, not your edited configuration;
reimporting it will not preserve those custom edits inside that new mapping.
Keep your edited configuration separately named.

The bridge can already implement conditional routing (Shift, pad modes and
touch ownership). Preview-aware routing does not require relying on a general
Djay conditional-mapping feature; the unresolved dependency is a real preview
seek command and, for toggle-based preview, reliable preview state.

## Beta user quick guide: current build

1. Start the menu-bar app and full bridge before opening Djay. Import its mapping
   and select it for **S4 MK3 MIDI Full**; do not run a second CLI bridge.
2. A/C and B/D select physical deck focus. Mixer channel strips remain fixed
   to their respective decks; changing deck focus does not move the mixer faders.
3. HOTCUE pads set/jump to cues; Shift clears them. SAMPLES uses shared A/C
   and B/D banks; Shift stops a sample. STEMS/MUTE select four mutes and four
   solos; Shift on the first four pads is exclusive solo. Stem numbers are
   native channel indices; confirm content with Djay's Neural Mix layout.
4. Channel FX1/FX2 select the left/right strip's target deck. Channel FX SELECT
   chooses the central instant-FX/reset target. These are control targets, not
   Traktor shared audio buses.
5. Use the implemented encoder layout above and its pending live checks.
   Backspin, REV hold, THRU startup, and Browse direction remain beta issues
   in the historical findings table. EXT and screens are not implemented.
6. Change LED appearance in Settings, Save, then restart the bridge. Ring chase
   is decorative; green flash indicates a returned loop state. Local mode
   lamps do not prove the software changed its visible pad panel.

For useful reports, include app/mapping name, Djay version, deck/side, pad mode,
Shift state, press versus rotation, expected result, and actual result. That is
more reproducible than a pitch label alone.

### Current STEMS/MUTE pad legend

These names come from the installed four-channel Neural Mix action labels.
They are not a claim that every track makes all four stems available.

| Pad number | Normal action | Shift action |
| --- | --- | --- |
| 1 | Drum mute | Exclusive drum solo |
| 2 | Bass mute | Exclusive bass solo |
| 3 | Harmonic mute | Exclusive harmonic solo |
| 4 | Vocal mute | Exclusive vocal solo |
| 5 | Drum solo | Same drum solo action |
| 6 | Bass solo | Same bass solo action |
| 7 | Harmonic solo | Same harmonic solo action |
| 8 | Vocal solo | Same vocal solo action |

Pads are the bridge's numbered physical order; the same layout is used on both
sides. Shift does not create a new action for pads 5-8 in this mode.

## Deferred screen ideas

| Idea | Proposed behavior | Design constraints for later |
| --- | --- | --- |
| Pad-mode legends | Display eight pad meanings for the active HOTCUE/SAMPLES/STEMS layer, independently of Djay's unchanged visible pad page. | Mirror the bridge's real mode/target and held ownership; verify stem labels/availability. The current screen protocol and renderer are not implemented. |
| Quick settings menu | A deliberate gesture opens an encoder-navigated on-device menu for settings such as jog feel. | A+C+B+D normally changes deck selection; a chord needs unambiguous detection and a policy for suppressing/committing those normal actions. Define persistence, apply/restart requirements, and an exit gesture before implementation. |

Both screen ideas stay deferred until the mapping/control work above is settled.
No UI chord or display command was added by this audit.

## Suggested order after design approval

1. Live-check the implemented encoder layout; investigate Browse polarity separately.
2. Momentary Reverse and startup selector reapplication.
3. Jog-feel/backspin measurements and separate tuning controls.
4. Confirm native REC, FX bank bypass and tempo-range actions.
5. Preview-state/seek feasibility and external-input feasibility.
6. Revisit pad layouts; then tackle the deferred screens.

Implementation should produce matched bridge and generated-mapping revisions
where MIDI behavior changes. Preserve existing user-edited mappings.
