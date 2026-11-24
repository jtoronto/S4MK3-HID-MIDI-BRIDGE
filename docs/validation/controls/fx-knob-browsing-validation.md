# Shift + FX knob browsing

October 26, 2026. Internal implementation and verification record.
See the [user guide](user-guide.md#manual-fx-strips) for the normal workflow.

## Behavior and evidence path

The agreed interaction uses physical-side Shift plus parameter knobs 1-3
to browse each slot's effects in both directions. Normal knobs retain parameter
pickup. Dry/wet and Shift + effect-button next shortcuts are unchanged.

The installed-version action inventory in `djay-action-reference.md` declares
`fx1SelectRotary`, `fx2SelectRotary` and `fx3SelectRotary` as stepped rotaries.
The catalog and generator assign these to canonical CCs 32-34 on channels 9-12.
The translator emits rotary-64 deltas on the FX target deck, not necessarily
the deck selected for the physical transport.

Four seven-bit position units produce one selection step, approximately 1/32
of full travel. Remainders stay relative to the last emitted step; duplicate
positions emit nothing. Shift and FX-target changes rebase the origin.
Without a baseline, the first physical sample only seeds it. Shifted parameter
movement suppresses parameter CCs and clears that knob's acquisition gate,
so releasing Shift cannot retain an acquisition from before browsing.

The evidence path is real translator input through emitted MIDI, plus parsed
generated mapping entries. Existing pickup, target routing and held-note tests
remain in the complete suite. This does not establish native Djay scrolling,
direction, perceived spacing, or physical pickup behavior.

## Regression evidence

Before production changes, the new relative-browsing test emitted no selector
where `[184, 32, 65]` was required. The acquired-parameter test emitted the
unwanted parameter `[176, 32, 80]` instead of selector `[184, 32, 68]`.
Both failed, establishing coverage of the missing routing and parameter leak.

After the implementation, all five focused `shifted_fx_` tests passed. They
exercise both strips, all four target decks and all three parameter knobs;
sub-step travel, repeated positions, multi-step motion and reversal; Shift
entry with simultaneous movement; target changes and end stops; first-sample
seeding; unchanged shifted dry/wet; real feedback and parameter reacquisition;
and unique generated rotary addresses with no pickup or output feedback.

## Ordered physical check

No app launch, MIDI injection, controller traffic or preference edit is part
of automated verification. These checks remain for the user:

1. Stop and quit the previous bridge app, then open
   `dist/FxKnobBrowse-20261026/S4 MK3 Bridge.app`.
2. Install its Djay mapping under a new name, preserving custom maps. Select
   that configuration for S4 MK3 MIDI Full, then start the bridge.
3. Select A for the left FX strip and B for the right. Note each slot's current
   effect and parameter in Djay.
4. Hold left Shift and turn knob 1 slowly clockwise, then counterclockwise.
   Expect next and previous effects in slot 1, with no parameter CC adjustment.
   Repeat for knobs 2 and 3, then right Shift and the right strip.
5. Make a larger turn and check useful multi-effect scrolling. Hold still at
   an end stop: selection must stop, not repeat.
6. Release Shift and reposition away from the software parameter value:
   selection must not change and parameter pickup must prevent a jump.
   Cross that value to resume parameter adjustment. Rehold Shift to continue
   browsing from the new physical position.
7. Retarget each strip to C/D while Shift is held. No selection should occur
   from retargeting alone; subsequent movement must affect only the new target.
8. Check normal effect buttons, their Shift + button next shortcuts, and
   shared dry/wet still work. Existing FX pickup warnings should remain useful.

## Source and package verification

All five new regressions pass. The full serial Rust run with `--no-fail-fast`
finished with 356 passing tests and two pre-existing diagnostic CLI failures:

- `explicit_free_play_startup_reaches_log_creation_before_hid`
- `wheel_selection_is_admitted_only_by_the_intended_profile`

Both expect valid diagnostic options to reach deliberate invalid-path log
creation. The user's existing RevFlxIdle bridge was running as process 57012.
The diagnostic's earlier ownership guard rejected that process instead.
A direct invocation with the same impossible `/dev/null/...` capture path
confirmed the competing-controller error, before any HID open. No app was
stopped, and no test or guard was changed to conceal this environment limit.

Formatting, strict locked all-target/all-feature Clippy, the locked all-target
build, all 55 Swift tests, build-script syntax, Info.plist lint and whitespace
checks passed. The initial Clippy run required extracting the browsing emission
and mapping-generation loops to keep their parent functions within the existing
100-line limit. No lint allowance was added. The two pre-existing SwiftPM
unhandled-file warnings remain.

The separate app is `dist/FxKnobBrowse-20261026/S4 MK3 Bridge.app`, with ZIP and
DMG alongside. The portable build, deep strict signature check, ZIP integrity
and DMG verification all exited 0. The app includes the newly generated mapping;
it has not been launched or substituted for the running app. Earlier packages
are preserved.

```text
3e8bb5089bc7fb533014a76ce3196c6ef6ff4e90b448c79bfee63b21003c0a13  S4 MK3 Bridge-arm64.zip
cf43b0297492e37ed69fa5a1827f1cb7572ed0eb22e4588f53d67eab24c3b401  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```

Native scrolling and perceived spacing remain untested on the controller.

After receiving this build, the user reported "Works like a charm" on October
26, 2026. This establishes general physical acceptance of Shift + FX knob
browsing, superseding the pending status above. It is not an instrumented pass
of every target/modifier combination.
