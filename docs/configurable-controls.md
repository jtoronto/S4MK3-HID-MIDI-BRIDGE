# Bridge settings and control-tuning candidates

The menu-bar app exposes operating flags, LED preferences, saved hardware
wheel tension, native Djay jog mapping preferences, and experimental backspin
controls. Ordinary Speed/Reaction preferences change exported mapping parameters;
backspin controls affect runtime detection and event-only scratch routing/gain.

## Available now

- The TT speed-Scratch diagnostic adds separate TT Vinyl startup (off by
  default; normal Vinyl stays on) and an integer TT pitch-bend floor from
  1 to 95% of target RPM, initially 50%. These are save/restart runtime
  preferences, not mapping changes or a measured CC4 limit. See the
  [calibration workflow](tt-speed-scratch-diagnostic.md).

- Current source adds app-managed screen on/off, optional automatic NI-driver
  stopping, restoration, and manual recovery. Screen controls save immediately;
  see [screen setup](screen-driver-handover.md).
- Full/minimal/HID-probe profiles, continuous run or a bounded duration,
  controller enumeration, raw HID logging, and log detail.
- Mapping installation/export. The installed file contains the same fixed
  input addresses and verified feedback outputs as the CLI generator.
- Jog-tab manual Scratch and Pitch Bend Speed/Reaction settings. Defaults are
  manual scratch 2.7% / 150% and pitch bend 2.7% / 17%, applied to decks A-D.
  Manual touch Scratch direction is independently reversible. The retained
  experimental absolute Scratch CC8 mapping has a separate sensitivity setting,
  default 14.0% and bounded to 0.1-100%; current motorized playback emits relative
  CC4 and speed-owned CC5 instead, so the absolute sensitivity does not tune that path. These values
  are encoded as `rotarySensitivity`, `rotaryAcceleration` and `flipped` in the
  generated mapping.
- Current-source Jog > Backspin controls: detection enabled by default,
  filtered backward-acceleration threshold **150000 counts/s^2**, and additive
  scratch-speed increase **5.6%**. Save/restart applies these without exporting
  a mapping. Zero increase preserves ordinary count gain; nonzero increase uses
  `(scratch_speed + backspin_speed_increase) / scratch_speed` only during a
  detected event. The normal Scratch speed must match the selected Djay mapping.
- Current-source **Motor assistance** is a separate default-on Backspin option.
  `backspin_assist_enabled` controls it; `backspin_assist_strength` is an integer
  **0-100%**, default **100**, scaling the existing maximum on only the detected
  wheel. Zero is equivalent to assist off and preserves ordinary tension.
  Save/restart lets the user repeat their own level comparisons. After touch
  release its one-second fade cannot be extended by
  movement. Retouch, stop/reversal, mode cancellation, freshness loss or Stop
  clears the override; normal tension and software scratch speed remain separate.
- Jog > Hardware > Jog Wheel Tension: a saved integer from -100 to 100,
  shared by both wheels with independent motion/touch response. Negative is
  assistance, zero neutral, positive braking. Full profile only; save before
  starting or Save and restart while running. No mapping export is needed.
  See [hardware integration](jog-tension-integration.md).
- Every setting in `examples/led-config.json`: deck/stem/hotcue palettes,
  cue-color policy, lamp/palette brightness, inactive display, mute meaning,
  quantize mixed state, calibrated tempo-center indicator, loop color/enabling,
  normal-mode playback-ring period, and channel-meter curve/brightness/freshness.
  The ring period does not affect TT's physical-position display.

LED edits only need a bridge restart, which the app performs on Save and
restart. They do not require mapping regeneration or a Rust toolchain.

Current-source startup volume/EQ synchronization and shared-FX pickup are
runtime behavior, not additional sensitivity settings. They need the matching
feedback mapping. See [operation](controls-state-sync.md) and the
[user guide](user-guide.md#settings-what-needs-saving-or-a-new-mapping).

Ordinary scratch/bend settings are different: save preferences, install the newly generated
mapping under a new filename, then select it in Djay. Export uses the current
editor values and the bundled generator; it can run while the source bridge
is running. A bridge restart alone cannot change Djay's selected mapping.
Preferences live in `~/Library/Application Support/S4 MK3 Bridge/jog-config.json`;
bundled defaults come from `examples/jog-config.json`. Existing LED/run files
and user-edited mappings remain separate.

For CLI export:

```sh
cargo run --locked -- --generate-mapping "S4 Jog Custom.djayMidiMapping" --jog-config examples/jog-config.json
```

The exporter refuses an existing output path. Jog configuration is accepted
for mapping export and full-profile runtime, not minimal MIDI/probe commands.
At runtime, `--jog-config` supplies backspin settings and the configured normal
Scratch speed used to calculate optional event gain. Speeds must be finite
and positive; reactions are integers from 0 to 150. Shift-seek stays at its
existing sensitivity 20 and is not exposed by this increment.

Backspin detection keeps qualified released motion on its original scratch
owner until rest, reversal, retouch or Stop. Optional motor assistance overlays
the normal force law without changing the saved tension preference in
`~/Library/Application Support/S4 MK3 Bridge/tension-config.json`.
Nonzero assist opt-in selects the paced motor runtime even if no tension file is
supplied, using neutral as the normal selection. Minimal MIDI never accepts these options.
See [the jog investigation](jog-wheel-investigation.md).

## Testing-only and future control settings

### Current-source motorized playback testing

The Jog preferences now include `vinyl_enabled_at_startup` (default true)
and testing-only `platter_*` runtime values for target RPM, maximum force,
feed-forward base/slope, Kp/Ki, startup duration and rise rates, and overspeed
cutoff. They use the saved jog configuration and apply on the next bridge
start. `platter_position_sensitivity` is a separate mapping-time percentage for
the retained CC8 experiment, not the current CC4 motorized path; save, export and
select a new Djay mapping to apply it. TT independently
toggles motorized playback and starts off, while JOG toggles Vinyl red/on and
blue/off. These testing controls are not a commitment to expose every
parameter in a release. See the
[user workflow](user-guide.md#testing-motorized-playback).

These are candidates, not options silently applied by this release.

| Candidate | Current value/location | How a setting would apply |
| --- | --- | --- |
| Shift-jog seek sensitivity | `20.0`, same catalog | Generate/install a new mapping; test seeking independently of scratching |
| Grid-edit precision | `0.125` count multiplier, `Translator::jog` in `src/full/mod.rs` | Runtime preference; test fractional residuals and mode changes |
| Fader pickup | Tempo binding has `pickup=true`, same catalog | Mapping-generation preference; test software/hardware mismatch |
| Fader direction | Tempo binding has `flipped=true`, same catalog | Mapping-generation preference; do not change ADC normalization |
| Initial pad/FX targets | Hotcue, A/B FX targets, A instant-FX target in `Translator::default` | Runtime startup defaults, subsequently overridden by physical selectors |
| Encoder actions/step behavior | Fixed catalog bindings and relative MIDI encoding | A separate mapping editor; larger scope than sensitivity preferences |

Mapping-generation controls are separate from runtime settings, since
restarting the bridge alone cannot apply them inside Djay.

## Keep these as protocol invariants

- Recovering S4 jog counts with the `1000.0` factor compensates for Encdr's
  fixed decoder scaling. It is not a user sensitivity knob.
- The S4 ADC range (`4095`), report lengths, pad bit order, wheel position
  range (`0..2879`), physical sides, palette bases, and MIDI addresses describe
  the hardware/protocol. Exposing them as ordinary preferences risks breaking
  routing or the installed mapping.
- Held-note ownership, release-before-selector ordering, fractional jog
  residuals, feedback queue bounds, and output pacing protect correctness and
  smooth input. They are not appearance or musical-control choices.
- No on-controller settings menu, audio-forwarding, guessed end-warning, or
  track-position chase setting is exposed. Motorized playback testing is
  separate from app screen controls and motor-backed tension;
  normal rings illustrate playback with Scratch movement and pause continuity;
  TT ring phase follows physical wheel movement. Neither is a track playhead.
