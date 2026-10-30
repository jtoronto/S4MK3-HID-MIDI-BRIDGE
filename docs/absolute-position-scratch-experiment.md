# Absolute-position Scratch experiment

This document preserves the abandoned Djay absolute Scratch CC 8 experiment
and describes the source inspected on October 30, 2026. It is a reconstruction,
not a setup guide for an active feature. The current runtime computes angle
telemetry but doesn't emit this absolute-position CC 8 stream. Generated
mapping entries, configuration, converter code and tests remain.

## Counter-to-position algorithm

The current implementation is in
[`platter_position.rs`](../src/full/platter_position.rs), called by
`Translator::log_platter_position` in [`mod.rs`](../src/full/mod.rs).
The scale is defined in [`tension/playback.rs`](../src/tension/playback.rs):
2,880 encoder counts per physical revolution.

To reproduce the conversion independently:

1. Accept Report 3 only. The runtime requires exactly 60 bytes, including
   report ID `3` at index 0.
2. Read an unsigned little-endian 32-bit counter for each physical side:
   bytes `12..16` for left and `40..44` for right, using zero-based,
   end-exclusive ranges that include the report ID in their indexing.
   These are the `offset + 4..offset + 8` fields for offsets 8 and 36.
3. Keep separate state per side: active deck, previous counter, phase in
   counts, and last published bin. On activation, set the previous counter
   to the sample, set `phase = counter % 2880`, clear the last bin, and
   produce the initial bin.
4. For each subsequent sample, subtract the previous counter with unsigned
   32-bit wrapping arithmetic. Reinterpret those 32 bits as a signed `i32`;
   don't clamp the result or subtract after converting both counters to
   signed integers. Advance phase using
   `phase = (phase + signed_delta).rem_euclid(2880)`, with a wide signed
   intermediate as the source does.
5. Quantize with integer arithmetic:
   `value = floor(phase * 128 / 2880)`. The result is `0..127`.
   Return a value only when it differs from the last returned bin.

The signed delta preserves forward and backward motion across `u32`
rollover. For example, `u32::MAX - 10` followed by `10` advances 21 counts.
As with any signed modular difference, interpreting direction assumes less
than half the counter range of travel between observations. Recomputing
`counter % 2880` for every sample would not preserve accumulated phase at
the sensor counter's rollover.

One MIDI bin represents 22.5 counts on average, or 2.8125 degrees. Starting
at counter 0 returns bin 0; counter 23 returns 1; counter 24 is suppressed;
counter 46 returns 2. Crossing phase 2879 to 0 returns `127 -> 0`, with the
reverse crossing returning `0 -> 127`. Large sample jumps produce the
destination bin only, not interpolated intermediate messages.

Here, "absolute" means a cyclic encoder-derived angle value, not an absolute
track position or a calibrated physical index mark. Activation uses the
counter's modulo phase. Ownership rebasing can deliberately separate that
phase from subsequent raw-counter modulo values.

## MIDI and Djay mapping contract

The endpoint is **`S4 MK3 MIDI Full`**. The following messages travel from
the bridge to Djay, as controller input, not from Djay as feedback. Decks
A, B, C and D use user-facing MIDI channels 1, 2, 3 and 4 respectively.
Their plist `midiChannel` values are 0, 1, 2 and 3.

| Numeric address | MIDI type | Djay target | Mapping interpretation |
| --- | --- | --- | --- |
| CC 8 | Control Change, plist type 3 | `turntableN.scratchingMove` | `rotary-absolute`, values 0..127 |
| Note 7 | Note, plist type 1 | `turntableN.scratchingMode` | Scratch gate, held during the corrected historical experiment |
| CC 5 | Control Change, plist type 3 | `turntableN.scratchingMove` | Separate manual `rotary-64` Scratch input |
| CC 4 | Control Change, plist type 3 | `turntableN.pitchBendMove` | Separate `rotary-64` Pitch Bend input |

`N` is the deck number 1 through 4. For CC 8, the wire message is
`[0xB0 | (N - 1), 8, value]`. Message type is part of the address: CC 8
isn't Note 8, and Note 7 isn't CC 7. CC 8 has no shifted duplicate.
The existing shifted Note 7 on user channels 9 through 12 instead maps to
`jogSeekMode`; it isn't the motor experiment's Scratch gate.

[`catalog.rs`](../src/full/catalog.rs) retains
`PLATTER_POSITION_BINDING` and `SCRATCHING_MODE_NOTE`.
[`mapping.rs`](../src/full/mapping.rs) emits four CC 8 entries with
`controlType=rotary-absolute`, `rotarySensitivity` from
`platter_position_sensitivity`, and `rotaryAcceleration` from
`scratch_reaction`. It doesn't flip CC 8. The configurable manual Scratch
direction flips CC 5 only.

The initial absolute package sent CC 8 without holding Note 7 during
untouched powered playback. Historical replay showed no Scratch-mode note
until physical touch. The follow-up held the existing Scratch gate before
the position stream, shared ownership with touch, followed deck changes,
and released on pause, gate closure and Stop. This was a required correction
to that experiment, not proof that Djay interpreted the resulting angle
stream correctly.

## Ownership, rebasing and gates

The retained telemetry path provides a precise model for avoiding duplicate
motion and cross-deck jumps:

- If playback isn't enabled for a side, `suspend(counter)` clears its active
  deck and last bin and records the counter. It doesn't emit a position.
- If another jog owner holds that side and the active deck still matches,
  `rebase(counter)` updates only the previous counter. Phase and last bin
  remain unchanged, so touch-owned movement isn't counted again on release.
  If the deck doesn't match, the state is suspended instead.
- An enabled, unowned side observes deltas only while its active deck matches.
  A newly selected deck or resumed suspended stream activates from a fresh
  counter baseline; it doesn't inherit a delta from the old deck.

In current `playback_state`, enablement requires TT motor mode, authorization
for the selected deck, neither Grid nor Shift, and no touch, wheel or
platter-Scratch ownership belonging to another deck. `owned` includes a
wheel owner or platter-Scratch owner, not just a capacitive touch bit.
Authorization comes from cached Djay feedback: loaded, playing and Internal
must each be `Some(true)`, and Thru must not be `Some(true)`. Feedback silence
alone doesn't revoke this cached authorization. These are the current
telemetry gates, not a claim that every later ownership condition existed
in the original experiment.

Historical Note 7 release on pause, gate closure and Stop was checked by
the recorded replay. Current bridge teardown calls `release_all`, which
clears jog ownership and releases held MIDI addresses. The angle converter
itself neither owns nor releases MIDI notes. Any restored experiment must
explicitly coordinate the gate and position stream across stop, restart,
deck changes and manual takeover.

## Sensitivity 14.0 and its limits

The historical record reports about 4 seconds of track advancement during
20.74 seconds of platter rotation at 145 BPM on October 29, 2026:

```text
observed average playback ratio = 4 / 20.74 = approximately 0.193
first-pass sensitivity estimate = 2.7 * 20.74 / 4 = approximately 14.0
```

This assumes sensitivity scales playback rate linearly. It isn't a measured
Djay sensitivity law, a verified one-to-one calibration, or a mapping from
BPM to angle. The speed-tuning build record explicitly says no physical
Djay/audio retest established 14.0 as a final setting. The earlier package's
missing Scratch gate also limits conclusions drawn from early playback
timings.

[`jog.rs`](../src/full/jog.rs) still defaults
`platter_position_sensitivity` to 14.0 and validates a finite value in
`0.1..=100.0`. It is independent of manual `scratch_speed`, whose default is
2.7. These sensitivities are mapping-time settings: changing them doesn't
rewrite an already installed mapping. The current generated CC 8 mapping
also uses the configured Scratch reaction, currently defaulting to 150.

Audio-derived frequency or elapsed-track ratios measure the combined host
response, mapping, gating and physical motion. An average ratio can conceal
a cyclic rate dip. It can't establish phase alignment or show that each
angle bin advances audio equally. The experiment read no current-BPM output
and didn't implement Djay's private T7/RANE integration.

## Why the Djay trial was abandoned

The supported observations are recorded in
[`motorized-playback-integration-validation.md`](motorized-playback-integration-validation.md),
especially "Absolute Scratch CC experiment", "Adjustable Scratch tuning",
"Scratch-mode gate follow-up", "Adjustable sensitivity and touch-direction
build", and "CC 8 wrap-seam phase-offset experiment (completed)".

The phase-offset trial added 32 modulo 128 to emitted bins, moving the
numeric wrap to a different physical sector without changing target RPM or
sensitivity. The paired `-3.wav` analysis and filtered log placed the dip
near emitted values 0 through 15, at about 940 Hz, rather than values 32
through 47, at about 1002 Hz. Sector step timings were nearly equal,
about 14.35 versus 14.39 ms. The tone trace also had a strong
once-per-revolution component near 0.548 Hz.

That supports a response associated with the low numeric CC region or wrap,
rather than a comparable physical speed dip at a fixed platter sector.
It remains an inference: the log doesn't identify the app binary, so the
interpretation assumes the `wrap-offset-32` build was used. The record
doesn't prove Djay's internal cause or that absolute jog input fails in
every configuration.

The experiment therefore hadn't established stable, calibrated playback
from absolute angle. Subsequent work replaced its output with a relative
CC 4 path and removed the experiment's continuous motor-owned Note 7 hold.
The historical "Deviation-only correction follow-up" and the later
raw-relative trial summary explicitly record no absolute CC 8 output.
The raw-relative trial's separate report of continuous CC 4 with no track
advancement mustn't be attributed to the absolute CC 8 experiment.

## What remains in source

The converter and four unit tests remain. They cover quantization and
duplicate suppression, bidirectional revolution wrap, sensor counter
rollover, and touch rebasing. Generated-mapping tests retain the four
absolute Scratch entries, Note 7 mapping, independent CC 5/CC 8 sensitivity,
and manual-only polarity reversal.

`log_platter_position` still computes changed bins and logs
`MOTOR_JOG_POSITION` with side, deck and position. It has no MIDI output
argument and doesn't append CC 8 messages. A telemetry record isn't
evidence that a MIDI message was sent. The fixed `+32` diagnostic offset
is absent from the current converter.

The same `Position` type is also retained for separate ring-position state;
removing it as supposedly dead experiment code would require reviewing that
use. Historical test counts and package replay results describe their
recorded revisions, not a fresh test run of today's tree.

## Reusing the idea with another host

No other DJ software is claimed tested here. Before trying another host,
establish its actual input contract: cyclic absolute angle versus bounded
position or relative velocity, supported resolution, direction, wrap
interpretation, Scratch/touch gate requirements, and sensitivity or
acceleration behavior. Djay's `rotary-absolute` plist string is a Djay
mapping instruction, not a portable host protocol.

A deliberate experiment should distinguish encoder phase from audio
playhead position, test both wrap directions and counter rollover, measure
response across all bins rather than only average speed, and verify manual
takeover and release without double-counting motion. Higher-resolution
input might be worth investigating if a host supports it, but that would
be a different mapping contract, not a proven fix for the observed dip.

Resurrection needs a deliberately separate experimental path with its own
explicit enablement, mapping and evidence. It must coordinate Scratch-gate
ownership before sending angle values, rebase at ownership transitions,
release on stop and gate closure, and prevent competing production jog
routes from driving the same gesture. Merely reconnecting telemetry to CC 8
wouldn't recreate the corrected historical experiment.

This documentation change doesn't reactivate code, run hardware, or validate
the UI. Release option pruning is deferred.
