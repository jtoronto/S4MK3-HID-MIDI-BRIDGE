# Serato mode: investigation and implementation plan

November 3, 2026. Source baseline: `34cf873b873865863c29840e9a5185f5278e926f`.
Worktree: `.slim/worktrees/serato-mode`; branch: `omos/serato-mode`.

## Current status: deferred

The Serato investigation is closed for now. See the
[November 3 results and deferred-work record](serato-investigation-results-20261103.md)
for the installed-Serato probes, recovered native profile, USB assembly gate
and USB-emulation feasibility assessment. No production Serato mode was built.
The proposal below is retained for context, not scheduled implementation.

## Original recommendation

Build separate **Djay** and **Serato DJ Pro (experimental)** software modes in
the same app. Keep **Full**, **Minimal**, and **HID probe** as separate technical
profiles, not alternative names for DJ software. Existing installations and
CLI commands must continue to default to Djay.

Use a sibling Serato backend rather than sending the existing Djay translator's
output through a new XML file. Share verified hardware decoding and low-level
MIDI/report helpers, not Djay action names, startup rules or software state.
Do not undertake a wholesale plugin framework or rename/move the entire Djay
backend before establishing the second backend.

The original direction was implementation before Serato installation/testing.
The later minimal-probe approach and eventual deferral superseded that
sequence. Current-version XML loading, return routing, audio and physical
control behavior remain unclosed acceptance gates.

This document began as a research and planning deliverable before Serato was
installed. Later probes and checks are recorded separately in the results
record above. No product behavior was changed and no Serato backend was
implemented.

## What the sources establish

Evidence classes below deliberately distinguish:

- **Official:** Serato documents the workflow or feature, not necessarily its XML.
- **Example:** an original author's XML exists; S4/current-version behavior is
  still untested.
- **Unknown:** neither a usable target-version contract nor an acceptance result
  was established. Unknown does not mean impossible.

| Question | Finding | Evidence |
| --- | --- | --- |
| Generic MIDI controller input | Serato documents secondary-controller MIDI Learn and preset management. Enable a generic device with **Enable MIDI Device**. Hardware Remapping is a different setting for supported Serato hardware. | Official [S1], [S2] |
| Operation with only the S4 | Not established. Practice mode offers two decks without compatible hardware, but its description does not establish generic MIDI mapping, four decks or S4 headphone routing. The published hardware list reviewed does not list the S4 MK3. | Official [S3], [S4], [S5]; unresolved setup question |
| S4 audio output | The external-soundcard master-output option is documented, but the article says compatible hardware must be connected. This does not establish standalone S4 use or independent master/headphone buses. | Official [S6] |
| Relative encoders | Five MIDI data modes are documented, including binary offset. Existing bridge relative bytes are centered on 64; binary offset is the first candidate, not a tested sensitivity calibration. | Official [S7]; `src/full/mod.rs:1604` |
| Basic output lighting | Supported functions can return lighting state. This does not prove which virtual destination Serato selects, initial-state delivery, meters or a complete playback-state contract. | Official [S2]; example [X1] |
| Stems | Third-party Stems MIDI Learn is documented alongside primary hardware. XML examples expose `codfather_*` actions, but pad layout and slot semantics need validation. | Official [S8]; examples [X1], [X2] |
| Jog/scratch | Jogwheels/platters are excluded from official remapping. An original 2018 experiment demonstrates legacy `platter_move` and `platter_touch` XML with an XDJ-700. Current S4 compatibility is unknown. | Official [S1]; original experiment [X4] |
| High-resolution tempo | No generic paired 14-bit contract was established. A saved preset contains `pitch_slider`, but uses relative input, not proof of an absolute or 14-bit S4 fader mapping. | Example [X2]; unknown |
| DVS/THRU | Deck modes depend on compatible DVS hardware; software THRU is hardware-dependent. An XML action or UI change cannot establish working input audio. | Official [S9], [S10] |
| Meters, key, range, position and screens | No verified generic MIDI contract for the complete data needed by this bridge was established. Do not claim universally impossible or emulate software truth from button presses. | Limits of reviewed sources; current requirements in `src/full/feedback/info.rs` and `src/full/screens.rs` |

The largest product uncertainty is therefore not XML serialization. It is
whether the eventual Serato setup permits useful MIDI control and the audio
workflow the user wants without an additional supported primary device.
Do not require purchasing one now; test S4-only and supported-primary scenarios
separately later.

### Corrections and additions to the supplied research

The corrected October 30 review in
[the supplied research document](research-%20Serato%20DJ%20Pro%20XML%20Mapping.md)
was the starting point and is preserved with this investigation. The primary
sources used here are listed below. Its preserved Gemini draft is not an
action specification.

New direct inspection found a complete Push preset declaring app version
`3.3.5.29`, with `pitch_slider`, `load_track`, `load_track_feedback`,
`cue_point`, `cue_point_set`, sampler, loop and Stems actions [X2].
This improves the evidence for those tokens beyond the small wiki index.
It does **not** validate them on this S4 or whatever version is installed later.
In particular, `pitch_slider` is no longer merely a draft-suggested name, but
its correct absolute-fader configuration remains unproven.

Both the wiki Play example and complete saved presets nest feedback inside
`userio event="output"` under a control. Do not generate the Gemini draft's
invented universal top-level output schema. Actual samples use **`behaviour`**,
not `behavior` [X1], [X2], [X3].

Observed wrapper values differ: a complete S9 preset uses `app=" 2.1.1.25"`,
and the Push preset uses `app=" 3.3.5.29"` [X2], [X3]. The original jog author
also describes a bare `midi` root [X4]. Generate a documented experimental
wrapper without claiming an installed version; do not fabricate a current
Serato version string. Compare with a locally saved seed after installation.

The original jog article's text says 1536 while its code uses 1546. More
consequentially, its movement example uses legacy `deck_id="1"` alongside
touch using modern `deck_id="0"` [X4]. Do not globally rewrite all jog deck IDs
under the assumption every XML element has the same indexing convention.
Preserve separate legacy-movement and modern-touch target conventions in the
experimental generator and verify left/right isolation before judging feel.

The original article reports MIDI-channel collisions between devices [X4].
Distinct virtual endpoint names are useful isolation, not proof of isolation
inside Serato. Test alongside any primary device; do not assume the existing
channel allocation is free in a multi-controller setup.

## Coupling assessment against current source

References below are to the baseline revision, not future line numbers.

| Surface | Existing coupling | Recommended boundary |
| --- | --- | --- |
| CLI and HID selection | `src/main.rs:30-34,119-195,204-280` distinguishes minimal/full, selects a corrected descriptor for full and exports only Djay mappings. | Add software selection independently of profile. Validate the combination before HID enumeration. |
| Physical input | `src/input.rs:24-146` supplies full snapshots/stale-interrupt rejection. `src/full/raw.rs:25-145` parses the full physical controls; `Control` is private in `src/full/mod.rs:49-68`. | Extract the full physical parser and event type only as needed; preserve current descriptor corrections and startup held-control suppression. |
| Address/action catalog | `src/full/catalog.rs:63-74,123-223` combines physical name/address with Djay key paths, pickup and direction. | Separate Serato address/action definitions. Existing numeric addresses may be reused deliberately, but Djay targets are not an intermediate language. |
| Full runtime | `src/full/mod.rs:242-309,333-360` unconditionally creates Djay feedback and source endpoints, then synchronizes software state during reports and idle service. | Select a complete backend at startup, not just a mapping exporter. Keep the Djay object unchanged where possible. |
| Ownership and routing | `src/full/mod.rs:639-732,1192-1297` orders releases before selector changes and retains emitted addresses until release/Stop. | Preserve these invariants for Serato, including multiple owners of the same MIDI address. Extract a small ownership helper only if both paths actually need it. |
| MOVE/LOOP | `src/full/mod.rs:780-830,1422-1435` uses feedback revisions and Djay jump-size restore pulses; `mapping.rs:186-195` generates their targets. | Serato-specific encoder behavior; never send Djay restore pulses accidentally. Ordinary mapping cannot recreate this whole workflow. |
| Browse/preview | `catalog.rs:175-223`; `mod.rs:667-672,833-860,1234-1272` implement Djay preview toggles, seek pulses and source navigation. | Define Serato load/browse separately. Do not transfer toggle-on-release preview logic to an arbitrary Serato action. |
| Mixer/startup | `mod.rs:560-572,1383-1444,1551-1578` gates mixer and front-panel synchronization on Djay playback feedback. | A separate Serato readiness policy is essential: reusing the default full translator can leave ordinary mixer controls gated indefinitely. |
| FX and Neural Mix | `catalog.rs:247` onward; `mod.rs:1045-1189,1411-1421` tie targeting, pickup, browse and EQ-mode selection to Djay state. | Independently map Serato FX banks/slots. Neural Mix EQ is not automatically Serato Stems EQ. |
| Feedback | `src/full/feedback.rs:28-193,245-294` interprets Djay addresses; `feedback/info.rs:50-94` decodes Djay numeric info. | Separate Serato decoder. Share MIDI framing/queue mechanics only, never address interpretation. |
| LEDs and meters | `src/full/leds.rs:269-320` consumes local plus software state; `src/full/output.rs:34-109` owns S4 report initialization/pacing. | Reuse S4 palette/report writing selectively. Separate Serato feedback-to-light rules; absent state stays unknown/dark. |
| Jog and motor | `mod.rs:1299-1338,1383-1410` and `platter_gesture.rs` depend on Djay scratch/bend actions and loaded/playing/Internal/THRU. | Reuse physical measurement, not Djay calibration or playback authorization. No automatic Serato TT authorization. |
| Screens | `src/full/screens.rs:75-153` adapts Djay state to pages. | Do not start this adapter in Serato mode. Later local-only or feedback-backed pages need an explicit adapter. |
| Native preferences | `macos/S4BridgeApp.swift:30-53,486-560,689-705` persists run, LED, jog and tension settings. | Persist software with backward-compatible decoding. Keep existing Djay files and tuning intact. |
| Native lifecycle | `S4BridgeApp.swift:709-749,843-965` stops via SIGINT and restarts only after completion/screen-driver handling. | Use this lifecycle for mode changes. Snapshot active software at launch; pending settings are not the running child's identity. |
| Mapping install | `S4BridgeApp.swift:1062-1166` hard-codes Djay arguments, filename, directory and open-with-default-app behavior. | Keep separate exporters/installers; Serato XML is saved then loaded through Serato's MIDI settings, not opened as a Djay document. |
| Other app entry points | `BridgeStatusMenu.swift:4-37`, `JogSettingsView.swift:157-179`, `S4BridgeApp.swift:1290-1440` expose Djay-specific labels/actions. | Audit menu, main mapping section and inline Jog installer together; hide or explain unsupported settings. |
| Bundle | `macos/build-app.sh:57-90,105-119` stages defaults and generates only the Djay artifact. `macos/Package.swift:10-19` lists Swift sources explicitly. | Package both artifacts using the same bundled bridge; add new Swift files to SwiftPM as well as the app build. |

The bridge is well separated from USB audio, but its **full profile is a Djay
backend**, not currently a neutral control translator. The important separation
is physical events versus software semantics, not merely plist versus XML.

## Proposed implementation contract

### Selection, lifecycle and compatibility

- Add `DjSoftware { Djay, Serato }` in a small `src/software.rs`, exposed as
  `--software djay|serato`. Omission means Djay.
- Keep bare `--midi` minimal and `--midi full` Djay-compatible. Serato uses
  `--software serato --midi full`. Reject explicit Serato with minimal/probe
  or Djay-only diagnostic/export combinations before opening devices.
- Keep `--generate-mapping PATH` working for existing callers; select format
  with `--software`. Djay produces its current plist; Serato produces XML.
  Both must remain hardware-free and refuse accidental overwrite.
- Persist software in `RunPreferences`. A missing key in old JSON means Djay;
  Swift synthesized Codable defaults alone do not migrate a new required key.
  Unknown/corrupt values follow the existing preserve-and-report policy.
- Software selection is edited while stopped. When running, retain the current
  selection until an explicit Save and restart operation; disable direct
  selection while starting/stopping. Do not hot-swap endpoints or route held
  notes to a different backend.
- Track active software separately from edited preferences. Restart must
  release the old child's MIDI ownership and finish its existing cleanup
  before starting the selected mode; failed stop means no replacement child.
- Keep Djay source/destination names unchanged. Use a distinct matching Serato
  source/destination display name, proposed `S4 MK3 MIDI Serato`. Create the
  return destination before publishing the source, as the current Djay path
  does. Actual Serato return-port association remains an acceptance test.

### Runtime shape

Add `src/serato/{mod,catalog,mapping,feedback,runtime}.rs` for the second backend.
Dispatch to it from the software/profile selection in `main.rs`.
Start with a dedicated Serato runtime rather than broadening the Djay motor
loop into a generic framework. Both runtimes must use the existing verified
full input acquisition/corrected descriptor and competing-process check.

Extract `Control` and the raw physical parser into a narrowly named common
module, proposed `src/controller/{mod,raw}.rs`. Keep existing parser regressions
at that boundary. Extract MIDI address/relative/ownership helpers only where
needed to preserve actual shared behavior. Keep the Djay catalog, decoder,
translator, screen adapter and motor authorization in place.

For LEDs, share palette bytes and low-level S4 report-writing helpers where
useful. A small Serato renderer should consume Serato state plus local mode/
held state; it must not call the Djay decoder or pass fabricated Djay feedback
through the full Djay renderer. Do not duplicate the whole LED preferences
editor: reuse only clearly hardware-local fields, such as brightness/colors,
and expose fixed slot colors where actual Serato cue colors are unavailable.

The experimental Serato runtime does not enter `bridge_runtime::run` or
instantiate its motor engine. Do not pass saved tension, Djay jog/backspin
assistance or screen flags into it. Preserve those preferences for return to
Djay; disable their application in Serato with a visible explanation.
Manual jog input and motorized playback are separate capabilities.

### Mapping and feedback rules

Use one typed Serato binding inventory for the generator and runtime. Each
binding records physical control, endpoint/direction, one-based MIDI channel,
message type/number, action/attributes, translation semantics, source and
evidence status. The inventory must cover every physical control as mapped,
local, MIDI-Learn-only, experimental or unavailable; no silent omissions.

Reuse the existing deck channels 1-4 and shifted channels 9-12 where practical,
mixer channel 5 and control numbers documented in the supplied review.
Convert indexing exactly once. Keep local deck/Shift/pad modes in the bridge;
Serato need not remap its own hardware Shift or pad-mode selectors.
Deck C/D can have emitted addresses without claiming a two-deck Serato setup
exposes four software decks.

Native toggle actions such as Play may be sent without knowing prior state:
Serato performs the toggle. The bridge must not infer the resulting state,
queue a state-dependent restore, or claim that a lamp proves playback.
Held actions must preserve their original deck/address through all local mode
changes, repeated snapshots and Stop.

Initial generated mappings should include only source-backed candidates.
Unknown actions remain explicitly MIDI-Learn-only in the inventory, with a
documented emitted address; do not invent XML names to fill coverage.

| Function | Candidate and source | Offline implementation / later gate |
| --- | --- | --- |
| Play | `play`, press/toggle [X1], [X2] | Deck Notes 0, one toggle per press; release must not toggle again. Return state can drive Play LED only. |
| Cue and Sync | No sufficient complete contract established in reviewed examples | Emit distinct learnable Notes 1/2; omit guessed default XML bindings. Capture target-version Learn output later. |
| Reverse/Flux | `censor` exists in S9 example [X3], but is not proof of reverse-hold or Slip parity | Keep separate addresses; require a correct semantic contract before assigning. |
| Hotcues | `cue_point`, `cue_point_set` [X2]; `cue_point_delete` [X4] | Eight slots per intended deck; retain held explicit behavior and shifted delete, with output aliases interpreted as states, not generic RGB. |
| Sampler | `sample_player_slot_play`, `sample_player_bank_select` [X2] | Treat Sample Player deck/slot numbering as a separate namespace; do not blindly clone per-deck Djay sampler behavior. |
| Stems | `codfather_st/ac/in/fx` [X1], [X2] | Build a Serato-specific pad layout from sourced slots. No claim of Djay mute/solo or Neural Mix equivalence. |
| Load/Browse | `load_track`, `load_track_feedback`, library examples [X1], [X2] | Separate loading confirmation from track-present truth. Binary-offset encoder candidate needs direction/step testing. |
| Tempo | `pitch_slider` [X2] | Keep emitted CC0 as 7-bit, learnable. Example's relative pitch configuration does not validate absolute-fader XML; capture/verify before promotion. |
| Mixer EQ/upfaders | `deck_eq_lo/mid/hi`, `upfader` explicitly marked untested [X1] | Generate only as separately identified experimental candidates. Gain, PFL/filter and other missing targets remain Learn-only. |
| Crossfader | Historical original forum XML [X5] | Absolute CC20 candidate; software mixing and audible routing require independent acceptance. Curve/assignment switches remain unmapped unless sourced. |
| LOOP/MOVE | Specific-length loop actions exist [X2], [X6]; a full encoder/jump/active-loop-move contract does not | Provide learnable encoder/press addresses without Djay restore logic. Keep independent size-selection UI unavailable until its semantics are implemented, not a misleading local number. |
| FX | `effect_level_1_slot_enable` plus parameter/bank examples [X1], [X2] | Separate Serato bank/slot routing; no Djay pickup, effect-browser stepping or Instant FX multi-deck assumptions. |
| Jog | Legacy `platter_move`, modern `platter_touch` [X4] | Separate opt-in experimental mapping export and calibration. Keep out of the baseline XML until enabled; do not substitute touchstrip seeking or pause/resume-on-touch. |
| EXT, TT, GRID and unknown advanced functions | No accepted Serato contract | Consume unsupported selectors cleanly, keep indicators honest, emit no Djay commands or force. Hardware-local audio controls remain hardware-local. |

Baseline feedback can decode Play and source-backed pad states on the addresses
actually emitted by the generated Serato controls. It must have its own Note
On/Off and zero-velocity handling; a Play Note must never be mistaken for
Djay channel-7 playback/load authorization. Unknown input packets must not
create known state. Record which state arrived and when in Diagnostics.

Do not infer loaded/playing/Internal/THRU from a load button, Play request or
absence of output. Do not promote a Play LED mapping into TT motor permission.
Leave meters and software-dependent screen values unavailable pending real
contracts. No memory scraping, accessibility scraping, audio analysis,
licensed-device impersonation or new network telemetry is part of this plan.

### Native app and packaging

Add the software picker to Bridge settings and show active software in status/
Diagnostics. Route the menu installer, Bridge mapping section and inline Jog
installer consistently. Unsupported Serato controls should explain the missing
capability rather than retain Djay-specific actionable controls.

Keep Djay export/import behavior intact. Serato export should suggest
`S4 MK3 Bridge Serato.xml`, initially under the community-documented
`~/Music/_Serato_/MIDI/Xml` path [X3]. Use a save panel, preserve user filenames,
and require normal overwrite confirmation. Do not modify Auto_Save, mark live
files read-only or reset the user's Serato presets. Completion instructions:
enable the MIDI device, select the saved preset in Setup > MIDI and Load.
Treat this directory as community evidence until verified locally.

Bundle both mode artifacts, generated by the same release binary. Keep the
experimental jog preset separate from the baseline preset. Add separate
Serato configuration files only for actual independent settings; do not copy
every Djay preference schema or retune existing Djay files.

## Archived implementation sequence

These are dependency-ordered implementation increments from the original
proposal, not active tasks. The closeout commits preserve research and
diagnostics only; implementation is deferred.

1. **Introduce software selection and compatibility tests.**
   Touch `src/main.rs`, new `src/software.rs`, `RunPreferences` and the existing
   Swift/CLI test boundaries. Add missing-key migration, explicit unsupported
   combination errors and active-versus-pending mode identity.
   Acceptance: old arguments/preferences still mean Djay; Serato selection
   survives restart; invalid combinations fail without HID/MIDI access.

2. **Isolate physical input and implement the Serato emitter.**
   Depends on 1. Extract only the physical event/raw parser boundary, add
   `src/serato/mod.rs`, `catalog.rs`, `runtime.rs`, and preserve snapshot pacing.
   Implement deck/Shift/pad ownership, analog values, relative deltas and
   shutdown releases. Add learnable addresses for unresolved mappings.
   Acceptance: captured/raw fixtures traverse the production decoder and
   Serato translator; no Djay startup, restore, preview or authorization
   behavior leaks; the minimal and Djay regressions remain intact.

3. **Generate the experimental Serato mapping and coverage inventory.**
   Depends on 2. Add `src/serato/mapping.rs` and a focused
   `docs/serato-mapping-reference.md` as implementation artifacts.
   Record action provenance and unverified cases. Produce deterministic XML
   with parser-verified structure, channel/deck/slot attributes, exact
   `behaviour` spelling and no overwrite. Keep legacy jog experiments separate.
   Acceptance: runtime addresses and parsed generated entries agree; every
   physical control has a stated classification; no unknown token is presented
   as validated.

4. **Add isolated return state and truthful local lighting.**
   Depends on 2 and 3. Implement `src/serato/feedback.rs` and narrow S4 LED
   output reuse. The return endpoint exists before the source.
   Acceptance: actual generated output addresses decode through real MIDI
   framing; absent or unrelated data remains unknown; no motors/screens,
   meter guesses or Djay state can be authorized. A native toggle command
   remains usable when feedback is absent.

5. **Connect app workflow and portable packaging.**
   Depends on 1-4. Update `S4BridgeApp.swift`, `BridgeStatusMenu.swift`,
   `JogSettingsView.swift`, `Package.swift`, `build-app.sh`, the canonical user
   guide and focused macOS docs. Preserve the existing lifecycle and Djay
   preferences, expose capability limits and route each installer correctly.
   Acceptance: event-driven process tests prove old child exit before new
   launch; export cannot disrupt the running child or overwrite on failure;
   fresh/legacy preferences and both bundled maps work without Cargo on the
   destination laptop. Inspect the native UI in both modes.

6. **Complete offline verification and prepare physical acceptance.**
   Depends on 1-5. Run repository-required checks and isolated package checks
   below. Write a dated Serato validation record distinguishing emitted MIDI,
   XML structure, app behavior and untested software/hardware outcomes.
   Acceptance: a complete experimental build is ready before Serato install;
   no statement of performance usability or feature parity is made.

The later installed-Serato phase confirms or corrects candidate mappings and
closes workflow gaps, especially Cue/Sync, tempo, encoder/loop/jump behavior
and audio. This is tracked work, not something offline tests can prove.
If standalone S4 operation fails but a supported-primary setup works, describe
that limitation explicitly before treating the mode as generally supported.

## Verification plan

Use existing test locations, not prose/source-text assertions:

- `src/full/raw.rs` regressions move with the shared decoder, preserving
  captured-field behavior, stale rejection and startup held suppression.
- New Serato module tests exercise the real decoder/translator and generated
  binding inventory: press/release, duplicate snapshots, simultaneous Shift/
  deck changes, pad changes while held, overlapping address owners, relative
  chunking, analog endpoints and all-note cleanup.
- Generator tests parse XML and assert machine-consumed structure/addresses,
  action/deck/slot/translation values, uniqueness and deterministic ordering.
  They do not claim Serato accepted the file.
- CLI tests alongside `tests/screens_cli.rs` and `tests/tension_cli.rs` verify
  admission and hardware-free export, including invalid flags and preserved
  existing output files.
- Swift tests extend `macos/Tests/BridgeTests.swift:64-83,234-431,1315-1390`
  for old JSON migration, selected arguments, capability filtering, export
  failures and serialized mode switches. Subscribe to real child start/exit
  events before triggering actions; no fixed sleeps.
- CoreMIDI tests subscribe before sending and use bounded delivery timeouts.
  Capture traffic in each direction separately. Verify virtual ports disappear
  on Stop and the other software's packets do not affect selected-mode state.

Run from the worktree after implementation:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
swift test --package-path macos
bash -n macos/build-app.sh
plutil -lint macos/Info.plist
git diff --check
```

Build into an owned, previously unused output directory, for example:

```sh
S4_BRIDGE_OUTPUT_DIR="$PWD/dist/serato-offline-20261103" bash macos/build-app.sh
codesign --verify --deep --strict "dist/serato-offline-20261103/S4 MK3 Bridge.app"
unzip -tq "dist/serato-offline-20261103/S4 MK3 Bridge-arm64.zip"
hdiutil verify "dist/serato-offline-20261103/S4 MK3 Bridge-0.3.0-beta.1-arm64.dmg"
```

Use the actual versioned DMG filename if implementation changes the version.
Confirm the app and archive copies contain both generated modes and correct
defaults. Do not overwrite the user's existing `dist` artifacts, saved settings
or installed mapping to perform this check.

### Later Serato acceptance, in execution order

These actions are **not run now**. Before hardware activity, prepare the build
and capture setup, explain exactly what will happen, and wait until the user
confirms they are watching. No motor experiment is included.

1. Record installed Serato version/build, macOS version, license/trial state,
   connected hardware and audio outputs. Preserve existing mappings. Close
   other bridges and start the new app in Serato Full mode with screens,
   tension and motor assistance unavailable.
2. With the S4 as the only controller, inspect Setup > MIDI for
   `S4 MK3 MIDI Serato` and Enable MIDI Device. Record whether assignment/preset
   loading is available in Practice mode. If blocked, record it; repeat later
   with supported primary hardware without confusing the two outcomes.
3. Save a separate target-version seed preset from MIDI Learn. Press left
   Play once, then release. Capture `90 00 7F` and `80 00 00` on bridge output;
   verify only the intended deck toggles once. Capture the learned wrapper,
   action and release semantics before loading the generated preset.
4. Load a separate copy of the generated baseline XML. Repeat Play on every
   available deck, save/reload/restart Serato, and compare rewritten XML.
   A load/rewrite failure changes the candidate generator, not the user's map.
5. Enable Play output lighting for that mapping. Toggle Play using Serato's
   GUI, not the S4. Capture packets at the bridge return destination and verify
   deck-specific LED state. Restart/close Serato and document initial/stale
   feedback behavior; do not infer state from the last button press.
6. Verify master sound and independent PFL/headphones with labeled left/right
   test tracks. Record exact Serato/CoreAudio routing and any compatible
   primary device required. Master audio alone is not headphone success.
7. Check ordinary controls one family at a time: Cue/Sync, tempo endpoints/
   center/pickup, mixer EQ/gain/filter/faders, Browse/Load, pads/sampler/Stems,
   FX and LOOP/MOVE. Capture missing Learn contracts and compare GUI and audible
   behavior. Confirm four-deck availability separately from MIDI channel output.
8. Test held controls across Shift/deck/pad changes, simultaneous presses and
   Stop, then stop-and-switch Djay -> Serato -> Djay. Confirm old ports close,
   no stuck notes remain and saved Djay tuning/mappings are intact.
9. Only after basic control/audio acceptance, enable the separate legacy jog
   experiment. Verify deck indexing and touch release first, then direction,
   stationary touch, bend versus scratch and calibration. Test save/reload
   rewriting separately. Do not enable TT or pause/resume-on-touch workarounds.

Meters, full screens, THRU/DVS and playback-driven motors require separate
evidence and implementation before acceptance, not a checkbox on this run.

## Sources and reproducibility

Checked November 3, 2026. Official pages describe public behavior; community
repositories and articles are primary evidence of their authors' examples,
not official schema documentation or S4 acceptance.
The support article HTML requests returned HTTP 403; their actual article
bodies were retrieved successfully through Serato's public Help Center API,
using `/api/v2/help_center/en-us/articles/<article-id>.json`. The hardware list
was checked in its full HTML because its text extraction omitted the cards.

- [S1] [Serato: MIDI mapping with Serato DJ Pro](https://support.serato.com/hc/en-us/articles/209377487-MIDI-mapping-with-Serato-DJ-Pro).
- [S2] [Serato: MIDI settings and output lighting](https://support.serato.com/hc/en-us/articles/223446808-MIDI).
- [S3] [Serato: Practice mode](https://support.serato.com/hc/en-us/articles/360001274635-Practice-mode).
- [S4] [Serato: hardware and licensing](https://support.serato.com/hc/en-us/articles/202260274-Understanding-Serato-DJ-compatible-hardware-licensing).
- [S5] [Serato: hardware list](https://serato.com/dj/hardware).
- [S6] [Serato: laptop/external master output](https://support.serato.com/hc/en-us/articles/360001860415-Use-Laptop-Speakers-or-External-Soundcard-as-Master-Output-with-Serato-DJ-Pro).
- [S7] [Serato: MIDI data types](https://support.serato.com/hc/en-us/articles/225065768-MIDI-Data-Types).
- [S8] [Serato: MIDI-mapping Stems](https://support.serato.com/hc/en-us/articles/5732598890127-MIDI-mapping-Stems-with-Serato-DJ-Pro).
- [S9] [Serato: virtual deck play modes](https://support.serato.com/hc/en-us/articles/360004125475-Virtual-Deck-Play-Modes).
- [S10] [Serato: Through mode](https://support.serato.com/hc/en-us/articles/234760848-Through-Mode).
- [X1] [Kovarsk: SERATO-XML-WIKI](https://github.com/Kovarsk/SERATO-XML-WIKI/tree/ad8cf815edcf9e942c14104d11f5bc3a57bcd07c):
  action inventory, Play example and original example files. Revision
  `ad8cf815edcf9e942c14104d11f5bc3a57bcd07c`.
- [X2] [rpong: Ableton Push MK1 preset](https://gist.github.com/rpong/6e70dcfb7242199d99e59eadf076a375/e9ebd3d29a1d2c610d3df76875713a1ca842a761).
  Revision `e9ebd3d29a1d2c610d3df76875713a1ca842a761`.
- [X3] [marscanbueno: saved S9 preset and installation notes](https://github.com/marscanbueno/serato-dj-pro-midi-maps/tree/b6d1bfb767b063416de7390f853b170b2dbd5e70).
  Revision `b6d1bfb767b063416de7390f853b170b2dbd5e70`.
- [X4] [Teo Tormo: original jog/touchstrip/modifier experiment](https://djtechtools.com/2018/04/11/hacking-serato-djs-midi-mapping-jogwheels-touchstrips-and-modifiers/),
  April 11, 2018.
- [X5] [Serato forum: original user-posted output/crossfader XML](https://serato.com/forum/discussion/1066513).
  Historical discussion; not a current support commitment.
- [X6] [Themitchell: Launchpad preset](https://gist.github.com/Themitchell/2fbeea02d3937cbb631a).
  Historical `1.7.2.4139` wrapper and loop/roll/hotcue examples.

Searches for additional complete mixer/Cue/Sync contracts did not establish
the missing target-version semantics. Mixxx schema references in the supplied
draft do not establish Serato behavior. At the time of this initial plan,
Serato was not installed and its binary was not inspected. The later
[results record](serato-investigation-results-20261103.md) supersedes that
limitation for the examined Starlight discovery/assembly path.
