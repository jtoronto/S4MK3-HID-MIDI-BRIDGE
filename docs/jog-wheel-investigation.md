# Jog feel and backspin investigation

October 6, 2026. Initial capture and subsequent calibration. The
[jog issue](issues.md) remains open for released backspin, final stop-response
acceptance, and mechanical tension.

## Calibrated settings and configurable defaults

The user adjusted Djay's MIDI editor and reported:

| Action | Speed | Reaction | Observed result |
| --- | ---: | ---: | --- |
| Scratch | 2.7% | 150% | Travel roughly matches Traktor; delay is much reduced and response feels sharper than at 50% |
| Pitch bend | 2.7% | 17% | Better gain and the previously noticed notchiness disappears |

At scratch Reaction 50%, releasing the platter stops the backspin immediately,
with less than about one beat of backward travel. At 150% this is less
noticeable, not established as fixed. Reaction is therefore a per-action
response choice, not a reason to force every jog action to its maximum.

The user's mapping is named **S4 MK3 Bridge 2**. Its saved deck-B scratch
`rotarySensitivity` was observed as `2.7000000476837158`, matching the
displayed 2.7% within float precision. The other selected values come from
the user's reported editor experiments, not a claim that every file entry
was already updated.

Those four values are now the generated defaults, exposed in the app's Jog
tab and saved separately as `jog-config.json`. Speed maps to
`rotarySensitivity`, Reaction to `rotaryAcceleration`. Export invokes the
bundled generator with the current settings; installing/selecting the new
mapping in Djay applies them. Runtime counts, seek, and released-touch
routing are unchanged. The initial comparison below predates this calibration.

## User-observed comparison

The comparison used a 140 BPM track. Bar-to-beat calculations below assume
4/4. Nonmotorized operation and Slip/Flux off were requested; Traktor's
version and exact settings were not recorded.

| Movement | Djay | Traktor |
| --- | --- | --- |
| One slow backward revolution | Bar 33 to 27.5: approximately 22 beats | 2.5 beats backward |
| One fast backward revolution | Bar 33 to 28: approximately 20 beats; kept reversing for nearly a second after physically stopping | Same 2.5 beats; no continued reversal after physically stopping |
| Released backspin | Bar 33 to 30: approximately 12 beats; physical revolutions not recorded | 2.5 bars, with four revolutions before the platter stopped |

The user subsequently noted that Traktor exposes configurable S4 jog tension,
the bridge feels more resistant, and the same force produces only approximately
2.5 backward revolutions under the bridge versus four in Traktor. This was
additional physical context, not a synchronized revolution measurement of the
captured Djay backspin. Equal-force flicks are therefore not equivalent inputs.

The user confirmed that Traktor continues reversing after lifting their hand,
tracks platter movement exactly, and stops reversing when the platter stops.
Slowly turning the same number of revolutions as the backspin lands at exactly
the same track position.

Thus Traktor's four revolutions at 2.5 beats/revolution account for its
10-beat backspin. The target is consistent distance per physical revolution
through touch release, not an arbitrary speed-dependent boost or an audio
tail extrapolated after the platter stops. The two applications' backspin
distances are not a same-gesture comparison: only Traktor's physical
revolutions were observed.

## Passive MIDI capture

An existing monitor listened to `S4 MK3 MIDI Full`, without opening HID,
creating another source, restarting the bridge, or changing preferences.
The first raw capture exceeded terminal retention and cannot establish
whole-trial totals. A second capture used bounded movement summaries.

These are delivered MIDI counts on channel 2, not independently captured raw
HID counters. Durations come from monitor log receipt times, not hardware,
CoreMIDI packet timestamps, or physical stop markers.

| Segment | Action | Net backward counts | Messages | Receipt span | Largest message |
| --- | --- | ---: | ---: | ---: | ---: |
| Slow one-turn movement | CC5 scratch | 2936 | 1280 | 4442 ms | 6 counts |
| Fast one-turn movement | CC5 scratch | 2704 | 466 | 1775 ms | 8 counts |
| Backspin while touched | CC5 scratch | 992 | 61 | 255 ms | 63 counts |
| Backspin after release | CC4 nudge | 4606 | 377 | 1273 ms | 52 counts |

Fast/slow net counts are about 92%; reported track travel is about 91%.
Neither controlled turn reaches the 63-count message boundary. This does not
show a large-motion MIDI encoding cap in those turns, but it does not prove
the absence of decoder, delivery, or native processing problems.

Approximately 82% of the backspin's net backward counts were delivered as
nudge after touch release. Current routing selects scratch while touched in
TT/vinyl mode, then returns to nudge when released. This is an evidenced
method difference from the desired continuous movement-based backspin;
it does not establish which native Djay settings could compensate for it.

The owned listener was stopped with SIGINT, allowing the summary process to
flush. Exit 130 reflects that intentional stop, not a bridge failure.

## Native mapping evidence

Before calibration, the bridge used `rotary-64`, with sensitivity 25 for
scratch, 7 for nudge, and 20 for seek, and omitted `rotaryAcceleration`.

Installed Djay presets provide these examples:

- NI Traktor Kontrol S3: scratch sensitivity 25, acceleration 150.
- Pioneer DDJ-SX2: scratch sensitivity 7, acceleration 150.

Those values are controller-specific. The acceleration field's units,
missing-field default, and effect on our delay have not been established.
Copying a preset value alone is not a verified fix.

## Findings and next experiment

1. **Calibration:** the original scratch response was roughly 8-9 times
   Traktor's travel. The user's 2.7% Speed now roughly matches it; Reaction
   150% improves scratch immediacy. Pitch bend is separately calibrated to
   2.7% / 17%. These values are defaults, not a bridge velocity curve.
2. **Stop response:** the user reports much less delay with the calibrated
   settings, but complete stop-response acceptance is not established.
   If delay persists, correlate a physical stop marker with MIDI delivery
   and Djay response. Receipt spans alone do not locate the original delay.
3. **Released movement:** verify whether native settings can maintain the
   reference's movement-based backspin. If they cannot, design a scratch
   continuation policy for a touched-and-released spinning platter while
   preserving ordinary rim nudge, original deck ownership, and immediate
   termination at rest/Stop. Do not invent extra distance or a timed audio tail.
4. **Mechanical tension:** shorter equal-force spins also reflect the reported
   resistance difference. Separate that from musical distance per revolution
   and post-stop latency; do not compensate for resistance with excessive
   software gain. Tension configuration is a separate hardware-control
   investigation. No motor/tension commands were authorized or sent.

The user explicitly wants to address haptic motors eventually and suspects
they establish wheel tension. This is recorded as deferred hardware work in
the issues checklist. Active resistance via Haptic Drive is a plausible
mechanism, not a verified command protocol; the Traktor tension value used in
the comparison was not recorded.

Do not mark the issue complete until slow/fast calibration, stop response,
released backspin, and normal nudge/seek behavior pass live checks. Preserve
the working mapping under its existing name when creating test variants.
