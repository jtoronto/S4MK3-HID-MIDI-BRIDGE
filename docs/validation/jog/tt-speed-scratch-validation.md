# TT speed-to-Scratch diagnostic validation

October 30, 2026. This records the new diagnostic, not completion of the
canceled integration closeout or publication of a release.

## Implemented scope

Normal mode defaults to Vinyl on; TT defaults to Vinyl off. Both startup
preferences save independently and legacy normal-mode preferences are preserved.
Each physical side remembers separate per-mode choices during a bridge session.

With authorized TT and Vinyl off, the qualified 100 ms speed window selects
held Scratch Note 7 and raw signed CC5 below the configurable speed floor,
replacing CC4 until forward recovery passes a hysteresis threshold. At the
initial 33 RPM/50% settings, entry is 16.5 RPM and exit is 18.5 RPM.
The 50% value is provisional recording evidence, not a verified CC4 floor.

TT touch, stationary holding and reverse movement retain bounded forward
assistance. At the supplied settings the command is 5280 units. Normal tension
and backspin assistance cannot add to it. The PI integrator is reset at the
policy transition and is not updated while assistance owns the motor.
Untouched startup retains speed regulation. Freshness recovery requires a new
20 ms sensor window; duplicate reports cannot refresh the deadline. Playback,
Shift/GRID and deck-owner gates still neutralize the selected TT motor.

See the ordered [attended runbook and calibration procedure](tt-speed-scratch-diagnostic.md).

## Source evidence

- Formatting and strict all-target/all-feature Clippy passed after correcting
  new test and state-layout findings without lint suppressions.
- All Rust unit targets passed: 50 library, 207 bridge, 175 standalone
  diagnostic and two screen tests (434 total).
- Production parser/feedback/output replay covers speed-floor entry, stationary
  Scratch hold, signed reverse movement, recovery, original deck ownership,
  touch assistance, Stop and freshness recovery.
- Remaining independent CLI targets and doctests were executed after the
  full-suite command stopped. All-target build passed.
- All 62 Swift tests passed, including legacy preference merging, saved
  independent defaults and integer floor validation. Two existing Swift
  unhandled-file warnings remain. Shell syntax and plist lint passed.
- Static diagnostics were clean for the new routing and motor-policy modules.

`cargo test --locked -- --test-threads=1` did not pass globally:
`explicit_free_play_startup_reaches_log_creation_before_hid` failed before its
expected capture-path error. Running its deliberately invalid-path CLI command
directly returned the competing-controller guard for PID 3432, the existing
`dist/motorized-playback-braking-2x/` bridge. The independent
`wheel_selection_is_admitted_only_by_the_intended_profile` test has the same
guard-before-capture-path assumption and also failed. No test or guard was
weakened, and the user's running bridge was not stopped.

## Portable artifact evidence

Output directory: `dist/tt-speed-scratch-diagnostic-20261030/`.
Build command:

```sh
CARGO_PROFILE_RELEASE_STRIP=none \
S4_BRIDGE_OUTPUT_DIR="$PWD/dist/tt-speed-scratch-diagnostic-20261030" \
bash macos/build-app.sh
```

Build, deep/strict signature verification, ZIP test, DMG verification, bundle
plist and mapping plist lint all exited zero. Bundled Jog defaults match
`examples/jog-config.json` byte-for-byte. The generated mapping is byte-identical
to the stronger-braking package: the handoff uses existing addresses.

The exact bundled bridge, with its bundled Jog configuration, ran
`--replay-motor-fixture` without HID or MIDI ports: exit zero, 8848 records and
580 MIDI messages. Deck C speed-Scratch entered at 3076001 us and released at
4462001 us. The stopped/reverse assistance window contained only force 5280.
Reverse Scratch produced 115 signed negative CC5 messages; the stationary
interval produced none. Both tested nominal windows produced zero CC4
messages. There were no absolute CC8 messages, and the final motor report was
neutral.

SHA-256:

- Native executable: `db28e602c0a26e2d9c0c7828e0b042d66767d6bc58de008b8eaeaf954a959d8b`
- Bundled bridge: `3a7e2650b362b4dc5dc1f54f2202107ed219cdbba1366e164db27403fc8e45dd`
- ZIP: `a31dcfbed52bcece12358603cf802d42bd1fac6794a8891c762dcb9cb84032b5`
- DMG: `bac2c6bb5d608b80a6b29a464b2b04109e8cc161177d98da53e69413884826f6`

## Verification and cleanup boundaries

UI/rendered settings verification was skipped as requested. Physical motor
feel, actual CC4 saturation and audible handoff continuity remain untested for
this new build; no agent-triggered hardware run occurred.

The previous packages, user mappings, saved preferences and existing staged
changes were preserved. New work remains unstaged; no commit was made.
The build script cleaned its owned staging directory. Test fixtures clean
their own temporary files; no preview app, live bridge, capture or background
watch was started for physical testing. Package/source-check subscriptions
have completed. The packaged app comes from the shared working tree, including
its existing input-reader and standalone diagnostic work; this is not a
clean-release or isolated-commit verification claim.
