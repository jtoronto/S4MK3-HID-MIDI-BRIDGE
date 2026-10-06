# LED feedback setup and verification

The full bridge now has a virtual MIDI destination as well as its existing
source, both named `S4 MK3 MIDI Full`. The encoder increment changes input
routing as described below. The minimal
`--midi` profile retains its original behavior.

Earlier software and live-test records below cover actual Djay
feedback, both decks' Play/Cue lamps, channel level meters, and the colors of
the user's current hotcues. The user also confirmed deck-colored segments
across A/C and B/D, green whole-ring loop flashing and exit, preserved hardware
master meters, and smooth jog response with feedback active.
**Remaining controls, white-token behavior, and master clip indication
still need confirmation.**
Successful HID writes alone do not prove those behaviors.

## Verified hotcue palette

On October 6, 2026, the user confirmed all eight selectable Djay hotcue
colors match the physical pads with this correspondence:

| Djay token | S4 palette color |
| --- | --- |
| 1 | red |
| 2 | orange |
| 3 | blue |
| 4 | yellow |
| 5 | green |
| 6 | azalea |
| 7 | cyan |
| 8 | purple |

These are now the Rust and bundled JSON defaults. Existing saved preferences
are preserved; fixed-slot, stem, and deck palettes are unchanged.
Djay's hotcue picker does not offer white. Token 9 retains the existing
`djay_white=white` fallback, but its use and physical feedback remain
unverified. White used by bridge-local lamps or fixed slots does not verify
a Djay white hotcue token.

The calibrated-default increment passed 81 Rust tests and 12 Swift tests,
formatting, strict Clippy, locked all-target build, shell syntax, plist lint,
and diff checks. A regression feeds all eight tokens through the real MIDI
decoder, feedback cache, and pad renderer with both Rust and bundled JSON
defaults, checking the calibrated physical report bytes.
The rebuilt app passed deep strict signature and ZIP integrity checks;
its bundled LED JSON is byte-identical to `examples/led-config.json`.
Saved LED preferences and the user's `S4 MK3 Bridge 2` mapping retained
their hashes. The previous bundle/archive are preserved at
`/tmp/s4mk3-pre-palette.BZPApM`.

## Current MOVE selection indicator

In the full profile, MOVE press toggles a local size-select flag for the
logical deck and emits no MIDI. There is no timeout or deferred save:
unshifted MOVE immediately changes native jump size through CC 7 on
channels 1-4 (`skipDurationRotary`). Outside selection it sends CC 2
(`skipRotary`). Shift+MOVE always jumps one beat per detent via note 5
forward / note 14 backward on channels 9-12, even while selecting,
preserving the chosen size.

Only the currently selected deck selector pulses, on a 1.2-second
palette-brightness cycle with a dark trough. A hidden deck keeps its mode
without flashing its inactive selector. Exiting restores ordinary selector
lighting; bridge restart initializes every mode flag off. Native A/B/C/D
jump and loop values remain independent Djay values, not bridge counters
or shared resets. LOOP retains its independent native size and existing
shifted actions. This local pulse isn't software-state feedback or the
green jog-ring loop flash. No screen or motor traffic is added.

Physical pulse, hidden-deck restoration, and fresh MIDI/native-action checks
are pending. The user's mouse check established beat jump moving an active
loop, not this LED behavior. Follow the focused
[encoder live check](full-midi-test.md#encoder-layout-live-check), including
both sides, all four decks, duplicate press/release, Shift while selecting,
mode exit, and stop/restart.

The user's Windows USBPcap capture is analyzed in `traktor-ring-capture.md`.
After mode 5 also failed with the captured initialization, the user chose a
stationary deck-colored segment normally and a green whole-ring loop flash.
The renderer uses captured modes 2/3 for that revised design. The user confirmed
the integrated result. Firmware owns the loop flash cadence; software does not
alternate back to the deck segment until the loop exits.

The user subsequently chose a moving chase, accepting decorative spinning
while playing rather than exact track-position tracking. The chase uses
verified raw `turntable1..4.playing` feedback, not the blinking Play-lamp state.
It advances at a fixed configurable speed, returns to a stationary 12:00
segment while paused or playback state is unknown, and retains whole-ring
green loop flashing. It does not represent seeking, tempo, or track position.
The user confirmed the integrated chase across A/B/C/D, stopping on pause,
green loop flashing and return to chase on exit. Jog response and hardware
master meters remained normal with the moving chase active.

## Start and select the feedback mapping

Reinstall the newly generated encoder mapping under a new filename, preserving
custom maps under their existing names. Generate a new file rather than
overwriting a working mapping:

```sh
cargo run --locked -- --generate-mapping "S4 MK3 MIDI Full LEDs.djayMidiMapping"
cargo run --locked -- --midi full --led-config examples/led-config.json --seconds 3600
```

Open the exported file in Djay. Under **MIDI > Configure S4 MK3 MIDI Full**,
select the new LED mapping rather than the older input-only mapping. Keep the
controller enabled and audio routed directly through the S4. No per-control
MIDI Learn is needed.

During development, Djay imported and saved the separate mapping at
`~/Music/djay/MIDI Mappings/S4 MK3 MIDI Full LEDs-20261005.djayMidiMapping`.
The prior `S4 MK3 MIDI Full.djayMidiMapping` was preserved. Selecting the new
mapping and establishing real feedback remain part of the live check.

Chase requires a newly generated mapping: raw playing outputs were added on
human MIDI channel 7, CC 4-7 for A-D. The earlier LED mapping lacks these outputs
and will leave the chase stationary. That chase addition didn't change input
addresses; the subsequent encoder layout does require the new mapping.

If Djay shows only a blank `S4 MK3` configuration while the bridge's virtual
ports are live, restart Djay with the bridge already running. In the live
chase test, this restored `Configure S4 MK3 MIDI Full` and the full control
table without changing or deleting the saved mappings.

The bridge initializes only ring LED reports `0x30`; it never sends motor report
`0x31`. Full mode writes button report `0x80`, channel-meter report `0x81` when
levels arrive, and ring output report `0x32`. Software-state lamps start dark
until Djay actually sends state. Local deck/mode/FX selectors light immediately.

## Change preferences afterward

Use `examples/led-config.json` as the editable starting point, or pass your own
JSON file with `--led-config PATH`. Omit the flag for built-in defaults.
Partial files are allowed; missing fields keep their defaults. Restart the
bridge after edits. Changing presentation requires no rebuild or Djay input
remapping. Invalid fields and values fail startup rather than being ignored.

For example:

```json
{
  "deck_colors": ["red", "blue", "yellow", "purple"],
  "inactive_display": "dark",
  "palette_active_intensity": 3,
  "mute_color": "magenta"
}
```

| Setting | Meaning |
| --- | --- |
| `deck_colors` | A/B/C/D colors shared by deck selectors and rings |
| `hotcue_colors` | `djay` by default; `fixed_slots` only when explicitly chosen |
| `djay_pad_colors`, `djay_white` | Verified correspondence for tokens 1-8; token 9 retains an unverified white fallback |
| `fixed_hotcue_colors` | Eight colors for the optional fixed-slot policy |
| `stem_colors`, `mute_color` | Stem/solo colors and the engaged-mute color |
| `stem_mute_bright` | True: bright means muted; false: bright means unmuted, not effective audibility under other solo states |
| `active_brightness`, `inactive_brightness` | Fixed-color lamp levels, 1-127 active and 0-127 inactive |
| `palette_active_intensity`, `palette_inactive_intensity` | Palette lamp/pad/ring intensities, 0-3 |
| `inactive_display` | `dim` or `dark`; applies to software and local inactive indicators |
| `quantize_mixed` | `dim` or `dark`; bright means all four known deck states are on |
| `tempo_center_tolerance` | Disabled (`null`) until physical center tolerance is calibrated |
| `loop_enabled`, `loop_color` | Firmware flashes the whole ring green while looping; exit restores the deck-colored segment. Flash rate is controlled by hardware, not configurable. |
| `chase_period_ms` | Decorative revolution duration while playing, default 2000 ms (30 RPM); 250-10000 ms supported. |
| `meter_brightness` | 0 off, 125 dim, or 127 bright |
| `meter_gamma` | Level curve exponent, default 1.0; 0.1-5.0 supported |
| `meter_stale_ms` | Continuous-level freshness deadline, default 500 ms; validate against actual Djay update cadence |

Available palette names: `red`, `carrot`, `orange`, `honey`, `yellow`, `lime`,
`green`, `aqua`, `cyan`, `sky`, `blue`, `purple`, `fuchsia`, `magenta`, `azalea`,
`salmon`, `white`. These refer to the S4 palette, not arbitrary RGB colors.

## Known limits, not simulated states

- Djay-matched hotcue output uses native `padColorsActive` tokens 1-8 and white
  token 9, off 0. Tokens 1-8 are user-verified; white is unavailable in the
  hotcue picker and remains unverified. Unknown tokens stay dark; binary
  velocity 127 is not silently interpreted as a cue color.
- Channel meters use native `turntable1..4.monoMeter` CC output, fixed to decks
  A-D independently of deck selection. The 14 level-byte layout and curve need
  physical calibration. There is no verified per-deck clip output; the bridge
  does not invent clipping from maximum normalized level.
- Master meters already work through hardware. The meter report's tail is
  zero, following the reference, but its effect on hardware master clip
  indicators still needs testing. Hardware master bars were confirmed preserved
  in live use; clip indication was not separately verified.
- There is no verified Djay end-warning/remaining-time binding. Red end warnings
  are not implemented, and no nonfunctional end-warning settings are exposed.
  The bridge reports `END_WARNING_UNAVAILABLE` once at startup. No wall-clock
  playback estimate substitutes for real state.
- REC, library buttons, and Filter reset have momentary indicators where
  persistent software-state output is unresolved. STEMS and MUTE remain aliases
  of the same layer. Sampler loaded/empty and Neural Mix availability are not
  inferred from silence.
- Track loading-success output is a candidate state signal: a false value
  invalidates cached cue colors and channel level. Replacement/load/unload
  ordering and initial snapshots still need observation.
- Latched button state never expires merely because no messages arrive.
  Feedback queue overflow is an error, not silently dropped state.

## Live checklist when available

Capture actual returned packets while the normal bridge runs:

```sh
RUST_LOG=info,s4mk3_hid_midi_bridge::full::feedback=debug \
  cargo run --locked -- --midi full --led-config examples/led-config.json --seconds 600
```

1. Select the new mapping. Change Play/Cue/PFL with the mouse and verify
   `MIDI_FEEDBACK` packets and corresponding lights. This establishes actual
   Djay output and destination association, not just physical input mirroring.
2. Assign all available Djay cue colors, including white; edit a color without
   triggering the cue, clear it, change pad layers, switch A/C and B/D, and load
   another track. Calibrate `djay_pad_colors` against observed token/color pairs.
3. Verify A/B ring segments are red/blue by default and C/D selection redraws them.
   Enable/exit automatic and manual loops, including paused loops. Confirm
   green whole-ring flashing and `loop_enabled=false` restores
   the deck-colored chase while playing or stationary segment while paused.
   Confirm that mouse Play/Pause controls also start/stop the chase and that
   seeking does not imply track-position tracking.
4. Play each deck and confirm the corresponding channel bar responds. Measure
   update gaps before accepting the freshness deadline. Confirm master bars
   and hardware clip indication keep working with direct S4 audio.
5. Verify FX indicators use each strip's selected target, not the visible deck;
   check pads, quantize's mixed state, mode aliases, held modifiers, and graceful
   stop/reconnect.
6. Compare jog MIDI cadence with feedback active using the existing independent
   `midi-monitor` example. The prior smooth-jog result is not evidence that the
   new HID write traffic has no effect.

For a feedback-only diagnostic, with the normal bridge stopped:

```sh
cargo run --locked --example feedback-capture -- --seconds 300
```

This publishes paired virtual ports and logs raw Djay output without opening
HID. Do not run it alongside the normal full bridge: duplicate endpoint names
would make association ambiguous.

## Historical software and live evidence

These records predate MOVE size selection and its selector pulse. They don't
establish a fresh MIDI/native or physical LED pass for the encoder increment.

An isolated live probe established `turntable1.playing`: CC values 127/0 tracked
mouse playback/pause repeatedly. `turntable1.isPlaying` did not track transitions
and is not used. The documented `display.songProgress` emitted coarse normalized
whole-track progress and changed during seeks, not a usable platter phase.

The chase implementation passed 62 tests, strict all-target/all-feature
Clippy, formatting, locked all-target build, and Rust diagnostics. Coverage
includes a real CoreMIDI virtual-destination receive test with a bounded event
wait, no echo source, MIDI stream decoding, state routing, cached deck/layer
rendering, preference overrides, ring loop transitions, decorative chase
positions on an injected clock, raw playback/lamp-state isolation, and meter
freshness. Full-profile ring initialization was accepted by the connected S4.
The separate generated plist passed Apple's `plutil -lint` and Djay imported
it. Selecting it produced initial state snapshots, loading-success, Play state,
cue-color tokens 1-6, and deck-A level messages around every 20 ms. The user
confirmed both Play/Cue pairs, current hotcue colors, and channel meters.

The initial ring calls incorrectly treated Mixxx's third argument as feature
transport. It actually selects non-skipping FIFO for ordinary output reports.
Both `0x30` initializations and `0x32` updates/clears now use ordinary output.
That correction lit both complete rings; mode 3 was then physically confirmed
to flash even without a loop. Mode 5 appeared steady only after a mode-3 packet,
but retained red/blue when green was requested and remained dark on a fresh
normal-bridge start, and remained dark with captured 28-byte initialization.
The user approved stationary mode-2 segments plus mode-3 green loop flashes
instead of full steady rings. The initial software phase alternation interrupted
the green flash; the renderer now holds firmware mode 3 until loop exit.
The user confirmed this final loop behavior, selected-deck colors, preserved
master bars, and smooth jog response. No complete palette/control pass,
master-clip verification, or instrumented jog-cadence comparison is claimed.
