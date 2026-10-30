# S4 MK3 bridge issues

Current checklist as of October 30, 2026. This is the source of truth for issue
status; [the mapping audit](mapping-audit.md) retains detailed findings and
historical snapshots.

Update this document as work completes. Check an item only when the agreed
behavior is implemented and verified. For physical controller behavior,
software checks alone are insufficient: record live confirmation as well.
Keep completed items and add their completion date and a short evidence note.

## Completed

- [x] **Preview encoder scrubbing** - Preview toggles playback and
  Shift+BROWSER emits directional preview seek pulses. The user confirmed
  the current scrubber works on October 21, 2026. A proposed one-handed
  replacement is tracked below.
- [x] **Saved hardware wheel tension** - Jog > Hardware exposes a shared
  Assist / Neutral / Brake setting with independent per-wheel motion/touch
  response. User-confirmed assistance/braking, Djay/MIDI/LED coexistence,
  Stop/Quit neutral, saved-setting restore while stopped, and explicit restart
  after USB reconnect, October 20, 2026. See
  [normal-app integration](jog-tension-integration.md).
- [x] **Mixer ON / Neural Mix EQ selection** - Filter is now a one-shot
  EQ deck-selection latch, not reset-to-neutral. The next ON toggles that deck's
  native EQ mode without retargeting Instant FX; normal ON still selects the
  FX target. White/cyan mode, armed brightness, active-effect color, and dim
  cyan idle Filter replace the former deck colors/reset indication.
  Live MIDI feedback was captured independently for A-D; the user accepted
  the physical behavior and idle refinement on October 20, 2026.
  The user confirmed working LED/FX behavior in their test set on October 25,
  2026, with no LED/FX fault reported. The exhaustive
  [edge-case check](neural-mix-eq-check.md) is optional regression coverage,
  not a known defect or an outstanding acceptance requirement.

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

- [x] **Experimental screen pages** - Implementation authorized October 21,
  2026. Raw dual-screen transfers succeeded after temporarily quitting the
  NIHardwareAgent process that owned interface 4; S4 audio stayed enumerated.
  An attended blank-screen report led to delaying screen startup until
  initialized input; the user then confirmed both deck pages appear from a
  cold firmware-logo baseline on October 21. Static bars/labels also passed.
  The approved legibility pass removes redundant headers, cue/sample grids,
  and control-role rows. STEMS has larger text, real mute/solo activation,
  separate held-press outlines, and unknown state. Hotcue/Samples now use the
  approved info page described below.
  On October 22, the user approved smooth anti-aliased sans-serif typography
  for both info and STEMS/MUTE; the shared renderer now embeds native-size
  coverage masks instead of enlarging an 8 x 8 font. The user then visually
  accepted the updated font ("Looks good!"). The app's full-controller launch
  now includes the accepted screens. The user tested the integrated app and
  reported "Tested and it seems good!" on October 22. On October 25, the user
  explicitly confirmed screen states across decks worked in the test set.
  This closes normal-use acceptance; it is not inferred from USB completion.
  See [screen implementation and ordered checks](screens.md).
- [x] **Numeric screen content** - Implemented October 22, 2026: non-STEMS
  modes show JUMP / LOOP / KEY / RANGE in four quadrants. Active LOOP uses
  green fill and black text; inactive says OFF. MOVE size selection has steady
  SELECT and an amber/dark background flash, never a disappearing number.
  A full paused-track calibration established all 24 numerical key labels.
  Generator, decoder, selected-deck projection, native pixels and worker
  timing have regressions. The user confirmed cross-deck screen behavior in
  the test set on October 25, 2026, completing physical acceptance.
  Inactive selected loop length remains unexposed by tested
  routes and is deliberately not shown. A fresh generated mapping is required. See
  [evidence and probe boundaries](screen-feedback-investigation.md).
- [x] **Shift+CUE: jump to beginning** - Implemented October 21, 2026:
  `turntableN.gotoZero` on Note 1, channels 9-12. Normal CUE is unchanged.
  Generated mapping and ownership regressions cover it. The user confirmed
  it worked in the test set on October 25, 2026.
- [x] **Swap A#-1 / B-1 actions** - Implemented October 21, 2026 on input
  endpoint S4 MK3 MIDI Full, Note 10/11, channels 1-4: Playlist now
  marks/unmarks selected songs. Current source supersedes Star's Queue action
  with Switch Library Table and adds Shift+BROWSER source navigation.
  Native MIDI/GUI checks passed; the user confirmed the library controls
  worked in the test set on October 25, 2026.
- [x] **One-handed held preview** - Implemented October 21, 2026: pressing
  Preview opens/starts the preview player, releasing it closes/stops preview,
  and BROWSER scrubs while Preview is held, without Shift. Normal BROWSER
  browsing resumes on release. Use paired preview toggles on press/release;
  the user accepts occasional state mismatch if preview was already active
  or changes externally, with closing preview using the mouse as sufficient
  recovery. Preview-state feedback is not required for this design.
  Duplicate snapshots, simultaneous press/turn, deck/Shift changes,
  overlapping holds, release, and shutdown are regression-covered.
  The user confirmed held Preview scrubbing worked in the test set on
  October 25, 2026, completing physical acceptance.
- [x] **Intermittent unknown key** - Fixed October 24, 2026 after an attended
  capture showed new key feedback arriving before a loading-false
  notification. The deck-owned key now follows its own feedback,
  including the explicit no-key value on eject. Replacement and eject
  regressions pass; the user verified `11A -> 2A -> --` on the physical S4.
  No mapping change is needed. See [diagnosis and verification](key-feedback-order-validation.md).

## Current-source EXT support

The full profile now provides fixed EXT1-4 switching for Djay decks 1-4,
independent of deck/FX selection, with a newly generated matching mapping.
Each new unshifted press selects Internal when actual THRU feedback is true,
or Thru when false. Unknown feedback means no command and a dark lamp.
Holds are consumed until release, including presses made with either Shift
held or feedback missing; no deferred toggle is queued. Either Shift preserves
hardware Shift+EXT input-type selection by suppressing software commands.
Indicators use only actual per-deck THRU state, with false/unknown dark even
when dim-idle lighting is enabled.

The attended [EXT validation](ext-input-validation.md) confirms single-press
switching and lamps on all four channels, independent mouse-driven feedback,
and Shift suppression with no delayed toggle. This is not a new published
release. DVS must be enabled manually and left on; input assignments remain
GUI setup. The bridge doesn't detect global DVS enable.

- [x] **EXT switching and indicators** - Confirmed on all four channels.
  Broader external-audio validation remains separate below.

## Open

- [ ] **Djay coalesces zero-delay jump bursts** - A paused-deck MIDI-only check
  sent normal MOVE CC 2 deltas +4, -4, then four +2/-2 pairs with native one-beat
  size. All ten CCs were emitted, but the native waveform ended two beats behind
  the starting point. The unchanged normal-MOVE path reproduces this independently
  of Shift-size restoration. Separate four-detent commands work in both directions.
  See [reproduction and evidence](controls-state-sync-validation.md#native-burst-limitation).
- [ ] **Practice-session freshness exit** - A 50 ms motor-input timeout
  stopped the entire full-profile bridge during practice on October 20, 2026.
  The beta.3 fix clears force, discards old movement, and automatically
  resumes tension after a fresh baseline and newly qualified motion while
  keeping control/feedback service running. The underlying input delay
  is unconfirmed and physical recovery testing remains pending.
  See [timeout recovery](tension-timeout-recovery.md).
- [ ] **Wheel sensor-time reversal** - A diagnostic run exited with
  `wheel 0 sensor time reversed` on October 24, 2026. The cause remains
  uninvestigated; this is separate from the self-recovering 50 ms input-gap
  path and was not changed by the musical-key fix. Retain the exact log if it
  occurs again. See [the captured run](key-feedback-order-validation.md#evidence-and-diagnostic-boundaries).
- [x] **Jog feel and backspin** - User confirmed the current jog feel and
  backspin tuning are solved to their satisfaction on October 27, 2026.
  [October 6 comparison](jog-wheel-investigation.md): Traktor's target is
  2.5 beats per revolution at either speed, including released backspin,
  with no reversal after the platter stops. The user calibrated scratch
  Speed/Reaction to 2.7% / 150% and pitch bend to 2.7% / 17%; these are now
  configurable exported defaults in the app's Jog tab.
  The earlier routing ended backspin on touch release, especially at Reaction 50%.
  Current source adds an acceleration-qualified
  scratch continuation, detection toggle/threshold, optional additive scratch
  speed and an owning-deck screen indicator. The October 25 input-only capture
  distinguishes all four deliberate backspins from ordinary releases. The user
  confirmed live continuation, catching, indicator clearing, slow release and
  normal rim nudging on both wheels, plus the detection-disabled comparison.
  A 1.0% boost was not noticeable to them; 5.2% was closer to the desired feel.
  Optional boost deliberately changes
  event travel rather than guaranteeing a fixed distance per revolution.
  The normal motor-tension setting is preserved. A subsequent
  released-assist trial provides independent on/off and 0-100% strength with a
  non-renewable one-second fade. The user confirmed useful assistance at 100%,
  both-wheel operation, catching and wheel isolation, then requested default-on
  assistance at 100% with a 5.6% software boost for beta.2. Earlier short motion-stop
  cancellations remain unexplained, not a claimed fix; see the
  [focused physical results and limits](backspin-assist-validation.md).
  See [the experiment and ordered checks](backspin-detection-validation.md).
  Equal-force flicks are not equivalent across the two setups.
- [ ] **Leftmost FX button duplicates slot 1** - Verify native bank-enable
  behavior and investigate a whole-bank bypass that preserves individual
  slot states.
- [ ] **REC / Record Sample: confirmed Djay bug** - The action did not record
  a sample in the user's October 21, 2026 testing. On October 30, the user
  reported that Algoriddim confirmed this is a bug on their end. Keep this
  open pending an upstream fix and a successful retest; do not treat it as
  an unresolved bridge mapping assignment or claim that it is fixed.
  REC remains assigned to Record Sample, not the set recorder.
- [x] **Broader external-audio validation** - User confirmed external audio
  is already validated on October 27, 2026, closing this checklist item.
  The user has confirmed manually selected DVS THRU on channel 4 works with
  the bridge active, including channel fader, FX and mix/cue. Input-channel
  mixing is no longer a blocker. The user also confirmed the USB input map:
  A 1-2, B 3-4, C 5-6, D 7-8, with PHONO/LINE on A/B and LINE/MIC on C/D.
  The earlier stopped-bridge headphone result
  remains unexplained, not a current fault. See the
  [test record](external-input-routing-test.md),
  [manual workflow](user-guide.md#external-audio-through-a-deck) and
  [software comparison](external-input-software-comparison.md).

## Deferred ideas

- [ ] **MUTE button role** - Deferred by the user on October 21, 2026.
  No distinct assignment chosen; revisit after screen support is understood,
  when layered controls may provide a useful role. This defers the design
  decision, not a change to the current STEMS-duplicate behavior.
- [ ] **Motorized playback beta testing** - Current source implements TT drive,
  relative bend and lower/upper Scratch handoffs, with continued holding force.
  On October 30 the user reported the corrected build much better and began
  extended beta testing; this is not exhaustive fault/recovery acceptance.
  See [current evidence](tt-normal-ring-recovery-validation.md).
  Review experimental settings for removal or hiding before release.
  Cue effects and other new haptics remain deferred.
- [ ] **Jog-ring extended validation** - Normal playback/Scratch/pause
  continuity and TT physical tracking are implemented and software-tested.
  The user reported TT tracking good and the corrected combined build much
  better. Longer beta testing continues; see the same validation record.

## DJ-laptop validation

The October 24 hour-long set passed on the actual DJ laptop. On October 25,
the user explicitly confirmed held Preview, Shift+CUE, library controls,
cross-deck screens and working LED/FX behavior from that set. See the
[set validation](dj-laptop-set-validation.md) for scope and remaining known
faults. Optional exhaustive scenarios are not additional reported bugs.

On October 20, 2026, an initial launch reported that
`default-tension-config.json` could not be opened. The installed resource was
present, signature verification reported no error, and the same app then
launched normally without changes. The cause is unconfirmed; no code fix or
rebuild was made for that report.

On-device screens/pad legends are now authorized experimental work, tracked above.
A controller settings-menu gesture remains deferred.
Automatic Slip during REV is a separate design choice, not part of
the completed momentary-reverse fix. See the mapping audit for these ideas
and the Encdr upgrade candidate.
