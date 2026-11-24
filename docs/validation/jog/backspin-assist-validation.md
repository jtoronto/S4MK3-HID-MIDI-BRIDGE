# Released backspin motor-assist trial

Internal implementation and acceptance record, October 25-26, 2026.
The preceding motor-unchanged experiment is recorded in
[backspin detection validation](backspin-detection-validation.md).

The trial's default-off setting below is historical. For beta.2, the user
subsequently selected motor assistance on at 100% and a 5.6% scratch speed
increase. See the [release verification](release-0.2.0-beta.2-validation.md)
and [current controls](user-guide.md#backspin-detection).

## Approved trial

The user selected assistance continuing after physical touch release, then
approved a bounded first experiment:

- Optional `backspin_assist_enabled`, default false, in the jog preferences.
- Existing maximum assist ceiling, 7000, only on the detected physical wheel.
- Full assistance while the qualified gesture remains touched; after release,
  a linear fade to zero over at most one second, measured from that release.
- Continued or motor-driven movement cannot refresh that deadline.
- Retouch, measured stop/reversal, event/mode cancellation, input freshness loss,
  and Stop/Quit cancel the override. Normal tension selection is never changed.
- MIDI scratch continuation and software speed increase remain separate.
  The user's selected 5.2% software increase is preserved.

The original implementation and initial verification are recorded first.
Attended observations and the subsequent user-controlled strength adjustment
are recorded below, including focused physical acceptance at 100%.

## Evidence path

The real tension decoder and existing smoothing/force law remain in use.
Physical touch stays physical: a separate per-wheel gate authorizes the
bounded override. The normal-tension histories continue independently.
The paced runtime retains its immediate physical-release zero before MIDI
dispatch; current translator qualification is fed back after that dispatch.
Event cancellation then clears an override before the next paced force tick.
The one-second deadline is enforced by the output clock, not MIDI or screen
refreshes. Existing 8 ms movement inactivity and 50 ms input freshness limits
remain in force.

Deterministic regressions cover default-off compatibility, wheel isolation,
real released touch remaining false, the 7000 ceiling, linear fade and exact
deadline despite continued movement, retouch/rest/reversal, cancellation,
freshness recovery without old-event resumption, normal-setting restoration and
Stop priority. Translator/configuration tests cover detection/mode ownership and
CLI selection of the paced motor owner even without a saved tension file.
Native tests cover old preference merging, persistence and disabled-editor
preservation; isolated UI inspection must not open HID/MIDI or change NI state.

## Initial software verification and package

The continued implementation passed 348 Rust tests and 55 Swift tests. Formatting,
strict all-target/all-feature Clippy, the locked all-target build, build-script
syntax and Info.plist lint passed. The unchanged SwiftPM warnings concern
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`.

The force regressions use real numbered snapshots and the shared decoder/force
law. They cover either wheel and both with different release times, an output
tick exactly at the deadline without a new snapshot, cancellation before another
tick, unchanged other-wheel output, Stop preemption and watchdog recovery.
The extra direct-tick test compares the unaffected wheel with the normal
force-law reference: its smoothing output can change between snapshots, so a
fixed 1750 expectation there was incorrect and was replaced with that reference.

The initial package was built separately under `dist/BackspinAssist-20261026/`,
preserving the earlier artifacts. The build, deep strict code-signature check,
ZIP integrity and DMG verification all exited 0. Bundled jog defaults match the
eight-field source file, including assist disabled. The generated mapping matches
the prior motor-unchanged test package byte-for-byte.

```text
df750d4b95d19b5da3e95355b60cb6f468497e76e30cf7405408d4e895b652c9  S4 MK3 Bridge-arm64.zip
b3c0f48e1eb47055d8eff7c9f43d6610563365104b42aa9a9af1c85f667e2e94  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```

At this initial stage, no project commit, push or release had been authorized.

## Initial native verification pause

An isolated helper compiled the actual settings view and controller with copied
defaults, no bridge executable and a rejecting NI-agent implementation. It does
not use the user's saved preferences or open HID/MIDI.

Initial inspection found zero active displays through both Quartz and CoreGraphics.
A bounded macOS display-wake assertion restored the M32UC display. The actual
minimum-size window was captured at 1520 x 1248 pixels in
`.omo/evidence/assist-min-light-top.png`: the default-off checkbox, 5.2% software
increase and complete new assistance caption are visible without overlapping.
This one light-theme frame is partial layout evidence, not a completed state/theme
matrix or proof of interactive behavior.

AX window lookup remained unavailable. Native checkbox input was then rejected
because the interactive session is behind the lock screen. Updating the isolated
fixture's in-memory assist flag succeeded, but another capture failed; it is not
accepted as evidence. The display assertion expired normally, and the offline
helper was closed. Remaining captures and actual input checks await an unlocked
Mac; no unlock, keyboard-input or controller-output workaround was attempted.

Both display/unlock readiness prompts expired without an answer. The stalled
display/wake/console subscription was inspected and stopped: its predicate only
checks for an active display, not an unlocked interactive session or human
readiness. At that pause, no offline helper, wake assertion or acceptance watch
remained running. Native UI and attended hardware acceptance were incomplete.
Fresh watching confirmation is still required after hardware setup,
before starting any bridge or motor trial.

No assist-enabled bridge has been started, and no new motor behavior is claimed
physically accepted.

Read-only preflight found the old menu-bar app and NIHardwareAgent, but no bridge
backend. The real saved jog settings still contain the user's 5.2% software boost
and no assist key; normal tension remains -25. The UI helper uses separate
fixtures and has not changed those preferences.

## Resumed native verification

After the user returned, the unlocked native accessibility tree exposed the real
settings controls. Actual AX presses enabled assist, disabled detection while
retaining the checked assist value, re-enabled detection, and finally returned
assist to off. Cmd+S displayed `Preferences saved.` and wrote the correct
eight-key JSON to the helper's private fixture. The 5.2% software boost and
fixture tension stayed unchanged. No bridge executable was present in that
fixture, and no HID/MIDI or NI operation was started.

Eleven fresh compositor captures under `.omo/evidence/assist-resume-*.png` cover
off/on/disabled-retained states, light/dark themes, enlarged windows, settled
minimum geometry and scrolled bottom sections. Two independent read-only reviews
passed with no blocking product or evidence findings. Full keyboard focus-order
and system reduced-motion behavior were not separately exercised; the new option
uses standard native controls without custom motion. The helper was closed.

All five real preference files were preserved in
`.omo/evidence/assist-prefs-before-1793020417701/`. The real saved values remained
5.2% software boost and -25 tension. The package still matched current Rust/Swift
source modification times, and its deep strict code-signature verification
exited 0. Normal termination was requested for the stopped older app; its exit
notification watcher timed out, but a separate process listing confirmed the old
app and all bridge backends were absent. Djay was running.

Native settings verification is complete. Physical motor acceptance is not.

The real verified app was then opened without Start. Native AX and
`.omo/evidence/assist-real-stopped-baseline.png` confirm `Stopped`, full profile,
normal tension -25, detection enabled, threshold 150000, software boost 5.2% and
motor assist off. No bridge backend was present. Passive USB metadata confirmed
the S4 MK3 was connected, and Djay was open. Existing screen preferences remain
enabled with automatic NI-driver handover and restoration. All first-trial setup
is complete; Start still requires fresh watching confirmation.

## Attended sequence

1. Prepare the verified test app while stopped, leaving ordinary tension and
   software boost unchanged. Check that no competing bridge is running.
2. Describe the one-second powered tail and cancellation behavior, then obtain
   fresh confirmation that the user is watching before Start.
3. With assist off, confirm the previously accepted released backspin/catch on
   one loaded deck.
4. Enable assist and apply through Save and restart while attended. On that
   same wheel, compare a backward flick/release, catching during the tail, and
   letting the one-second fade finish while the wheel is still coasting.
5. Confirm the untouched other wheel does not receive assistance and ordinary
   scratch/nudge and saved tension remain normal.
6. Only after that wheel passes, repeat on the other wheel and then both.
7. Check normal Stop clears force and scratch ownership. Restore the user's
   chosen preference and leave the app stopped unless otherwise requested.

Passing writes, tests or requested force values are not physical acceptance.
Record actual feel and user observations, including any inconclusive result.

## Attended results

With fresh watching confirmation, the assist-off baseline was started. The app
reported `Running` and its packaged backend was observed. The user reported that
the left-wheel released backspin and catch matched previous behavior, including
the untouched right wheel staying still. This is focused baseline acceptance,
not acceptance of the new powered tail or every controller function.

The baseline was then stopped; backend exit and native `Stopped` state were
verified. Assist was enabled and saved while stopped for the next trial.
The saved files still show tension -25 and software boost 5.2%.

The subsequent assist-on attempts produced no clear improvement in feel. The
user reported that the wheel coasted to a stop in approximately the same time
as without assistance. The bridge was stopped before investigation, and backend
exit was observed. Assist was returned to off while preserving tension -25,
software boost 5.2% and all other saved Jog values.

The inspected info-level log confirms that the feature was enabled and armed:

```text
2026-10-26T13:37:53.947512Z  INFO TENSION_ACTIVE: shared physical wheels; mixer faders unchanged tension=-25 backspin_assist=true
2026-10-26T13:39:07.416677Z  INFO BACKSPIN_ASSIST_ARMED side=0
2026-10-26T13:39:07.468887Z  INFO BACKSPIN_ASSIST_RELEASED side=0 released_us=73522850 deadline_us=74522850
2026-10-26T13:39:07.550106Z  INFO BACKSPIN_ASSIST_CANCELLED side=0 reason="motion_stop"
2026-10-26T13:39:10.639118Z  INFO BACKSPIN_ASSIST_RELEASED side=0 released_us=76693265 deadline_us=77693265
2026-10-26T13:39:10.690651Z  INFO BACKSPIN_ASSIST_CANCELLED side=0 reason="motion_stop"
2026-10-26T13:39:12.304413Z  INFO BACKSPIN_ASSIST_RELEASED side=0 released_us=78358376 deadline_us=79358376
2026-10-26T13:39:12.329713Z  INFO BACKSPIN_ASSIST_CANCELLED side=0 reason="motion_stop"
```

These attempts cancelled 25-81 ms after release rather than reaching the
one-second deadline. A motion-stop cancellation is the current investigative
lead, not a confirmed implementation bug: the available lines do not distinguish
fresh measured rest from an input gap or duplicate. They also do not establish
the requested force magnitude, a native-write result or the physical force.
No cutoff was weakened and no behavioral fix is claimed.

## User-controlled strength adjustment

The user requested independent saved on/off and adjustable motor-assist strength
so they can make repeated attempts themselves, as with the software boost.
The implementation adds `backspin_assist_strength`, an integer percentage from
0 through 100, in Rust and the native Jog settings. Missing keys migrate to 100.
Assist remains off by default. At 100 the existing assist ceiling is unchanged;
0 bypasses the optional override. Other values scale only that override, leaving
ordinary saved tension, the software boost, independent wheel ownership,
one-second deadline and cancellation/freshness rules unchanged.

Existing phase-change logs now include strength and requested torque at release.
The bounded `BACKSPIN_ASSIST_MOTION_STOP` line reports the existing cached decoded
sample, its age and last movement time before cancellation. There is no
per-tick/raw-report log and no extra motion history. These fields remain useful
for interpreting repeated trials, but requested torque is not proof of delivered
physical force.

The adjustable build passed 351 Rust tests and 55 Swift tests, formatting,
strict all-target/all-feature Clippy and the locked all-target build. Native
regressions cover migration, strict percentage validation, saved reload and
disabled-editor retention. Real native slider decrement changed 100 to 99, and
the visible Save preferences button acknowledged success and wrote private
nine-key JSON with assist on, strength 99 and software boost 5.2%. This fixture
could not start HID/MIDI or change NI state.

Nine fresh `.omo/evidence/assist-strength-*.png` captures cover both appearances,
minimum/enlarged geometry, 0/37/100%, default-off and disabled-retained states,
and lower Jog sections. Both independent read-only reviews passed without a
blocking product finding. Keyboard focus traversal, hover/pressed states and
app-wide shortcuts were not fully exercised. Background Cmd+S did not produce
a save acknowledgement in this helper; button persistence is verified, not
shortcut persistence for this round.

## Adjustable package handover

The new package is preserved separately under
`dist/BackspinAssistControl-20261026/`. Packaging, deep strict signatures, ZIP
integrity, DMG verification, build-script syntax and Info.plist lint exited 0.
Bundled nine-key Jog defaults match the source file, and the generated Djay
mapping matches the first assist package byte-for-byte.

```text
fc09e6d3ca5caf430433b4e88c3668f680804f5354253f9dff929f832432f218  S4 MK3 Bridge-arm64.zip
ec0fd538038c2b1afde822965c25a0be03611c1ffd6ecce61a55f9bb4c9cf0b0  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```

The real package was opened through Launch Services rather than a timed terminal
session. Its process is owned by launchd. The native Bridge pane confirms no
fixed-duration limit; the real Jog pane and compositor capture show Stopped,
tension -25, software boost 5.2%, assist off and the migrated strength at 100%.
The user's existing eight-key Jog file was not rewritten merely by opening it.
No backend was started with this adjustable package.

The app is left open on Jog for the user to choose levels, save, Start/Stop and
repeat attempts. While running, Save and restart applies changed Backspin
settings; no new Djay mapping is needed. No further automatic motor test follows
from the request for controls.

At handover, physical gain at the new levels, the reason for early motion-stop
cancellation, retouch/deadline behavior on hardware and independent both-wheel
assist remained unverified. No preferred motor-assist strength had been accepted.

## User-led 100% trial result, October 26, 2026

The user selected "Noticeable change" and reported:

> Motor assist strength at 100% works well.

This is a positive physical result for the adjustable assist at 100%, following
the earlier inconclusive trial. The saved Jog file confirms detection and motor
assist enabled, strength 100 and software boost 5.2%.

On the follow-up covering both wheels, clean catching of the backspin and no
unwanted assistance on the other wheel, the user confirmed:

> All those checks pass

This accepts the useful 100% assist setting, both-wheel operation, catching and
wheel isolation in the user's trials. It does not measure the exact release-fade
duration or establish the cause of the earlier short motion-stop cancellations.
That historical lead is retained rather than pursued as a current defect after
these passing trials; no behavioral fix is claimed.

No source behavior or user preference was changed in response. The user's app
and hardware remain under their control; no Stop, restart or further motor test
was triggered.
