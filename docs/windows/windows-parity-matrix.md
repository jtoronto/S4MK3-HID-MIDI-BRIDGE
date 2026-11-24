# Windows source parity matrix

This is the November 4 task-1 inventory of source behavior, not Windows runtime or physical acceptance. Every row starts **UNVERIFIED**. Named proving scenarios are obligations for the delivering tasks, not claims that a runner or test already exists. Source-audit PASS in the companion coverage record never changes a row's Windows status. Later bounded testing is summarized in [Windows implementation: current state](windows-current-state.md); it does not update the historical rows below.

Source baseline: HEAD `7ec18cf8a85a5450d2bb97043cd0bfb34555807d`, plus the actual dirty contents identified by manifest SHA-256 `d9a0c304f63e4247f7f948e829d6a27fe1f58a05c5e496a20d6808eee1ae054a` and content aggregate `dc3684558657869206dfba1f2a9ee35a8d20155bff0c2fae2db2d42ced77ddd6`. Source date: November 4, 2026. Historical published downloads are not this baseline.

The assembly preserves all 31 runtime IDs R-01 through R-31 and all 70 host IDs H-01 through H-70 from the upstream audits. Added H-71 through H-80, R-32 through R-33 and L-01 through L-12 close source-backed gaps and make limitations explicit. IS-1 through IS-8 trace the plan's outcomes. There are 133 unique rows including those eight outcome rows. Paths are repository-relative; abbreviated line references inherit the preceding file within the same row. In H rows, all Swift host paths are under macos/ and test paths under macos/Tests/.

Assembly evidence, source-side QA, exact baseline commands, audit corrections and hashes are in `.omo/evidence/windows-bridge/20261104-01a1a4a4/task-1/parity-coverage.md`. Upstream files there are `runtime-parity.md`, `host-parity.md`, `baseline.json` and `snapshot.json`. Missing upstream evidence is a coverage failure.

## Profile and settings navigation

| Scope | Concrete rows | Boundary |
| --- | --- | --- |
| Full | R-04 through R-25; R-27,R-28; H-15,H-21 | Deck/layer translation, genuine feedback, LED/screens/jog ownership |
| Minimal MIDI | R-03; H-15,H-21,H-28 | Fixed Play/Cue and faders, no feedback, deck switching or jog translation |
| HID probe | R-32; R-26; H-15,H-17,H-21 | No MIDI; interrupt/raw/duration workflow, no full-profile configuration |
| Offline utilities | R-12,R-22,R-26; H-42,H-43 | Mapping, screen previews and motor replay do not open HID/MIDI |
| Bridge tab | H-14 through H-16,H-19 through H-27,H-35,H-43,H-44 | Start/stop, duration/profile, mapping and file access |
| Screens tab | H-45 through H-57,H-78 | Immediate-save toggles, consent, retry, owned restoration, manual actions |
| Jog tab | H-36 through H-42,H-68,H-79 | Motorized defaults, advanced tuning, wheel feel, mapping-only values |
| LEDs tab | H-28 through H-34 | Light settings, JSON import/export, full-profile warning |
| Palette tab | H-71 through H-77; H-31,H-32,H-33 | Separate editable colors/modes, persisted in LED JSON |
| Diagnostics tab | H-17,H-18,H-58 through H-63,H-69 | Log level/raw flag, live state, capture/reveal/copy/command preview |
| All tabs and tray | H-01 through H-14,H-26,H-34,H-80 | Six tabs, reset/save/restart and native tray lifecycle |

## Outcome traceability

| ID | Workflow | Source references | Current behavior | Expected Windows behavior | Delivery task | Named proving scenario | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IS-1 | Portable Windows 11 x64 product | macos/build-app.sh:60-74; R-30; R-33 | Only macOS product packaging exists in current source. | Self-contained Windows portable ZIP, no development tools on DJ laptop. | 2,19,20 | build-smoke | UNVERIFIED |
| IS-2 | Full, minimal and probe ownership | src/midi.rs:1-100; src/input.rs:66-97; R-03 through R-09; R-32; H-15 | Three distinct profiles with held-note and jog ownership. | Preserve all three, including release/stop semantics. | 5,6,7,10 | profiles-and-held-ownership | UNVERIFIED |
| IS-3 | Genuine feedback without self echoes | src/full/feedback.rs:238-300; R-10,R-11,R-13,R-15 through R-21 | Native virtual feedback cache; Windows shared-bus partition not established. | Only declared feedback changes state; echoes cannot authorize playback. | 4,7,8 | djay-feedback | UNVERIFIED |
| IS-4 | Both screens and managed recovery | src/screens.rs:146-200; R-22,R-23; H-45 through H-57,H-78 | Current pages and host consent/retry/recovery use macOS access. | Same pages/lifecycle with verified Windows interface and owned service handover. | 12,13,18 | screens-access-retry-restore | UNVERIFIED |
| IS-5 | Jog, tension, TT and neutral stop | src/bridge_runtime.rs:114-127; R-06,R-09,R-24,R-25; H-36 through H-41,H-68,H-79 | Shared Rust laws plus macOS timing and host controls. | Preserve laws, measure Windows timing, physically prove neutral. | 9,10,11,21 | motor-timing-neutral | UNVERIFIED |
| IS-6 | Tray, six tabs and persistence | macos/S4BridgeApp.swift:1284-1299; H-01 through H-80 | Native AppKit/SwiftUI host and saved JSON. | Avalonia host with every action, validation and diagnostic workflow. | 14,15,16,17,18 | host-six-tabs | UNVERIFIED |
| IS-7 | Audio and user data preservation | src/screens.rs:166-172; R-02,R-28; H-25,H-43,H-51,H-52; L-06 | No audio routing; native export replacement differs from Windows preservation contract. | Never claim audio, replace user mappings silently or restore unowned service state. | 3,9,15,18,21 | audio-user-data-preservation | UNVERIFIED |
| IS-8 | Existing macOS behavior and resources | macos/build-app.sh:60-74; R-12,R-30; H-70 | Captured dirty-source baseline is source of record, not published download. | Shared changes retain native mapping semantics, resources and macOS workflows. | 5,7,19,22 | macos-regression-and-resources | UNVERIFIED |

## Runtime workflows

R-01/R-02 and R-26/R-28/R-30/R-31 span profiles; the profile map above distinguishes full-specific behavior from minimal/probe. Runtime rows and host rows describe separate layers of the same workflow, so both remain test obligations.

### R-01 HID report framing, snapshot pacing, stale-interrupt rejection

- Source: `src/input.rs:22-109` (`read_analog`, `read_batch_inner`, report 1 = 23 B, report 3 = 60 B, report 2 = 79 B incl. ID); `src/input.rs:144-210` stale-interrupt test/floor.
- Current behavior: full mode polls reports 1/3 as snapshots with touch/buttons before movement; bounded `read_timeout(..., 2ms)` paces; queued analog interrupt 2 is demoted to a change notification and `read_analog` re-reads; `accepts_interrupt` rejects stale copies so newer snapshots win.
- Limitation: macOS hidapi framing assumed; no Windows HID proof.
- Expected Windows: task-9 adapter preserves IDs/lengths/order, normalizes only demonstrated framing deltas at ingress.
- Delivering task: 9. Proving scenario: `verify.ps1 -Scenario hid-reports` + `windows/qa/hid-reports.ps1`. Status: UNVERIFIED.

### R-02 HID open, collection selection, competing-process guard, audio non-claim

- Source: `src/main.rs:214-273` (VID/PID `0x17cc:0x1720`, single-device/collection `ensure!`, `HidApi::new`, `ownership::check()`, `S4_HID_OPEN: native HID access; audio interfaces are not claimed`); `src/jog_diagnostic/ownership.rs`; `src/screens.rs:146-173`.
- Current behavior: enumerates USB via Encdr hotplug, refuses zero/multiple devices/collections, checks competing owners before opening HID; audio interfaces never claimed.
- Limitation: `/bin/ps`-style ownership and first-match-adjacent selection are macOS formulations; Windows identity/capability selection unproven.
- Expected Windows: task-3 inventory then task-9 identity/capability selection, bounded read-only process inventory, exact executable identities; never claim audio.
- Delivering task: 9 (inventory 3). Proving scenario: `verify.ps1 -Scenario device-inventory` then `hid-reports`. Status: UNVERIFIED.

### R-03 Minimal MIDI profile (fixed decks, no feedback/jog)

- Source: `src/midi.rs:1-100` (`PORT_NAME "S4 MK3 MIDI"`, `create_virtual` via `midir::os::unix::VirtualOutput`, `button_message` left/right play/cue zero-based channels 0-1, `slider_message` all four faders CC 16-19 + crossfader CC 20 on MIDI channel 1, Drop releases notes 0-1 on ch 0-1).
- Current behavior: bare `--midi` keeps verified minimal mapping; encoders/touch/grid ignored.
- Limitation: Unix virtual-port construction only; no Windows transport.
- Expected Windows: preserved through task-5 transport seam; offline mapping/validation remains available; live minimal MIDI requires its transport; runtime backend flag defaults to LoopBe on Windows.
- Delivering task: 5/6. Proving scenario: native Offline suite + `verify.ps1 -Scenario midi-lifecycle` (S4-data-free). Status: UNVERIFIED.

### R-04 Full profile emission domains (decks 1-4, shifted 9-12, mixer 5, select 6)

- Source: `src/full/mod.rs:38-95` (Deck A-D channels 0-3, Address press/release); `:562-620` (`emit_mixer`); `:926` (mixer channel 5 context); `src/full/catalog.rs:247` (browse strips ch 9-12), `:440-570` (deck select/assignment/shift); `src/full/mapping.rs:40-118` (deck/mixer/select/curve push).
- Current behavior: input channels 1-6 and 9-12 retained per AGENTS.md mapping contract; mixer on 5, deck selection on 6.
- Limitation: Windows deck-feedback reservation (13-16), mixer feedback 8, telemetry 7 must be proven disjoint (task 4 provisional).
- Expected Windows: native layout unchanged; LoopBe layout consumed by one definition in task 7; commands stay on 1-6/9-12.
- Delivering task: 7 (contract 4). Proving scenario: `verify.ps1 -Scenario feedback-route` + layout-disjointness tests. Status: UNVERIFIED.

### R-05 Held-note ownership across deck/shift/pad/FX changes

- Source: `src/full/mod.rs:2276` (old ownership releases on A while new press targets C), `:2541` (same deck held vs transfer), `:3053` (shift-cue keeps original release deck), `:3591-3620` (LED routing follows selection while held notes keep old ownership; screens follow selectors without changing held ownership).
- Current behavior: releases route to the original deck/action owner, never the current selector; deck/shift/pad/FX changes cannot strand or steal notes.
- Limitation: Mac-side logic only; Windows timing/queue behavior unproven.
- Expected Windows: identical ownership; tasks 6/7/10 replay ownership regressions natively.
- Delivering task: 10. Proving scenario: native Offline suite + `verify.ps1 -Scenario djay-feedback` held-release cases. Status: UNVERIFIED.

### R-06 Jog-touch and wheel/coasting ownership

- Source: `src/full/mod.rs:96-130` (`WheelOwner { deck, action, coasting }`, `MotorGateState.owner`); `:2499` (coasting backspin retains ownership across deck/shift changes); `:2412-2416`,`:2545` (wheel/coast message expectations); `src/jog_diagnostic/ownership.rs`.
- Current behavior: jog touch binds wheel owner; coasting/backspin keeps the originating owner across selector changes.
- Limitation: no Windows motor/timing acceptance.
- Expected Windows: same WheelOwner semantics under measured Windows snapshot timing.
- Delivering task: 11 (after 10). Proving scenario: `verify.ps1 -Scenario djay-feedback` + motor/timing QA in task 11. Status: UNVERIFIED.

### R-07 Deck/layer routing (deck select, shift, pad modes, FX targets, browsing)

- Source: `src/full/catalog.rs:247-570` (FX left/right, quick, pad `pad_binding` Hotcue/Samples/Stems x8x4 decks, deck select A/C-B/D, assignment, curve); `src/full/mapping.rs:44-118`; `src/full/mod.rs:807-867` (jump select/restore), `:1383-1480` (sync_front_panel selector application).
- Current behavior: browsing strips on ch 9-12 separate from parameter CCs; shifted layer, pad modes, FX targets route without disturbing held owners.
- Limitation: Windows mapping identity/layout proof pending.
- Expected Windows: same routing; generated Windows mapping under separate name.
- Delivering task: 7/10. Proving scenario: `feedback-route` + full hardware parity scenario (task 21). Status: UNVERIFIED.

### R-08 Startup restoration (analog sync once, jump-size restore, front-panel ready)

- Source: `src/full/mod.rs:183-230` (`jump_restore: [Option<u64>;4]`, `jump_revision`); `:296-410` (startup path calls `sync_front_panel`); `:1383-1480` (`sync_front_panel`, `front_panel_ready` set once on first `playback_received`, `CROSSFADER_FEEDBACK_RECEIVED` one-shot, jump restore confirm/replace logic); `:2747-2780` (shifted MOVE restore-after-detents test).
- Current behavior: channel volumes and high/mid/low EQ, plus assignment/curve switches, synchronize once after genuine playback feedback; gain, filter, tempo and crossfader position are excluded; per-deck jump size restores after fixed one-beat jump unless newer non-one selection confirms external edit; startup does not guess from commands.
- Limitation: requires genuine Djay feedback; silent mapping stays unready.
- Expected Windows: same gate; readiness only after real LoopBe feedback callbacks and controller init (contract 7).
- Delivering task: 8/10. Proving scenario: `verify.ps1 -Scenario djay-feedback` launch-order cases. Status: UNVERIFIED.

### R-09 Shutdown neutral (motor neutral before teardown, note releases, no false ready)

- Source: `src/bridge_runtime.rs:114-133` (clear force before input teardown/MIDI/LED release; neutral-report check); `src/midi.rs:Drop` (releases mapped buttons); `src/full/mod.rs:3526` (shutdown releases ownership test).
- Current behavior: normal stop clears motor force first, releases held MIDI notes, tears down reader/worker/queue/child handles.
- Limitation: Windows close-order and LoopBe handle lifetime unproven.
- Expected Windows: same order; timeout reports failure and never falsely claims neutral; contract 7 `S4_BRIDGE_STATUS` stop semantics.
- Delivering task: 6/11. Proving scenario: `midi-lifecycle` open/close cycles + task-11 motor QA. Status: UNVERIFIED.

### R-10 Feedback decode, cache, bounded queue/overflow, channel-7 playback/meters/loading

- Source: `src/full/feedback.rs:1-140` (`FeedbackState` loaded/playing/internal/thru/eq/fx/meters, `playback_received`, `QUEUE_CAPACITY 4096`); `:238-300` (`FeedbackInput::new` virtual input + decoder callback, `drain`, overflow errors `restart to synchronize software state`); `src/full/feedback/info.rs`; `src/full/mapping.rs:80-118` (ch-7 monoMeter/playing/loading outputs).
- Current behavior: bounded crossbeam queue; running-status/realtime-tolerant decode; overflow fails loudly instead of silently desyncing; dedicated ch-7 telemetry/global outputs already on 7.
- Limitation: macOS virtual input (`midir::os::unix::VirtualInput`); zero/ambiguous Windows endpoint handling missing.
- Expected Windows: task-6 WinMM/LoopBe constructors, receive-before-ready, bounded queue retained; Windows layout normalizes only declared feedback then uses the real decoder/cache.
- Delivering task: 6/7. Proving scenario: `midi-lifecycle` + `feedback-route`. Status: UNVERIFIED.

### R-11 Feedback-silence policy (unknown stays unknown; echoes never authorize)

- Source: `src/full/mod.rs:1383-1420` (`playback_authorized` requires loaded+playing+internal+not-THRU; silence does not revoke; touch-scratch needs no pause proof; unknown never authorizes force); `src/full/output.rs:50` (`LED_FEEDBACK_ACTIVE: software state starts unknown`); plan GAP-3/contract warnings.
- Current behavior: no optimistic inference from button presses or own sends; missing outputs leave state `None`; Djay disappearance handling stays a warning, not silent revocation (separate behavior change explicitly out of scope).
- Limitation: provider-silence matrix only partially proven on macOS; Windows provider behavior entirely open (task 8).
- Expected Windows: identical policy; echo-only streams leave cache/LEDs/authorization unchanged; task 8 proves each family or leaves the node failed.
- Delivering task: 8. Proving scenario: `verify.ps1 -Scenario djay-feedback` echo/mute/absent-mapping cases. Status: UNVERIFIED.

### R-12 Native mapping generation identity and semantics

- Source: `src/full/mapping.rs:19-118` (`PORT_NAME "S4 MK3 MIDI Full"`, `generate`/`generate_with_jog`, `write` create-new-only, endpoint/channels/message types, FX/browse/jump/pad/mixer/select/curve/outputs); `:198-475` + tests `:489-570` (plist root/controls/outputs, native FX/mixer targets).
- Current behavior: deterministic native plist; `write` never overwrites existing user files; jog config varies only jog controls.
- Limitation: single native endpoint identity; Windows identity not yet generated.
- Expected Windows: native output semantically unchanged for every jog fixture; Windows file `S4 MK3 Bridge Windows.djayMidiMapping`, endpoint `LoopBe Internal MIDI`, offline generation accepts either layout.
- Delivering task: 7. Proving scenario: `cargo test --locked midi_layout` + mapping-fixture comparisons + `feedback-route`. Status: UNVERIFIED.

### R-13 Wire/endpoint separation (native vs LoopBe; no echo normalization)

- Source: `src/full/mapping.rs:19` + `src/midi.rs:10` + `src/full/feedback.rs:238-300` (native virtual source/destination construction today; direction is part of endpoint identity); contracts 1-3 (provisional 13-16/8/7 partition).
- Current behavior: macOS commands and feedback share the native definition; task-4 provisional Windows partition unproven against every command/startup/restore/minimal path.
- Limitation: any inherited embedded output, Note Off/note-on-zero, or startup-restoration collision can break the partition until task 4/7 audit passes.
- Expected Windows: one layout definition; ingress accepts only declared feedback, normalizes to canonical cache addresses, drops command addresses before state changes.
- Delivering task: 4/7. Proving scenario: `feedback-route` colliding-layout rejection + echo-only no-state-change assertions. Status: UNVERIFIED.

### R-14 Transport/cue/sync/loop/move/encoder controls

- Source: `src/full/catalog.rs:120-400` (note/CC/relative bindings incl. `EQ_MODE_NOTE 20`, `SCRATCHING_MODE_NOTE 7`); AGENTS.md encoder table (MOVE/LOOP nibbles bytes 19-21, presses byte 6/15 masks; custom Djay LOOP-length/MOVE-jump semantics); `src/full/mod.rs:2735-2780` (MOVE detents/jump), `:2817-2830`, `:2635-2650` (rotary expectations).
- Current behavior: LOOP selects length + press activates; MOVE independent jump length, press toggles per-deck size, Shift+MOVE always one beat.
- Limitation: encoder rename alone proves nothing about Djay behavior; Windows delivery unproven.
- Expected Windows: identical addresses/targets via shared catalog.
- Delivering task: 10. Proving scenario: full hardware parity scenario (task 21) + Offline emission tests. Status: UNVERIFIED.

### R-15 Mixer (EQ/filter/gain/faders/crossfader/curve, Internal/THRU, Neural EQ mode)

- Source: `src/full/catalog.rs:440-560` (mixer bindings incl. `EXT_THRU_CC 21`, `INTERNAL_MODE_CC 22`, `EXT_BINDINGS` thru/internal notes 27-28, `EQ_MODE_NOTE 20`); `src/full/feedback.rs:68-140` (thru/internal/fx apply, per-deck independence); `src/full/mod.rs:562-620`,`:926`,`:1383-1400`.
- Current behavior: per-deck THRU/Internal confirmed from feedback; crossfader sync once after genuine readiness; later GUI edits respected.
- Limitation: task-8 must prove each mixer feedback family on Windows.
- Expected Windows: same; mixer feedback normalized on channel 8 per provisional contract.
- Delivering task: 8/10. Proving scenario: `djay-feedback` mixer/GUI-vs-MIDI cases. Status: UNVERIFIED.

### R-16 FX values, slots, pickup, instant-FX order

- Source: `src/full/mod.rs:96-140` (`FxPickup` low/high/latest window); `:1383-1440` (pickup observe, `fx_values` adopt); `src/full/feedback.rs:68-140` (CC 32-37 per-deck FX, info outputs); `src/full/mapping.rs` FX outputs.
- Current behavior: physical FX does not jump until feedback window confirms; Instant-FX most-recent-active ordering kept.
- Limitation: outstanding-value window depends on genuine ordered feedback.
- Expected Windows: unchanged control law; only transport/layout changes.
- Delivering task: 8/10. Proving scenario: `djay-feedback` FX GUI-vs-MIDI + Offline pickup tests. Status: UNVERIFIED.

### R-17 Pads (Hotcue/Samples/Stems colors, mute/solo, tokens)

- Source: `src/full/catalog.rs:484-540` (pad bindings, mute/solo/exclusive shifted); `src/full/feedback.rs:22-28` (`PAD_COLOR_TOKENS 1-8`, `PAD_WHITE_TOKEN 9`); `src/full/mapping.rs:14` (pad color tokens in outputs); `src/full/leds.rs` + `src/full/screens.rs` (held-pad rasters).
- Current behavior: per-deck pad colors from genuine feedback; local deck/routing lights independent.
- Limitation: Windows pad-color Declarations unproven.
- Expected Windows: same tokens/semantics; feedback on declared addresses only.
- Delivering task: 8/10. Proving scenario: `djay-feedback` pad/matrix cases. Status: UNVERIFIED.

### R-18 Browse/load/preview (jump sizes, loadingSuccess, preview path)

- Source: `src/full/catalog.rs` jump bindings + `jump_restore_binding`; `src/full/mapping.rs:80-118` (`song.loadingSuccess` outputs, jump outputs via `push_jump_restores`/`push_info_outputs`); `src/full/feedback/info.rs` (jump_index/revision); `src/full/mod.rs:807-867`,`:1425-1440`.
- Current behavior: loading confirmed only by `loadingSuccess`; jump restore logic as in R-08.
- Limitation: preview/feedback declarations need task-8 proof on Windows.
- Expected Windows: same; startup volume/EQ and assignment/curve sync respects later GUI edits; crossfader position is excluded.
- Delivering task: 8/10. Proving scenario: `djay-feedback` load/preview cases. Status: UNVERIFIED.

### R-19 Button LEDs (local vs confirmed state, validation, pacing)

- Source: `src/full/leds.rs:86-260` (`LedConfig` load/validate/defaults, `LocalState`, `render_buttons`); `src/full/output.rs:64-130` (`BUTTON_INTERVAL 10ms`, change-only writes, `clear`); tests `:616-1080` incl. bundled-vs-builtin identity.
- Current behavior: deck/routing lights local; software state starts unknown; 10 ms change-only pacing; unknown fields rejected; brightness/palette validated.
- Limitation: macOS HID write path only.
- Expected Windows: same renderer; only HID adapter changes (task 9/10).
- Delivering task: 10. Proving scenario: Offline LED tests + hardware parity LED matrix. Status: UNVERIFIED.

### R-20 Jog rings and phase presentation

- Source: `src/full/output.rs:16-100`,`:201-227` (`RING_INTERVAL 20ms`, `ring_phase::Rings::positions`, `render_ring`, init `0x30` 28 B per side); `src/full/ring_phase.rs`, `src/full/ring_position.rs`, `src/full/ring_tests.rs` (physical vs playback phase, TT tracking, scratch deltas, pause continuity).
- Current behavior: playback-driven normal-mode ring phase with scratch deltas and pause continuity; TT physical tracking; no upper-speed handoff forcing sustained RPM (per Oct 30 corrections).
- Limitation: physical ring behavior unproven from writes alone; Windows timing unproven.
- Expected Windows: identical presentation; measured timing under load.
- Delivering task: 10/11. Proving scenario: hardware parity ring cases (task 21). Status: UNVERIFIED.

### R-21 Meters (level-gated writes, pacing, silence behavior)

- Source: `src/full/output.rs:100-130` (`METER_INTERVAL 50ms`, no meter report until Djay supplies a level); `src/full/feedback.rs:100-140` (ch-7 CC meters + `meter_updated`); `src/full/mapping.rs:80-100` (monoMeter outputs).
- Current behavior: meters never written pre-level; change-only 50 ms pacing.
- Limitation: Djay meter Declarations need task-8 proof.
- Expected Windows: same gate; mixer/meter feedback on declared addresses.
- Delivering task: 8/10. Proving scenario: `djay-feedback` meter cases. Status: UNVERIFIED.

### R-22 Screens pages, worker, control-stdin lifecycle

- Source: `src/screens.rs:1-145` (raster/page/font, `generate_screen_previews` offline path); `src/screens/worker.rs` + `worker/service.rs` + `worker/tests/*` (pages, enable/retry/restore state machine); `src/screens_control.rs:10-95` (`S4_SCREEN_CONTROL <id> ON|OFF|RETRY` parse/validation); `src/main.rs:102-160` (`--screens`, `--screen-control-stdin`, `generate_screen_previews` conflicts); `src/full/screens.rs` + `key.rs` + `tests.rs` (routing, held-pad rasters, selector-following without ownership change per `src/full/mod.rs:3605`).
- Current behavior: both screens show current pages; managed enable/retry/restore; EOF stops bridge; offline preview export without USB/MIDI; six-tab host exposes Screens settings (task-15 target).
- Limitation: Mac USB path only; Windows access/driver coexistence unproven (GAP-4).
- Expected Windows: reuse raster/pages/font/worker; claim only when compatible access proven; `S4_SCREEN_CONTROL` + `S4_SCREEN_STATUS` semantics preserved; new `S4_BRIDGE_STATUS` v1 does not alter screen lines.
- Delivering task: 13/18 (access 12). Proving scenario: screen-access contract QA + task-13/18 host integration scenarios. Status: UNVERIFIED.

### R-23 Screen USB transport (interface 4, bulk OUT 0x03, no audio/config/reset)

- Source: `src/screens.rs:146-200` (`claim_interface(4)`, `endpoint::<Bulk,Out>(0x03)`, no configuration change/reset/audio-HID detach; busy hint names NIHardwareAgent/Traktor/Mixxx).
- Current behavior: interface-scoped output independent of HID/audio; refuses to rebind or reset device.
- Limitation: macOS nusb behavior; Windows display access requires task-12 contract (no copied force-stop policy, restore only app-stopped services).
- Expected Windows: claim only interface 4/0x03 when proven compatible; never audio interfaces; restore only owned transitions.
- Delivering task: 12/13. Proving scenario: task-12 access-contract QA (read-only until authorized). Status: UNVERIFIED.

### R-24 Jog/tension/TT/backspin/braking/assistance and motor gates

- Source: `src/full/jog.rs:62-130`,`:222-340`,`:560-580` (platter target 33 RPM, base 7000, max/valid ranges, strict JSON validation; shipped-vs-builtin test `:280`); `src/tension.rs:111-370` (per-side backspin assist, qualify/set_backspin, torque); `src/jog/*` (decode/model/platter/continuity/damped/smooth); `src/tension/*` (backspin/playback/diagnostics); `src/full/mod.rs:1450-1560` (motor gate: TT enabled + loaded + playing + internal + not-THRU + owner/grid/shift; `tt_off/track_unloaded/playback_unknown/...` blocks); `src/full/platter_*.rs`, `src/full/backspin.rs`, `src/bridge_runtime/paced/*` tests.
- Current behavior: TT Vinyl startup off, Normal on; delayed scratch handoff at lower bend threshold; continuous apparent assistance while held; playback-driven normal ring phase with scratch deltas; backspin/nudge/brake laws depend on fresh snapshots + genuine playback/internal/THRU.
- Limitation: control laws validated on Mac timing/feedback; Windows snapshot framing/timing/motor cleanup without acceptance (GAP-5).
- Expected Windows: identical laws; 2 ms service target, 50 ms freshness cutoff, neutral-before-teardown; measure actual timing under combined load, never widen timeouts to hide violations.
- Delivering task: 11. Proving scenario: task-11 motor/timing QA + task-21 physical acceptance. Status: UNVERIFIED.

### R-25 Runtime pacing, freshness cutoff, watchdog, neutral-before-teardown

- Source: `src/bridge_runtime.rs:1-140` (service finish/neutral, `with_cleanup(result, neutral, "motor neutral")`, neutral-report check); `src/input.rs:91-109` (2 ms full-mode pace); `src/full/output.rs` (10/20/50 ms LED pacing); `src/bridge_runtime/paced.rs` + `paced/tests/{backspin,playback,recovery,tt_gesture,watchdog}.rs`; `src/jog/decode.rs:16` freshness and plan contract 6.
- Current behavior: paced service loop; stale data rejected; watchdog/recovery paths tested; motor force cleared before ports/threads torn down.
- Limitation: Mac timing only.
- Expected Windows: same targets; native measurement under combined load; no elevated thread priority without evidence.
- Delivering task: 11. Proving scenario: task-11 timing QA. Status: UNVERIFIED.

### R-26 Probe/raw/replay/diagnostic CLIs and managed-stdin extension point

- Source: `src/main.rs:38-160` (`--list`, `--raw`, `--seconds`, `--until-stopped`, `--replay-motor-fixture`, `--generate-mapping`, `--generate-screen-previews`, `--screen-control-stdin`; `--managed-stdin` does not exist yet - contract 7 to add `--managed-stdin` for full/minimal/probe while retaining `--screen-control-stdin`); `src/bin/s4mk3-jog-diagnostic.rs`, `src/bin/s4mk3-screen-diagnostic.rs`; `src/full/replay.rs`, `src/bridge_runtime/replay.rs`; `src/screens_control.rs`.
- Current behavior: hardware-free offline generation/preview/replay; probe paces minimal/interrupt transport.
- Limitation: `--managed-stdin` and aggregate `S4_BRIDGE_STATUS` are task-5/contract-7 additions, not current behavior. Existing `S4_MOTOR_CONFIG` (`src/main.rs:185-201`) and `S4_PLATTER_STATUS` (`src/full/mod.rs:365-379`) must be preserved.
- Expected Windows: same CLI surface plus narrow contract-7 additions; host generation tokens discard old launches; normal stop waits for exit/cleanup.
- Delivering task: 5/14. Proving scenario: `build-smoke` + host lifecycle QA (tasks 14/16). Status: UNVERIFIED.

### R-27 Preferences (LED/jog/tension JSON keys, validation, defaults, reset/export)

- Source: `src/full/leds.rs:86-260` + tests `:1043-1080`; `src/full/jog.rs:62-340`; `src/tension.rs`; `examples/led-config.json`, `examples/jog-config.json`, `examples/tension-config.json`; `macos/S4BridgeApp.swift` + `macos/ScreenPreferences.swift` (six settings areas per plan IS-6).
- Current behavior: same Rust keys/validation/defaults/reset semantics; `LedConfig::load(None)` defaults asserted; unknown fields denied; jog/tension strict typing; saved user Djay mappings never overwritten by generator (`create_new`).
- Limitation: macOS Application Support vs Windows `%LOCALAPPDATA%\S4 MK3 Bridge\` + Logs/Diagnostics split is a task-15 change.
- Expected Windows: identical keys/validation/defaults/reset; atomic staged replacement; no writes inside distributed app; motorized-default restoration preserved; unsaved-jog-mapping-export and hidden-Diagnostics refresh rows must appear in the shared matrix (task-1 QA-failure guard).
- Delivering task: 15 (contracts 4). Proving scenario: host preferences QA (tasks 15/16) + Offline config tests. Status: UNVERIFIED.

### R-28 Audio-local controls and no audio routing

- Source: `src/main.rs:244-280` (audio interfaces not claimed); `docs/traktor-djay-comparison.md`, `docs/user-guide.md` (master/booth/headphone remain hardware-local); `src/screens.rs:166-168` (no audio detach).
- Current behavior: bridge never routes audio; Djay/NI own audio; booth/master/headphone knobs stay local.
- Limitation: Windows composite-interface ownership/cleanup untested (GAP-7).
- Expected Windows: same; HID/display access must not replace or claim audio interfaces.
- Delivering task: 9. Proving scenario: `device-inventory` + `hid-reports` audio-coexistence observations. Status: UNVERIFIED.

### R-29 Known limits and non-goals carried into parity

- Source: `src/full/output.rs:50-52` (`END_WARNING_UNAVAILABLE`); `src/full/mod.rs:1383-1420` (silence semantics); AGENTS.md hardware boundaries; `.omo/plans/windows-bridge.md:79-110` scope boundaries; `docs/user-guide.md:589-608,656-668` limitations.
- Current behavior: no verified remaining-time/end-warning output; current motorized playback is included (R-24); no additional screen pages/haptics, Serato support or audio routing is added, and Windows publication is outside this plan.
- Expected Windows: same limits recorded as parity rows; any screen-driver change needs its own evidence/consent.
- Delivering task: 22 (docs). Proving scenario: final audits F1-F4 read-only review. Status: UNVERIFIED.

### R-30 Build/test/evidence baseline and dirty-tree attribution

- Source: `Cargo.toml`, `Cargo.lock`, `macos/Package.swift`, `macos/build-app.sh`, `tests/*_cli.rs`, `windows/` (untracked diagnostic probes), task-1 acceptance (required Mac checks before production edits).
- Current behavior at audit: no production edits made; the separately captured `baseline.json` records all eight required Mac commands at exit 0, 510 Rust and 80 Swift tests passed, and two SwiftPM unhandled-file warnings. This assembly verified those log hashes; it did not rerun the suites.
- Limitation: unit counts and historic packages do not describe the current tree; pre-existing failures must be captured before attribution.
- Expected Windows: task-2 proves Mac-to-Windows build/test routes with pinned tool/SDK/CRT versions/hashes; manifest-driven native suites; no Wine acceptance.
- Delivering task: 2/19. Proving scenario: `verify.ps1 -Scenario build-smoke`. Status: UNVERIFIED.

### R-31 Fault paths (queue overflow, malformed backend, absent ports, launch races, restore pending)

- Source: `src/full/feedback.rs:278-300` (overflow `ensure!`); `src/main.rs:200-280` (`ensure!` single device/collection, framing `ensure!`); `src/full/mod.rs` jump-restore pending (`jump_restore`), FX pending window (`FxPickup.pending`), `front_panel_ready` gating; `src/screens/worker/*` retry/restore; contract 7 (malformed managed input cannot authorize readiness/force; old generation tokens discarded).
- Current behavior: bounded reproducible failures at reader/worker/queue/child boundaries; timeout reports failure without false neutral claims.
- Limitation: Windows fault injection matrix not executed.
- Expected Windows: same boundaries; `verify.ps1` fails on missing assertions/evidence; echoes/muted LoopBe/absent mapping/stale messages stay unready/unknown.
- Delivering task: 5/6/8/20. Proving scenario: `midi-lifecycle` fault cases + `djay-feedback` fault cases + task-20 lifecycle QA. Status: UNVERIFIED.

## Host workflows

Editor means in-memory values; saved means persisted JSON; launched means the active child configuration. Most edits require Save/restart. Screens options save immediately. Mapping export reads the live jog editor.

| ID | Workflow | Source references | Current behavior | Expected Windows behavior | Delivery task | Named proving scenario | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| H-01 | Tray: first row shows live bridge status | macos/BridgeStatusMenu.swift:14 | Disabled item `Bridge: <state>` updates via `updateMenu` (S4BridgeApp.swift:1678) | Tray shows live worker status | 16 | host-tray-status | UNVERIFIED |
| H-02 | Tray: second row shows live screen status | macos/BridgeStatusMenu.swift:15 | Disabled item from `screens.state.rawValue` | Tray shows live screen state | 16,18 | host-tray-status | UNVERIFIED |
| H-03 | Tray: Start bridge | macos/BridgeStatusMenu.swift:18, macos/S4BridgeApp.swift:1685 | Enabled only when `!isBusy`; calls `start()` | Same gating on worker handle | 14,16 | host-start-stop | UNVERIFIED |
| H-04 | Tray: Stop bridge | macos/BridgeStatusMenu.swift:19, macos/S4BridgeApp.swift:844,1702 | Enabled only when `canStop` (`process != nil`, not stopping, screens not busy); SIGINT stop, `stopping` state | Same gating; cooperative stop via managed stdin/STOP | 14,16 | host-start-stop | UNVERIFIED |
| H-05 | Tray: Enable S4 screens toggle | macos/BridgeStatusMenu.swift:21-24, macos/S4BridgeApp.swift:1686 | Checkbox state mirrors `preferences.enabled`; disabled while `screens.busy` or `state == .stopping`; calls `setScreensEnabled(!enabled)` (BridgeController+Screens.swift:4) | Same toggle/gating; never restarts MIDI/LED/jog service | 16,18 | host-screens-toggle | UNVERIFIED |
| H-06 | Tray: Enable screens - stop NI driver (retry) | macos/BridgeStatusMenu.swift:25-28, macos/S4BridgeApp.swift:1690 | Visible only when `needsAttention`; enabled only while `state == .running && !busy`; calls `enableScreensByStoppingDriver()` (BridgeController+Screens.swift:24, requires running+full) | Visible/gated retry via Windows service handover; same running+full gating | 16,18 | host-screens-retry | UNVERIFIED |
| H-07 | Tray: Restart NI screen driver | macos/BridgeStatusMenu.swift:29-32, macos/S4BridgeApp.swift:1691 | Enabled when `!busy`, not stopping, and (screens disabled OR driver not running); `restartNIDriver()` asks stop-bridge confirmation when a child runs, else direct `restoreDriver` (BridgeController+Screens.swift:59) | Same gating; confirmation-before-stop; restore only app-stopped service | 16,18 | host-driver-restore | UNVERIFIED |
| H-08 | Tray: Settings… | macos/BridgeStatusMenu.swift:34, macos/S4BridgeApp.swift:1710 | Opens/activates 760×680 settings window | Opens settings window | 16 | host-settings-open | UNVERIFIED |
| H-09 | Tray: Install Djay mapping… | macos/BridgeStatusMenu.swift:35, macos/S4BridgeApp.swift:1705 | `NSApp.activate` then `installMapping()` save panel | Native save dialog export | 16 | host-mapping-install | UNVERIFIED |
| H-10 | Tray: Check controller | macos/BridgeStatusMenu.swift:36, macos/S4BridgeApp.swift:750,1703 | Disabled while busy; launches child `--list` as diagnostic (`isDiagnostic`, state `.checking`) | Same; `--list` via managed worker, no screens | 14,16 | host-check-controller | UNVERIFIED |
| H-11 | Tray: Quit | macos/BridgeStatusMenu.swift:38, macos/S4BridgeApp.swift:1704 + `prepareToQuit` macos/S4BridgeApp.swift:968 | `NSApp.terminate` → `prepareToQuit`: terminateNow only when idle and no pending owned restore; else terminateLater, stop child, restore owned driver, close motor capture, then reply | Same lifecycle: no quit while busy without stop+restore+close | 14,18 | host-quit-lifecycle | UNVERIFIED |
| H-12 | Menu-open driver refresh | macos/S4BridgeApp.swift:1693 | `menuWillOpen` calls `refreshDriverStatus()` (no-op when screens disabled; ScreenSupport.swift:76) and rebuilds menu | Refresh on menu open; no-op when disabled | 16,18 | host-tray-status | UNVERIFIED |
| H-13 | Status icon reflects state + screen attention | macos/S4BridgeApp.swift:1660,1681 | Green waveform when running, red when `needsAttention`, template otherwise; accessibility label with both states | Equivalent tray icon/label semantics | 16 | host-tray-status | UNVERIFIED |
| H-14 | Settings header Start/Stop + state | macos/S4BridgeApp.swift:1271 | Start disabled when busy or `requiresReset`; Stop disabled when `!canStop` | Same gating | 16 | host-start-stop | UNVERIFIED |
| H-15 | Bridge tab: profile picker (full/minimal/probe) | macos/S4BridgeApp.swift:1319, RunPreferences macos/S4BridgeApp.swift:24 | `BridgeProfile` full/minimal/probe; disabled while screens busy or capturing; screens/LED/jog args only for full (S4BridgeApp.swift:44) | Same three profiles; minimal/probe never enable screens | 14,16 | host-profiles | UNVERIFIED |
| H-16 | Bridge tab: fixed-duration toggle + seconds 1-604800 | macos/S4BridgeApp.swift:1326, validate macos/S4BridgeApp.swift:36 | `--seconds N` vs `--until-stopped`; out-of-range rejected with visible error | Same validation/args | 14,16 | host-duration | UNVERIFIED |
| H-17 | Diagnostics tab: raw HID reports flag | macos/S4BridgeApp.swift:1462 (`Raw HID reports`), args macos/S4BridgeApp.swift:49 | Appends `--raw` | Same flag | 14,16 | host-raw-flag | UNVERIFIED |
| H-18 | Diagnostics tab: log-level picker (warn/info/debug) | macos/S4BridgeApp.swift:1455, validate macos/S4BridgeApp.swift:39, env `RUST_LOG` macos/S4BridgeApp.swift:762 | Unsupported level rejected; passed as child env | Same validation/env | 14,16 | host-log-level | UNVERIFIED |
| H-19 | Start: save-first, fail-closed | macos/S4BridgeApp.swift:718 | `start()` saves; on save error sets `.failed` and reports without launching | Same save-first gate | 14,16 | host-start-stop | UNVERIFIED |
| H-20 | Start: screen handover before launch | macos/S4BridgeApp.swift:727 | `screens.prepare(full:consent:)`; nil (cancel) → restore owned + stopped; false → launch without screens; true → launch with `--screens` | Same prepare/launch matrix | 14,18 | host-screens-consent | UNVERIFIED |
| H-21 | Launch args reflect profile+screens+files | macos/S4BridgeApp.swift:44,520 | `arguments(ledURL:jogURL:tensionURL:screens:)`; `commandPreview` shown in Diagnostics | Same arg construction; preview shown | 14,17 | host-launch-args | UNVERIFIED |
| H-22 | Stop: SIGINT, restart flag, failure message | macos/S4BridgeApp.swift:843 | `kill(pid, SIGINT)`; failure → `.failed` + error, quit-reply false; `restart:true` from Save-and-restart (S4BridgeApp.swift:709) | Managed-STOP equivalent; same restart flag | 14,16 | host-start-stop | UNVERIFIED |
| H-23 | Exit classification + messages | macos/S4BridgeApp.swift:913 | status 0 or SIGINT-after-request = normal (`.stopped`, "Bridge stopped."); diagnostic → "Controller check complete…"; else `.failed` + status | Same classification/messages | 14,17 | host-exit-codes | UNVERIFIED |
| H-24 | Stale-launch guard: generation token + launchID | macos/S4BridgeApp.swift:753,780,811 + macos/ScreenSupport+Control.swift:7 | Pipe reader tags lines with launch `UUID`; `flushProcessOutput`/status/platter/receipt handlers ignore non-current tokens; stale screen lines dropped unless generation matches | Host generation/request IDs discard old-launch events; same guard | 14 | host-stale-launch | UNVERIFIED |
| H-25 | Save preferences after validation, atomic per file | macos/S4BridgeApp.swift:689 | Validates run/LED/jog/tension before writes, saves screens, then writes each JSON file atomically. This is not a transaction across all files; requiresReset blocks Save. | Same validation and reset gate; stage validation before atomic file replacements under LOCALAPPDATA. | 15,16 | host-save | UNVERIFIED |
| H-26 | Save and restart / Save preferences button | macos/S4BridgeApp.swift:1307 | Label switches on `isBusy`; disabled while stopping/checking/screens-busy; running → `stop(restart:true)` after save | Same label/disable/restart | 14,16 | host-save-restart | UNVERIFIED |
| H-27 | Invalid saved prefs → requiresReset gate | macos/S4BridgeApp.swift:560 | Load failure sets `requiresReset=true`; Start and Save blocked; editor reset required | Same fail-closed gate; visible error | 15,16 | host-corrupt-prefs | UNVERIFIED |
| H-28 | LED tab: full-profile notice for minimal/probe | macos/S4BridgeApp.swift:1360 | Notice that LED prefs apply to full profile only | Same notice | 16 | host-led-scope | UNVERIFIED |
| H-29 | LED editor: brightness/inactive/palette/quantize | macos/S4BridgeApp.swift:1366 | NumberRows + pickers with ranges/sets; unknown-key decode rejects (S4BridgeApp.swift:63) | Same controls/validation | 15,16 | host-led-edit | UNVERIFIED |
| H-30 | LED editor: jog rings + meters + tempo-center | macos/S4BridgeApp.swift:1379,1391,1404 | Chase 250-10000 ms, loop flash toggle+color, meter brightness/gamma/staleness, optional tempo tolerance; static limitation notes | Same controls/notes | 15,16 | host-led-edit | UNVERIFIED |
| H-31 | LED import JSON (merge over bundled defaults) | macos/S4BridgeApp.swift:1026 | `NSOpenPanel` .json; `LedPreferences.decode(data, defaults:)` merges over bundled base, rejects unknown keys; editor-only until Save | Same merge/reject semantics; separate filename | 15,16 | host-led-import | UNVERIFIED |
| H-32 | LED export JSON | macos/S4BridgeApp.swift:1043 | `NSSavePanel` default `s4-led-config.json`; writes encoded editor values | Same export | 16 | host-led-export | UNVERIFIED |
| H-33 | Invalid LED/palette values → visible error, no silent reset | macos/DESIGN.md:50, macos/S4BridgeApp.swift:683 | `report()` sets `errorMessage` + log line; saved config untouched | Same error policy | 15,16 | host-invalid-values | UNVERIFIED |
| H-34 | Reset preferences (editor only) | macos/S4BridgeApp.swift:1011 + confirm macos/S4BridgeApp.swift:1511 | Modal confirm; restores editor to bundled defaults incl. `RunPreferences()` and `screens.reset()` (preserves `restorePending`); "Save to apply" | Same confirm + editor-only + pending preserved | 15,16 | host-reset | UNVERIFIED |
| H-35 | Show preferences in Finder | macos/S4BridgeApp.swift:1036,1343 | Creates dir, `NSWorkspace.open(supportURL)`; failure → visible error | Open state dir in Explorer; same error | 16 | host-show-files | UNVERIFIED |
| H-36 | Jog tab: Motorized group + base-force slider | macos/JogSettingsView.swift:33 | Weak-Strong slider 0-25000 step 50 with readout; `Restore defaults` resets motorized group in editor only (S4BridgeApp.swift:1003, message "Save to apply") | Same slider + editor-only restore | 15,16 | host-motorized-defaults | UNVERIFIED |
| H-37 | Jog tab: Advanced checkbox reveals playback/touch + motor tuning without changing values | macos/JogSettingsView.swift:61, macos/DESIGN.md:44 | `@State showAdvanced=false`; hidden sections keep saved values | Same reveal; no value change | 16 | host-advanced-reveal | UNVERIFIED |
| H-38 | Jog: normal/TT Vinyl startup prefs (independent) | macos/JogSettingsView.swift:77, restore macos/S4BridgeApp.swift:331 | `vinylEnabledAtStartup` + `ttVinylEnabledAtStartup` persist independently per session choice | Same two prefs | 15,16 | host-vinyl-startup | UNVERIFIED |
| H-39 | Jog: wheel tension slider (-100..100) | macos/JogSettingsView.swift:106, TensionPreferences macos/S4BridgeApp.swift:386 | Integer-only schema (float tokens rejected); default -77; Save+restart applies | Same validation/default/apply | 15,16 | host-tension | UNVERIFIED |
| H-40 | Jog: backspin + assist controls | macos/JogSettingsView.swift:130 | Enable, threshold, extra speed, assist enable/strength with disabled-coupling | Same controls/coupling | 15,16 | host-backspin | UNVERIFIED |
| H-41 | Jog: scratch/pitch mapping values are export-only | macos/JogSettingsView.swift:165, macos/DESIGN.md:45 | scratchSpeed/Reaction/Direction, pitchBendSpeed/Reaction affect exported mapping only; bridge restart does not apply them; "Install Djay mapping…" in section | Same export-only semantics | 16 | host-jog-export-only | UNVERIFIED |
| H-42 | NEGATIVE PROBE: unsaved jog edits flow into mapping export | macos/S4BridgeApp.swift:1069 | `exportMapping` encodes the live **editor** `jog` (`try jog.encoded()`), not the saved file - unsaved slider changes ARE exported; bridge restart is not required and does not apply them | Windows exporter must read editor state, not the saved file | 16 | host-unsaved-jog-export | UNVERIFIED |
| H-43 | Install Djay mapping (save dialog + generate + open) | macos/S4BridgeApp.swift:1149 | Native save dialog selects destination. Generator uses a fresh temporary path and create-new; host atomically writes its result to the selected destination, including an existing file after save-panel confirmation, then opens it. isExporting guards concurrency. | Generate Windows mapping under separate name with LoopBe identity; preserve existing user mappings per plan contract 4. Do not silently copy the macOS destination replacement behavior. | 14,16 | host-mapping-install | UNVERIFIED |
| H-44 | Check controller diagnostic run | macos/S4BridgeApp.swift:750,913 | `--list` diagnostic; screens input null; completion message; screens `bridgeExited` only if `restoreAfterExit` | Same diagnostic path | 14,16 | host-check-controller | UNVERIFIED |
| H-45 | Screens tab: enable toggle saves immediately, worker-only | macos/ScreenSettingsView.swift:12, macos/DESIGN.md:36 | `setScreensEnabled` → `screens.setEnabled` persists now; toggles affect screen worker only, never restart MIDI/LED/jog | Same immediate-save + worker-only scope | 15,18 | host-screens-toggle | UNVERIFIED |
| H-46 | Screens tab: automation prefs save immediately | macos/ScreenSettingsView.swift:35, macos/BridgeController+Screens.swift:17 | `automaticallyStopNI` / `restoreNIOnStop` via `changePreferences` (atomic immediate write) | Same immediate persistence | 15,18 | host-screen-automation | UNVERIFIED |
| H-47 | Consent dialog: exact wording + 3 actions + remember | macos/ScreenPrompts.swift:14, macos/DESIGN.md:39 | "The Native Instruments screen driver software…"; Enable screens / Continue without screens / Cancel + "Do this automatically in future" checkbox → persists `automaticallyStopNI` | Same wording/actions/remember | 18 | host-screens-consent | UNVERIFIED |
| H-48 | Consent: remembered auto-stop skips dialog | macos/ScreenSupport.swift:89 | `automaticallyStopNI == true` → `.allow(remember:false)` without prompting | Same skip | 18 | host-remembered-consent | UNVERIFIED |
| H-49 | Consent: Cancel aborts launch cleanly | macos/ScreenSupport.swift:91, macos/S4BridgeApp.swift:731 | `.cancel` → state `.ready`, returns nil → bridge not started; owned-driver restoration is attempted and errors reported | Same abort; no launch | 14,18 | host-consent-cancel | UNVERIFIED |
| H-50 | Consent: Continue without screens | macos/ScreenSupport.swift:94 | `.withoutScreens` → state `.blocked`, returns false → bridge runs screenless | Same screenless run | 14,18 | host-consent-continue | UNVERIFIED |
| H-51 | Restoration ownership: record-before-stop | macos/ScreenSupport.swift:111 (stopDriver), macos/DESIGN.md:40 | `restorePending=true` persisted BEFORE stopping NI service; `restoreOwnedDriver` only when app stopped it | Record-before-stop; restore-only-owned | 18 | host-restore-ownership | UNVERIFIED |
| H-52 | NEGATIVE PROBE: pending-restore ownership across crash/quit | macos/ScreenSupport.swift:67 (reset preserves pending), macos/S4BridgeApp.swift:968 | Reset retains persisted restorePending; recreation retains the recovery record. Quit defers while busy or pending with restoreNIOnStop. Restore errors remain visible and pending; quit is not proof that restoration succeeded. | Persist app-owned pending recovery across reset/recreation; cooperative quit attempts owned restoration and reports failure. Do not clear pending or claim restored without success. | 18 | host-pending-restore | UNVERIFIED |
| H-53 | Blocking NI apps must close first | macos/ScreenSupport.swift:116 | `blockingApplications()` non-empty → visible error naming apps, no stop attempted | Same pre-stop check with Windows process names | 18 | host-ni-blockers | UNVERIFIED |
| H-54 | SIGKILL (not SIGTERM) for NIHardwareAgent; no login/registration changes | macos/DESIGN.md:40, macos/NIHardwareAgentNative.swift | Guarded kill of verified agent; never disable login registration or terminate NI applications | Windows equivalent: exact service/binary identity only; no broad service changes (plan contract 9) | 18 | host-driver-stop | UNVERIFIED |
| H-55 | Manual recovery always available | macos/ScreenSettingsView.swift:48, macos/DESIGN.md:37 | Explicit manual restore works even when disabled and unowned on macOS; running child prompts for stop before restore. | Keep a visible recovery workflow, but restore only a verified service this app stopped under plan contract 9. Unowned macOS recovery is a deliberate platform difference requiring task-12/18 review. | 18 | host-driver-restore | UNVERIFIED |
| H-56 | Retry generation: stale screen lines dropped | macos/ScreenSupport+Control.swift:7,37 | `S4_SCREEN_CONTROL <gen> <cmd>`; inbound `S4_SCREEN_STATUS` with mismatched generation ignored; `generation=0` reset on exit path (ScreenSupport.swift:153) | Same generation guard on Windows pipe protocol | 14,18 | host-screen-retry | UNVERIFIED |
| H-57 | Screen state machine + attention icon | macos/ScreenPreferences.swift:38 (off/ready/pending/active/blocked/failed/releasing), macos/BridgeStatusMenu.swift:26 | `needsAttention` = enabled && (blocked\|\|failed); red icon + retry row | Same states/attention | 18 | host-screen-states | UNVERIFIED |
| H-58 | Diagnostics tab: six areas (version, log detail, platter, capture, log, command) | macos/S4BridgeApp.swift:1449 | App version `S4BridgeReleaseVersion (build CFBundleVersion)`; log picker; live platter; capture controls; bounded log + copy; `commandPreview` | Same six areas; Windows version/build source | 17 | host-diagnostics | UNVERIFIED |
| H-59 | NEGATIVE PROBE: Diagnostics hidden-tab refresh suspension | macos/S4BridgeApp.swift:1292, macos/PlatterDiagnostics.swift:272, macos/Tests/SettingsVisibilityTests.swift | Off-tab diagnostics body is Color.clear; hidden/minimized/closed settings unmount it. accept retains state without publishing when inactive; active display coalesces refresh and publishes only changed presentations. | Suspend hidden presentation work while retaining incoming telemetry/logs; reactivation shows retained state. Do not suspend worker telemetry or capture. | 17 | host-hidden-refresh | UNVERIFIED |
| H-60 | Log bounds (60,000 chars) + incremental view | macos/S4BridgeApp.swift:429 (OutputBuffer), macos/S4BridgeApp.swift:443 (BridgeLog) | Both buffers trim to most recent 60,000; view applies insert/delete deltas, placeholder when empty | Same bound + incremental rendering | 17 | host-log-bounds | UNVERIFIED |
| H-61 | Copy log to clipboard | macos/S4BridgeApp.swift:1480 | `NSPasteboard` full `controller.log` | Same copy | 17 | host-copy-log | UNVERIFIED |
| H-62 | Motor capture gating (Debug + full profile) | macos/S4BridgeApp.swift:508,609,1475 | Start disabled unless `(process==nil ? prefs : active) == full+debug`; else visible error naming the requirement; header records launchID + `S4_MOTOR_CONFIG` receipt-or-pending + event list; stop appends `# Stopped` | Same gate + header/receipt semantics | 17 | host-capture-gating | UNVERIFIED |
| H-63 | Motor capture teardown on quit | macos/S4BridgeApp.swift:968,985 | `prepareToQuit`/`completed` close handle; reveal via `activateFileViewerSelecting` (S4BridgeApp.swift:679) | Close on quit; reveal in Explorer | 17 | host-capture-reveal | UNVERIFIED |
| H-64 | App launch failure shows alert, no silent exit | macos/S4BridgeApp.swift:1657 | Missing Resources / bad bundled defaults → `NSAlert` + terminate | Same fatal-with-message | 16 | host-launch-failure | UNVERIFIED |
| H-65 | State errors: launch failure, export failure, missing file | macos/S4BridgeApp.swift:800,1069,1149 | Child-spawn failure tears down + throws; export nonzero/missing-output → "Mapping export failed with status…"/"produced no file…"; mapping-saved-but-unopenable → explicit path message | Same distinct errors | 14,16,17 | host-state-errors | UNVERIFIED |
| H-66 | Concurrent-export guard | macos/S4BridgeApp.swift:1069 | `isExporting` reentry → "Mapping export already in progress." | Same guard | 16 | host-export-guard | UNVERIFIED |
| H-67 | Temp-dir hygiene for export | macos/S4BridgeApp.swift:1069 | Fresh `S4BridgeMapping-<uuid>` temp dir; deferred removal with error reporting | Same hygiene under `%LOCALAPPDATA%\Temp` | 16 | host-export-temp | UNVERIFIED |
| H-68 | TT response-curve editor (per-mode gains/shapes) | macos/TTResponseCurveView.swift | Braking/nudge gain + exponent editors with validation ranges (S4BridgeApp.swift:280) | Same editors/validation | 15,16 | host-tt-curves | UNVERIFIED |
| H-69 | Platter live diagnostics presentation (running vs stopped wheels) | macos/PlatterDiagnostics.swift:206 | `stoppedWheel` presentation with deck labels when not running | Same presentation | 17 | host-platter-display | UNVERIFIED |
| H-70 | Bundled defaults = shipped examples | macos/build-app.sh:71-74, macos/S4BridgeApp.swift:541, examples/{led,jog,tension}-config.json | `default-{led,jog,tension}-config.json` copied from `examples/`; `decode(x, defaults:x)` self-validates | Same defaults staged from same examples; self-decode must pass | 15 | host-bundled-defaults | UNVERIFIED |

## Additional source-backed workflows

| ID | Workflow | Source references | Current behavior | Expected Windows behavior | Delivery task | Named proving scenario | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| H-71 | Palette: deck colors | macos/S4BridgeApp.swift:1423-1426 | Four independent deck palette entries in the LED editor. | Same four entries; saved/exported with LEDs. | 15,16 | host-palette-decks | UNVERIFIED |
| H-72 | Palette: hotcue color mode | macos/S4BridgeApp.swift:1426-1433 | Choose genuine Djay tokens or fixed-slot colors; unknown tokens dark. | Same mode and unknown-token rule. | 15,16 | host-palette-hotcue-mode | UNVERIFIED |
| H-73 | Palette: white-token override | macos/S4BridgeApp.swift:1434 | Djay white token has an independent palette value. | Same editor and token mapping. | 15,16 | host-palette-white | UNVERIFIED |
| H-74 | Palette: token correspondence | macos/S4BridgeApp.swift:1436 | Eight Djay color-token entries. Full palette is not physically calibrated. | Same eight entries and calibration warning. | 15,16 | host-palette-tokens | UNVERIFIED |
| H-75 | Palette: fixed hotcue slots | macos/S4BridgeApp.swift:1437 | Eight optional fixed-slot colors. | Same editor values and mode dependency. | 15,16 | host-palette-fixed | UNVERIFIED |
| H-76 | Palette: stem colors | macos/S4BridgeApp.swift:1438 | Four stem colors. | Same four editor entries. | 15,16 | host-palette-stems | UNVERIFIED |
| H-77 | Palette: mute brightness and engaged color | macos/S4BridgeApp.swift:1439-1444 | Bright-means-muted toggle and engaged color; mute switch is not effective audibility under solo. | Same controls and explanation. | 15,16 | host-palette-mute | UNVERIFIED |
| H-78 | Screens: explicit Stop NI driver | macos/ScreenSettingsView.swift:60-63; macos/BridgeController+Screens.swift:40-59 | Busy-gated action: retry/consent if full bridge requests inactive screens, otherwise stopDriver; errors visible. | Keep explicit scoped handover action, subject to verified Windows identity and task-12 access contract; no macOS kill policy copied. | 18 | host-manual-driver-stop | UNVERIFIED |
| H-79 | Jog: advanced playback and motor-response fields | macos/JogSettingsView.swift:70-104; macos/S4BridgeApp.swift:280-331 | Target RPM, two startup Vinyl settings, scratch threshold, maximum/base force, speed compensation, proportional/integral correction, rise/startup ramp and overspeed settings remain editor values until save/restart. | Same controls, strict values and application timing; no control-law redesign. | 15,16 | host-advanced-motor-fields | UNVERIFIED |
| H-80 | All six settings tabs remain reachable | macos/S4BridgeApp.swift:1284-1299 | Bridge, Screens, Jog, LEDs, Palette and Diagnostics; only selected Diagnostics mounts its body. | Six areas with the actions in H-14 through H-79, accessible from tray Settings. | 16,17,18 | host-six-tabs | UNVERIFIED |
| R-32 | Probe profile distinct from minimal MIDI | src/main.rs:43-158; src/input.rs:66-97; macos/S4BridgeApp.swift:24-51 | No --midi selects HID probe; raw/duration options apply; original interrupt path uses 100 ms timeout. No MIDI profile or full LED/jog/screens configuration is enabled. | Preserve probe as a separate host profile and CLI workflow; managed stop added by task 5. | 5,9,14,16 | host-probe-profile | UNVERIFIED |
| R-33 | Portable distribution and resource integrity | macos/build-app.sh:60-74; .omo/plans/windows-bridge.md:340-351 | macOS bundles Rust worker, mapping, examples-derived defaults and screen-font notice; current Windows probes are not the product. | Self-contained Windows 11 x64 host, worker, mapping, defaults, font/dependency notices in portable ZIP; no DJ-laptop development tools or system .NET. Preserve prior build artifacts. | 2,19,20 | build-smoke-portable-resources | UNVERIFIED |

## Documented limitations

| ID | Workflow | Source references | Current behavior | Expected Windows behavior | Delivery task | Named proving scenario | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| L-01 | Startup sync exclusions and silence | docs/user-guide.md:83-105; src/full/mod.rs:1383-1446 | Volume/EQ and assignment/curve synchronize after feedback; gain/filter/tempo/crossfader position excluded. Last confirmed playing can persist after Djay closes. | Same exclusions and stop-TT/bridge-before-quitting-Djay warning. | 8,10,22 | djay-feedback-startup-and-silence | UNVERIFIED |
| L-02 | Preview resynchronization | docs/user-guide.md:152-173; src/full/catalog.rs:183-245 | Held Preview scrubs with BROWSER, wins over Shift; preexisting preview or mouse edits can desynchronize, requiring onscreen stop. | Same hold/seek precedence and documented recovery. | 10,22 | controls-preview-hold-recovery | UNVERIFIED |
| L-03 | Loop and jump information limits | docs/user-guide.md:189-210,667; src/full/mod.rs:1420-1435 | Inactive loop length is not displayed; unknown jump size cannot restore after Shift+MOVE; no manual loop-in/out binding; Djay may combine rapid jumps. | Keep independent lengths and these limits. | 8,10,22 | controls-jump-unknown-and-rapid | UNVERIFIED |
| L-04 | Manual FX main ON limitation | docs/user-guide.md:340-342; src/full/catalog.rs:252-284 | Main strip ON duplicates slot 1, not reliable whole-bank bypass. | Retain behavior and tell users to use slot buttons or Djay UI. | 10,22 | controls-fx-main-on | UNVERIFIED |
| L-05 | Sampler banks, MUTE alias and REC | docs/user-guide.md:364-385; src/full/catalog.rs:238,484-540 | A/C share bank 1, B/D bank 2; MUTE selects STEMS; Neural Mix depends on Djay/track. REC uses Record Sample with a known bug, not set recording. | Same bank ownership and mode semantics; retain REC limitation and onscreen-recording guidance. | 10,22 | controls-pads-banks-rec | UNVERIFIED |
| L-06 | External audio and DVS remain manual | docs/user-guide.md:245-295; src/main.rs:244-280 | DVS enable/input assignment manual; unknown EXT mode does nothing; Shift+EXT hardware input type; full phono/four-input mixer behavior not all verified. | No audio routing or audio-interface claims. Preserve manual setup and require Windows audio coexistence acceptance. | 3,9,21,22 | audio-ext-dvs-coexistence | UNVERIFIED |
| L-07 | Screen content and unknown key | docs/user-guide.md:589-600; src/full/screens.rs:76-140 | HOTCUE/SAMPLES JUMP/LOOP/KEY/RANGE; STEMS/MUTE legend; no title/artwork/BPM/time/waveform. KEY -- means no key, ? missing/unrecognized. | Same pages and unknown states; no invented track metadata. | 8,13,18,22 | screens-pages-and-unknown-key | UNVERIFIED |
| L-08 | Rings and end-warning limits | macos/S4BridgeApp.swift:1374-1382; src/full/ring_phase.rs:25-69; src/full/output.rs:50-52 | Normal playback animation with scratch continuity is not track position; TT follows physical rotation. Firmware loop flash cadence; no verified end warning. | Same presentation and limits; writes alone do not prove physical ring behavior. | 10,11,22 | led-rings-loop-end-warning | UNVERIFIED |
| L-09 | Palette, meters and tempo-center limits | macos/S4BridgeApp.swift:1384-1444; src/full/output.rs:99-114 | Master meters hardware-driven; optional tempo center needs calibration and is not pickup; full token palette uncalibrated, unknown cues dark; mute light not audibility. | Same controls/limits; physical LED calibration remains open. | 10,16,22 | led-calibration-and-unknown | UNVERIFIED |
| L-10 | Diagnostic numbers and LED export scope | macos/PlatterDiagnostics.swift:339-348; docs/user-guide.md:624-628 | Bend counts are outgoing MIDI, not audible speed; LED/Palette JSON export is not a complete settings or mapping backup. | Preserve labels and limited export scope. | 16,17,22 | host-diagnostic-units-export-scope | UNVERIFIED |
| L-11 | Disconnect and mapping reconnect recovery | docs/user-guide.md:635-648; src/full/mod.rs:1440-1446 | USB reconnect needs explicit Start; mapping reconnect needs bridge restart for feedback-dependent startup state. | Same recovery policy; no new automatic reconnect. | 5,14,20,22 | lifecycle-reconnect-explicit-start | UNVERIFIED |
| L-12 | Intentional controller differences | docs/user-guide.md:656-668; src/full/catalog.rs:124-181; src/full/mod.rs:2255-2292 | Shift+SYNC global range; independent REV/Slip; FX targets not audio buses; Filter then ON selects EQ mode, normal ON arms Instant FX; held effects retain original owners. | Same workflow differences; preserve all distinct full-profile routes. | 10,21,22 | controls-intentional-differences | UNVERIFIED |

## Overlap resolution and acceptance boundaries

| Shared workflow | Runtime owner | Host owner | Retained distinction |
| --- | --- | --- | --- |
| Profiles and launch | R-03,R-26,R-32 | H-15 through H-24,H-44 | Runtime flags/transport versus editor validation, launch tokens and child lifecycle |
| Mapping | R-12,R-13 | H-09,H-41 through H-43,H-65 through H-67 | Native generator create-new versus live editor export and destination installation |
| Preferences | R-19,R-24,R-27 | H-25 through H-40,H-68,H-70 through H-77,H-79 | Rust schema/defaults versus editor/saved/launched values and immediate screen saves |
| Screens | R-22,R-23 | H-05 through H-07,H-20,H-45 through H-57,H-78 | Pages/USB worker versus consent/service recovery ownership |
| Shutdown and fault handling | R-09,R-25,R-31 | H-04,H-11,H-22 through H-24,H-51,H-52,H-63,H-65 | Neutral/release order versus process exit, persisted pending restore and capture close |
| Diagnostics | R-26 | H-17,H-18,H-58 through H-63,H-69 | Existing runtime telemetry versus display visibility, bounded logs and capture |
| Physical output and editor presentation | R-17,R-19 through R-21 | H-28 through H-33,H-71 through H-77 | Genuine state and HID output versus palette/settings; no UI check proves hardware |

H-10/H-44 retain one controller-check scenario with two entry/lifecycle views; H-09/H-43 do the same for mapping installation. H-51/H-52 distinguish record-before-stop from reset/recreation/quit. R-08's jump restoration is not NI service restoration. These overlaps are intentional, not duplicate acceptance credit.

The native host permits explicit manual recovery without prior ownership (H-55), while the Windows plan requires restore-only-owned service transitions. Windows also preserves user mappings under separate names, whereas the current macOS host can atomically replace the user-confirmed destination (H-43). These platform differences remain explicit task-12/18 and task-16 review obligations, not permission to copy broader macOS actions.

At this November 4 baseline, all Windows rows remained UNVERIFIED until their proving scenarios produced current evidence. Source inspection, 510 Rust tests, 80 Swift tests and successful command exits alone did not establish Windows MIDI delivery, Djay feedback completeness, native tray behavior, screen-driver access, motor timing, audio coexistence or physical parity.
