# TT braking, live diagnostics and paused scratching

November 1, 2026. Current-source follow-up to the user's motorized-platter
feedback. This is not a published-release claim.

## Reproductions and changes

The previous bend path required measured speed within 0.75 RPM of target before
any later correction could qualify. A deterministic 32 or 34 RPM baseline with
a 33 RPM target, followed by a 27 RPM brake, produced no negative correction.
`quiet_band_startup_qualifies_following_braking` failed on that behavior.
Qualification now accepts the existing 2 RPM quiet band. The 100 ms measurement
window, 2 RPM gesture threshold, 0.75 RPM release threshold and 2x negative gain
are unchanged. This fixes a reproducible missing-output case; it does not
establish that this was the cause of the user's audible braking report.

With Vinyl off, the previous TT path suppressed ordinary jog movement while
its speed handoff required playing feedback. A paused deck therefore could not
move through the platter. The two-sided
`paused_tt_moves_the_track_without_vinyl_or_motor_authorization` regression
failed before the fix. Confirmed paused, loaded, Internal decks now receive
held Scratch and signed relative movement without requiring top-touch or
changing Vinyl. Pause does not authorize motor drive. Physical touch retains
its original deck owner until release; the synthetic paused owner ends on
resume, TT-off, Shift/GRID, loss of authorization or stale input. Invalid
counter jumps do not become Scratch bursts.

## Live indicators

The user chose the existing app Diagnostics tab, not controller screens.
Full mode emits `S4_PLATTER_STATUS` JSON at most 10 times per second independently
of log level. Each physical side carries its actual measured RPM, logical deck,
routing mode and recent successfully sent CC4 count rate. The RPM measurement
uses the same sensor-clock window as bend decoding, independent of whether
bending or motor drive is authorized.

CC4 counts are taken after successful MIDI sends; Scratch CC5, other channels
and other messages cannot contribute. This is outgoing MIDI evidence, not
feedback about Djay's audible speed. The app consumes machine lines before its
bounded display log, includes valid lines in an active motor capture, and
clears numeric presentation on stale sensor state, telemetry silence or process
stop. Zero and unknown are distinct.

## Software and package evidence

- All 484 Rust tests and 66 Swift tests passed. Formatting, strict locked
  all-target/all-feature Clippy, locked all-target build, shell syntax, plist
  lint and diff checks passed. The two existing SwiftPM unhandled-file warnings
  remain; no checks were suppressed.
- The real runtime replay now explicitly checks the paused Scratch interval
  as well as touch/speed handoff, without relaxing its nominal-rotation checks.
  It verifies that motor output stays zero while paused. Tests also cover
  physical-touch deck ownership, resume/mode/stale release and invalid counters.
- Native indicator cards were inspected with positive/negative output, reverse
  and zero RPM, missing data and all routing labels in light/dark appearances.
  The full tab was inspected at 760/1024 widths before compacting the new group.
  The final full-window compositor capture failed; bitmap-cache attempts omit
  child layers and are not accepted as whole-window evidence. Final native card
  renders remain valid. Artifacts and this limit are recorded under
  `.omo/evidence/platter-diagnostics-20261101/`.
- Built `dist/tt-braking-diagnostics-20261101/S4 MK3 Bridge.app` and companion
  ZIP/DMG, preserving older builds. Version remains 0.2.1-beta.1, numeric
  0.2.1/build 7. Deep strict signing, ZIP integrity, DMG verification, bundled
  defaults and executable UUID checks passed. The complete bundled mapping
  is byte-identical to the first-gig-controls test package.
- The exact packaged executable's hardware-free replay produced 9,850 records
  and 165 diagnostic frames. Every reported CC4 rate matched the corresponding
  outgoing MIDI counts; the 10 Hz bound held. Two update intervals contained
  negative output, and Bend, Scratch and Paused scratch were observed.
- ZIP SHA256: `c476641ea9c8a4991a86f0e07fd046143dfb2512468777a24b6ba8c8465c2534`.
  DMG SHA256: `cdf81fb1caf81f6354c616602856c1f485185ae97a698d23b81a4b2ee1040828`.

No live HID, MIDI, screen or motor test was started, no saved user preferences
were changed, and nothing was committed or published. Audible braking strength
and physical paused-scratch acceptance remain unverified.

## Ordered physical comparison

No agent-triggered hardware run is part of these source checks.

1. Stop the old bridge, quit its app, and open the separately built test app.
2. Select the Full profile and keep the existing matching Djay mapping selected.
   No mapping reinstall is needed for the routing and indicators alone.
3. Open Diagnostics. Before Start, both live indicator panels should be stopped
   with no current numbers. Normal log detail is enough for the live indicators.
4. When ready to watch the controller, start the bridge. Load an Internal deck,
   select TT, and start playback. The platter will rotate with the existing
   motor settings. Let it settle and confirm measured RPM and Bend mode.
5. Brake gradually. Beyond the existing quiet band, slowing should produce a
   negative Bend output. If RPM falls but the indicator stays zero, note the
   displayed mode and RPM. Scratch mode uses a separate route and normally has
   no CC4 output.
6. If negative MIDI output is present but audible slowdown is weak, compare
   Pitch Bend Speed/Reaction in Jog settings with the mapping selected in Djay.
   Changing those mapping settings requires exporting/selecting the new mapping,
   not merely Save and restart. Absolute platter sensitivity does not tune CC4.
7. Pause playback with TT still selected and Vinyl off. The motor should be
   neutral; moving the platter forward or backward should move the track.
   Diagnostics should report Paused scratch and no bend output.
8. Repeat on the other physical side. Resume playback and confirm that ordinary
   TT behavior returns without changing the saved Vinyl choice.
9. Stop the bridge and confirm the live numeric values clear. Keep any failed
   observation with its displayed mode, RPM and Bend output; do not infer audible
   behavior from a passing write or indicator alone.

## Adjustable TT gain follow-up

The user reported negative bend working, then requested a gain control to
investigate Djay's actual response floor. `tt_pitch_bend_gain` now accepts
0.1-10, default 1.0, in Rust and app preferences. It multiplies TT correction
before residual accumulation/quantization, retaining the existing 2x braking
bias. Existing JSON without the field loads as 1x and preserves its saved floor.
Gain does not require a mapping change; Save/restart loads it.

At a 33 RPM target, 1% handoff floor and near-zero forward movement, 1x requests
roughly -3136 counts/s; 2x can request roughly -6273 counts/s before handoff.
These are bridge MIDI rates, not audible speed or proof of a Djay ceiling.
The gain-scaling runtime regression failed while the field was ignored, then
passed once wired. Regressions compare braking and acceleration at multiple
gains, unchanged measured RPM, unchanged normal jog/paused Scratch, legacy
defaults, round-trip persistence, bounds and invalid numeric values.

For an attended comparison, keep target RPM, floor and Djay Speed/Reaction
fixed. Start at 1x, then try 2x and 4x, saving and restarting between trials.
Re-engage TT after each restart, let it settle, then brake to the same measured
RPM. Compare signed MIDI rate with measured audible speed. If MIDI requests
increase while audio no longer slows, retain the audio evidence and exact
settings before calling that Djay's response floor. No audio-limit measurement
has been performed by the agent.

Gain follow-up source checks passed: 487 Rust tests, 67 Swift tests, formatting,
strict Clippy, locked all-target build, shell/plist and diff checks. The existing
SwiftPM unhandled-file warnings remain. Native gain controls were rendered in
light/dark at the unchanged default and higher values using the actual
`DecimalRow`/`NumberRow` components; no user settings or hardware were touched.

The gain test app is under `dist/tt-bend-gain-20261101/`, alongside its ZIP/DMG,
still version 0.2.1-beta.1 (numeric 0.2.1/build 7). Signing, archive integrity,
bundled defaults and current executable checks passed. The generated mapping
is byte-identical to the preceding diagnostics package.

The exact packaged default replay matches the preceding package's complete
replay output. At 4x, TT negative counts changed from 83 to 335 and positive
counts from 396 to 1590, within accumulated per-gesture rounding. The 31 normal
jog counts, motor reports, Scratch messages and RPM/routing snapshots remained
unchanged. The oracle partitions TT and normal-jog traffic using fixture state;
applying the TT multiplier to their combined CC4 total would be incorrect.

Gain package ZIP SHA256:
`0d8db656a863f0482b205da3c1c674d23cdd76840aff8b41ebfc8ec27e302124`.
DMG SHA256:
`2802934935e1f9da7f31eac0ff58ee2529133833b4b30e037dcf65bcfb31dd00`.
This establishes software gain behavior only, not Djay's minimum audio speed.
