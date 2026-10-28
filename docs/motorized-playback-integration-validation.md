# Motorized playback testing integration

## Scope and evidence status

This record follows integration of the attended platter diagnostic into the
full-profile bridge. Implementation and integrated validation are in progress.
The diagnostic feel settings are provisional, not physical acceptance of
motorized playback while DJing.

The selected testing values are 33 RPM, maximum force 20000, feed-forward
base 1650 and slope 110, Kp 4000, Ki 3000, force rise 200000/second, startup
200 ms with an immediate capped step, and overspeed abort disabled. The
diagnostic's 15-second powered window is not a playback duration setting.

## Intended control contract

TT independently enables motorized playback per physical wheel, initially off.
JOG toggles Vinyl: red means top-touch scratch enabled, blue means disabled.
Vinyl starts on unless changed through the startup preference. Mode controls
must consume a press until release, and must not disturb held-note ownership.

Rotation authorization uses actual selected-deck playback/loading/THRU
feedback, not Play-button guesses or blinking lamps. The existing motor
output owner arbitrates playback, tension and backspin rather than summing
their forces. Powered counts must not become unconditional bend/seek MIDI.

## Explicit testing limitation

The user accepted cached real playback authorization during Djay feedback
silence for this testing build. Quitting Djay without a final paused-state
message may leave cached playback true. Use TT-off or Stop bridge before
quitting Djay; no reliable disconnect heartbeat has been established.

Rim movement relative to nominal rotation is a sensor-based approximation:
the available counter cannot independently identify hand contact, friction
and motor regulation. Physical comparison is still required.

## Baseline

Before integration edits, `cargo test --locked` and
`swift test --package-path macos` passed, including 175 diagnostic tests and
55 Swift tests. Two existing Swift unhandled-file warnings remain.
No motors were started in baseline validation.

## Planned integrated checks

- Feedback plus captured input through the production motor/MIDI path:
  TT-off and unknown feedback neutral; known playback permits rotation beyond
  15 seconds; nominal powered travel emits no jog MIDI; pause removes force.
- Touch, held scratch ownership through deck changes, released backspin,
  independent wheels, stale sensors, queued stale frames and Stop.
- Native testing controls and Vinyl startup preference save/reload using
  isolated preferences, with screenshot and action evidence.
- Required source checks and rebuilt portable app integrity.

Integrated results and exact invocations will be added after execution.

## Executable production replay

Parent invocation, with no HID or MIDI ports opened:

```sh
cargo run --locked --bin s4mk3-hid-midi-bridge -- --replay-motor-fixture \
  > .omo/evidence/motorized-integration/replay.jsonl
```

The command exited 0. Its 8,393 records contain 8,212 motor writes, 133 MIDI
messages and 24 labelled state transitions. Direct transcript checks found:

- All outputs through the unknown-Internal feedback interval were neutral.
- Both wheels retained positive drive after 16 seconds, beyond the old
  diagnostic pulse limit.
- No jog CC4-6 messages appeared after the fixture returned to nominal
  powered travel at one second.
- The deliberate rim disturbance emitted `[176,4,68]` at 70,001 microseconds.
- The final motor report was neutral for both wheels.

The fixture uses the production descriptor/parser, feedback cache, translator,
regulator and output service with external sinks substituted. Scheduler
Stop/short-write and frozen-sensor scenarios remain separately covered in
the runtime regressions. This does not establish physical feel, real motor
torque, or Djay's actual delivery of the new Internal feedback.

## Matching mapping requirement

The generated mapping adds `turntableN.controlModeIsInternal` on CC22,
deck channels 1-4 (one-based), in addition to existing THRU feedback.
Motor permission requires explicit Internal true; false THRU alone could
also describe DVS. Install/select a new mapping under a new filename.

## Packaging and isolation plan

Use `S4_BRIDGE_OUTPUT_DIR` with a new integration-specific directory under
`dist/`, rather than replacing or moving the shared existing bundle and
archives. The build script supports this override and refuses an existing
app or DMG. Package integrity must be checked on that exact output.

The native GUI check must use a separate support/preferences directory and
must not start the bridge against hardware. Its persistence tests already
provide isolated fixtures; the native implementation lane will supply a
concrete application invocation for screenshot-based checks.

The implementation lanes own disjoint scopes: Rust and bundled jog defaults
in one lane, native Swift in another. Combined tests and packaging remain
parent-owned after both results are inspected.

## Combined verification results

The final source checks passed formatting, strict all-target/all-feature
Clippy, locked Rust tests (190 bridge tests and 175 diagnostic tests),
all-target build, 58 Swift tests, shell syntax, plist lint and diff checks.
Two existing Swift unhandled-file warnings remain.

Self-review found that a mode button held during startup could toggle TT or
Vinyl on the initial snapshot. The parser now seeds those mode-button states
without emitting an initial press; release and a new press are required.
`held_startup_modes_require_release_and_a_new_press` covers that boundary.
The first check after this change caught only new-test formatting, which was
corrected before the full checks passed.

An index-only source snapshot was separately tested with `cargo test --locked`
to verify the integration commit does not depend on the shared unfinished
input-reader and standalone diagnostic work. That run passed.
Implementation commit: `52c0c50` (`Integrate motorized playback and independent Vinyl mode`).

The app, ZIP and DMG in `dist/motorized-playback-testing/` passed deep strict
signature verification, `unzip -tq` and `hdiutil verify`. The prior owned build
is preserved in `dist/motorized-playback-testing-before-startup-fix/`.
No version bump, publication or installed user-app replacement was performed.

The exact bundled bridge also passed `--replay-motor-fixture --jog-config`
using its bundled defaults, which matched `examples/jog-config.json`
byte-for-byte. SHA-256:

- Bundled bridge: `f0fa2095aaea53dee6aa2809b8243e0a997d5d6e7b476e487a2315b629b37504`
- ZIP: `59ff2910c1f0aad4667ef887f5feb2f1dcacbdc7b053025b1e014171e333047a`
- DMG: `888ce894c0e952a6dbd7bb6f65c27e2db1d9c5e77989d6292be92446ddb85a27`

## Native GUI evidence limit

The actual Jog settings were compiled into an isolated native preview with
no bridge executable. Launch succeeded and its process was stopped afterward.
Window capture failed and desktop capture reported
`Quartz reported no active displays`, despite capability permissions reporting
granted. The display-availability question timed out; a later check returned
the same error. Native visual/save-reload QA is **not passed**.

The 58 Swift tests cover legacy field merging, bounds/types, persistence,
invalid-file preservation and event-driven restart forwarding. These do not
substitute for a rendered screenshot. The preview harness and test log were
retained under `.omo/evidence/motorized-integration/native/`; the owned preview
process and original temporary directory were removed.

## Attended acceptance sequence

No physical test was started by the agent. Before testing:

1. Use the testing app and install its generated mapping under a new filename,
   preserving existing custom mappings. Select it in Djay.
2. Save the chosen Jog preferences, enable full profile, and start the bridge.
   Confirm TT remains off and Vinyl indicates the selected startup default.
3. Load an internal deck, keep the other wheel clear, and confirm normal
   playback feedback. Do not use DVS/THRU as a motorized-playback test.
4. Before an agent-triggered test, explicitly confirm watching after setup is
   complete. No automated trigger is authorized by this record.
5. Press the selected wheel's TT button. Check rotation beyond 15 seconds
   without hands-off pitch drift, then repeat controlled rim braking/nudging.
6. With Vinyl red, briefly touch the flat top and perform a small scratch.
   Check torque interruption, correct deck ownership and release recovery.
7. Toggle JOG blue/red and verify touch scratch is disabled/enabled independently
   of TT. Pause playback, turn TT off, and Stop bridge separately; verify force
   clears. Repeat on the other side and with A/C and B/D changes.
8. Turn TT off or Stop bridge before quitting Djay. Cached playing state is a
   known testing limitation; silence is not a disconnect signal.

Record physical feel, observed lamps and audio behavior separately. Native
write success and generated MIDI do not establish those results.
