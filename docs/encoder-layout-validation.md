# Encoder layout validation

October 6, 2026. This records the working-tree encoder update, not a physical
controller acceptance test.

## Implemented behavior

The full profile corrects all four captured rotation names and only the left
press names. The minimal profile is unchanged.

- LOOP retains native independent loop-duration selection and activation.
- Shift+LOOP press now uses native `reloop` (note 6, channels 9-12), replacing
  manual `loopInOut`. Manual loop in/out is unmapped. The user observed
  Shift+LOOP turn moving inactive loop regions too, apparently by loop length;
  the bridge neither supplies nor restores that length.
- MOVE rotates through native beat jumps, whether or not a loop is active.
- MOVE press toggles size selection for that logical deck. Rotation then changes
  Djay's jump length immediately; the next press exits. There is no timeout.
- Shift+MOVE emits a forward/backward one-beat action for each detent, even in
  selection mode, without setting or resetting either native length.
- The visible selected deck button pulses over 1.2 seconds while selecting.
  Hidden deck modes remain cached; inactive deck buttons do not pulse.
  Bridge restart clears the local selection modes, not Djay's length values.

## Software and package evidence

### Subsequent REV, Browse, and crossfader startup increment

REV now uses native `reverseHold`, preserving held ownership through deck,
Shift, and Stop. Browse has one native `flipped=true` mapping correction on
both layers; encoder decoding and wire polarity are unchanged.

Physical crossfader state is cached separately from emitted state. The latest
snapshot is sent once raw playback feedback arrives, including zero/paused
feedback, regardless of whether the snapshot or feedback arrived first.
Subsequent physical changes emit normally, without periodic resynchronization.
This feedback signal is not an acknowledged MIDI input connection; live
startup acceptance remains necessary. Same-process reconnect has no reliable
epoch signal and requires a bridge restart, not a guessed silence timeout.

Formatter, strict Clippy, all-target Rust build, 74 Rust tests, 7 Swift tests,
shell/plist checks, and whitespace checks passed for this increment.
New regressions use the real decoder, feedback cache, translator, and mapping.
They cover both launch orders, all assignment/curve positions, the latest
pre-feedback snapshot, zero playback followed by an unloaded track, no later
replay, native Browse flip flags, and REV release/shutdown ownership.
Follow the focused live check in [full-midi-test.md](full-midi-test.md).

The user's partly checked palette corrections in saved app preferences are
not promoted to bundled defaults and are not overwritten by packaging.

### Original encoder increment

Formatter, strict all-target/all-feature Clippy, 70 Rust tests, all-target Rust
build, 7 Swift tests, shell syntax, plist validation, and whitespace checks
passed. The whitespace check disables Git's filesystem monitor; the earlier
combined check had timed out without confirming that step.

Regressions use the corrected real descriptor and Encdr's wrap16 decoder,
real translator messages, serialized generated mapping addresses, and explicit
clock values for the LED pulse. They cover per-deck selection, duplicate
presses, held presses across deck switches, simultaneous selector/rotation
ordering, Shift overrides, both directions, and pulse restoration.

The portable app and arm64 ZIP were rebuilt. Deep strict code-signature
verification, archive integrity, and bundled mapping plist validation passed.
Signing remains ad-hoc. The previous app and ZIP were preserved under
`/var/folders/vr/c8tb_f111857tlwv0vr2z0rr0000gn/T/s4mk3-pre-move-mode.41VMq7/`.

## Native and physical limits

The user's subsequent test found that tightening an active 8-beat loop to
4 beats leaves the original 8-beat auto-loop selection visible after exit.
The bridge stores neither size; this is observed native Djay behavior.
The new Shift+LOOP Reloop assignment still requires a live controller check:
create and exit a loop, move its inactive region, and reloop on each deck.
Native metadata enables `reloop` only when `song.masterLoopRegion.hasRange`.

The actions are present in installed Djay metadata and shipped mappings.
The user verified with the mouse that beat jump moves an active loop. Earlier
MIDI probes observed native Skip retaining an active loop, but did not establish
that both loop boundaries moved. Djay's main window was unavailable during
this implementation's final checks, so the new MIDI routes have not received
a fresh native acceptance test.

The controller's pulse brightness, physical direction/press pairing in the
rebuilt package, and active-loop one-beat adjustment remain to be tested with
the updated mapping. Unit tests and successful builds do not establish those
physical results. No screen or motor reports were added.

Use the focused checks in [full-midi-test.md](full-midi-test.md), installing the
new mapping under a new name and preserving existing customized mappings.
