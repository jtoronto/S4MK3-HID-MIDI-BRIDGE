# Physical rings and upper-speed Scratch diagnostic

October 30, 2026. Follow-up to
[the first speed-Scratch diagnostic](tt-speed-scratch-validation.md).
The user reported rings continuing during held TT and normal-mode scratching,
and returning to noon on pause. They also requested the same Scratch handoff
at the symmetric upper pitch-bend limit.

## Changes and evidence

The ring renderer was using elapsed time while playing and position zero
while paused. Two new regressions reproduced those behaviors before the fix:
`stationary_ring_does_not_follow_playback_clock` and
`pausing_does_not_reposition_stationary_ring` both failed.

The new per-physical-side phase tracker observes raw encoder counters before
gesture ownership or playback routing. Rendering uses that phase, not a clock.
Normal/TT mode, held touch, pause, deck changes and raw counter rollover do not
reset it. Tests drive raw Report 3 through the translator and native ring
renderer, checking forward/reverse movement, stationary holds and rollover.
Existing deck colors and firmware whole-ring loop flash remain unchanged;
loop exit uses the latest physical phase. The legacy chase-period preference
remains accepted for file compatibility but no longer animates the ring.

The upper Scratch ceiling is `200% - floor`; there is no new independent
preference. At 33 RPM and the default 50% floor, Scratch entry is at or below
16.5 RPM or at or above 49.5 RPM. Return to pitch bend requires the measured
speed inside 18.5-47.5 RPM. Both entries hold Scratch Note 7 and route raw CC5
without concurrent CC4. Force tuning and mapping addresses are unchanged.
Symmetry is diagnostic, not a confirmed CC4 playback-rate limit.

## Source checks

Formatting, strict all-target/all-feature Clippy and all-target build passed.
All Rust unit targets passed: 50 library, 212 bridge, 175 standalone diagnostic
and two screen tests (439 total). All 62 Swift tests passed. Shell syntax,
source plist and final staged/unstaged diff whitespace checks passed.
Static diagnostics were clean on the renderer and translator.

The full Rust command still failed
`explicit_free_play_startup_reaches_log_creation_before_hid`, the previously
recorded live-CLI admission test sensitive to a competing running bridge.
That test and guard were not weakened, and the user's bridge was not stopped.
Two existing Swift unhandled-file warnings remain.

## Portable package and exact-binary replay

Output: `dist/tt-ring-upper-diagnostic-20261030/`, built with
`CARGO_PROFILE_RELEASE_STRIP=none` and a separate `S4_BRIDGE_OUTPUT_DIR`.
Build, deep/strict signature verification, ZIP integrity, DMG checksum and
bundle/mapping plist lint all passed. Bundled Jog defaults match the example;
the mapping is byte-identical to the preceding diagnostic.

The exact bundled bridge ran `--replay-motor-fixture` with its bundled Jog
configuration, without HID or MIDI ports. Exit zero; 9675 records and 1399
MIDI messages. Lower Scratch entered at 3076001 us and released at 4462001 us;
upper Scratch entered at 6060001 us and released at 7010001 us. There were
zero CC4 messages during either Scratch hold and zero CC4 messages in the
tested nominal windows. All 32 reported physical-phase samples were within
0-2879; the final motor report was neutral. Ring report correctness is covered
by the raw-input/native-report regressions, not merely those metadata samples.

SHA-256:

- Native executable: `9c8aba048bf18d1e42629013b6940ff956a01b17637fe3574a0e5f5d01c6c136`
- Bundled bridge: `5458d4bbbccc81dbf25410eef62aaef2ac5b29b49bcb9f73bb26efceab763e30`
- ZIP: `76db43972bdc2df7fb9013a125069fb698a4a53d0961bcc8ddb0c0b54aa7471c`
- DMG: `53fefbde03232f9541595417e9f86a10f4f0ef77c9579b96e82bf45e93cbb79a`

## Remaining physical checks and cleanup

UI verification remains excluded for this diagnostic. No agent-triggered
hardware run occurred. Physical ring alignment/direction and upper-handoff
audio continuity still need the ordered
[attended checks](tt-speed-scratch-diagnostic.md).

Previous packages, user mappings, saved preferences, staged work and unrelated
dirty files were preserved. No commit or staging operation was made.
The build cleaned its temporary staging directory; verification subscriptions
finished. The app comes from the shared working tree and is not a clean-release
or isolated-commit validation claim.
