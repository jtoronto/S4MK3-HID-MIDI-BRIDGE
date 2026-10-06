# S4 MK3 HID-MIDI Bridge

A proposed standalone hardware bridge for using a Native Instruments Traktor
Kontrol S4 MK3 with DJ software other than Traktor.

The primary goal is to use the owner's S4 MK3 with Djay Pro on macOS. The bridge
would translate the controller's HID controls into MIDI and translate software
feedback back into controller LEDs. Its intended first usable version must also
carry audio to the S4's master and headphone outputs.

## Status

Stage 1: a terminal Encdr connectivity probe. Stage 2 adds optional minimal
macOS virtual MIDI output for Play/Cue, channel volume, and crossfader.
The minimal custom Djay mapping has been learned, saved, and tested with the S4.
There is no selected bridge board or audio forwarding yet.

The first live macOS test received button and jog events, drove the left Play
LED, and left audio playing through the S4. See the test record for the exact
evidence and remaining unverified controls.

The design record captures the initial discussion on October 4, 2026. On October
5, the Encdr connectivity stage and minimal macOS MIDI stage were authorized.
Stage 3 adds an automatically generated full custom Djay mapping. Its live
controller validation is pending; standalone hardware remains a proposal.

## Run the connectivity probe

With Rust installed and the S4 connected by USB and its own power adapter:

```sh
cargo run --locked -- --list
cargo run --locked -- --seconds 60
```

Press and release the left Play button, move a fader, and touch/turn a jog.
The probe logs control events and writes left Play LED updates matching the
button's held state. Confirm the light physically; a successful HID write is not
visual confirmation. It uses native HID access on macOS and Encdr for decoding
and LED report construction.

See [docs/connectivity-test.md](docs/connectivity-test.md) for setup, pass criteria,
limitations, and troubleshooting.

## Test MIDI in Djay

```sh
cargo run --locked -- --midi --seconds 600
```

Configure the `S4 MK3 MIDI` virtual source with Djay's MIDI Learn.
See [docs/midi-test.md](docs/midi-test.md) for the message profile, mapping steps,
independent MIDI monitor, and verification limits.

## Automatically map the full controller

```sh
cargo run --locked -- --generate-mapping "S4 MK3 MIDI Full.djayMidiMapping"
cargo run --locked -- --midi full --seconds 3600
```

Open the generated mapping in Djay and select `S4 MK3 MIDI Full`. This separate
profile preserves the verified minimal mapping. It includes four-deck controls,
EQ, loops, library navigation, pad layers, jogs, and FX targeting without learning
each control manually. See [docs/full-midi-test.md](docs/full-midi-test.md) for
the layout, hardware-local controls, unsupported functions, and live test steps.

**Shift+Sync cycles Djay's tempo-fader range globally:** pressing it on either
side changes the range for all decks, not just the selected deck. Normal Sync
remains deck-specific. Install/select the updated generated mapping to use it.

## Agreed initial direction

- Start with a custom Djay MIDI mapping, not another controller's USB identity.
- Target macOS first.
- Support ordinary touch scratching and pitch bending before powered platter
  rotation.
- Require the S4's own master and headphone audio outputs for the first usable
  standalone version.
- Do not require the screens to work with alternate software.
- In the full custom profile, channel FX1/FX2 buttons select the deck controlled
  by the left/right FX strip respectively.

The longer-term goal remains a hardware bridge without a laptop-side bridge
application. Guest plug-and-play and support for other DJ applications are
desirable, but personal Djay use takes priority. Importing a custom mapping is
acceptable. DDJ-SX2 emulation can be explored later.

## macOS menu-bar app

The standalone Apple Silicon app bundles the bridge and a Djay mapping, so the
DJ laptop does not need Rust or Cargo. Build with `bash macos/build-app.sh`;
copy `dist/S4 MK3 Bridge.app` to the laptop. It provides Start/Stop, mapping
installation, complete LED preferences, and diagnostics.
See [macos/README.md](macos/README.md) for installation and
[docs/configurable-controls.md](docs/configurable-controls.md) for control-tuning
candidates.

Default Djay hotcue colors use the physically verified token order: red,
orange, blue, yellow, green, azalea, cyan, purple. Saved LED preferences
remain unchanged. White is not available in Djay's hotcue picker; its
separate token retains an unverified fallback.

## Mapping reference and beta guide

The user confirmed the standalone app works on the DJ laptop. Control-level
beta findings, explanations, and proposed changes are recorded in the
[mapping audit and beta guide](docs/mapping-audit.md). The audit is documentation,
not a new mapping release.

- [Djay input/output master reference](docs/djay-action-reference.md):
  software actions, state outputs, human descriptions, and current MIDI usage.
- [Bridge-owned function table](docs/bridge-function-reference.md):
  local routing, modes, conversion, and LED behavior.
- [Traktor factory comparison](docs/traktor-djay-comparison.md):
  supplied S4 MK3 manual references and achievable or unresolved equivalents.

## Design record

The full profile includes LED feedback, channel meters, and a decorative jog-ring
chase while playing, with green loop flashing. These behaviors have been tested
with Djay and the connected S4; full palette calibration and remaining control
checks are documented separately. See
[docs/led-feedback.md](docs/led-feedback.md) for setup, editable JSON preferences,
verification steps, and current limits. Input-only mappings must be regenerated
to include feedback. The minimal profile remains separate.

See [docs/led-mapping-plan.md](docs/led-mapping-plan.md) for the approved LED
design and its hardware/software evidence.

See [docs/design.md](docs/design.md) for requirements, architecture options,
controller comparisons, open questions, proposed validation stages, and sources.

The initial references are [Encdr](https://github.com/RufusIbiza/Encdr), the
[Mixxx S4 MK3 mapping](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js),
and the [NIME 2026 S4 MK3 motor-control paper](https://nime.org/proceedings/2026/nime2026_123.pdf).
