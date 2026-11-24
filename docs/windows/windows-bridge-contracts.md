# Windows bridge shared contracts

Contract revision 1, November 4, 2026. This freezes task 4 against the
322-file task-1 dirty-source aggregate
`dc3684558657869206dfba1f2a9ee35a8d20155bff0c2fae2db2d42ced77ddd6`.
The companion parity matrix remains authoritative for workflows and limitations.
The machine specifications live in `windows/contracts/`; audit receipts and
source-traced cases live in
`.omo/evidence/windows-bridge/20261104-01a1a4a4/task-4/`.

“Current” means inspected source, not Windows acceptance. The original task
numbers below identify the contract's baseline. Transport selection and managed
STOP/status are now implemented; the current Windows default is BLOOP, with
LoopBe retained as an explicit alternative.

## 1. Transport, CLI and identity

`src/midi_transport.rs` owns the concrete
`midir::MidiOutputConnection` and `MidiInputConnection<Decoder>` lifetimes.
Windows WinMM connects to exactly one input and exactly one output named
`S4 MK3 Bridge BLOOP`, or `LoopBe Internal MIDI` for the explicitly selected
legacy backend. Zero or ambiguous matches fail with direction and
candidate identities. Open the receive callback before exposing the send port
or reporting readiness. Close every acquired handle on partial failure.
Keep the 4,096-message bounded callback queue and sticky overflow failure.
Do not add a backend framework, implicit backend fallback, timed echo
suppression or automatic reconnect.

`--midi-backend core-midi|loopbe|bloop` defaults to `core-midi` on macOS and `bloop`
on Windows. The selected enum identifies both transport and layout.
`--generate-mapping PATH --midi-backend ...` is offline and accepts either
layout on either build host. A runtime backend unavailable on that target fails
before HID enumeration/open/initialization. Invalid enum syntax fails in CLI
parsing. Probe has no MIDI handles but still validates an explicit backend
before hardware access. List and offline replay/preview retain their no-open
behavior. No backend option creates a port during export.

The Windows host creates BLOOP endpoints through the supported
Windows.Devices.Midi2 0.99.92-preview.11 API package with Preview 13 transports.
`Core/BloopEndpoint.cs` keeps the ownership interface free of SDK types;
`Platform/BloopEndpointOwner.cs` contains the version-specific SDK and
transport-configuration boundary. Prepare writes the preassigned association
receipt before publication. The supervisor takes the lease before Create,
records the real worker identity, and retains it through confirmed worker exit,
output drain and exact removal. A pending SDK call is awaited, never abandoned
on a timeout. Cleanup failure retains the lease and receipt for Stop retry.
Probe, list and mapping export do not create an endpoint.

Current arguments come from `src/main.rs:28-160`: no `--midi` means probe;
bare `--midi` means minimal; explicit `--midi full` selects full.
Default duration is nonzero 30 seconds; `--until-stopped` conflicts with
`--seconds`. LED/tension/screens/screen-control require full, jog config
requires full, mapping export or motor replay. Preserve these constraints.
`--managed-stdin` is an optional runtime flag for all three profiles, conflicts
with list and offline exports/replay, and shares one reader with screen control.
`--screen-control-stdin` retains full-only behavior and EOF stop, even without
the new flag. Unmanaged stdin EOF retains current behavior.

Native names remain `S4 MK3 MIDI` (minimal), `S4 MK3 MIDI Full` (full,
both directions), and `S4 MK3 Bridge.djayMidiMapping`. Windows full mapping
is `S4 MK3 Bridge Windows.djayMidiMapping`, endpoint `S4 MK3 Bridge BLOOP`
(`LoopBe Internal MIDI` only for an explicit LoopBe export).
Do not install the full mapping as a claim of minimal-profile equivalence:
minimal faders have different channels. Generator output stays create-new-only
(`mapping.rs:21-36`); Windows host export/install must preserve user mappings
under existing names. Native `generate()` and macOS packaging defaults retain
their current semantic results for every jog fixture.

API basis: locked midir is **0.11.0**, checksum
`77c12e74a8604bc07f9a3fcf3d5889a81bc96ca6e07d6a114d63fc0371f3e5a4`
(`Cargo.lock:900-903`). Its primary API documentation describes callback-owned
input and connection lifetime; virtual ports are not provided on Windows.
References: `https://docs.rs/midir/0.11.0/midir/struct.MidiInput.html` and
`https://docs.rs/crate/midir/0.11.0`. Context7 discovery was unavailable in this
session; primary documentation was used instead. No tool, SDK or asset was
downloaded, and no new license/redistribution conclusion is made.

## 2. One explicit wire layout

All channels here are **one-based**; plist channels are channel minus one.
An address includes endpoint, direction, channel, message family and numeric
data number. Plist type 1 is Note; type 3 is CC. Both `0x8n` Note Off and
`0x9n` Note On, including velocity zero, occupy the same Note address for
collision analysis. CC and Note with the same number remain distinct.

`src/midi_layout.rs` is the single task-5/7 owner of effective output allocation,
declared ingress membership and normalization. The generator and ingress must
consume that definition; a separate permissive channel rewrite is forbidden.
First decode raw stream framing, then check wire membership and normalize the
decoded message, then enqueue/apply through the current real cache path.
Never rewrite arbitrary bytes before decoding running status.

| Family | Commands, unchanged | Windows feedback | Canonical cache |
| --- | --- | --- | --- |
| Deck A/B/C/D | 1/2/3/4; shifted 9/10/11/12 | 13/14/15/16 | 1/2/3/4 |
| Mixer | 5 | 8 | 5 |
| Deck selection | 6 | None | None |
| Dedicated telemetry/global | None | 7 | 7 |
| Minimal Play/Cue | 1/2 | None | None |
| Minimal faders | 1 | None | None |

The allocation **passes the source-domain audit** with no correction to its
channel partition. It remains **provisional for actual Djay output behavior
until task 8**. Native generation has 582 input entries, 168 embedded outputs
and 177 root outputs: 345 effective outputs. Resolving inheritance gives 192
native command/output address intersections, including 24 FX CC addresses.
Applying this table gives zero intersections, also including minimal faders,
Note Off and note-on-zero. There are no commands on 7, 8 or 13-16.
Exact expanded domains are in `windows/contracts/wire-layout.json`.

Effective embedded outputs inherit parent type/data/channel unless explicitly
overridden. Task 7 must override all three address fields where needed, resolve
the final effective address, and audit root AND embedded outputs. Changing
only root outputs leaves 168 direct echoes. Preserve `controlType`, pickup,
flipped, sensitivity/acceleration, numeric/boolean bounds and color semantics.
Hotcue output dictionaries currently omit numeric min/max deliberately.
Do not create shifted duplicate outputs or right-strip aliases for the
canonical left FX output addresses.

The allowlist is the effective generated output set, not all addresses in the
reserved channels. Current mixer consumption is **only Note 0-3**.
The earlier isolated channel-8 CC20 crossfader proof is not a production cache
family; reject it in this revision. New required declarations found in task 8
reopen the layout audit, generator/ingress tests and affected consumers.

## 3. All emission families

The catalog (`src/full/catalog.rs:1-570`) supplies action strings, type,
numeric address, pickup/flipped flags and optional shifted target. Expand
`{deck}` to 1-4 and `{bank}` to 1 for A/C, 2 for B/D.
`mod.rs:1575-1581` uses the shifted deck channel only when a binding has a
shifted action; pressing Shift alone never moves every address.

| Producer/family | Numeric addresses and routing |
| --- | --- |
| Deck transport/library/EXT/grid | Deck Note 0-15, 22-30, excluding 21; Note 20 EQ and 16-19 Instant FX are separate quick bindings. Specific base Note set is in the JSON. Shifted Cue 1, Sync 2, fixed MOVE 5/14, LOOP press 6, wheel touch 7, load/back 12, preview 23/24 and source 25/26 have explicit shifted mappings. |
| Tempo/LOOP/MOVE/browser/jog | Deck CC0 absolute tempo (Shift CC0 relative speed); CC1 LOOP length (Shift loop move); CC2 MOVE; CC7 MOVE-size; CC3 browser; CC4 bend, CC5 scratch, CC6 seek. Runtime jog movement uses the original owning deck's base channel even during Shift seek; shifted Note7 enters seek mode. |
| Left FX strip | Target deck Note32-35; shifted Note33-35; absolute CC32-37 (parameters then each wet/dry). |
| Right FX strip | Target deck Note36-39; shifted Note37-39; absolute CC40-45. Feedback for both strips is canonical Note32-35/CC32-37, not the right aliases. |
| Shift FX browsing | Target deck shifted-channel CC32-34, rotary-64. Both strips share the same target selectors; `mod.rs:757-778` steps after four position units. |
| HOTCUE pads | Deck Note40-47; Shift clears using same numbers on shifted channels. Preserve output color tokens 1-8, white 9, off 0. |
| SAMPLES pads | Deck Note48-55; Shift pauses on shifted channels. Shared banks A/C and B/D. |
| STEMS/MUTE pads | Deck Note56-59 mute, 60-63 solo; Shift56-59 exclusive solo. Shift60-63 stays on base channel because no shifted action exists. MUTE is a mode alias. |
| Instant FX and EQ | Notes16-19 to each independently armed deck (zero to four targets); EQ-selection pulse Note20 to selected deck, not mixer channel. Quantize Note15 fans out to all four decks. Held owners survive target edits. |
| Mixer | Channel5 CC0-3 gain, 4-15 EQ, 16-19 volumes, 20 crossfader, 21-24 filter, 25 curve. Notes0-3 PFL; assignment pulses Note80-91 = 80 + 3*(deck-1) + selector state0-2. |
| Deck selectors | Channel6 Note0-3 pulses; A/C left, B/D right. Selection never steals a held address. |
| Startup analog/switches | Seed report2 without immediate emission (`main.rs::initialize_full_analog`, `mod.rs:518-574`). First genuine channel7 playback output, including false, sets sticky `playback_received`; emit physical EQ/volume CC4-19 on channel5 and assignments/curve. Gain/filter/tempo/crossfader position are excluded. Delivered values suppress repeats; later physical changes still send, GUI edits are not repeatedly overwritten. |
| Shift+MOVE restoration | `mod.rs:799-833`: base-deck Note67 on/off selects one beat, base CC2 carries delta, then saved base Note64-73 on/off restores size if known. Wait for newer non-one jump selection/revision before accepting restored/external size; unknown cannot restore. |
| Preview press/release | `mod.rs:1230-1254`: first physical Preview holder sends Note13 pulse; last release sends another pulse. Both buttons share logical preview ownership. This is not an ordinary held release. |
| Wheel/TT/backspin | Touch Note7 and movement CC4/5/6; grid Note29/30 pulses. Motor scratch uses the same base-deck Note7 and CC5 (`platter_gesture.rs:111-162`); original touch/coast owners and deduplicated note ownership remain intact. TT force is not a MIDI family. |
| Stop/release | `mod.rs:1258-1297`: ordinary release goes to saved channel/number; final shared owner releases once. `release_all` ends preview through its pulse behavior, clears wheel state and sends sorted deduplicated Note Off for remaining held owners. Full Drop sends these before closing MIDI. |
| Minimal and probe | `midi.rs:49-99`: Play/Cue Note0/1 on channels1/2, faders CC16-20 all on channel1; positions round(value*127), reject nonfinite/outside0-1. Minimal Drop sends Note Off0/1 on both channels regardless of held set. Probe emits no MIDI; encoders/jog/deck changes are absent in minimal. |

Press/pulse is `90|channel0,number,127`; ordinary release is
`80|channel0,number,0`. Relative CC encodes `64 + chunk`, chunk clamped to
-63..63 and repeated until all steps consumed; zero steps emit nothing.
The JSON inventory includes mapping-declared controls that runtime rarely
emits (for example Shift fixed-MOVE notes), so audit membership is conservative.
No new MIDI numbers are introduced for startup, restore or shutdown.

## 4. Consumption and cache semantics

Current producer/consumer trace: `mapping.rs:198-354,406-475` ->
`feedback.rs:200-235` Decoder -> callback queue -> `FeedbackState::apply` ->
`mod.rs:1383-1445`/LEDs/screens/motor gating.

| Wire input | Normalization and current effect |
| --- | --- |
| Deck13-16 Note0-4,6,8,15-20,32-35,40-63 | Canonical deck Note, preserve velocity/color; Note Off forces zero even with release velocity64. Note-on-zero also means inactive. EQ Note20 additionally sets `eq_mode`. Instant FX16-19 keep most recently newly activated still-active slot; duplicate positive values do not reorder. |
| Deck13-16 CC21/22 | Canonical per-deck THRU/Internal booleans, nonzero true. Independent of loading; false THRU alone is not proof of Internal. |
| Deck13-16 CC32-37 | Six canonical FX values per deck. Right strip aliases CC40-45 are inputs only. Pickup needs genuine known values and crossing/within-one acquisition; existing pending value-window semantics remain. |
| Deck13-16 CC80-89 | Jump booleans; revision increments on every received jump message, including unchanged values. Exactly one true gives known selected size; zero/multiple true means unknown. |
| Deck13-16 CC90-111 | Active-loop booleans; exactly one true selects active loop size; no active loop gives no inactive-length estimate. Interval order is `feedback/info.rs:17-43`. |
| Deck13-16 CC112 | Key raw numeric value. Screen interpretation stays in `full/screens/key.rs`; do not invent text/title/time fields. |
| Mixer8 Note0-3 | Canonical channel5 PFL note state only. |
| Telemetry7 CC0-3 | Meter value and timestamp update on every accepted message, including repeated value. |
| Telemetry7 CC4-7 | Playing booleans; sets session-sticky `playback_received` even at zero. This is startup synchronization authority, not standalone motor authority. |
| Telemetry7 Note0-3 | Loaded: Note On nonzero true; Note Off or note-on-zero false. Unload clears that deck's active-loop flags, hotcue notes40-47, meter/timestamp and playing; keeps deck key, jump flags, mode and other note/FX caches. |
| Telemetry7 CC8 | Global tempo range: raw5/8/10/13/20/32/64/95 -> 4/6/8/10/16/25/50/75 percent. Other raw values produce unknown. |

Sampler48-55 mirrors deck index XOR2. All fields begin unknown (`None`),
except bookkeeping booleans/counters. `apply`'s changed boolean is not an
acceptance/readiness signal: duplicates can refresh a meter or jump revision.
The decoder retains status/data across calls, ignores realtime >=F8, resets
on system common/SysEx and emits only complete Note Off/On/CC messages.
Unsupported families/data, incomplete packets and unrelated channels must
never authorize state. Queue overflow remains an error before/after drain,
not a silently dropped synchronization event.

Playback force authorization is loaded=true AND playing=true AND
internal=true AND thru!=true, then current TT/grid/Shift/owner/freshness gates.
Unknown playing can permit a manual touch-scratch gesture under the existing
loaded/Internal condition, but never authorizes playback motor force.
Silence preserves last-confirmed state; this port does not add revocation.
Stop TT/the bridge before quitting Djay, as the existing guide warns.
Wire partition rejects command echoes regardless of elapsed time or previous
cache contents. A genuine stale channel7 packet on a shared bus cannot be
distinguished from a fresh same-address packet using MIDI1 bytes alone:
task 8 must prove fresh-session initialization and stale-queue isolation.
No timestamp heuristic or invented session ID is added to MIDI packets.

## 5. Managed child protocol v1

Current `screens_control.rs:7-55` parses only exact, single-space commands,
decimal ASCII u64 IDs, uppercase actions; invalid lines warn and do nothing.
EOF/read error requests the existing bounded stop channel. Preserve these
screen lines exactly:

```text
S4_SCREEN_CONTROL <id> ON|OFF|RETRY
S4_SCREEN_STATUS <id> active|inactive|failed|pending
S4_MOTOR_CONFIG {"jog":<current serialized Rust Config>}
S4_PLATTER_STATUS {"host_us":<integer>,"wheels":[<left>,<right>]}
```

Screen ID0 is startup status. Latest request generation supersedes older
requests; Off/Retry requires release even if later On supersedes it.
Opening waits for controller-initialized pages; inactive follows release,
active follows successful frame work; failures leave MIDI/jog running.
Worker-internal generation is separate from public request ID
(`screens/worker.rs:34-85`, `worker/service.rs:5-22,160-270`).

Motor receipt is stdout diagnostic confirmation of parsed jog settings,
printed before hardware open (`main.rs:185-201`), never readiness.
Each platter wheel has `deck` (left1/3, right2/4), nullable finite `rpm`,
finite `bend_counts_per_second`, `mode` in off/waiting/bend/scratch/
paused_scratch/manual_scratch/blocked/stale/seek/grid. `host_us` is elapsed
microseconds; frame cadence is at most10 Hz. Preserve field names, units and
optional RPM. Swift ignores `host_us` for presentation, validates wheels,
and marks displayed telemetry stale after0.5 s; none of these lines grants
force (`platter_telemetry.rs:10-38,96-151`, `PlatterDiagnostics.swift:52-123`).

Required new input is exactly:

```text
S4_BRIDGE_CONTROL <id> STOP
```

Managed EOF and valid STOP in **full, minimal and probe** converge on the normal
stop channel. Repeated STOP is idempotent; it cannot start/restart a bridge.
Screen commands remain full-only. A single reader dispatches both grammars;
it is detached rather than joined on an idle blocking read. Neither invalid
input nor stdin text resembling a status/receipt grants readiness or force.

New output is a UTF-8 line, `S4_BRIDGE_STATUS ` followed by one JSON object.
Required fields, with no inference from human log messages:

| Field | Exact type/meaning; producer -> consumer |
| --- | --- |
| `version` | Integer 1; Rust lifecycle emitter -> host v1 decoder. Reject unsupported versions. |
| `state` | `starting`, `ready`, `stopping`, `failed`; runtime -> supervisor. |
| `profile` | `full`, `minimal`, `probe`; resolved Args -> matching launch spec. |
| `backend` | `core-midi`, `loopbe`; resolved Args -> matching launch spec, including probe. |
| `request_id` | Null for startup/error/EOF/duration/signal, otherwise ASCII decimal u64 **string**, copied from STOP for `stopping`. A string avoids JSON precision loss. |
| `readiness` | Object with booleans `hid`, `controller_initialized`; nullable booleans `midi_input`, `midi_output`; `feedback` enum `unknown`, `received`, `not_applicable`. Runtime handle/init/cache milestones -> supervisor display. Null MIDI fields mean not required for this profile. |
| `error` | Null except failed; then object with nonempty string `stage` and `message`. Runtime causal failure -> host visible error. No motor-neutral assertion field. |

Readiness fields report completed milestones, not guessed software state.
Full ready requires HID, output/input handles and callback registration,
LED/controller initialization, startup analog acquisition and successful
snapshot-reader start; feedback can still be unknown. Minimal requires
HID/output, descriptor/parser/LED-builder setup and a service loop able to
receive stop; input is null. Probe requires that same non-MIDI setup and stop
service; both MIDI fields null. No new initialization writes are implied.
Only full can report feedback received, sourced from this launch's
`playback_received`; it does not imply all decks or motors are authorized.
When stopping/failed, host treats readiness as revoked regardless of retained
milestone booleans. Screens have their own pending/failed/active state and do
not hold the working control service hostage.

Child transitions: launch -> starting -> ready; startup failure -> failed;
any normal stop source before or after ready -> stopping -> process exit.
Errors during run/cleanup -> failed -> nonzero exit. Failed cannot return to
ready; start needs a new process. No child `stopped` event substitutes for
exit. Emit starting after argument/config validation and managed-reader setup;
early validation failure may produce only nonzero exit/stderr. Stop accepted
during initialization must prevent later ready. Emit stopping at most once;
first accepted STOP ID owns the event. Duplicate requests do not change it.

Host creates a fresh launch UUID before process start, tags every stdout,
stderr, EOF and exit callback with the UUID AND process identity, and only
mutates the matching current launch. This follows
`S4BridgeApp.swift:759-819,882-961`; launch UUID is transport-local and is
not added to the unchanged screen/motor/platter lines. Request counters start
at 1 per launch, never wrap/reuse within a launch, and reserve screen 0 startup.
Aggregate and screen ID namespaces are separate; the host correlates each.
Unknown fields may be ignored in a v1 status object, but missing/wrong-type
required fields, mismatched profile/backend/ID, invalid enum, or impossible
transition are rejected and logged. No rejected event changes readiness.

Normal stop waits for neutral attempt, reader shutdown, MIDI releases, screen
worker/port disposal and **process exit**. Existing ordering is
`bridge_runtime.rs:114-127`: service finish neutral before reader stop and
bridge Drop. A timeout is a failed/unconfirmed stop, never neutral or stopped.
Freeze a 10 s host stop deadline and 5 s screen-release deadline (the latter is
current `ScreenSupport+Control.swift:45-64`). Keep process identity and
pending restoration after timeout; no automatic kill/service recovery.
Restart waits for successful prior exit/cleanup, not merely stopping status.
This new lifecycle is a task-5/14 obligation, not current SIGINT host behavior.

## 6. Configuration and persisted user data

Use the **existing** Rust schemas/default implementations and shipped examples,
not a second runtime defaults inventory in the host. The field index
`windows/contracts/config-fields.json` enumerates every accepted key, type,
range, source/default owner and legacy handling. Values come from
`examples/{led,jog,tension}-config.json` staged as `default-*-config.json`,
with Rust `LedConfig::default`, `jog::DEFAULT`, `tension::Config::default` as
the runtime owners. LED/jog partial objects use Rust defaults. Tension has an
intentional distinction: no runtime file/empty object gives neutral 0;
the app's bundled saved selection is -77. Do not replace one with the other.

LED keys are snake_case with exact arrays/enums; unknown fields rejected.
Jog accepts the retired `platter_position_sensitivity` numeric/null field,
ignores it, and never serializes it. Optional brake/nudge gains fall back to
2*shared/shared gain. If an override supplies `tt_pitch_bend_gain` without
`tt_nudge_gain`, Rust load clears the default nudge override; the Swift merge
removes absent optional gain overrides before merging. Preserve this migration.
Decode then run validation at use/export boundaries; `jog::Config::load`
alone is not the complete range gate. Finite numbers only; strict integer
tokens must reject booleans, fractional/exponent/float token encodings where
the existing Rust integer schema rejects them.

Run settings retain current host JSON camelCase keys:
`profile=full`, `limitDuration=false`, `durationSeconds=3600` integer 1-604800,
`rawReports=false`, `logLevel=info` in warn/info/debug.
Current synthesized Swift decoding requires fields and ignores extra keys;
LED/jog/tension and screen decoders reject unknown keys. Preserve this
distinction rather than claiming uniform unknown handling.
Screen camelCase keys/defaults remain `enabled=true`,
`automaticallyStopNI=false`, `restoreNIOnStop=true`, `restorePending=false`;
strict booleans, reject unknown keys, missing keys filled from defaults
(`ScreenPreferences.swift:3-34`).

Windows root is `%LOCALAPPDATA%\S4 MK3 Bridge\`; exact files:
`run-settings.json`, `led-config.json`, `jog-config.json`,
`tension-config.json`, `screen-settings.json`. Logs go in `Logs\`, motor
captures in `Diagnostics\`. Resolve local app data through the platform;
absence/unwritable state root is a visible failure, no fallback to the
distributed app/current directory. Bundle defaults/resources are read-only.

Editor, persisted and launched values remain distinct. Validate all pending
run/LED/jog/tension/screen values and encode all data before replacement;
write a fresh sibling temporary file, flush/close, then atomic replace.
Each file replacement is atomic, **not a transaction across five files**.
Report partial I/O failure and do not launch/restart. Current host behavior is
`S4BridgeApp.swift:689-716`; task 15 owns Windows implementation. Corrupt saved
values set the reset gate and leave user files intact; no silent overwrite.
Screens save immediately through a validated copy, updating editor only after
successful write (`ScreenSupport.swift:55-65`). Ordinary Save applies by
stop/restart; mapping sensitivity/direction changes require re-export/import.

Reset-all restores editor defaults for run/LED/jog/tension/screens but preserves
pending restoration. Save is required; do not erase ownership to clear errors.
Motorized-only reset affects exactly the fields assigned by
`JogPreferences.restoreMotorized` (`S4BridgeApp.swift:331-350`): startup Vinyl,
TT floor/gains/curves and all platter tuning; it leaves scratch/pitch mapping,
backspin, tension and LED/run editors alone.
LED import merges over bundled defaults into the editor, validates before
assignment, and may clear the existing recovery gate except invalid tension;
LED export emits the validated LED/Palette editor only. It is not a complete
settings backup. Mapping export validates and uses **unsaved jog editor**
values, fresh create-new output, nonzero/missing-file failure and owned temp
cleanup; preserve existing user destinations on Windows.

## 7. Platform collection, screens and ownership requirements

Task 3 must supply actual VID/PID 17cc:1720, serial/container identity, HID path,
usage page/usage, interface number, input/output/feature report capabilities,
USB/audio children, driver/service and exact binary identities. Task 9 selects
the single collection satisfying that recorded evidence; no guessed usage
numbers, interface substring or first VID/PID match is frozen here.
Absent/multiple/ambiguous capabilities fail before open. Current source only
proves that multiple HID collections are refused (`main.rs:241-253`).
Normalize demonstrated Windows framing once: reports 1/3/2 are 23/60/79 bytes
including ID; touch/buttons precede movement; queued analog interrupts cause
a fresh report 2 read, never overwrite newer snapshots with stale copies.
Keep 2 ms service target and 50 ms freshness cutoff.

Screens keep current raster/pages/font/latest-state mailbox, deferred open,
worker-local USB ownership, superseded-request cancellation and release-before-
retry. Claim only verified interface 4/bulk OUT 0x03; transfer bound 500 ms
(`screens.rs:146-200`). No reset/configuration change, audio claim, driver
detach or guessed NI service action. Task 12 must identify installed service/
binary, dependents, prior running/startup state, access and rollback evidence
before any separately authorized transition. This task performs none.

`restorePending` means a verified app-owned transition or unresolved recovery,
not “NI currently absent.” Record an intent before a scoped stop; mark ownership
confirmed only after verified running->stopped transition by this app.
Keep a durable pending record if the result is uncertain. Windows ownership
metadata must include exact service/binary identity, previous running/startup
state, operation ID and observed transition/verification evidence; task 12
supplies actual values and task 18 stores them beside the screen preferences.
Do not guess a service name here. Restore only a verified app-stopped service;
do not change startup type. Persist pending until successful verified restore.
An uncertain intent requires review, not automatic restoration. Reset/crash/
quit cannot erase it. This explicitly narrows the current Mac manual restore,
which can restore unowned NI state (matrix H-55); it is not permission to copy
the Mac force-stop policy. Cancel launches nothing; continue-without-screens
leaves control service available; remembered consent is not transition evidence.

## 8. Edit-path ownership and evidence gate

| Owner/task | Paths |
| --- | --- |
| Integrator5 | `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/main.rs`, `src/full/mod.rs`, `src/full/feedback.rs`, `src/screens_control.rs`, common CLI tests; new `src/midi_transport.rs` and initial `src/midi_layout.rs`. Other tracks send changes through this owner. |
| MIDI6 | Windows transport constructors and `windows/qa/midi-lifecycle.ps1`, transport tests; shared-file integration through integrator. |
| Mapping7 | `src/midi_layout.rs`, `src/full/mapping.rs`, layout fixtures, `windows/qa/feedback-route.ps1`; ingress integration through integrator. |
| Hardware3/9/12/13 | Inventory/device-access docs and QA; new `src/platform.rs`, `src/platform/windows.rs`; screen transport in `src/screens.rs` and optional Windows module. Main/input/shared integrations through integrator. |
| Supervisor14 | `windows/bridge-app/` project shell, `Core/BridgeSupervisor.cs`, supervisor tests. |
| Preferences15 | `windows/bridge-app/Models/`, `Core/PreferenceStore.cs`, preference tests. No competing project-file edits. |
| UI16/Diagnostics17/Screens18 | Views/ViewModels/tray; `Diagnostics/`; `Platform/NIServiceControl.cs` and screen integration respectively. Shared project/model changes go through their owner. |
| Build2/19 | Build/test/verify scripts, pinned toolchain manifest and smoke harness; packaging scripts/resources in19. |
| Contract4 | This document, `windows/contracts/`, task4 evidence only. Parity matrix stays read-only through its owner. |

Any contract revision reopens direct consumers5/7/12/14/15 plus dependent
8/13/18 checks. Task7 cannot proceed with an allocation conflict; task8 cannot
pass a silent declaration by substituting command echoes or cache guesses.
Source-only/fixture inspection proves address arithmetic and coverage, not
Djay provider behavior, hardware, audio coexistence or physical neutral.

## 9. Adversarial classes and verification limits

| Exact class | Task4 evidence or N/A justification |
| --- | --- |
| malformed_input | Existing decoder/screen/config regressions run unchanged; packet cases include SysEx, split data, Note Off velocity and invalid managed grammar. |
| prompt_injection | Managed text resembling status/“enable motor” is data, not an authorized state transition; no report text is executed. Case specifications are not runtime acceptance. |
| cancel_resume | Latest screen generation and old launch UUID cases traced through real worker/Swift handlers; cooperative STOP lifecycle specified. Windows implementation remains future work. |
| stale_state | Current source hashes and effective mapping checked; unload/repeated value/jump revision and stale launch cases retained. MIDI1 same-address stale-provider ambiguity remains task8's obligation. |
| dirty_worktree | Task1 dirty manifest and parity corrections retained; source and matrix read-only, explicit per-file hashes in audit. |
| hung_or_long_commands | Tests run under bounded monitor subscriptions; timeout cannot establish success or neutral. No hardware watcher/process was started. |
| flaky_tests | Initial default parallel Rust run failed the pre-existing invalid-path CLI assertion; exact isolated command reached expected create-file failure; unchanged serial suite passed510. Root cause not proven; concurrent process ownership is a plausible source-backed explanation, not a fix. |
| misleading_success_output | Full stdout/stderr and true command exits retained; a success-looking exit7 probe is required by artifact inspection. Fixture constants and test-name substrings are not passes. |
| repeated_interruptions | Repeated Off/Retry/On, duplicate STOP, old launch events and no-ready-after-stop cases are specified/traced. No physical interruption was applicable: this is a source/data-only contract, with no hardware session. |

All classes have explicit evidence in `contract-audit.json`; none is omitted.
No Windows status/STOP implementation, provider acceptance or physical
controller trial is claimed. The first Rust failure remains visible alongside
the successful serial result; no test was modified, skipped or weakened.
