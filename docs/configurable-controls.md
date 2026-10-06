# Bridge settings and control-tuning candidates

The menu-bar app exposes existing operating flags and all LED preferences.
It does not change the verified MIDI input mapping or add unimplemented
control-tuning switches.

## Available now

- Full/minimal/HID-probe profiles, continuous run or a bounded duration,
  controller enumeration, raw HID logging, and log detail.
- Mapping installation/export. The installed file contains the same fixed
  input addresses and verified feedback outputs as the CLI generator.
- Every setting in `examples/led-config.json`: deck/stem/hotcue palettes,
  cue-color policy, lamp/palette brightness, inactive display, mute meaning,
  quantize mixed state, calibrated tempo-center indicator, loop color/enabling,
  decorative chase period, and channel-meter curve/brightness/freshness.

LED edits only need a bridge restart, which the app performs on Save and
restart. They do not require mapping regeneration or a Rust toolchain.

## Useful future control settings

These are candidates, not options silently applied by this release.

| Candidate | Current value/location | How a setting would apply |
| --- | --- | --- |
| Jog pitch-bend sensitivity | `7.0`, `src/full/catalog.rs` | Generate/install a new Djay mapping; preserve the wire address |
| Jog scratch sensitivity | `25.0`, same catalog | Generate/install a new mapping; test touch ownership and both directions |
| Shift-jog seek sensitivity | `20.0`, same catalog | Generate/install a new mapping; test seeking independently of scratching |
| Grid-edit precision | `0.125` count multiplier, `Translator::jog` in `src/full/mod.rs` | Runtime preference; test fractional residuals and mode changes |
| Fader pickup | Tempo binding has `pickup=true`, same catalog | Mapping-generation preference; test software/hardware mismatch |
| Fader direction | Tempo binding has `flipped=true`, same catalog | Mapping-generation preference; do not change ADC normalization |
| Initial jog/pad/FX targets | Vinyl, Hotcue, A/B FX targets, A instant-FX target in `Translator::default` | Runtime startup defaults, subsequently overridden by physical selectors |
| Encoder actions/step behavior | Fixed catalog bindings and relative MIDI encoding | A separate mapping editor; larger scope than sensitivity preferences |

Jog bend/scratch/seek sensitivity is the best first control-tuning addition:
it is already represented explicitly by native `rotarySensitivity` values.
Mapping-generation controls should be clearly separated from runtime settings,
since restarting the bridge alone cannot apply them inside Djay.

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
- No screen, motor, audio-forwarding, guessed end-warning, or track-position
  chase setting is exposed. Decorative chase does not claim actual position.
