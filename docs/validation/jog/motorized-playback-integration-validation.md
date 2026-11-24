# Motorized playback testing integration

## Scope and evidence status

This record follows integration of the attended platter diagnostic into the
full-profile bridge. Current integrated source is in extended user beta testing.
The diagnostic feel settings are provisional, not physical acceptance of
motorized playback while DJing.

The selected testing values are 33 RPM, maximum force 20000, feed-forward
base 1650 and slope 110, Kp 4000, Ki 3000, force rise 200000/second, startup
200 ms with an immediate capped step, and overspeed abort disabled. The
diagnostic's 15-second powered window is not a playback duration setting.

## Intended control contract

TT independently enables motorized playback per physical wheel, initially off.
JOG toggles Vinyl: red means top-touch scratch enabled, blue means disabled.
Normal Vinyl starts on and TT Vinyl starts off, with independent startup and
session choices. Mode controls
must consume a press until release, and must not disturb held-note ownership.

Rotation authorization uses actual selected-deck playback/loading/THRU
feedback, not Play-button guesses or blinking lamps. The existing motor
output owner arbitrates playback, tension and backspin rather than summing
their forces. Powered counts must not become unconditional bend/seek MIDI.

## Explicit testing limitation

Current implementation and package evidence are summarized in
[normal-ring and nudge recovery validation](tt-normal-ring-recovery-validation.md).
It supersedes the earlier behavior described in dated trial sections below.
On October 30, after testing the correction, the user reported it much better
and chose extended beta testing. This does not establish every individual
fault/recovery path. UI validation remains explicitly skipped. Before release,
review which Testing settings should remain exposed; this is a deferred product
decision, not authorization to remove saved settings during beta testing.

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

## First physical attempt: timestamp failure

On October 28, 2026, the user reported an immediate bridge exit when touching
the left jog: `wheel 0 sensor time reversed`. After restarting, TT lit blue
but produced no rotation or motor-related diagnostic explanation.
This is a failed physical acceptance attempt, not a passing integration test.

The runtime regression
`touch_across_sensor_rollover_neutralizes_without_exiting` reproduced the
exact timestamp error using valid 20 ms sensor intervals with touch on the
first post-rollover sample. The decoder previously accepted rollover only
when both timestamps were within one million ticks of zero. At the measured
100 ticks/microsecond, legitimate samples can cross outside that window.
Modular forward-distance comparison fixes this reproduced defect while
retaining backwards-timestamp rejection. The regression failed before the
fix and passed afterward; the existing seven decoder tests also passed.
The user's original timestamp values were not captured, so this establishes
a concrete matching failure, not proof of the exact hardware sequence.

The user subsequently confirmed that the newly generated mapping had not been
installed. The older mapping lacks CC22 Internal-mode output, so TT selection
correctly remains neutral rather than guessing whether the deck is in Internal
or DVS mode. Install and select the testing app's generated mapping under a
new name, then restart the bridge. The missing explanation in normal logs is
corrected with transition-only `MOTOR_TT_SELECTION` and `MOTOR_PLAYBACK_GATE`
events at Info level; the authorization gate is retained. The latter reports
actual loaded/playing/Internal/THRU feedback and local blockers, including
`internal_mode_unknown` when the required mapping output is absent.

## Previous ratio-deadband trial (not absolute Scratch CC)

The currently generated mapping/cache does not expose per-deck current BPM.
The installed Djay metadata describes a tempo-fader value, which is not the
track's BPM. This experiment does not need BPM readback: nominal platter
rotation is removed using the configured target RPM, and relative jog
movement carries the speed ratio. Ideally,
`playback BPM = current BPM * measured platter RPM / target RPM`; at the
33 RPM default, a 1 RPM deviation is about a 3% playback-rate change.

The first implementation discarded up to two counts independently in every
20 ms window. At 33 RPM, a sustained 1 RPM deviation is only about 0.96
counts per window, so it could be discarded indefinitely. The revised filter
accumulates signed deviation against the same two-count chatter budget. A
deterministic regression was added to require both +1 RPM and -1 RPM deviations
to produce corresponding jog input within 100 ms, while nominal motion and
alternating one-count chatter remain neutral. This establishes the sensor-to-
MIDI response in software, not Djay's actual pitch change or physical feel.
The positive regression failed before the filter change because no MIDI steps
were emitted during the 240 ms simulated run. The focused test set now passes.
At Debug log detail, nonzero bridge jog output records `MOTOR_JOG_NUDGE` with
side, selected deck and relative step count; that proves the bridge emitted
jog input, not that Djay applied it to audio. No physical motor or audio test
was started for this experiment.

Formatting, strict all-target/all-feature Clippy, the 194 bridge unit tests,
LSP diagnostics and `git diff --check` passed. A separate test app was built
at `dist/motorized-playback-ratio-deadband/`. Its signature, ZIP and DMG
integrity checks passed, and its bundled executable completed the production
motor replay with 8,393 records. The bundled jog defaults matched
`examples/jog-config.json`.

SHA-256:

- Bundled bridge: `051cd2c9dae60c1641e17d5463c67dfce9ab7b8f3b6ad090150311d4c3c2148c`
- ZIP: `a885f169e74a5a2528bf8ac71fff95d6f7c5552425566c3ead665fcd29c32583`
- DMG: `efaf9c070bcd490b8beaef4e8d16a2791505a6c1a4efde86bcf38532f4d0e5d6`

The repository-wide `cargo test --locked` was not rerun because its last
attempt failed at
`tests/jog_play_cli.rs::explicit_free_play_startup_reaches_log_creation_before_hid`
while a bridge process was running. This did not affect the passing focused
bridge test target or package checks.

The app and archive here still send motorized deviations through `Pitch
Bend (CC)`. They do not test absolute platter position or the Scratch CC path.
Keep these artifacts as the previous experiment; do not use them to evaluate
the absolute-position mapping.

## Absolute Scratch CC experiment

The generated Djay mapping now assigns an additional deck-channel CC 8 input
to `turntableN.scratchingMove` with `controlType=rotary-absolute`. The runtime
converts the S4 counter into 128 platter-angle positions per revolution and
sends changed positions while TT playback is authorized and the wheel is not
touch-owned. Manual touch retains its separate relative Scratch CC or Pitch
Bend route. This replaces the prior motorized Pitch Bend path.

The mapping address changed, so install this build's generated mapping under
a new filename and select it in Djay. No current-BPM output is read; this sends
generic absolute jog position, not Djay's private T7/RANE integration. The
software position/mapping tests do not establish playhead alignment,
turntable-mode gating, or audible rate response; these require the attended
physical test.

## Adjustable Scratch tuning

On October 29, 2026, the user measured about 4 seconds of track advancement
over 20.74 seconds of platter rotation at 145 BPM. The observed playback ratio
was `4 / 20.74 = 0.193`; scaling the prior default 2.7% sensitivity linearly
gives an initial absolute-platter sensitivity of about 14.0%
(`2.7 * 20.74 / 4`). This is a first-pass estimate, not a measured Djay
rotary-sensitivity law.

`platter_position_sensitivity` is now an adjustable 0.1-100% Jog preference
for absolute CC 8, independent of `scratch_speed` for manual relative CC 5.
Both are mapping-time values: save preferences, export a new Djay mapping,
and select it. `scratch_direction` selects normal or reversed polarity for
manual touch-scratch CC 5; it does not reverse the absolute platter stream.
It defaults to Normal pending
retest with the Scratch-mode gate fix. The native settings screenshot remains
unverified because desktop capture is unavailable.

## Scratch-mode gate follow-up

The first absolute-Scratch package emitted CC 8 position values but did not
hold Djay's `scratchingMode` state during untouched motorized playback. The
offline production replay showed CC 8 output and no corresponding scratch-mode
note until physical touch. The current source now holds the existing mapped
`scratchingMode` note while TT playback is authorized. It shares note ownership
with capacitive touch, follows selected-deck changes, and releases on pause,
gate closure or Stop. Debug state transitions are logged as
`MOTOR_SCRATCH_MODE`. The existing generated CC 8 mapping is unchanged.

The production replay regression checks that the scratch-mode note is sent
before the absolute-position stream, remains held across touch ownership,
tracks deck changes, and releases on pause and Stop. Djay's actual response is
still unverified.

This package predates the Scratch-mode gate follow-up below. Its replay emits
CC 8 but not the motorized `scratchingMode` hold, so it is not a valid audible-
response test build.

The isolated app, ZIP and DMG are at
`dist/motorized-playback-absolute-scratch/`. Signature verification, ZIP
integrity, DMG checksum and plist lint passed. The bundled mapping contains
four deck-channel CC 8 absolute Scratch inputs and retains four CC 5 relative
Scratch controls. Its bundled executable completed the production replay with
10,725 records, including 2,283 absolute Scratch CC position messages. Bundled
jog defaults matched `examples/jog-config.json`.

SHA-256:

- Bundled bridge: `feee735c02bf9b76b78b893d32320dd985829b247ad23af8224ce655ff4176ee`
- ZIP: `92d190618ca5d2a9d4155397cf18319d47c2774413af4252908eaf7447d93b7b`
- DMG: `7460e07541cc0d42821ec4b1ee92ed3cb79b5b5f0121003c9a5c0f08c0a7bfde`

The Scratch-mode gate app is at
`dist/motorized-playback-scratchmode-gate/`. It reuses the same CC 8 absolute
Scratch mapping; the already-installed absolute mapping remains compatible.
The bundle passed signature, ZIP, DMG and mapping-plist checks. Its production
replay produced 10,739 records, 2,283 absolute position CCs and 16
`scratchingMode` note messages. Formatting, strict Clippy, all 195 bridge
tests and `git diff --check` passed after the mode-gate change.

SHA-256:

- Bundled bridge: `9935b75627c1efe88f4c9edc04b43ec5e8a9633ce915bcf1060e2406082b6f19`
- ZIP: `433f364b2960e256e69c3e4b18dee5e9ed479e5e13536f6fa43f6bc42e7a9bf2`
- DMG: `77aa43f0606b4f180c6ab3631489e2e9fcfe4d1f8b724a6d850a107fca51a26a`

## Adjustable sensitivity and touch-direction build

The separate app at `dist/motorized-playback-speed-tuning-v2/` defaults
absolute Scratch CC 8 sensitivity to 14.0%, retains manual CC 5 Scratch at
2.7%, and stores touch direction as `normal` or `reversed`. The generated map
was checked for four CC 8 absolute-rotary Scratch entries at 14.0%, four
unflipped relative CC 5 entries, and four existing `scratchingMode` Note 7
entries. The UI defaults and saved JSON schema were exercised in Swift tests.

Rust formatting and strict Clippy passed; all 196 bridge tests and 59 Swift
tests passed. The full `cargo test --locked` suite was not rerun; its prior run
failed in `tests/jog_play_cli.rs::explicit_free_play_startup_reaches_log_creation_before_hid`
while a bridge process was active. The app signature, ZIP, DMG, generated
mapping and bundled production replay passed verification. The replay emitted
10,739 records, 2,283 absolute Scratch CC 8 values and 16 Scratch-mode Note 7
messages. Default Jog JSON matched `examples/jog-config.json`.

SHA-256:

- Bundled bridge: `5b59e8c558dba74d4637212e200c85c50a166f675f7522c69d331e4c3015c1ef`
- ZIP: `30dddb875dd0736080b70eefe67e1e274aade8cbf8e73e4e2dc8a886b0b68e4a`
- DMG: `713550bea51eecc12b262f7b1c81039336345b151b3196587e8678d44ac85f5b`

Native visual capture is still unavailable because computer control is off.
No physical Djay/audio retest was performed; 14.0% is a first-pass linear
estimate from the reported 4.0s/20.74s timing, not a confirmed final setting.

## Filtered app-side motor diagnostic capture

Diagnostics now has an explicit Start/Stop capture control. It writes
timestamped `MOTOR_JOG_POSITION`, `MOTOR_SCRATCH_MODE`, `MOTOR_TT_SELECTION`
and `MOTOR_PLAYBACK_GATE` records to
`~/Library/Application Support/S4 MK3 Bridge/diagnostics/`. Capture requires
the Full profile and Debug logging; the UI's bounded 60,000-character buffer
remains unchanged.

Two Swift regressions cover rejecting capture without Debug, filtering out
`TENSION_WRITE`, and retaining more than 60,000 characters of position data
in the file. All 61 Swift tests passed. The isolated app was visually
spot-checked on its Diagnostics pane while stopped; no hardware or bridge
launch was performed.

The package at `dist/motorized-playback-diagnostic-file-capture/` passed
signature, ZIP, DMG, and mapping verification. The bundled fixture replay
produced 10,739 records, including 2,283 absolute Scratch CC 8 values and 16
Scratch-mode Note 7 messages.

SHA-256:

- App executable: `dd1e88db08cea98b6f6edf8354cbb4ff55853c2d42779ea85d2e37a681a6014b`
- ZIP: `353046c86d6a77fcf1516deb5c9d93cff8fef0507b5830bcd7fd733befca3d96`
- DMG: `694b63863eb0e20be8320ea76924ce2338e5a2ff08f3c081090c9c1eb0f8d017`

## CC 8 wrap-seam phase-offset experiment (completed)

The next isolated test build adds 32 modulo 128 to emitted CC 8 positions.
This moves the physical location of the numeric `127 -> 0` seam from raw
position 0 to about raw position 96, without changing motor target RPM or
mapping sensitivity. The initial absolute position shifts by a quarter turn.
The paired `-3.wav` and filtered log show the dip remains near emitted CC 8
positions `0-15` (about 940 Hz), not `32-47` (about 1002 Hz). Step timing is
nearly equal in those sectors (about 14.35 vs 14.39 ms), so the pitch change
does not track a comparable platter-speed dip. The tone trace has a strong
once-per-revolution component near 0.548 Hz. This supports a Djay response to
the low numeric CC 8 region/wrap, rather than a fixed physical platter sector.
The log does not identify the app binary, so this conclusion assumes the
`wrap-offset-32` build was used; it remains a diagnostic inference.

The isolated app at `dist/motorized-playback-wrap-offset-32/` contains this
fixed diagnostic shift. Formatting and the five `full::platter_position`
tests passed. The app signature, Info.plist, generated mapping and bundled
fixture replay passed; replay produced 10,739 records, 2,283 absolute Scratch
CC 8 values and 16 Scratch-mode Note 7 messages. No hardware test was run.
`macos/build-app.sh` created the app and ZIP but stopped at DMG creation with
`hdiutil: create failed - Directory not empty`; the app bundle was verified
separately and is the only artifact required for this experiment.

SHA-256:

- App executable: `e0b05b3905f96e13a64cd24b97672d818aa865c06e86479727278b738a1897ab`
- Bundled bridge: `a6d2998881818c9b4d152ecbbbc97d682f94cdce82e7df4a8ea4379a1d90b491`

## Relative CC4 jog-bend experiment

### Deviation-only correction follow-up

The raw-relative trial sent normal powered rotation as CC4 instead of removing
the nominal travel. The user reported continuous CC4 and no track advancement.
The corrected path subtracts the configured target's expected counts over
100 ms sensor-clock windows. A temporary +/-0.5 RPM neutral band includes the
observed roughly 32.7 RPM steady baseline at the 33 RPM target. Outside it,
signed count deviation becomes relative CC4, preserving fractional residuals.
Touch, closed playback gates, deck changes and invalid/stale sensor intervals
discard the measurement history. Manual jog routes remain separate.

The absolute-Scratch experiment's motor-owned Note 7 hold is also removed:
untouched TT playback must not remain in Scratch mode while waiting for
relative CC4 corrections. Physical touch/backspin still owns its normal note.
These changes require an attended Djay/audio retest; software replay cannot
establish audible bend gain or whether the mode hold caused the reported stop.

The isolated corrected app is
`dist/motorized-playback-deviation-cc4/S4 MK3 Bridge.app`.
All 199 bridge-target tests, formatting and strict binary-target Clippy passed.
The production replay regression checks silence at 33 and 32.7 RPM and signed
corrections at 32 and 34 RPM; separate regressions cover sensor/counter rollover,
stale duplicates, deck changes and touch ownership.

The exact bundled executable completed hardware-free replay with 8,442 records,
182 MIDI messages, no CC4 after the fixture's return to nominal motion at one
second, and no absolute CC8 output. Note 7 appeared only for touch/backspin,
from 200,001 through 424,001 microseconds. Deep strict code-signature verification,
ZIP integrity, DMG verification and bundle plist lint passed. Packaging used
`CARGO_PROFILE_RELEASE_STRIP=none` after the default release build failed to load
stripped proc-macro libraries on this Mac. No hardware or audio test was started;
the broader Rust/Swift suites and native visual QA were not rerun for this fix.

### Physical deviation-CC4 retest: playback works, bend response fails

On October 29, 2026, the user confirmed playback now advances with TT enabled,
but rim braking/nudging has no audible effect. Ordinary rim jogging with TT off
does bend playback with the same selected mapping. This is a failed motorized
bend acceptance test, not a passing audio result.

The complete capture is
`~/Library/Application Support/S4 MK3 Bridge/diagnostics/motor-diagnostic-2026-10-29T22-59-47Z-31A1D4CF.log`.
The selected mapping is `~/Music/djay/MIDI Mappings/S4 MK3 Bridge 5.djayMidiMapping`.
Its four deck CC4 inputs target `turntableN.pitchBendMove` with `rotary-64`,
sensitivity 2.7 and reaction 17, matching the generated/runtime contract.
The mapping was inspected only; it was not changed.

The capture has 159 bend events (150 negative, nine positive). On deck 2,
braking reaches a measured 14.259 RPM with -92 steps at 23:00:11.001223Z;
forward nudging reaches 70.250 RPM with +182 steps at 23:00:13.909194Z.
Playback permission remains open and no touch owner appears during those
gestures. The bridge therefore observes the disturbances and constructs signed
corrections; the filtered capture does not independently prove each MIDI message
arrived at Djay or establish its audible interpretation.

Small corrections also appear during apparently steady rotation, commonly
32.08-32.49 RPM against the 33 RPM target. The provisional +/-0.5 RPM band does
not keep this physical baseline silent. Startup also emits a large negative
correction before the platter reaches nominal speed.

That tested build emits accumulated corrections in roughly 100 ms bursts,
whereas manual jog output follows individual accepted input reports. Cadence is
a candidate explanation for the differing audio response, not a confirmed root
cause. The user selected quiet rotation for the next experiment: a 2 RPM entry /
0.75 RPM exit neutral band. Current source uses a rolling 100 ms sensor-clock
window and integrates its speed error over each fresh report interval, retaining
fractional counts rather than emitting the entire window in one burst.
Startup, gate reopening, deck changes and post-touch recovery require measured
speed within 0.75 RPM of nominal before corrections can begin. Gate/touch/stale
resets discard qualification, samples and residual corrections. The selected
mapping and manual jog gain remain unchanged.

This remains a testing configuration, not a final release setting. Native UI
verification remains deferred at the user's request.

The new isolated app is
`dist/motorized-playback-quiet-cc4/S4 MK3 Bridge.app`, with ZIP and DMG alongside.
The user's existing `S4 MK3 Bridge 5` mapping remains compatible; the generated
deck CC4 bindings retain `rotary-64`, sensitivity 2.7 and reaction 17.
All 200 bridge-target tests, formatting and strict binary-target Clippy passed.
The regressions cover startup silence, nominal/wobble silence, signed gestures,
hysteretic release, rollover and stale/deck resets. The production replay checks
that sustained +/-3 RPM deviations emit individual signed steps with at most
10 ms between messages, rather than 100 ms bursts.

The bundled executable passed hardware-free replay: 8,442 records, 182 MIDI
messages, no nominal CC4 after one second, no absolute CC8, normal touch/backspin
Note 7 ownership, and a final neutral motor report. App signature, ZIP, DMG,
Info.plist, mapping plist, bundled-default equality, shell syntax and diff
whitespace checks passed. Packaging used `CARGO_PROFILE_RELEASE_STRIP=none`.
No hardware or audio test was started by the agent. Broader Rust/Swift suites
were not rerun for this experiment.

### Quiet-CC4 physical acceptance

On October 29, 2026, the user selected "Works as intended" for the quiet-CC4
retest. That option described quiet steady rotation, audible braking/nudging,
and return to normal after release. Preserve
`dist/motorized-playback-quiet-cc4/S4 MK3 Bridge.app` as the working baseline
with the existing `S4 MK3 Bridge 5` mapping.

This is user-reported acceptance of that workflow, not an independent audio
measurement or confirmation that every wheel, deck, touch/backspin transition,
Stop/freshness fault or recovery path was physically exercised. It also does not
isolate which of cadence, startup qualification and hysteresis resolved the
earlier failure; those changes were tested together. The focused software and
package evidence above remains separate from the physical report. No further
tuning, UI verification or hardware action was performed for this acceptance.

### Two-times motorized braking experiment

After accepting the quiet-CC4 workflow, the user requested stronger slowdown and
selected a 2x braking test. Current source doubles only negative motorized CC4
corrections after the existing quiet filter; positive nudge corrections remain
1x. The 2 RPM entry / 0.75 RPM exit thresholds, startup qualification, input
cadence, mapping sensitivity/reaction and manual jog routes are unchanged.
This is not a motor-force change or a measured doubling of audible slowdown.
The accepted `motorized-playback-quiet-cc4` package is preserved.

The new deterministic gain regression first failed against the accepted source:
one settled second at 30 RPM produced -144 steps rather than the requested -288.
It also checks that 36 RPM retains +144 steps/second and that startup and release
remain silent. After the braking-only multiplier was applied, all 201
bridge-target tests, formatting and strict binary-target Clippy passed.

The isolated app is
`dist/motorized-playback-braking-2x/S4 MK3 Bridge.app`, with ZIP and DMG alongside.
App signature, ZIP, DMG, bundle/mapping plist, bundled-default equality, shell
syntax and diff whitespace checks passed. Its generated mapping is byte-for-byte
identical to the accepted quiet-CC4 package's mapping; keep the existing selected
`S4 MK3 Bridge 5` mapping. The bundled executable's hardware-free replay exited
0 with 8,442 records and 182 MIDI messages, no nominal CC4 after one second,
no absolute CC8, and a final neutral motor report. That nominal replay does not
measure braking strength; the asymmetric-gain regression above establishes the
command change.

Packaging used `CARGO_PROFILE_RELEASE_STRIP=none`. No hardware was started,
the accepted quiet-CC4 package was not replaced, and native UI verification
remains deferred. Broader Rust/Swift suites were not rerun for this experiment.
Actual slowdown, feel and release recovery with 2x braking remain untested.

The preceding raw-relative trial sent motorized platter deltas through the
existing relative `turntableN.pitchBendMove` CC 4 mapping. The bridge no longer
emits absolute CC 8 MIDI in this path; `MOTOR_JOG_POSITION` remains as angle
telemetry, and `MOTOR_JOG_BEND` records the relative bend steps. The
wrap-offset experiment is removed from current source. Djay's CC 4 sensitivity
must be tuned separately from the absolute CC 8 setting. Physical audio
acceptance remains pending.

The isolated test app is at `dist/motorized-playback-relative-cc4/`. Formatting,
the focused powered-platter regression, and all four platter-position unit
tests passed. The app signature, ZIP, Info.plist, mapping plist and default Jog
configuration checks passed. Its offline replay emitted 16,216 relative CC 4
messages and no absolute-position CC 8 messages. No hardware run was performed.

SHA-256:

- App executable: `f9d526435a21884235e1fc03d2d7ed2b213918cd90365759826a659d3953a172`
- Bundled bridge: `be03139bc40cb1eed411aaa9740ad76dcf49c01fc3da624e0e67d4644ba8ff91`
- ZIP: `7ffa940fb051a55698a42edcedee0a4f28dd74d09998182c3dd8fb15d38a1c6f`

## Native GUI evidence limit (previous build)

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

The adjustable platter-sensitivity and touch-direction controls were compiled
and covered by Swift preference tests, but no fresh screenshot was captured.
The desktop computer tool was off, so native visual QA for these controls is
not passed.

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

## October 30 source closeout

At that closeout attempt, quiet-CC4 was the accepted physical baseline and no
physical result had yet been received for the 2x-braking package. The user later
reported it was "much better", then requested a different TT touch/force design;
that report is not coverage of every fault or ownership path. Native UI verification remains
deferred. Neither package nor the selected Djay mapping was modified during
closeout.

The staged integration source was exported independently of the shared dirty
input-reader and standalone diagnostic work. That index-only snapshot passed
formatting, strict all-target/all-feature Clippy, all Rust unit targets
(46 library, 198 bridge, 112 diagnostic and two screen tests), all-target build,
61 Swift tests, shell syntax and plist lint. Two existing Swift unhandled-file
warnings remain. All-target Clippy identified four test-only issues missed by
the earlier binary-only check; they were corrected without changing runtime
behavior or rebuilding the accepted package.

The shared-tree `cargo test --locked` run reached the known
`explicit_free_play_startup_reaches_log_creation_before_hid` failure while the
quiet-CC4 bridge was running. Its competing-process guard runs before the
deliberately invalid capture-path check. The bridge was left running; no guard
or test was weakened. A subsequent subscribed full-suite run also reached the
same assertion failure. The user canceled closeout before any further commit.
That subscription has exited; closeout is not waiting or running automatically.

Only integration files and the integration hunks of the user guide were staged.
`docs/issues.md`, `src/input.rs`, standalone jog-diagnostic changes, their guide
section, capture logs and audio recordings remain outside this commit scope.
The index-only counts differ from shared-tree counts because those separate
unfinished changes are intentionally excluded. Physical acceptance belongs to
the preserved tested bundle; the isolated source snapshot was not physically
tested. Its earlier temporary snapshot is separate from the new diagnostic.

## Subsequent TT speed-to-Scratch diagnostic

The user authorized building mode-specific Vinyl defaults, a configurable
speed-floor Scratch handoff and continuing bounded TT assistance, explicitly
excluding UI verification. This does not reopen the canceled commit closeout.
See [the diagnostic runbook](tt-speed-scratch-diagnostic.md) and
[its software/package evidence](tt-speed-scratch-validation.md).
