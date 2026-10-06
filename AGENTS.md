# Agent guide

## Project goal

Use the Native Instruments Traktor Kontrol S4 MK3 with Djay Pro on macOS.

The current product is an Apple Silicon menu-bar app bundling a Rust
HID-to-MIDI bridge, generated Djay mapping, and LED preferences. The DJ
laptop must not require Rust or Cargo.

A standalone hardware bridge remains a longer-term goal. Screens and
motors are deferred unless explicitly requested.

## Architecture

- `src/main.rs`: CLI, HID opening, report framing, descriptor corrections,
  and bridge lifecycle.
- `src/input.rs`: full-profile input snapshots and stale-interrupt rejection.
- `src/midi.rs`: minimal MIDI profile.
- `src/full/catalog.rs`: shared physical-control and Djay-action catalog.
- `src/full/mod.rs`: deck/layer routing, held-note ownership, and MIDI emission.
- `src/full/mapping.rs`: generated native `.djayMidiMapping` plist.
- `src/full/feedback.rs`: virtual MIDI destination, decoding, and state cache.
- `src/full/leds.rs`: validated preferences and button/pad rendering.
- `src/full/output.rs`: native LED reports, ring presentation, and meters.
- `macos/S4BridgeApp.swift`: AppKit status item, SwiftUI settings, child process.
- `macos/build-app.sh`: standalone bundle and transferable archive.
- `examples/`: MIDI diagnostics and example LED configuration.
- `reference/`: supplied controller documentation.
- `docs/`: design, mapping references, capture evidence, and validation records.

## Read before changing behavior

- `docs/mapping-audit.md`
- `docs/encoder-capture.md`
- `docs/djay-action-reference.md`
- `docs/bridge-function-reference.md`
- `docs/traktor-djay-comparison.md`
- `docs/led-feedback.md`
- `macos/README.md`

Historical validation records describe their tested revisions. Do not assume
the packaged app or older test counts describe the current working tree.

## Commands

Run from the repository root:

       cargo fmt --all -- --check
       cargo clippy --locked --all-targets --all-features -- -D warnings
       cargo test --locked
       cargo build --locked --all-targets
       swift test --package-path macos
       bash -n macos/build-app.sh
       plutil -lint macos/Info.plist
       git diff --check

Build the portable app:

       bash macos/build-app.sh

Outputs:

- `dist/S4 MK3 Bridge.app`
- `dist/S4 MK3 Bridge-arm64.zip`
- `dist/S4 MK3 Bridge-0.1.0-beta.1-arm64.dmg`

Cargo package/executable: `s4mk3-hid-midi-bridge`, release `0.1.0-beta.1`.
The bundle uses numeric version `0.1.0`, build `1`, and custom
`S4BridgeReleaseVersion` for the complete prerelease identifier.
`macos/Assets/AppIcon.png` generates the bundled `.icns` during packaging.

The build script refuses to overwrite an existing app bundle. Preserve an
owned previous build before rebuilding. Do not delete unrelated artifacts.

Verify a rebuilt package:

       codesign --verify --deep --strict "dist/S4 MK3 Bridge.app"
       unzip -tq "dist/S4 MK3 Bridge-arm64.zip"
       hdiutil verify "dist/S4 MK3 Bridge-0.1.0-beta.1-arm64.dmg"

Signing is currently ad-hoc, not Developer ID signing or notarization.

## Mapping contracts

Keep runtime emission and generated mappings consistent. Read both the catalog
and generator when changing an address or action.

User-facing MIDI channels are one-based; plist channels are zero-based:

- Decks A-D: channels 1-4.
- Shifted deck actions: channels 9-12.
- Mixer: channel 5.
- Deck selection: channel 6.
- Dedicated playback/meter/loading feedback: channel 7.

A MIDI address includes endpoint, direction, channel, message type, and numeric
note/CC. Pitch names such as C2 are not sufficient identifiers.

Preserve the minimal profile. Preserve held-note and jog-touch ownership across
deck, Shift, pad-mode, and FX-target changes.

## Verified encoder fields

Payload offsets exclude the report ID:

| Physical knob | Rotation            | Press             |
| ------------- | ------------------- | ----------------- |
| Left MOVE     | Byte 19 low nibble  | Byte 6 mask 0x04  |
| Left LOOP     | Byte 19 high nibble | Byte 6 mask 0x20  |
| Right MOVE    | Byte 20 high nibble | Byte 15 mask 0x20 |
| Right LOOP    | Byte 21 low nibble  | Byte 15 mask 0x04 |

Clockwise is +1 and counterclockwise -1 modulo 16. Shift does not change these
fields. The pinned upstream rotation names are reversed on both sides, but
only the left press names are reversed. Do not blindly swap right presses.

Current custom Djay behavior: LOOP selects loop length; pressing LOOP activates
it. MOVE uses an independent jump length, jumping without a loop and moving
the loop when active. MOVE press toggles per-deck jump-size selection, with
Shift+MOVE always jumping one beat. The earlier shared-length proposal is
superseded. An encoder rename alone does not prove software behavior.

## Hardware and verification boundaries

- Check for competing bridge processes before opening HID.
- Do not claim audio interfaces for control translation.
- Master, booth, and headphone controls remain hardware-local.
- Do not send motor commands or screen traffic without explicit scope.
- Do not equate physical platter position with Djay playback position.
- Do not infer software state from button presses when real feedback is needed.
- Successful HID writes, generated keys, and passing unit tests do not establish
  physical behavior. Label untested behavior plainly.

Use deterministic regressions at existing test boundaries. Preserve real
decoder, translator, mapping, and feedback behavior in tests. Subscribe before
triggering asynchronous actions and use bounded timeouts; do not use fixed
sleeps to make tests pass.

## Workspace discipline

The working tree is shared and can contain user edits and unfinished work.

- Read files before editing; preserve changes you did not make.
- Use `apply_patch` for source and documentation edits.
- Never commit, push, rewrite history, or discard changes unless requested.
- Preserve user-edited Djay mappings under their existing names.
- Keep generated bundles, local capture logs, and large packet captures out
  of commits.
