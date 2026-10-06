# S4 MK3 bridge issues

Current checklist as of October 6, 2026. This is the source of truth for issue
status; [the mapping audit](mapping-audit.md) retains detailed findings and
historical snapshots.

Update this document as work completes. Check an item only when the agreed
behavior is implemented and verified. For physical controller behavior,
software checks alone are insufficient: record live confirmation as well.
Keep completed items and add their completion date and a short evidence note.

## Completed

- [x] **Default Djay color-token correspondence** - All eight selectable
  hotcue colors physically matched the user's saved correspondence,
  October 6, 2026. Rust and bundled JSON defaults now match it; saved
  preferences are preserved. White is not an option in Djay's hotcue picker:
  token 9 retains its existing unverified white fallback.
  See [palette evidence](led-feedback.md#verified-hotcue-palette).

- [x] **LOOP/MOVE layout and beat jump** - Correct physical encoder pairing;
  independent per-deck loop/jump sizes; MOVE press toggles jump-size selection
  with a visible selector pulse; Shift+MOVE jumps one beat. Implemented and
  user-tested, October 6, 2026.
- [x] **Reloop** - Shift+LOOP press restores the stored loop range; manual
  loop in/out remains unmapped. User confirmed "It works," October 6, 2026.
- [x] **Momentary REV** - Reverse ends on release, retaining original deck
  ownership across deck/Shift changes and bridge Stop. Slip is independent.
  User confirmed the three-fix build tests good, October 6, 2026.
- [x] **Browse direction** - Correct direction on both sides and both
  modifier layers, with one mapping inversion. User confirmed the three-fix
  build tests good, October 6, 2026.
- [x] **Crossfader startup state** - Apply the latest physical assignments
  and curve after Djay playback feedback, then emit physical changes only.
  User confirmed the three-fix build tests good, October 6, 2026.
  Mapping reconnect/reselection still requires bridge Stop/Start.
- [x] **Shift+Sync tempo-range cycling** - Either side cycles Djay's global
  tempo-fader range for all decks via `application.tempoSliderRangeNext`;
  normal Sync remains deck-specific. Djay owns the choices and cycling order.
  Software routing/export checks passed and the user confirmed "Works,"
  October 6, 2026. See [validation](shift-sync-validation.md).

## Open

- [ ] **Jog feel and backspin** - Reduce excessive slow-movement sensitivity
  while improving fast-backspin response. Measure raw counts,
  emitted MIDI, and Djay response before choosing tuning behavior.
  [October 6 comparison](jog-wheel-investigation.md): Traktor's target is
  2.5 beats per revolution at either speed, including released backspin,
  with no reversal after the platter stops. The user calibrated scratch
  Speed/Reaction to 2.7% / 150% and pitch bend to 2.7% / 17%; these are now
  configurable exported defaults in the app's Jog tab.
  Backspin still ends on touch release, especially at Reaction 50%.
  Released-spin routing, final stop-response acceptance, and the reported
  higher mechanical tension remain unresolved; equal-force flicks are not
  equivalent across the two setups.
- [ ] **Leftmost FX button duplicates slot 1** - Verify native bank-enable
  behavior and investigate a whole-bank bypass that preserves individual
  slot states.
- [ ] **REC / Record Sample** - Establish the native action's recording
  source, destination/slot, and press/hold requirements.
- [ ] **Preview encoder scrubbing** - Find a genuine preview-seek action and
  establish the preview-state or held-modifier interaction before routing
  the requested encoder to it.
- [ ] **MUTE/STEMS behavior** - Decide whether MUTE should become a held
  modifier instead of duplicating the STEMS layer, and settle pad actions.
- [ ] **FX SELECT / Filter Reset** - Settle the intended behavior and verify
  reset-to-neutral and knob pickup. Currently resets only the selected deck.
- [ ] **EXT / microphone / line inputs** - Establish S4 input channels and
  hardware source switching, then determine which routing Djay can support.

## Deferred ideas

- [ ] **Haptic motor and wheel-tension control** - The user wants to address
  this eventually. Investigate the S4's tension/motor protocol and match
  configurable resistance independently of software jog sensitivity.
  Motor-driven rotation and other haptic behavior need their own agreed
  scope; the current jog investigation sends no motor commands.

On-device screens/pad legends and a controller settings-menu gesture remain
deferred. Automatic Slip during REV is a separate design choice, not part of
the completed momentary-reverse fix. See the mapping audit for these ideas
and the Encdr upgrade candidate.
