# TT Shift-only jog-seek suppression

November 2, 2026. The user reported CC 6 when pressing Shift with TT selected,
then identified the output as Jog seek.

## Cause and correction

CC 6 is the relative jog-seek command on the selected deck's MIDI channel,
from the bridge to Djay. The catalog maps it to `turntable{deck}.jogSeekMove`.
Shift itself does not emit this command. Instead, Shift selected the seek route
for subsequent raw wheel movement.

The raw TT rotation gate in `Translator::jog` previously excluded Shift.
Consequently, holding Shift let automatic or coasting platter travel through
the normal relative jog path, emitting CC 6 without a top-touch gesture.
The motor's existing Shift cancellation did not prevent the still-moving wheel
from producing those commands.

The correction removes that Shift exception. With TT selected, no GRID route
and no held wheel owner, raw movement remains suppressed whether or not Shift
is held. Hold Shift first, then touch and turn the top to seek deliberately.
Release the top to stop seeking even while the wheel still turns.

Normal-mode Shift+jog, explicit seek ownership, existing Scratch ownership and
the GRID route retain their separate behavior. MIDI addresses, mapping,
preferences, motor policy and the preceding fast-stop/touch-hold changes are
unchanged.

## Regression evidence

The real translator fixture produces matching raw sensor travel and decoded
jog movement with deterministic 2 ms timestamps.
`tt_shift_requires_top_contact_for_seek_but_normal_shift_does_not` covers both
physical sides, decks A-D, both Vinyl choices, and TT/Normal mode.
It checks Shift alone, deliberate top-touch seeking in both directions, and
untouched rotation after touch release.

Before the correction, it failed with 100 CC 6 commands from 33 RPM untouched
rotation, including `[176, 6, 67]` and `[176, 6, 68]` on deck A. The same test
passed after removing the raw routing exception. Normal-mode control cases
still emit CC 6, so the test does not pass by disabling all seek output.

## Source and package verification

Formatting, strict all-target/all-feature Clippy, all-target build, all 71 Swift
tests, shell syntax, plist lint and diff whitespace checks passed. Changed
Rust files had no LSP errors.

The complete Rust run executed 509 tests: 507 passed and two existing diagnostic
CLI tests failed because the user's bridge was running:

- `explicit_free_play_startup_reaches_log_creation_before_hid`
- `wheel_selection_is_admitted_only_by_the_intended_profile`

Both expect valid diagnostic options to reach an intentionally invalid capture
path and fail there before HID. A competing live bridge is rejected earlier.
The direct startup probe returned `competing controller process` for the running
`tt-touch-stop-20261102` bridge instead of the expected `create new capture`.
No tests, process guards or user processes were modified to make this green.
The second run used `cargo test --locked --no-fail-fast` to run every target
despite those environmental failures.

The standalone app, ZIP and DMG are in `dist/tt-shift-seek-20261102/`, with
previous builds preserved. Version remains `0.2.1-beta.1`, build 7.
Deep/strict signatures, ZIP integrity and DMG verification passed.
The bundled mapping and LED/Jog/tension defaults match the preceding
touch-stop build byte-for-byte.

Both actual packaged executables were run through the existing offline playback
fixture using the same custom curve configuration. All 12,915 timestamped motor
reports match. Both runs contain 17,068 structured records; this fixture
preserves the preceding playback/Scratch behavior, while the new targeted
translator test owns the untouched Shift regression.

## Physical limits

No HID session, hardware start, motor test or saved preference edit was
performed. This establishes MIDI routing in software, not a completed physical
Shift/top-touch comparison in Djay. No mapping reinstall or preference reset is
needed for the comparison build.
