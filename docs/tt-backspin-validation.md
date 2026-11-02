# TT backspin detection and MIDI speed increase

November 2, 2026. The user requested backspin detection in TT after reporting
improvement with the first-load Scratch/native CPU comparison build.

## Change

The detector now accepts the existing TT speed-floor Scratch owner as well as
physical Scratch owners. TT no longer requires Vinyl on for the event.
The existing two-window acceleration threshold and saved additive speed
increase are unchanged. Bend, Shift seek and GRID do not start events.

Physical jog movement and TT speed-floor movement share the same `jog_steps`
quantization and event scale:
`1 + backspin_speed_increase / scratch_speed`.
Zero increase preserves raw travel; an increase equal to Scratch speed doubles
event travel. Only MIDI CC5 travel is scaled, not the physical RPM estimate,
handoff floor, bend response or motor force.

Top release preserves an active TT floor backspin's measured motion window.
Retouch clears detection and extra speed without releasing/repressing its
Scratch note. Stop or reversal clears the event; TT floor Scratch itself still
uses its existing speed-based exit. Deck changes and stale input release that
original TT speed owner rather than retargeting it. Explicit touch-owned TT
backspins retain their released owner until measured settling, as in Normal mode.

The BACKSPIN fields on existing screen pages use the same event/owner source.
No screen layout, mapping, MIDI address or preference field is added.
`backspin_assist` remains restricted to Normal mode. TT's playback-driven motor
policy is unchanged; the Normal one-second backspin assist pulse is not enabled
by this feature.

## Regression evidence

The new TT speed-floor test first failed with `None` instead of `Some(0)` for a
qualified left backspin. It now covers both physical sides, disabled detection,
an excluding threshold, zero increase and doubled travel, before and after top
release. It checks exact signed CC5 count sums, the actual screen-page owner,
retouch, rest and forward recovery. The Normal assist flags remain false in TT.

The existing released-coast regression also runs in TT with Vinyl on and off,
retaining its exact MIDI expectations and original Normal-mode cases.
The existing TT deck-switch/stale-owner table now includes active backspins
with and without top contact, and verifies no fractional boost carries into a
later Normal-mode packet.
A separate same-frame TT-off/movement regression failed when a fractional
backspin remainder suppressed the first one-count Normal bend packet. Clearing
the routing remainder at the mode transition fixes that case.

## Source and packaged verification

All 512 Rust tests and 72 Swift tests passed. Formatting, strict
all-target/all-feature Clippy, all-target build, shell syntax, plist lint and
diff whitespace checks passed. Production Rust diagnostics were clean.
The first strict-Clippy run rejected duplicate return arms; those were merged
without suppression before the successful complete run.

The final app, ZIP and DMG are in `dist/tt-backspin-20261102/`.
The initial owned package was preserved in
`dist/tt-backspin-20261102-pre-clippy/` before rebuilding the exact final source.
Version remains `0.2.1-beta.1`, build 7, with no preference or mapping migration.
Deep/strict signature verification, ZIP integrity and DMG verification passed.
Bundled mapping and LED/Jog/tension defaults match the preceding build
byte-for-byte.

Both actual packaged Rust executables were run through the same playback
fixture, using an isolated `backspin_acceleration=5000` configuration so the
fixture's slow 6 RPM reverse segment qualifies. This does not change saved
preferences or the default 150000 threshold, which the translator test exercises
with a faster flick.

All 13,060 timestamped motor reports match the preceding first-load/CPU build.
For deck C over 4.1-4.3 seconds of the TT reverse segment, raw Scratch travel
changes from -58 to -179 counts. The configured gain is
`1 + 5.6 / 2.7 = 3.074074...`, within one count of the expected scaled travel.
This demonstrates the new MIDI response without a different motor command.

## Physical limits

No hardware session, motor command, screen write or saved preference change was
started for this work. Software detection, MIDI gain and owner assertions do
not establish physical backspin feel, screen visibility during a fast gesture,
or audible Djay travel. TT retains its existing playback motor behavior, not
Normal-mode free coasting or a new assist pulse.
