# EXT switching and indicator validation

## Scope

Current-source full-profile EXT controls select explicit Djay Internal/THRU
actions for the fixed corresponding mixer deck. Actual Djay THRU feedback
chooses the next action and lights its EXT indicator. Global DVS enablement
and audio input assignments remain manual setup; no global-enable feedback
flag is assumed. Shift+EXT source selection remains hardware-local.

This record separates native indicator identification, software regressions
and integrated physical acceptance. It does not establish every external
source, phono gain/staging or four simultaneous audio sources.

## Attended indicator identification, October 27, 2026

The temporary `ext-led-probe` example used the existing competing-controller
guard and opened only the unique S4 HID collection, after the user's fresh
watching confirmation. No HID was opened on startup. The normal bridge was
not running; the idle menu-bar app remained open.

Each commanded write was a 95-byte button report: ID `0x80`, followed by
94 payload bytes. Only the selected candidate byte was set to `0x47`; the
other button bytes were zero. The user identified each lamp individually:

| Payload offset | Report index, including ID | User-confirmed indicator |
| --- | --- | --- |
| 80 | 81 | EXT A / channel 1, second mixer strip from left |
| 84 | 85 | EXT B / channel 2, third mixer strip from left |
| 88 | 89 | EXT C / channel 3, leftmost mixer strip |
| 92 | 93 | EXT D / channel 4, rightmost mixer strip |

The probe was then sent `quit`, clearing the button report and releasing
HID. It sent no MIDI, motor, screen, audio-routing or ring-initialization
commands. Its temporary source was removed.

## Implementation contract

- New presses use the latest confirmed per-deck THRU flag: true selects
  Internal; false selects THRU. Unknown feedback consumes the press without
  sending a switch. A later feedback message does not turn that hold into
  a new press.
- Either Shift suppresses the software EXT command. Releasing Shift while
  EXT remains held does not emit a delayed command.
- Input actions use Notes 27/28 on human deck channels 1-4, with complete
  press/release pulses. Output CC 21 on those channels carries
  `turntable1..4.controlModeIsThru`.
- EXT lamps light only for confirmed THRU and stay dark for non-THRU or
  unknown state, even with dim idle lighting enabled. No optimistic state
  change follows a physical press.
- Deck selectors and FX targets do not retarget these four mixer controls.
  The minimal MIDI profile is unchanged.

## Integrated physical acceptance

The user enabled global DVS and assigned all four S4 stereo pairs to the
corresponding decks. The newly generated `S4 EXT INT-THRU 20261027` mapping
was imported and selected under its own name, preserving previous mappings.
A MIDI-only setup capture received initial CC 21 values for all four decks;
that setup endpoint was stopped before opening HID.

After fresh watching confirmation, the updated full bridge ran with
`.omo/evidence/ext-no-motor-jog.json` setting
`backspin_assist_enabled=false`, without tension or screen flags. Normal
LED output and physical mixer startup synchronization remained enabled.

The attended procedure selected C/D on the deck sides, then exercised each
physical mixer EXT twice. The user reported all four working. Logs confirmed
independent `EXT_MODE_REQUEST` THRU/INT pairs followed by actual returned
CC 21 values 127/0 on MIDI channels 1, 2, 3 and 4.

**Normal operation is a single press:** it switches immediately; a second
press returns to Internal. Holding in the procedure was a repeat-suppression
check, not a required gesture. Deterministic regressions separately cover
repeated held snapshots, feedback arriving mid-hold and consumed unknown-state
presses.

The user also confirmed both follow-up checks:

- Changing deck mode in Djay's on-screen menu changed the corresponding EXT
  lamp without a physical EXT press. The captured flags changed independently
  of an `EXT_MODE_REQUEST`, confirming that the lamp follows software state.
- Shift+EXT did not change Djay's mode; releasing Shift while EXT remained
  held did not issue a delayed switch. The original physical input type was
  restored. Regressions cover either Shift on every EXT channel.

The test bridge was stopped with Ctrl-C and exited with code 0. It is not left
running alongside the user's menu-bar app. The user's four input assignments
and installed new mapping remain available for normal use.

This establishes the requested button/indicator path, not every source's
audio processing. Other mixer-control coverage, phono gain/staging and four
simultaneous external audio sources remain outside this acceptance.

## Software and standalone-package checks

The following required checks passed:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
swift test --package-path macos
bash -n macos/build-app.sh
plutil -lint macos/Info.plist
```

The unused Notes 27/28 preserve REC's existing Note 22. The uniqueness
regressions caught the initial proposed address collision before export
or live testing. No test was weakened or skipped.

The standalone app, ZIP and DMG were rebuilt under
`dist/ext-int-thru-20261027/`, preserving prior packages. The build used
`CARGO_PROFILE_RELEASE_STRIP=none` for the existing local SDK/toolchain
workaround. Version `0.2.1-beta.1`, numeric `0.2.1` and build `7` were not
changed; this is a local source build, not a new published release.

Deep strict code-signature verification, ZIP integrity, DMG verification and
bundled mapping plist lint passed. The bundled mapping is byte-identical to
the installed mapping used for the attended physical test. Djay's import
moved the exported file into `~/Music/djay/MIDI Mappings/`; comparison used
that installed file rather than assuming the export remained in place.

The normal app's UI and saved preferences were not changed. The new app was
not launched during this test; physical acceptance used its source-equivalent
debug bridge with motor assist and screens explicitly disabled.
