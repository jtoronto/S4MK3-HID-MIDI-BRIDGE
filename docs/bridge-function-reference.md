# Bridge-owned functionality

Snapshot: October 6, 2026, full profile bundled with the macOS app. This table
describes the implementation, not a promise of factory Traktor compatibility.
The user confirmed the app runs on the actual DJ laptop; subsequent control
notes are tracked in [mapping-audit.md](mapping-audit.md).

## Logic implemented inside the bridge

### Current encoder overlay

The encoder increment is full-profile only. Descriptor normalization corrects
all rotation names and only left press names. LOOP turn/press use native
`autoLoopDurationRotary` / `autoLoopOnOff`; shifted LOOP retains
`autoLoopMoveRotary` on turn and uses `reloop` on press. Manual loop in/out
is unmapped. The user observed native Loop Move also moving inactive loop
regions, apparently by their loop length; the bridge supplies no size.

MOVE normally sends CC 2 on channels 1-4 to `skipRotary`, whether a loop is
active or not. Its press emits no MIDI and toggles size-select mode per
logical deck, with no timeout. Unshifted MOVE in that mode sends CC 7 on
channels 1-4 to `skipDurationRotary`: native jump size changes immediately,
and the second press only exits. Shift+MOVE always emits a fixed one-beat
jump per detent, note 5 forward or note 14 backward on channels 9-12,
including during selection. Base-channel aliases in the native catalog
aren't used by runtime Shift routing.

Djay owns the independent A/B/C/D jump and loop values; the bridge keeps
only independent mode flags, initialized off on restart. No shared-size
counters or resets are used, and fixed Shift jumps preserve chosen jump size.
The selected deck selector pulses on a 1.2-second palette-brightness cycle
with a dark trough while selecting. Hidden deck modes persist without
flashing inactive buttons. No screen or motor traffic is included.

These routes are implemented, not physically validated here. The user
mouse-verified beat jump moving an active loop; fresh MIDI/native and LED
checks remain pending. Reinstall the generated mapping under a new filename,
preserving custom maps. See [live checks](full-midi-test.md#encoder-layout-live-check).

### Historical base snapshot

The table below retains the earlier snapshot and source line references.
The encoder and selector-mode overlay above supplements it; it doesn't
establish which revision is installed in an existing app bundle.

These decisions are not ordinary Djay MIDI Learn assignments. Where a row
ultimately controls playback, Djay still performs the audio operation.

| Bridge function | Human-readable behavior | MIDI/software consequence | Source |
| --- | --- | --- | --- |
| HID decoding and descriptor corrections | Read native S4 reports without impersonating another controller. Correct payload sizes, 12-bit slider ranges, pad bit order, and added EXT/headphone fields. | Physical input becomes named events; EXT is decoded but unassigned. | `src/main.rs:380`, `src/main.rs:422` |
| Full-profile snapshot transport | Request buttons/touch before jog movement; discard stale interrupt copies of those reports. | Avoid replaying old touch/motion; ordinary slider interrupts remain accepted. | `src/input.rs:5` |
| Physical deck selection | Left side selects A/C; right side B/D. Defaults A/B. | Route deck controls on human channels 1-4; selection also emits application deck-selection notes on channel 6. | `src/full/mod.rs:101`, `src/full/mod.rs:299` |
| Shift layer | Each physical side has its own held Shift state. | Bindings with a shifted action use channels 9-12; other bindings retain normal channels. Shift is not itself a Djay conditional. | `src/full/mod.rs:299`, `src/full/mod.rs:651` |
| Pad modes | HOTCUE, SAMPLES, STEMS choose which eight actions the pads address. Mode is per physical side, not per software deck. | Normal notes 40-47, 48-55, or 56-63 respectively. | `src/full/mod.rs:299`, `src/full/catalog.rs:359` |
| MUTE alias | MUTE selects exactly the same layer as STEMS. | It does not mute the deck or all stems by itself. | `src/full/mod.rs:343` |
| Pad slot order | Correct raw bit masks to physical pad order. | Numbered pads address slots 1-8 rather than the descriptor's original order. | `src/main.rs:434` |
| Sampler bank addressing | A/C target sampler bank 1; B/D target bank 2. | These deck pairs share sampler players; they do not have four independent banks. | `src/full/mapping.rs:221` |
| FX strip target selection | Mixer channel FX1 selects the left strip's deck; FX2 selects the right strip's deck. Defaults A/B. | Changes control destination, not a Traktor-style shared FX audio bus. | `src/full/mod.rs:410` |
| Mixer quick-FX target | Channel FX SELECT selects the target for central FX 1-4/Filter buttons. Defaults A. | Filter resets that selected deck's filter, not all four filters. | `src/full/mod.rs:410`, `src/full/mod.rs:509` |
| Dry/wet fan-out | One hardware dry/wet knob emits three CCs. | Sets wet/dry for FX slots 1, 2, and 3 on the strip's target deck. | `src/full/mod.rs:487`, `src/full/catalog.rs:185` |
| Global Quantize fan-out | One button is sent to all four decks. | Toggles each deck separately; it is not an absolute "make all on" operation. Mixed initial states can remain mixed. | `src/full/mod.rs:509` |
| JOG/TURNTABLE local mode | JOG chooses nudge/CD behavior; TURNTABLE chooses touch scratching. Default is touch scratching. | Changes routing of touch/motion; neither button enables a motor. | `src/full/mod.rs:365`, `src/full/mod.rs:385` |
| Touch-owner routing | At touch-down, latch deck and scratch/seek/nudge destination. | Held touch continues controlling its original deck even if a deck selector changes. | `src/full/mod.rs:385`, `src/full/mod.rs:570` |
| Jog count recovery | Recover integer counts from normalized HID floats before converting movement. | Avoid delayed counts and reversal cancellation caused by float truncation. | `src/full/mod.rs:584` |
| GRID+jog | While GRID is held, accumulate 0.125 grid steps per recovered count and emit directional pulses. | Beatgrid editing replaces normal jog motion. | `src/full/mod.rs:585` |
| Relative MIDI encoding | Encode signed increments around CC value 64; split large deltas into chunks of up to 63. | No explicit maximum total jog speed is imposed here; per-message chunk size is not a total-motion cap. | `src/full/mod.rs:676` |
| Held-note ownership | Remember every MIDI destination owning a press. Release old notes before selector changes/new presses. | Deck/mode changes cannot strand a held Cue/pad/effect note; duplicate owners do not release each other. | `src/full/mod.rs:238`, `src/full/mod.rs:525` |
| Shutdown release | Release all owned MIDI notes when the bridge closes. | Prevent stuck held actions after Stop/Quit. | `src/full/mod.rs:183`, `src/full/mod.rs:559` |
| Front crossfader selectors | Cache physical assignments and curve until a raw playback output has arrived from Djay's mapping, then send the latest snapshot once. | After startup, send only physical changes. A playback packet is evidence of feedback, not an acknowledged input connection. Restart the bridge after mapping reconnect to initialize a new session; no silence-gap inference or periodic replay overrides UI choices. | `src/full/mod.rs`, `src/full/feedback.rs` |
| MIDI source/destination lifecycle | Create matching virtual input and feedback endpoints for the full profile. | Exposes `S4 MK3 MIDI Full`; does not claim the S4 audio device. | `src/full/mod.rs:131`, `src/full/feedback.rs:152` |
| Feedback decoding/cache | Parse received MIDI, including running status; retain supported software state. | Shifted/unsupported addresses cannot overwrite normal lamps; A/C and B/D sampler feedback is shared. | `src/full/feedback.rs:45`, `src/full/feedback.rs:111` |
| Local selector LEDs | Render deck, pad mode, jog mode, grid, and selected FX targets from bridge state. | These can light without a Djay feedback message. | `src/full/leds.rs:246` |
| Software-state lamps/pads | Translate supported returned software values into S4 palette/intensity bytes. | Unknown state remains dark. Bright mute indication describes the mute switch, not effective audibility under solo. | `src/full/leds.rs:309`, `src/full/leds.rs:377` |
| Momentary local indicators | REC, library controls, and Filter reset indicate held button state where persistent software state is unresolved. | A lit lamp here does not prove recording or playback is active. | `src/full/leds.rs:367` |
| Hotcue color interpretation | Decode Djay palette tokens or use explicitly selected fixed-slot colors. | Token-to-S4 palette correspondence is configurable; unrecognized tokens stay dark. | `src/full/leds.rs:470` |
| Jog-ring presentation | Choose selected deck color; fixed-period decorative chase only when raw playing state is true; green whole-ring firmware flash while looping. | A 2-second chase by default, not track position, physical platter motion, seek position, or BPM. | `src/full/output.rs:138` |
| Channel meter presentation | Convert normalized deck level into 14 S4 meter segments with brightness/gamma and freshness rules. | Fixed A-D meters; stale values go dark; no invented clipping signal. | `src/full/output.rs:168` |
| HID output pacing/cleanup | Coalesce unchanged reports; pace button, ring, and meter writes; clear outputs on shutdown. | Native reports only; no motor report `0x31`. | `src/full/output.rs:26` |
| Mapping generation | Expand the shared catalog into concrete deck/shift addresses and declared feedback. | Ensures emitter and generated mapping agree structurally; does not prove an action has the right musical meaning. | `src/full/mapping.rs:19` |
| LED preferences | Validate JSON and render from configured colors/levels/timing. | Presentation changes do not change input action assignments. | `src/full/leds.rs:158` |
| macOS app process management | Bundle the executable, install/open mapping, save/import/export settings, drain logs, and stop the child with SIGINT. | Runs without Rust on the laptop; closing Settings is not Stop, quitting the app is. | `macos/S4BridgeApp.swift` |

## Hardware-local, not bridge-managed audio

| Function | Owner | Current bridge behavior |
| --- | --- | --- |
| Master output level | S4 audio hardware | Do not map again to software master attenuation. |
| Booth output level | S4 audio hardware | Do not add duplicate software attenuation. |
| Headphone level and mix | S4 audio hardware | Decode where available, but do not assign duplicate software controls. |
| Master meter bars | S4 audio hardware | Preserve hardware behavior; separate master clip validation remains open. |
| Audio playback, sample recording, effects, Neural Mix | Djay and the selected CoreAudio device | HID/MIDI bridge controls operations but does not process, capture, or forward audio. |
| Live mic/line routing | Djay/CoreAudio and potentially S4 firmware | EXT switching is not implemented; decoding a press is not an audio-routing command. |

## Not implemented

Screens, screen settings menus, pad legends, motors/haptics, playback-position
platter tracking, live-input deck replacement, preview-dependent encoder
routes, Shift+Sync tempo-range cycling, and automatic same-process mapping
reconnect detection are not present. Startup crossfader initialization is
gated on raw playback feedback. Keep these limits separate from the
current-behavior reference.
