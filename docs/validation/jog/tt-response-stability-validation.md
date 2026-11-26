# TT response-stability validation

Date: November 2, 2026.

Status: source checks, development and packaged executable replay, and portable
package verification passed. Physical acceptance and the ordered morning test
remain unperformed.

The verified morning bundle is
`dist/tt-response-20261102/S4 MK3 Bridge.app`.
The version remains `0.2.1-beta.1`, numeric version `0.2.1`, build `7`.
The unchanged version alone cannot distinguish it from previous test builds;
use this separate path and the hashes below. This is an unpublished diagnostic.

## Evidence available before the change

The reported November 2 capture used a 1% TT floor and contained four automatic
upper Scratch entries near 66 RPM, with maximum measured RPM about 90.3.
The reported 10x-gain run was not captured. Don't attribute the recorded trace
to that run or infer its actual runtime gain from an editor value.
These observations motivate the change; they don't establish that the proposed
response fixes physical acceleration or Djay's audible slowdown limit.

Source inspection for this documentation located:

- `src/full/platter_bend.rs`: rolling 100 ms RPM measurement, qualification,
  fractional relative-step accumulation and the existing 2x braking/1x forward
  full-error law.
- `src/full/platter_gesture.rs`: floor calculation, Scratch Note 7 ownership,
  raw CC5 motion and CC4 routing.
- `src/tension/playback.rs`: continuous assistance selection, 20 ms motor
  measurement window, force calculation and touch-release fallback.
- `src/jog/platter.rs`: normal speed regulation and PI state.
- `src/full/jog.rs`: saved floor range 1-95/default 50, gain range 0.1-10/default
  1, and the default motor values that give 5280 force units.
- `src/full/mod.rs`: `S4_PLATTER_STATUS` emission after MIDI sends.
- `macos/S4BridgeApp.swift`: capture filtering, launch lifecycle and saved
  configuration handling.

At the initial inspection, the core still contained the old 2/0.75 RPM bend
hysteresis, symmetric upper Scratch entry and constant hold assistance. Core
and app changes are separate work tracks. This draft records the approved
replacement contract below, not a claim that those changes were already
verified at that inspection.

## Approved contract to verify

For `e = measured_rpm - target_rpm`, the bend path is neutral when
`abs(e) <= 2.25`. Neutral clears fractional residual; there is no active bend
hysteresis. Across the next 1 RPM, multiply the existing full-error law by
`smoothstep(x) = x*x*(3 - 2*x)`, where
`x = clamp((abs(e) - 2.25) / 1, 0, 1)`.
At 3.25 RPM deviation and beyond, the original full-error law is retained:
braking multiplier 2, forward multiplier 1, then saved gain before quantization.
Gain remains 0.1-10 with default 1. No gain or floor preference is reset.

With TT Vinyl off, automatic Scratch entry is only at or below
`floor = target_rpm * floor_percent / 100`. Exit is at or above
`floor + min((target_rpm - floor) * 0.25, 2)`.
At 33 RPM and 50%, entry is 16.5 RPM and exit 18.5 RPM. At 1%, entry is
0.33 RPM and exit 2.33 RPM. Forward nudges must not enter automatic upper
Scratch. Explicit Vinyl-on top touch still owns Scratch.

Hold assistance is
`F0 * clamp((target_rpm - measured_rpm) / min(2, target_rpm), 0, 1)`, where
`F0 = min(max_force, base_force + force_per_rpm * target_rpm)`.
With defaults, `F0 = 5280`. Full stopped/reverse assistance remains; the final
2 RPM below target taper linearly to zero. At/above measured target, hold
assistance and the touch-release fallback request zero. The PI integrator must
not accumulate against the hand. Playback, freshness and ownership gates
still apply; paused TT movement does not authorize drive.

Diagnostics must retain the actual runtime configuration receipt
`S4_MOTOR_CONFIG` for each launch, including when capture starts after launch.
A receipt must not be manufactured from later unsaved editor values.
Bounded `MOTOR_TT_STATE` reports expose touch, authorization, owner and
requested force. Requested force is not measured applied torque.
Existing `S4_PLATTER_STATUS` supplies physical RPM and outgoing CC4 evidence.
There is no `MOTOR_SCRATCH_MODE` runtime emitter to require in a capture.

The mapping is unchanged by this response update. No export is necessary.
The already-selected mapping must supply the existing Internal-mode feedback.

## Ordered morning test

Perform these steps in order after the lead has identified the built artifact.
This is an attended motor test: TT selection can start platter rotation.
If an agent is to trigger a control, finish setup first, state readiness, and
wait for the user to confirm they are watching before that trigger.

1. Finish the old session. Choose **Stop bridge**, then **Quit S4 MK3 Bridge**
   in the old app. Check that no separate CLI bridge remains running. Closing
   Settings alone does not quit the app.
2. Open `dist/tt-response-20261102/S4 MK3 Bridge.app` in Finder. Don't open the
   older Applications copy for this comparison. Confirm the app's diagnostic
   launch path points into this bundle; the unchanged version alone cannot
   distinguish the two builds.
3. Record the existing Jog settings before editing. Don't reset preferences.
   Set **TT pitch-bend gain** to **1x** for the baseline. Initially keep the
   current saved **TT pitch-bend floor**, target RPM, motor forces and Djay
   Pitch Bend Speed/Reaction unchanged. Record whether the current floor is
   1%, 50% or another value. The default is still 50%, not an instruction to
   overwrite the saved floor.
4. Select **Full controller + LED feedback** and **Debug** log detail. Save
   preferences and restart the bridge, or save then start if stopped. Leave
   TT off while preparing. Don't export or reinstall the mapping for this
   update; retain the current Internal-feedback mapping and custom maps.
5. Open **Diagnostics > Live platter diagnostics**, then choose **Start motor
   diagnostic capture**. Use **Show file** to locate the capture. Confirm it
   includes the actual launch's `S4_MOTOR_CONFIG` receipt and that loaded gain,
   floor and target match the saved baseline. If the receipt is absent or
   mismatched, stop and resolve that before treating a trial as evidence.
   Keep each later launch's receipt with its trial.
6. Load the original steady 1 kHz reference track into an Internal deck, not a
   recording containing an earlier sweep. Prepare an audio recording of Djay's
   output and note the deck/side and trial start time. Keep the other platter
   clear. Start playback, select TT yourself, and confirm JOG is blue
   (Vinyl off). If a saved choice makes it red, use JOG to select blue.
7. Leave the platter hands-free until Diagnostics shows qualified **Bend**.
   Record physical RPM and bend output. Within target +/-2.25 RPM, expect
   neutral bend output after the display's recent-output interval clears, not
   repeated CC4 correction or Scratch. Correlate the trace with steady audio;
   zero MIDI bend alone is not proof of constant audible speed.
8. Brake gently at the rim without touching the top. Record RPM, negative
   CC4 output and audio. Return toward target and confirm bend becomes quiet
   again inside the same 2.25 RPM band. Repeat with a controlled forward rim
   nudge: expect positive CC4 above the onset, no automatic upper Scratch, and
   recovery toward target after release. Don't chase the earlier 66/90 RPM
   excursion to provoke a removed upper handoff.
9. At settled rotation, lightly touch the top while allowing it to turn, then
   release. With Vinyl off, contact alone must not select Scratch. Record touch,
   authorization, owner, requested force and RPM together. Assistance must
   taper near target and request zero at/above measured target, including the
   release fallback. Look and listen for acceleration or a release kick;
   repeat the contact/release while the capture and audio recording continue.
10. Progressively slow through the recorded lower floor, hold stopped, then
    make a small controlled reverse movement. Expect lower Scratch entry,
    signed CC5 motion when moving, no invented movement when stationary, and
    retained forward resistance. Allow recovery past the calculated exit and
    confirm Scratch releases, pitch bend resumes, and neutral rotation is quiet.
    Judge the audible transition separately from `MOTOR_SCRATCH_HANDOFF`.
11. Select JOG red while TT remains selected. Briefly touch the top to confirm
    explicit Vinyl-on Scratch still works. Release, then restore JOG blue.
    This deliberate Scratch ownership is not an automatic upper-speed entry.
12. Pause the selected deck. Expect motor drive to stop; turning the paused TT
    wheel should still scratch. Resume and check fresh qualification, smooth
    recovery and quiet hands-free rotation. Switch TT off and verify drive
    removal. Re-enter TT for the next trial only while attending the controller.
13. Stop the capture and audio recording. Record the outcome of each gesture
    and retain the log, audio, loaded config receipt and mapping identity
    together. Repeat steps 5-12 on the other physical side with a separate
    capture so side-specific behavior remains identifiable.
14. Only after the current-floor baseline, compare a different floor. If the
    baseline was 1%, compare 50%; if it was 50%, compare the previously used
    1% setting. For another baseline, compare 50% and record the choice.
    Keep gain 1x and all other settings fixed. Save/restart, start a new capture,
    verify its new receipt, then repeat the lower-floor and recovery checks.
    Don't infer an audio floor from the selected percentage.
15. Finish by turning TT off, pausing playback and choosing **Stop bridge**.
    Confirm both motors stop. Stop the capture and recording if still active,
    retain their files, then quit the app. Turn TT off or stop the bridge
    before quitting Djay; cached playback feedback silence is not a disconnect
    signal. Record the floor/gain left saved for the next session.

For a later audio-floor comparison, follow
[Calibrate against the actual CC4 path](tt-speed-scratch-diagnostic.md#calibrate-against-the-actual-cc4-path).
Keep that calibration separate from this gain-1x baseline.

## Performed source evidence

- `cargo fmt --all -- --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo test --locked` (495 tests), `cargo build --locked --all-targets`,
  `bash -n macos/build-app.sh`, `plutil -lint macos/Info.plist` and
  `git diff --check` passed. Capture:
  `.omo/evidence/tt-response-20261102/source-final.log`.
- `swift test --package-path macos` passed all 68 tests after receipt forwarding
  was restored and verified. Existing SwiftPM warnings about the icon and jog
  diagnostic build script remain. Capture:
  `.omo/evidence/tt-response-20261102/swift-final.log`.
- The real app-controller/process lifecycle regression launches gain 1, truncates
  the display log, changes the editor and saved file to gain 10, then starts
  capture. The capture still identifies launched gain 1. A restart produces a
  second launch ID with gain 10. Disabling receipt forwarding fails this check
  with zero receipts; restoring it passes. The test subscribes before launch
  and uses one-shot readiness events, not sleeps.
- A temporary obsolete upper exit bound failed the abrupt lower-Scratch
  recovery regression. The unbounded exit was restored before the full checks.
- The initial extended fixture caught a 32-bit fractional-counter overflow at
  66 RPM; its accumulator now uses widened arithmetic. Strict lint also caught
  an oversized fixture function; the complete RPM profile is a const helper.
- The development executable's production-service replay passed all six
  combinations of floor 1/50 and gain 1/5/10. Each produced 13,060 motor records,
  with identical motor/RPM streams across gains. It verified zero neutral CC4
  after braking and nudging, no upper Scratch, signed reverse on both decks,
  bounded near-target assistance, zero overspeed touch/release force and final
  neutral. Capture:
  `.omo/evidence/tt-response-20261102/development-replay.jsonl`.

## Verified portable package

Built with:

```sh
S4_BRIDGE_OUTPUT_DIR="$PWD/dist/tt-response-20261102" \
  CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh
```

The app, ZIP and DMG are under `dist/tt-response-20261102/`; previous builds
were preserved. Build output:
`.omo/evidence/tt-response-20261102/package-build.log`.

`codesign --verify --deep --strict` passed for the app, extracted ZIP copy and
read-only mounted DMG copy. `unzip -tq` and `hdiutil verify` passed. The packaged
Swift and Rust executables, Info.plist, three default configuration files and
mapping matched in all three copies. Bundled defaults matched source; the
mapping is byte-identical to the preceding gain-test package, so no mapping
export is needed. Version is 0.2.1-beta.1, numeric 0.2.1, build 7.
The packaged Rust UUID matches the release build:
`6D6F64E7-B7A9-3123-8540-12E7096561A6`.

The exact bundled `Contents/Resources/s4mk3-hid-midi-bridge` ran
`--replay-motor-fixture playback --jog-config <input>` for all six combinations
of floor 1/50 and gain 1/5/10. Inputs contain only
`tt_pitch_bend_floor_percent` and `tt_pitch_bend_gain`; remaining settings use
the shipped defaults. Each run emitted one matching full configuration receipt
and 13,060 actual output-service motor records. The independent oracle checked
the same noise, touch/release, reverse, routing, final-neutral and cross-gain
invariants as the development run. All six passed. Captures:

- `.omo/evidence/tt-response-20261102/packaged-replay.jsonl`
- `.omo/evidence/tt-response-20261102/package-verification.log`

SHA256:

| Artifact | SHA256 |
| --- | --- |
| ZIP | `f816a706402528f261957ef51a240182577dfac1027d0f4dfdc9289c53809fcc` |
| DMG | `6cc7e1dea95da10d76534dc8904d0cffe99c898a906f3e371568d748b98e0248` |
| Bundled Rust executable | `f8209cf52ebefaa5f818f49df02e076d7482ecb30801cc90b5a88cabbe1a158b` |
| Bundled app executable | `3927a2269e711d62ccef98b0c9d04c7ff1b73d2697203148e76cb949dc9f9621` |

The DMG was detached and temporary archive extraction removed, with
`PACKAGE_QA_CLEANED` recorded. macOS reported deprecation warnings for hdiutil
attach/detach, not verification failures. Signing remains ad-hoc, not Developer
ID signing or notarization.

## Physical and verification limits

- The morning sequence above has not been performed by this documentation
  task. There is no new hardware, audio, force-feel or UI acceptance claim.
- Source tests and offline replay can exercise math, ownership, gates and
  output requests. They cannot prove physical stability, applied torque,
  Djay's audible floor or continuity at the Scratch transition.
- The November 2 historical capture does not include the approved replacement
  behavior, and the uncaptured 10x report cannot serve as a measured comparison.
