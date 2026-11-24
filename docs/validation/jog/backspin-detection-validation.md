# Backspin detection experiment

October 25, 2026. Internal development and acceptance record.
Normal operation is described in the [user guide](user-guide.md#backspin-detection).

The zero-increase, motor-unchanged defaults below describe the original
experiment. Beta.2 uses a 5.6% scratch speed increase with motor assistance
enabled at 100%, as subsequently requested by the user. See
[release verification](release-0.2.0-beta.2-validation.md).

## Agreed first experiment

- Detect backward acceleration while the top is touched in Turntable mode.
- Retain scratch ownership through release while actual backward motion continues.
- Expose detection enable, acceleration threshold and additive scratch-speed
  increase, with the increase defaulting to zero.
- Show BACKSPIN and the owning logical deck on both Info and STEMS/MUTE pages.
- Keep motor/tension behavior unchanged. Assistance is not part of this build.

Normal nudge, shifted seek, grid edits, minimal MIDI and held-note ownership
must remain intact. Optional gain is an intentional effect, not a claim of more
accurate distance per revolution.

## Attended input-only evidence

The user performed slow backward turn/release, free backspin and released
backspin caught by hand, on both physical wheels. The capture sent no output
reports and created no MIDI ports. It completed with exit code 0 and 10487
snapshots. The raw file is ignored local evidence:
`.omo/evidence/jog-diagnostic/backspin-detection-20261025.jsonl`.

The existing decoder replay also exited 0. Slow turns peaked below 3900
counts/s using a 24 ms window; deliberate flicks reached approximately
16000-23000 counts/s. Comparing successive filtered windows gave backward
acceleration peaks around 38500-51355 counts/s^2 for the slow/catch gestures,
versus 419655-598573 for the four deliberate flicks. The starting threshold is
150000 counts/s^2. These are measured separations for the captured gestures,
not universal scratch-style calibration.

Zero or duplicate samples occur during real spin, and two-count chatter
appears near rest. Continuation therefore uses filtered displacement and
direction, not one zero reading or an invented audio tail.

## Software evidence

The pre-fix translator regression compiled and failed with:

```text
backspin released scratch mode: [[128, 7, 0]]
```

The corrected focused backspin run passed six tests covering continuation,
disable/threshold/mode gating, deck/Shift ownership, independent wheels,
retouch, reversal, discontinuity, freshness, Stop and additive event gain.

A temporary real-parser replay of the entire attended capture classified its
eight release edges as:

```text
left:  ordinary=false, free=true, caught=true, catch-release=false
right: ordinary=false, free=true, caught=true, catch-release=false
```

That replay used USB enumeration only to obtain Encdr's opaque device identity;
no interface was opened and no MIDI or hardware output was created. This proves
captured-input routing, not Djay audio behavior. The temporary replay test is
removed after saving the evidence.

## Integrated source, UI and package verification

The final source passed 333 Rust tests and 55 Swift tests. Formatting, strict
all-target/all-feature Clippy, locked all-target build, build-script syntax and
Info.plist lint passed. The combined source-check monitor remained open at its
final Git step; a separate `git --no-pager diff --check` exited 0. No failed test
was ignored or weakened. SwiftPM still reports the two unchanged unhandled
files, `macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`; all 55 tests
completed with exit code 0.

Native UI inspection used the real `BridgeSettingsView` and `BridgeController`
with isolated copied defaults, a rejecting NI-agent implementation and no bridge
executable. The actual detection checkbox disabled the threshold and gain
editors. Minimum/large windows, light/dark appearance, zero/nonzero gain and
bottom scrolling were inspected through the real window compositor. No saved
user preferences, MIDI/HID ports or real NI processes were changed.
Local evidence is `.omo/evidence/backspin-native-{min,large}-{light,dark}.png`.
An initial view-cache attempt missed SwiftUI compositing and was replaced by
these valid compositor captures; it is not counted as visual evidence.

The CLI exported all 48 offline screen previews. The four backspin variants
were inspected at native 320 x 240 resolution: Info and STEMS, with and without
FX warnings. Both headers, all four numeric readouts and all eight pads remain
legible without overlap. Inactive-state raster regressions preserve old output.
Local evidence is `.omo/evidence/backspin-screen-previews-final-20261025/`.

The separate test bundle and archives are in `dist/Backspin-20261025/`; the
prepared release artifacts at the ordinary `dist/` paths were not replaced.
The app passed deep strict code-signature verification, ZIP integrity and DMG
verification. Bundled jog defaults match source byte-for-byte. The generated
mapping is byte-identical to the previous prepared app's mapping, so a mapping
reinstallation is not required for this runtime experiment.

```text
0a6fc7141ba499d43ff3a9368fe65dec8aca4603f71511d0b054fe4e8fce81fd  S4 MK3 Bridge-arm64.zip
21391c2fb132b0ee3658bcdb3ba59c61d37bd5306b9136f05f907a6fc4f625e5  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```

## Attended baseline acceptance

The user confirmed watching before the experimental package was started.
The app then reported Running and Screens active, with full MIDI and feedback
ports ready and Djay feedback received. Existing tension remained -25; detection
was enabled at 150000 counts/s^2 with 0.0% extra scratch speed.

The user reported:

> Done. Backspin indicator worked and cleared correctly. Backspin itself seems to work (continues playing backwards even after release).

They separately confirmed:

> Catch also works as expected.

On the targeted follow-up covering both wheels, free continuation, catching,
ordinary slow release and rim nudging, they selected **All checks pass**.
These are positive live audio/feel observations for the baseline configuration,
not an inference from the software tests.

A passive MIDI capture completed with exit code 0 and 4566 messages. Its first
wheel's Deck B trace contains 3332 scratch CC5 messages across ten completed
touch-note intervals, with no touch hold left open at capture end. That capture
does not cover the later other-wheel check; that evidence is the user's report.
The ignored local file is
`.omo/evidence/backspin-live-midi-1792951580823.log`.

The live native boost slider advanced in 0.1% steps to 1.0%. Save and restart
persisted `backspin_speed_increase: 1`, left normal Scratch speed at 2.7%, and
replaced the bridge process successfully; the app returned to Running. This
verifies the real preference and restart path, not the audible boost effect.
The user then reported:

> No noticable difference at 1%. Closer to desired behavior at 5.2%

Their actual saved preferences confirmed 5.2% extra speed, still with 2.7%
normal Scratch speed and threshold 150000. The 1.0% audible comparison is
inconclusive; 5.2% is the user's better-feeling trial value, not a measured
distance calibration or a change to the shipped zero default.

Detection was disabled through the real app, saved, and the bridge process
restarted; the user's saved 5.2% value was retained in the disabled configuration.
The user selected **Works as expected** for scratch ending on touch release,
no BACKSPIN indicator and normal touched scratching. The separate 120-second
passive capture contained zero messages; it is not proof of that behavior.
The user's attended observation supplies the physical evidence.

Detection was then re-enabled and saved through the app. The replacement bridge
ran with detection enabled, threshold 150000 and the user's 5.2% boost, retaining
normal Scratch 2.7% / 150%, bend 2.7% / 17% and tension -25. The shipped default
boost remains zero.

For the final check, the user selected **Bridge stopped cleanly** on the
clarified question requiring the bridge app's Stop while BACKSPIN was still
visible, followed by normal playback using Djay's own Play button. The process
exit subscription completed with exit code 0, and the real app showed Stopped
and "Bridge stopped." NIHardwareAgent was running again, with
`restorePending: false`.

The final passive capture contained 1081 MIDI messages. Its last message was
Deck B scratch-touch release `[81, 07, 00]` at
`2026-10-25T18:46:02.372962Z`, with no touch note left held. The recorder was
deliberately terminated after observing bridge exit, rather than left running
to its duration limit. The ignored local file is
`.omo/evidence/backspin-stop-midi-1792953406756.log`.

The candidate is left stopped with detection enabled, threshold 150000, user
boost 5.2%, normal jog values unchanged and tension -25. Temporary replay/UI
helper sources and the root debug journal were removed. Project changes remain
uncommitted; no push or release was performed.

### Verification limits

The first motor-unchanged experiment has live acceptance for continuation,
catching, ordinary release/nudge on both wheels, indicator clearing, disabled
detection and shutdown during an event. The boost feedback is subjective, not a
distance-per-revolution calibration. Both screen layouts and deck/Shift ownership
transitions have deterministic/raster coverage; the live reports did not
separately identify both screen pages or a deck switch during coasting.
Freshness/discontinuity and minimal-profile boundaries have automated coverage,
not a new physical fault-injection or minimal-profile session.

## Ordered physical check

Finish setup before asking the user to watch. Do not start the bridge until
the user confirms readiness.

1. Open the experimental app while stopped. Select Full controller, enable
   screens, enable Backspin detection, set acceleration to 150000 and speed
   increase to 0.0%. Preserve the user's hardware tension.
2. Confirm Djay has the working mapping selected. Its normal Scratch speed
   must match the app's configured Scratch speed. Load a local track, use
   Turntable mode and turn Slip off for the comparison.
3. Tell the user that Start will run normal control/LED/screen service with
   saved tension, but no new motor assistance. After watching confirmation,
   start immediately.
4. On the left wheel, compare one slow and one fast backward turn while touched.
   Check normal distance and stopping response. Make an ordinary slow
   release: playback must not remain latched in scratch.
5. Flick backward and release. BACKSPIN must appear for the original deck;
   audible reversal must follow the coasting platter and end at rest.
   Repeat while catching the platter by hand, then check normal rim nudge.
6. Repeat steps 4-5 on the right wheel. Check Info and STEMS/MUTE indicators.
   During a separate event, switch the displayed deck and confirm that the
   indicator and movement retain the original owning deck.
7. Set speed increase to 1.0%, Save and restart, then repeat one backspin.
   It should cover more track than zero boost without altering ordinary
   scratch/nudge. Restore 0.0% after comparing unless the user chooses otherwise.
8. Disable detection, Save and restart. A released flick must return immediately
   to nudge as before. Restore the user's selected detection setting.
9. Stop the bridge during a detected event. Confirm normal Stop releases scratch
   and leaves the existing tension lifecycle unchanged.

Record the user's actual observations and exact limits rather than inferring
audio success from MIDI messages, screen pixels or unit tests.
