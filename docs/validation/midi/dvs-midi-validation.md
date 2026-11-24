# DVS MIDI auto-enable and mode feedback validation

Attended test on October 27, 2026, using installed Djay Pro 5.6.10,
build 67091. The user operated Djay's UI; the agent sent only the two
requested MIDI actions through an isolated virtual endpoint.

## Result

**Sending `turntable4.controlModeThru` did not automatically enable global
DVS in this configuration.** With the global checkbox disabled, Deck 4
continued to display INT. The user reopened Settings and confirmed the
checkbox was still off.

After manually enabling DVS, the user reported Deck 4 was THRU, and the
captured mode feedback changed to THRU-active/Internal-inactive.

That sequence is observed, but its explanation is not isolated. There was
no disable/re-enable control without an intervening THRU command, so this
run does not distinguish a queued mode selection from restoration of a
remembered DVS mode. Do not describe the THRU command as simply ignored.

## Isolated setup

- Endpoint: `S4 DVS Probe 20261027-4d73`, paired virtual source/destination.
- No startup MIDI; commands were accepted only through the probe's stdin.
- Input actions on human MIDI channel 1: Note 100 selected
  `turntable4.controlModeThru`; Note 101 selected
  `turntable4.controlModeInternal`.
- Each trigger sent Note On velocity 127, followed by Note Off velocity 0.
- The imported mapping belonged only to this endpoint. Production S4
  mappings were not replaced or edited.
- This helper did not open HID or start the bridge, motors or screens.

The note/CC numbers below were chosen for this diagnostic. They are not
fixed Djay addresses or a proposed production EXT mapping.

## Starting state and sequence

Initial read-only UI inspection established:

- Global Enable Digital Vinyl Control: checked.
- Decks 1, 2 and 3 input sources: None.
- Deck 4: Traktor Kontrol S4 MK3, channels 7-8; mode THRU.
- DVS buffer: 256 frames; other DVS options were left unchanged.

After importing the isolated mapping and obtaining fresh confirmation
that the user was watching:

1. Sent the Internal action. Deck 4's accessibility value changed to INT,
   and the expected mode feedback changed. This confirmed a working MIDI
   connection and the Internal binding.
2. The user unchecked global DVS, applied if necessary, closed Settings,
   and confirmed readiness for the next trigger.
3. Sent the THRU action. Deck 4 still showed INT; no mode-feedback change
   was observed before the subsequent manual re-enable.
4. The user reopened Settings and reported that the global checkbox
   remained off. They manually re-enabled it and reported Deck 4 was THRU.
   The capture then emitted THRU-active/Internal-inactive feedback.

The THRU action was not separately triggered with global DVS already
enabled during this run. Audio quality, actual input monitoring, other
decks and other DVS modes were not retested.

## Captured feedback

All messages used human MIDI channel 1. Top-level outputs explicitly bound
the state paths below, with minimum 0 and maximum 127. Embedded outputs on
the two action bindings also emitted their corresponding notes.

| Output | Initial THRU | After MIDI Internal | After manual DVS re-enable |
| --- | --- | --- | --- |
| CC 100: `turntable4.controlModeIsThru` | 127 | 0 | 127 |
| CC 101: `turntable4.controlModeIsInternal` | 0 | 127 | 0 |
| Note 100: THRU action's embedded output | 127 | 0 | 127 |
| Note 101: Internal action's embedded output | 0 | 127 | 0 |
| CC 102: `turntable4.controlMode`, `controlType=control` | 127 | 0 | 127 |

The note feedback used Note On status `90` with velocity 127 or 0;
the CC feedback used status `b0`. For example, the Internal action produced
`b0 65 7f` and `90 65 7f`, while clearing THRU with `b0 64 00` and
`90 64 00`. Manual re-enable reversed those flag values.

CC 102 is the observed result of the configured MIDI conversion, not a
decoded raw mode enum. Absolute, Relative and Wireless values remain
unmeasured. No global-DVS-enable feedback key was identified by this test.

## Restoration and cleanup

Final read-only inspection confirmed global DVS checked, Decks 1-3 set to
None, Deck 4 still assigned to S4 channels 7-8 and showing THRU, and the
256-frame buffer and other inspected DVS options unchanged. Apply and
Revert were disabled, with no pending input-assignment changes.

The probe received `quit` and exited with code 0. Its temporary example
source and uniquely named imported mapping were removed. The inactive
diagnostic mapping remains under ignored `.omo/evidence/` as evidence.
No production bridge behavior, bundle or user mapping was changed.

The temporary helper passed language-server diagnostics, formatting,
focused Clippy with warnings denied, and compilation; its plist passed
`plutil -lint`. The behavioral evidence is the attended UI/MIDI sequence,
not those static checks.
