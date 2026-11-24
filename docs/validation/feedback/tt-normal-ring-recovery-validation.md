# Normal rings and released-nudge motor recovery

October 30, 2026. Corrections to
[the ring/upper-handoff diagnostic](tt-ring-upper-validation.md).

## Reproductions and corrections

The user clarified that normal-mode rings should advance with playback, follow
Scratch and freeze on pause. TT's physical tracking was reported good.
`normal_playback_advances_with_stationary_untouched_wheels` failed before the
fix because both frames remained at the same physical position.

Normal presentation now keeps continuous phase per logical deck. Playback
advances at the configured ring period; Scratch applies signed raw counter
deltas to its original owning deck and suppresses clock advancement while
held. Pause freezes phase and release resumes from the same position. TT
still selects physical phase. Tests cover paused scratching, playing-flag
holds, owner/deck changes, counter rollover and retained TT behavior.
The new presenter has no device I/O or allocations.

The supplied capture
`motor-diagnostic-2026-10-30T04-26-33Z-E6B98886.log` showed upper Scratch entry
at roughly 55 RPM after both nudges. MIDI ownership was selecting constant
motor assistance regardless of speed. The old exact bundled replay confirmed
5280 commanded force during untouched upper Scratch.
`upper_scratch_without_touch_keeps_speed_regulation` reproduced that error:
5280 instead of zero above target.

Motor policy now distinguishes physical contact from MIDI ownership. Contact
retains assistance; qualified lower-speed braking/holding can also retain it.
Upper ownership alone does not bypass speed regulation. Both regressions now
pass. Target speed, motor tuning, Scratch bounds and saved preferences were
not changed.

## Source checks

All required commands passed individually: formatting, strict all-target/
all-feature Clippy, full Rust tests with one test thread, all-target build,
Swift tests, shell syntax, plist lint and whitespace checks.
The first formatting check found layout-only differences, which were corrected
and rechecked. The full Rust test command passed in one run, including the
previously competing-process-sensitive CLI tests; no guard/test was weakened.
There were 51 library, 216 bridge, 175 standalone diagnostic and two screen
unit tests (444 total), plus the integration targets. All 62 Swift tests passed.
Two existing Swift unhandled-file warnings remain.

Static diagnostics were clean for the ring presenter and motor policy.
UI verification remains excluded for this diagnostic.

## Package and exact-binary replay

Output: `dist/tt-normal-ring-recovery-diagnostic-20261030/`, built with
`CARGO_PROFILE_RELEASE_STRIP=none` into a fresh output directory.
Build, deep/strict signature verification, ZIP integrity, DMG checksum and
bundle/mapping plist lint passed. Bundled defaults match the example; the
mapping is byte-identical to the preceding diagnostic.

The exact bundled bridge ran the offline fixture with bundled defaults:
exit zero, 9674 records and 1399 MIDI messages. Lower and upper Scratch entries
remain at 3076001/6060001 us, with releases at 4462001/7010001 us.
During the stopped/reverse lower window, commanded assistance remains 5280.
During the untouched upper window, commanded force is now zero rather than
5280 while Scratch MIDI remains active. There is no concurrent upper CC4,
and final motor output is neutral. Normal ring presentation is verified through
the raw-input/presenter/native-report tests, not through this motor-only replay.

SHA-256:

- Bundled bridge: `be43a745e026acf190d3d7690d48f0d9daf164c43c674611b8343e2ba2326700`
- ZIP: `8a98cfbdd4a2355ae4f84bb4dda23a8cf79de5fc5edfc4558409c7220fadfd9d`
- DMG: `cea47d70ccbd46eb870ab939ac6751ba41fdc06b6d0913feb2275e5d9270e7ec`

## Physical checks and cleanup

Physical ring transitions and actual nudge return to 33 RPM await the
[attended runbook](tt-speed-scratch-diagnostic.md). No hardware was started,
and no saved settings, mappings or previous packages were replaced.
No staging or commit operation was made; existing dirty/staged work remains.
The build cleaned its owned staging directory and verification subscriptions
completed. The bundle is a diagnostic from the shared working tree, not an
isolated-commit or published-release claim.

Subsequent user feedback on October 30: the corrected combined build was much
better, and the user chose to spend time beta testing. This is positive
normal-use evidence, not confirmation that each individual scenario or fault
path passed. UI validation remains skipped, and end-user settings reduction is
deferred until release preparation.

The subsequent commit checkpoint was verified from an export of the staged
index, excluding the older standalone reader/diagnostic edits. That export
passed formatting, strict all-target/all-feature Clippy, full Rust tests,
all-target build, 62 Swift tests, shell syntax and plist lint. It contains
378 Rust unit tests (51 library, 213 bridge, 112 diagnostic, two screen),
plus the integration targets. These counts differ from the shared-tree build
above because the unrelated standalone changes remain uncommitted.
