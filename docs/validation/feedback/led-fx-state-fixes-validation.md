# Fixed-color lamps and FX pickup warning corrections

October 26, 2026. Internal verification record. Normal operation is covered by
the [user guide](user-guide.md), [LED reference](led-feedback.md) and
[pickup reference](controls-state-sync.md).

## Reports and mechanisms

The user reported FLX/REV always bright red, Shift always bright white, and
brief FX SYNC warnings during ordinary knob movement, especially dry/wet.

### Fixed-color intensity

The saved LED preferences contained active brightness 127 and inactive
brightness 50. The renderer copied those values directly into the native report.
The controller-specific Mixxx reference uses NI base/intensity packing for these
button outputs, rather than an unconverted MIDI-like brightness byte.

The new regression feeds real MIDI mode feedback through the decoder/cache,
selects C/D, applies physical Shift state and renders both sides. Before the
production correction it failed at lamp payload offset 55:

```text
left: 50
right: 69
```

The corrected fixed-color path maps a nonzero seven-bit preference into a
white-base byte with four intensity steps. Zero, unknown software feedback and
the inactive-dark preference retain genuinely dark output. Factory lamp colors
remain fixed. The saved values and mapping addresses are unchanged.

Three earlier assertions expected raw Play brightness bytes. They were updated
to the corrected native bytes, without changing their unknown-state,
hidden-deck or stale-meter behavior.

### FX pickup projection

The warning renderer compared physical positions directly with cached software
values. This could report a mismatch after the translator had already acquired
pickup and sent the new value, but before Djay returned it.

The correction publishes the existing per-slot acquisition gates into the
shared LED/screen warning state. It does not change MIDI emission or introduce a
delay/debounce. Unknown feedback remains unknown. Retargeting and external
software changes still re-arm the appropriate pickup gate.

The worker reproduced a pre-fix failure in
`acquired_fx_turn_does_not_warn_while_software_feedback_catches_up`.
Its focused `cargo test --locked fx_ -- --nocapture` passed 2 library and
13 binary tests afterward. The final regression also examines separate
parameter and wet-slot feedback arrivals, not only the final batched state.

## Combined software verification

The final combined source passed 353 Rust tests and 55 Swift tests.
Formatting, strict locked all-target/all-feature Clippy, the locked all-target
build, build-script syntax, Info.plist lint and whitespace checks exited 0.
The two existing SwiftPM unhandled-file warnings are unchanged.

The first full run exposed one remaining old Play-byte assertion in the
stale-meter test. Only that expected byte changed to the corrected format;
the complete run then passed. No test was removed, skipped or weakened.
The FX regression includes partial wet feedback, actual MIDI emission,
unknown state, genuine external edits and independent slot pickup.

## Separate test package

The corrected app is
`dist/LedFxFixes-20261026/S4 MK3 Bridge.app`, with ZIP and DMG alongside it.
Build, deep strict signature verification, ZIP integrity and DMG verification
all exited 0. Bundled LED/Jog defaults match the source examples byte-for-byte.
The generated mapping matches the accepted BackspinAssistControl package,
so mapping reinstallation is unnecessary.

```text
8e689f8b02823633c7dd94c38dd6a0daed64baed4900772361304002f21b502e  S4 MK3 Bridge-arm64.zip
5259d6d40ae7946942419fb0559551f1170eb2f3fe997e85ec4ff223f7c56dc8  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```

Earlier packages are preserved. This app has not been launched or substituted
for the user's current running app.

## Ordered physical check

Use the separately built corrected app rather than the still-running old build.
No new mapping export is required.

1. In the old bridge app, Stop the bridge and Quit the app. Merely closing its
   settings window leaves the menu-bar app running.
2. Open `dist/LedFxFixes-20261026/S4 MK3 Bridge.app`. Check that the existing
   saved preferences are loaded; do not reset them. Leave normal tension and
   Backspin settings as chosen.
3. While watching the controller, Start the full-profile bridge. This starts
   normal MIDI/LED/screens and the saved tension behavior, not an LED-only mode.
4. With Slip off in Djay, check that FLX is dim red. Toggle FLX and confirm bright
   red when Slip is on, then dim when off. Repeat on the other physical side.
   A mouse change of Slip in Djay should update the same lamp.
5. Hold REV and confirm bright red during reverse, returning to dim on release.
   Check Shift is dim white at idle, bright while held and dim again on release,
   on each physical side. Check C/D selection uses their own cached mode states.
6. Start FX testing with the left strip targeting A and the right strip B. Pick
   up each parameter and dry/wet value, then turn normally. FX SYNC and blinking
   controls should not briefly reappear just because feedback is returning.
7. Change a parameter with the mouse in Djay: the corresponding genuine pickup
   warning should still appear and clear when the physical knob crosses it.
   Change a strip's target and check the same pickup protection for the new deck.
8. If both strips target one deck, moving one can legitimately make the other
   require pickup. That is a real mismatch, not the feedback-lag flash.

## Physical limits

Actual lamp intensity and live warning behavior are not established by the
renderer regression or a passing build. The user's old app, saved preferences,
controller and MIDI/HID ownership were not changed during software verification.
No hardware test or app restart was automatically triggered.

The physical-result question expired after 30 minutes without an answer.
No controller pass or failure is inferred from that timeout. Software and
package verification are complete; the ordered physical check remains available
for the user, without another automatic prompt or hardware action.

## REV/FLX idle contrast follow-up

The user subsequently tested the LedFxFixes build and reported that REV/FLX
idle was still too bright, requesting the same idle intensity as JOG.
Their saved fixed-lamp inactive brightness is 50 (intensity 1), whereas
JOG uses palette inactive intensity 0. The follow-up targets only known-off
REV/FLX: use that same palette backlight setting, keep their active level and
unknown-state darkness, and leave Play, Shift, saved preferences and FX behavior
unchanged. The existing real-renderer regression now compares the REV/FLX
intensity bits directly with the actual JOG output on each physical side.

The pre-fix comparison failed with REV/FLX intensity 1 versus JOG intensity 0.
The corrected test covers default and non-default palette idle settings,
unchanged active output, unchanged Play/Shift levels and unknown-deck darkness.

Formatting, strict Clippy and all 353 Rust tests passed, followed by the locked
all-target build, 55 Swift tests, shell/plist and whitespace checks. Rust tests
used `cargo test --locked -- --test-threads=1`: the initial parallel run failed
an existing diagnostic-startup assertion, while a standalone invalid-path
probe produced the expected pre-HID rejection. These diagnostic tests launch
processes subject to a process-global ownership guard; the full serial run
passed without changing tests, weakening the guard or stopping the user's app.

The separately preserved follow-up app is
`dist/RevFlxIdle-20261026/S4 MK3 Bridge.app`. Build, deep strict signatures,
ZIP integrity and DMG verification exited 0; the mapping and bundled LED
defaults match the previous LedFxFixes build/source. No mapping reinstall or
preference change is needed. The app was not launched automatically.

```text
ae07d59ad149db130958c59513ed3a0aecd650fd266533e805650e63a7a682c0  S4 MK3 Bridge-arm64.zip
1aa5a5e4a2cba446bff52e323b546ac61af08c18405f5843800655535c566c0e  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```

The user tested this follow-up build and reported "Much better now" on
October 26, 2026, accepting the revised REV/FLX idle contrast and requesting
the changes be committed. This confirms that brightness refinement, not
a separate physical acceptance result for Shift or FX behavior.

The subsequent October 26 report, "Tested on the DJ laptop. All checks out
good.", establishes general acceptance of the current LED/FX build on the
intended laptop. This supersedes the pending general physical status above,
without claiming exhaustive per-control or fault-path coverage.

## First-gig TT and multi-deck Instant FX follow-up

November 1, 2026. The user reported their first gig went well, then requested
red TT indicators, better deck-selector contrast and multi-deck Instant FX.
Linked LOOP/MOVE sizing was explicitly deferred and is in
[the issue backlog](issues.md#deferred-ideas).

Current source fixes TT to red without changing its enabled/disabled intensity
or JOG's Vinyl colors. Channel ON independently toggles arming on A-D, with only
A initially armed. Central Instant FX presses address all armed decks and keep
their original addresses until release or Stop. Empty-target presses are also
consumed until release. Filter-then-ON preserves the armed set. Shared FX lamps
aggregate actual armed-deck feedback without inventing missing state.

The TT regression and three multi-deck routing/ownership regressions failed
against the old behavior. The shared-lamp regression also failed under a
temporary mutation that ignored armed B-D; that mutation was restored.
All 477 Rust tests passed under ordinary `cargo test --locked`, with no serial
override, and all 62 Swift tests passed. Formatting, strict locked all-target/
all-feature Clippy, locked all-target build, shell syntax, plist lint and diff
checks passed. The initial Clippy run flagged tuple-to-array syntax in a changed
test fixture; the fixture was corrected before the complete passing Rust run.
SwiftPM still reports the two existing unhandled files, `Assets/AppIcon.png`
and `build-jog-diagnostic.sh`; no Swift source or package definition changed.

The local saved LED preferences and bundled defaults both use inactive palette
intensity 0, the lowest nonzero level in the current encoding. No lower steady
backlight level is established. Deck-selector rendering and saved preferences
were left unchanged. The newer Traktor captures use dim orange for C/D rather
than our yellow/purple, with the same intensity bits. A color-related perceived
brightness difference remains a hypothesis. Selector contrast is pending
investigation; linked loop/jump sizing remains deferred.

The initial software verification did not rebuild or launch an app, install a
mapping, or start a live HID/MIDI/LED/motor session. The requested test package
was subsequently built under `dist/first-gig-controls-20261101/`, preserving
earlier packages, with version 0.2.1-beta.1 (numeric 0.2.1, build 7).
Build, deep strict signature verification, ZIP integrity and DMG verification
passed. Bundled defaults and Info.plist match current source; the packaged
bridge's UUID matches the release executable. All 16 Instant FX mapping
addresses match the previous test package, so these two changes alone require
no mapping reinstall.

On November 1, 2026, the user responded to that package with "Looks good. Commit
it." This records general acceptance of the current test build, not a claim
that every individual control or fault/recovery path was exercised.
