# Bridge-owned functionality

Reference for the current full-controller source build, not a promise of
factory Traktor compatibility. Start with the [user guide](user-guide.md)
for everyday workflows. Current sections below supersede the October 6 base
snapshot retained later in this document.

## Logic implemented inside the bridge

### Current-source multi-deck Instant FX

Channel ON independently toggles that deck's Instant FX arming; only A starts
armed and an empty target set is allowed. Each central FX 1-4 press emits its
existing Note 16-19 on every armed deck channel 1-4. All addresses are retained
until release or Stop, even after selection changes. An empty-target press is
consumed until release. Filter-then-ON still changes one deck's EQ mode without
changing the armed set. Manual FX-strip targeting is unchanged.

ON lamps show each deck's actual EQ mode and arming, with actual active-effect
color taking precedence. Central buttons aggregate actual FX state across armed
decks rather than choosing one deck's feedback. Missing state is not guessed.
These changes use the existing generated mapping addresses.

### Current-source motorized playback testing

TT toggles motorized playback independently per physical side, initially off.
Its lamp stays red across deck changes, bright when selected and dim when off,
subject to the inactive-display preference.
Untouched TT rotation is excluded from raw jog output even while Shift is held.
Deliberate Shift-then-top-touch movement still emits jog seek CC 6 on the owning
deck's channel; releasing the top suppresses subsequent coasting seek output.
Normal-mode Shift+jog and held gesture/deck ownership are unchanged. GRID retains
its separate route. This does not change the motor's existing Shift gate.
See [Shift seek regression and validation limits](tt-shift-seek-validation.md).
Current source also enables Scratch movement for a confirmed paused, loaded
Internal deck in TT mode, independently of Vinyl and capacitive touch. This does
not authorize motor force while paused. Physical touch retains its original
Scratch owner through deck changes; fresh sensor deltas drive untouch-owned
paused movement. Resume, mode cancellation and stale input release that
synthetic Scratch owner and require a new bend baseline.
For a newly loaded, confirmed Internal deck with playing feedback still unknown,
TT also permits explicit top-touch Scratch with Vinyl off. This uses a separate
`playback_touch_scratch` permission; `playback_paused` and motor authorization
still require their actual feedback values. It does not create an untouch-owned
paused Scratch owner. The telemetry readiness mode is `manual_scratch`, with
`scratch` during the held gesture. Unloaded, THRU and unknown/non-Internal decks
do not receive this additional permission.

Current-source Jog settings provide independent braking and nudge curves for
the playback-driven TT CC4 route. Optional `tt_brake_gain` accepts 0.1-20 and
`tt_nudge_gain` accepts 0.1-10. Missing or null gains inherit the existing
`tt_pitch_bend_gain` (default 1, range 0.1-10), multiplied by 2 for braking
and unchanged for nudging. Explicit gains replace that combined strength;
there is no additional braking multiplier. Thus legacy shared gain 5 retains
effective strengths 10 for braking and 5 for nudging.
`tt_brake_curve_exponent` and `tt_nudge_curve_exponent` each accept 0.25-4,
defaulting to 1. All non-null values must be finite numbers.
Save/restart applies these runtime preferences without regenerating a mapping.
They do not change ordinary jog, Scratch, handoff thresholds or motor control.

The response-stability law in `src/full/platter_bend.rs` uses a true
2.25 RPM deadband for qualification and neutral output. There is no active
bend hysteresis; neutral clears the fractional residual. For error
`e = measured_rpm - target_rpm`, set `x = clamp((abs(e) - 2.25) / 1.0, 0, 1)`
and multiply the existing full-error law by `x*x*(3 - 2*x)`.
For selected strength `G`, selected exponent `P`, target RPM `T`, and
`q = min(abs(e) / T, 1)`, each fresh report contributes
`e * onset * G * q^(P - 1) * (2880 / 60) * elapsed_seconds`
before relative-step quantization. Exponent 1 preserves the earlier response;
higher exponents soften small deviations and lower exponents strengthen them.
At 3.25 RPM deviation and beyond, onset is 1; it does not subtract the deadband.
Above 100% target-speed deviation, the shape factor stays 1 for linear
continuation rather than unbounded power extrapolation. Qualification, sign
change/neutral residual clearing, report timing and speed measurement are
unchanged. The native graph evaluates this rate in signed CC4 counts/second;
it does not predict Djay's audible tempo.
See [curve implementation and verification limits](tt-response-curves-validation.md).
Full-profile `S4_PLATTER_STATUS` lines provide a 10 Hz Diagnostics snapshot with
physical RPM, the successful outgoing CC4 count rate and current routing mode.
This is MIDI output evidence, not audio-speed feedback.
The response-stability diagnostic adds an actual-runtime `S4_MOTOR_CONFIG`
receipt per launch and bounded `MOTOR_TT_STATE` reports for touch, authorization,
owner and requested force. Requested force is not applied torque. The app
retains the receipt for a capture started after launch. `MOTOR_SCRATCH_MODE`
is not a runtime emitter.
Normal/Info logging emits TT-state initialization and boolean-state transitions
only. Debug additionally emits periodic TT-state force samples at the bounded
100 ms cadence. The native app keeps log and live-platter observation separate
from preference observation, and only mounts the Diagnostics tab while selected.
See [idle logging and Settings performance validation](idle-ui-performance-validation.md)
for evidence and measurement limits.
The RPM diagnostic adds Debug-only `MOTOR_REGULATOR data={...}` JSON samples
at most every 20 ms per side, plus regulator-mode edges. Side 0 is left and
side 1 is right. Records include raw encoder counter/timer, current report
delta/status, authorization/contact flags, the motor window's signed count
travel and sensor-tick span, measured motor RPM and requested force after sample
arbitration. A regulating record also contains the actual PI state: baseline,
proportional, integral, unclamped and rate-limited/clamped force. PI state is
null when inactive; qualifying can include the existing assistance fallback
while the regulator rebuilds its window. This is sampled telemetry, not every
input report or proof of a successful motor write. Sensor ticks are 100 per
microsecond; the motor window is at least 20 ms. `S4_PLATTER_STATUS` now includes
`host_us` in the same bridge-start clock domain, supplying the separate 100 ms
RPM estimate and sent bend rate at 10 Hz. See the
[hands-free baseline](tt-rpm-baseline.md).
JOG toggles Vinyl red/on or blue/off. Normal-mode `vinyl_enabled_at_startup`
defaults true; `tt_vinyl_enabled_at_startup` defaults false. Each mode remembers
its own Vinyl choice during the session.
The testing `platter_*` preferences use the existing saved Jog configuration.
Their provisional feel defaults and application steps are in the
[user guide](user-guide.md#testing-motorized-playback).

The motorized-playback mapping supplies Djay-to-bridge CC22 on deck channels 1-4
(one-based) for `turntableN.controlModeIsInternal`. Loaded and raw playing
feedback plus explicit Internal true and THRU false authorize rotation;
false THRU alone is insufficient because it may mean DVS. Install/select a
matching mapping only if the current one lacks that feedback, preserving custom
mappings. The response-stability update does not change the mapping.
Active wheel output uses the existing CC4/CC5/Note7 routes. The retained CC8
absolute-position mapping is experimental and receives no runtime stream;
see the [archived method](absolute-position-scratch-experiment.md).

The production motor owner arbitrates rotation, tension and owned gestures;
nominal powered motion is removed before jog MIDI translation. The TT diagnostic
retains bounded forward assistance through touch, braking and Scratch rather
than summing tension/backspin force. Freshness, playback and deck-owner gates
still neutralize drive. With TT Vinyl off, the provisional configurable
`tt_pitch_bend_floor_percent` (default 50, range 1-95) selects only the lower
speed-driven CC4-to-CC5 Scratch handoff in `src/full/platter_gesture.rs`.
Entry is at or below `floor = target * percent / 100`. A qualified abrupt stop
from the normal-speed quiet band can also enter after 4 ms of fresh zero-count
sensor time, before the 100 ms RPM average falls through the floor. The candidate
starts from a measured speed within 2.25 RPM of target; valid nonzero movement
or a measurement reset clears it. Duplicate reports cannot advance confirmation,
and gradual low-speed count quantization does not start a candidate.
Top touch retains floor-entered Scratch across fast threshold crossings.
Without top contact, exit is at or above
`floor + min((target - floor) * 0.25, 2)` RPM, unless the abrupt stop remains
confirmed. Releasing below exit speed waits for recovery. Playback/mode/deck
cancellation and stale-input release still take precedence over touch.
Forward nudges do not trigger automatic upper Scratch. Explicit Vinyl-on touch
still owns Scratch.
See [touch retention and abrupt-stop verification](tt-touch-stop-validation.md).
In `src/tension/playback.rs`, hold assistance uses
`F0 = min(max_force, base_force + force_per_rpm * target_rpm)`, default 5280,
scaled by `clamp((target_rpm - measured_rpm) / min(2, target_rpm), 0, 1)`.
This preserves full stopped/reverse assistance, tapers the final 2 RPM below
target, and gives zero assistance at/above measured target, including the
touch-release fallback. The PI controller does not integrate while assistance
owns the motor.
New automatic underspeed assistance requires both the existing 20 ms motor RPM
and a separate sensor-clock window of at least 100 ms to be strictly below
`target_rpm - 2`. Qualification remains short RPM at least `target_rpm - 0.75`;
MIDI ownership can bypass qualification, not either admission-speed check.
Touch or negative short RPM bypasses long-window confirmation. Once assistance
is active, it continues below `target_rpm - 0.75` or during touch/reverse; the
long estimate cannot delay release when current speed recovers.
Both windows use signed count travel over actual sensor time. Real assistance
transitions still reset the PI controller; rejected short dips do not. Existing
full resets discard both histories. The 20 ms regulator, force taper, startup,
overspeed check, authorization and freshness rules are unchanged.
`MOTOR_REGULATOR` exposes the new `assistance_rpm` admission estimate separately
from `motor_rpm`. This reduces false admission in the recorded hands-free
fixture; it is not proof that all physical ripple or recovery chatter is fixed.
Saved floor/gain values persist; the response changes require no mapping export.
The floor still needs actual CC4/audio calibration;
see the [diagnostic runbook](tt-speed-scratch-diagnostic.md).
Implementation, package evidence and physical-test limits are
tracked separately in the [validation record](tt-response-stability-validation.md).
Cached real playback authorization survives feedback silence in this testing
build: use TT-off or Stop bridge before quitting Djay.
See [integration evidence](motorized-playback-integration-validation.md).

Ordinary jog Speed/Reaction are mapping-generation preferences, not count scaling in
the translator. Calibrated defaults are scratch 2.7% / 150% and pitch bend
2.7% / 17%, configurable through the app's Jog tab and CLI `--jog-config`
mapping export. Installing/selecting the generated mapping in Djay applies
them; restart alone does not.

Current-source backspin detection uses two 24 ms movement windows to recognize
backward acceleration while a Scratch gesture owns the wheel. This includes
TT physical touch owners and the TT Vinyl-off speed-floor Scratch owner. The
runtime options share `jog-config.json`: enabled, acceleration threshold
150000 counts/s^2, and additive speed increase 5.6%. A detected event retains
its deck/scratch ownership after release until measured near-rest, reversal,
retouch, mode cancellation or Stop. Input freshness loss also releases the
synthetic hold. Zero increase preserves measured counts; event gain is
`(scratch_speed + backspin_speed_increase) / scratch_speed`, with fractional
residuals confined to the event. Normal-mode bend, seek and grid actions cannot
start it.
Both screen pages show the original owning deck, independently of current deck
selection. Physical touch data remains unchanged.
See [backspin evidence and live checks](backspin-detection-validation.md).
TT floor Scratch shares `jog_steps` scaling with physical jog movement, so the
same saved additive speed increase applies before and after top release.
Retouch clears detection without chattering its Scratch note. The TT speed-floor
owner still cancels on deck change or stale input; clearing BACKSPIN alone does
not bypass the existing Scratch exit floor. Fractional travel clears with that
owner and cannot carry into Normal-mode bend.

`backspin_assist_enabled` defaults true. A separately gated per-wheel
override is restricted to Normal mode and uses the existing maximum assist
ceiling, 7000, and force qualification/
smoothing. `backspin_assist_strength` is an integer 0-100, default 100, scaling
only that override. Zero behaves as assist off; old saved files retain 100.
Real release starts a fixed one-second linear fade on the motor output
clock. Driven motion cannot renew it; retouch, measured stop/reversal, event/mode
cancellation, Stop or the 50 ms freshness cutoff cancels it. One real touch gesture
can produce only one pulse. The normal-tension histories continue independently,
and the saved tension selection is never rewritten. The ordinary recovery path
cannot resume an old pulse after freshness loss. See
[assist implementation and attended checks](backspin-assist-validation.md).
Existing bounded assist event logs include selected strength, requested release
torque and the latest decoded sample/age at a motion-stop cancellation. Requested
torque and accepted native writes do not establish physical force or feel.

### Current-source EXT switching and feedback

Full profile only: physical mixer EXT1-4 always address Djay decks 1-4,
regardless of deck focus or FX targets. A new unshifted press selects explicit
`turntableN.controlModeInternal` if the latest actual THRU flag is true, or
`turntableN.controlModeThru` if false. Unknown feedback sends nothing.
Every press is consumed until EXT release, including presses suppressed by
either Shift or missing feedback. Repeated snapshots, Shift release or later
feedback cannot replay a held press. There is no guessed mode or queued toggle.

On **S4 MK3 MIDI Full**, bridge-to-Djay Note 27 selects Thru and Note 28 selects
Internal on deck channels 1-4. Djay-to-bridge CC 21 on those same channels
carries `turntableN.controlModeIsThru`. Channels here are one-based.
EXT lamps light only for actual true THRU feedback; false/unknown stay dark
regardless of dim-idle settings.

Either Shift suppresses software EXT commands so hardware Shift+EXT can select
the input type. DVS enable remains manual and must stay on; no global DVS-enabled
detection is provided. Input assignment stays in Djay's GUI: A 1-2 PHONO/LINE,
B 3-4 PHONO/LINE, C 5-6 LINE/MIC C, D 7-8 LINE/MIC D.
These are USB audio pairs, not MIDI channels.

Install a newly generated matching mapping. This is current-source support,
not a newly published release. Physical EXT LED payload offsets 80/84/88/92
are confirmed for A-D; integrated acceptance is tracked in the
[EXT validation record](ext-input-validation.md). Broader audio validation is separate.
This overlay supersedes the historical base snapshot's EXT-unassigned claim.

### Current encoder overlay

Preview uses a complete `musicLibrary.togglePreview` pulse on press and another
on release or shutdown. While that side's Preview is held, BROWSER emits one
directional preview-seek pulse per detent, without requiring Shift. Notes 23/24
on channels 1-4 (9-12 with Shift) target `musicLibrary.skipForwardPreview` /
`musicLibrary.skipBackwardPreview`. Otherwise Shift+BROWSER sends source-next/
previous pulses on notes 25/26, channels 9-12; unshifted BROWSER browses normally.
Overlapping Preview holds keep preview open until the last release.
No playback state is guessed; the user accepts mouse-close recovery if external
preview changes leave the toggles out of sync. The previous Shift+BROWSER
scrubber was physically confirmed on October 21; the new held route needs
physical acceptance. LOOP and Shift+BROWSER press are unchanged.

Shift+CUE uses `turntableN.gotoZero` on Note 1, channels 9-12, for the beginning
of the song rather than a saved start cue. Normal CUE is unchanged. Library
Note 10 marks/unmarks selected songs; Note 11 switches the library table on channels
1-4. These assignments do not change HID fields.

The encoder increment is full-profile only. Descriptor normalization corrects
all rotation names and only left press names. LOOP turn/press use native
`autoLoopDurationRotary` / `autoLoopOnOff`; shifted LOOP retains
`autoLoopMoveRotary` on turn and uses `reloop` on press. Manual loop in/out
is unmapped. The user observed native Loop Move also moving inactive loop
regions, apparently by their loop length; the bridge supplies no size.

MOVE normally sends CC 2 on channels 1-4 to `skipRotary`, whether a loop is
active or not. Its press emits no MIDI and toggles size-select mode per
logical deck, with no timeout. Unshifted MOVE in that mode sends CC 7 on
channels 1-4 to `skipDurationRotary`: native jump size changes immediately,
and the second press only exits. Shift+MOVE selects one beat, sends native CC 2
with the full signed detent count, then restores the last confirmed selection
using normal-channel notes 64-73. Missing/conflicting feedback is not guessed.
The fixed-jump notes 5/14 remain mapping aliases, not runtime Shift destinations:
Djay coalesced repeated fixed-action pulses in a native multi-detent probe.

Djay owns the independent A/B/C/D jump and loop values; the bridge keeps
independent mode flags, initialized off on restart, plus confirmed jump feedback
and in-flight restoration ownership. No shared-size counters are used.
The selected deck selector pulses on a 1.2-second palette-brightness cycle
with a dark trough while selecting. Hidden deck modes persist without
flashing inactive buttons. This encoder overlay adds no screen or motor traffic;
the separate saved-tension increment below adds motor output.

The user mouse-verified beat jump moving an active loop. Native MIDI/GUI,
multi-detent jump restoration, and production LED-report checks are recorded in
[control validation](controls-state-sync-validation.md). Those checks are not
a complete physical A/C/B/D gesture matrix. When changing mappings, install
under a new filename and preserve custom maps. See
[focused live checks](full-midi-test.md#encoder-layout-live-check).

### Startup positions and FX pickup

The full profile seeds its real parser from an initial numbered analog report 2
GET without emitting bootstrap slider MIDI. After raw playback feedback opens
the existing mapping-ready gate, sixteen channel-volume/EQ CCs apply their latest
physical positions once. Full-mode analog interrupts trigger fresh snapshots
rather than replaying queued values older than the startup GET. Minimal is unchanged.

FX uses six actual values per deck from explicit root control outputs on CC 32-37:
three parameters and three wet/dry values. Shared physical knobs are not replayed
at startup or retarget. Per-strip/slot pickup gates require known software state
and matching/crossing movement, retaining ownership through ordinary feedback lag
and reacquiring after confirmed external edits. Native pickup flags remain enabled.
Known mismatch blinks the corresponding FX button; dry/wet uses main ON.
Info/STEMS adds an affected-deck `FX SYNC` header only while needed.
Current-source Shift + FX parameter knobs use native `fx1SelectRotary` through
`fx3SelectRotary` on the strip's target deck. Both strips share canonical browse
CCs 32-34 on channels 9-12 (Djay plist channels 8-11), encoded as rotary-64.
Four seven-bit physical position units produce one relative step. Shift or
target changes reset the travel origin; the first unknown position seeds only.
Browsing suppresses parameter output and re-arms that knob's pickup. Dry/wet,
effect-button held ownership and the existing Shift + button next-effect
shortcuts are unchanged. A newly generated mapping is required.
See [operation](controls-state-sync.md) and
[verification boundaries](controls-state-sync-validation.md).

### Musical-key feedback

The screen key follows deck-owned `displayKeyIndex` feedback independently of
song-owned `loadingSuccess`. Djay can send a new key before loading-false;
that notification must not erase it. Genuine eject reports key 0, displayed
as `--`. Missing or unrecognized key feedback remains `?`, without guessing.
The captured ordering regression and physical replacement/eject acceptance are
recorded in [key feedback validation](key-feedback-order-validation.md).

### Saved hardware tension

The full profile accepts `--tension-config PATH`, with a strict saved integer
`{"tension":0}` in -100..100. Negative values select motion-qualified,
held-touch assistance up to 7000; positive values select braking up to 30000.
Zero is neutral. This is one saved setting for both physical wheels, not a
logical-deck value or a mixer-fader assignment. The shared law is the same
decoder, qualification and smoothing used by the physically validated
diagnostic. Djay jog MIDI and held-touch ownership remain separate.

An explicitly paced bounded reader delivers full snapshots and preserves
ordinary analog interrupts. One bridge thread writes motor and LED reports.
Startup neutral precedes native initialization; queued zero gates precede
overdue output ticks. Stop and detected faults attempt full neutral before
reader teardown. Input older than 50ms fails closed. There is no automatic
reconnect: restart the bridge to initialize a fresh session. A disconnected
device cannot acknowledge cleanup; the bridge reports that failure.

The app saves this value separately from mapping sensitivity preferences and
applies it with Save and restart. First installation defaults to neutral.
No mapping regeneration is needed. Minimal/probe profiles do not use tension.
On October 20, 2026, the user confirmed normal-app assistance, braking,
Djay/MIDI/LED coexistence and Stop, followed by Quit, saved-setting restore
while stopped, and USB disconnect/reconnect with explicit Start. These are
user-reported physical passes on the integrated build, separate from the
earlier diagnostic passes. The full UI regression battery remains deferred.

### Historical base snapshot

The Mixer ON/Filter overlay supersedes the base table's quick-FX/reset row:
Filter arms/cancels a one-shot cyan-flashing deck selection. The next ON toggles
that deck's Neural Mix EQ via Note 20 on channels 1-4 without changing the
Instant FX target. Normal ON still selects that target. ON uses confirmed
white/cyan EQ mode and dim/bright FX arming; active Instant FX color overrides
both. Unknown mode stays dark. MIDI-only capture verified initial states and
independent mouse-driven mode transitions for A-D on October 20, 2026. The user
subsequently accepted the physical workflow and dim-cyan idle Filter;
exhaustive edge cases remain unverified. See [the focused check](neural-mix-eq-check.md).

The table below retains the earlier snapshot and source line references.
The encoder and selector-mode overlay above supplements it; it doesn't
establish which revision is installed in an existing app bundle.

These decisions are not ordinary Djay MIDI Learn assignments. Where a row
ultimately controls playback, Djay still performs the audio operation.

| Bridge function | Human-readable behavior | MIDI/software consequence | Source |
| --- | --- | --- | --- |
| HID decoding and descriptor corrections | Read native S4 reports without impersonating another controller. Correct payload sizes, 12-bit slider ranges, pad bit order, and added EXT/headphone fields. | Physical input becomes named events; EXT is decoded but unassigned. | `src/main.rs:380`, `src/main.rs:422` |
| Full-profile snapshot transport | Request buttons/touch before jog movement; discard stale interrupt copies of those reports. | Avoid replaying old touch/motion; ordinary slider interrupts remain accepted. | `src/input.rs:5` |
| Physical deck selection | Left side selects A/C; right side B/D. Defaults A/B. | Route deck controls on human channels 1-4; selection also emits application deck-selection notes on channel 6. | `src/full/mod.rs:101`, `src/full/mod.rs:299` |
| Shift layer | Each physical side has its own held Shift state. | Bindings with a shifted action use channels 9-12; other bindings retain normal channels. Shift is not itself a Djay conditional. | `src/full/mod.rs:299`, `src/full/mod.rs:651` |
| Pad modes | HOTCUE, SAMPLES, STEMS choose which eight actions the pads address. Mode is per physical side, not per software deck. | Normal notes 40-47, 48-55, or 56-63 respectively. | `src/full/mod.rs:299`, `src/full/catalog.rs:359` |
| MUTE alias | MUTE selects exactly the same layer as STEMS. | It does not mute the deck or all stems by itself. | `src/full/mod.rs:343` |
| Pad slot order | Correct raw bit masks to physical pad order. | Numbered pads address slots 1-8 rather than the descriptor's original order. | `src/main.rs:434` |
| Sampler bank addressing | A/C target sampler bank 1; B/D target bank 2. | These deck pairs share sampler players; they do not have four independent banks. | `src/full/mapping.rs:221` |
| FX strip target selection | Mixer channel FX1 selects the left strip's deck; FX2 selects the right strip's deck. Defaults A/B. | Changes control destination, not a Traktor-style shared FX audio bus. | `src/full/mod.rs:410` |
| Mixer quick-FX target | Channel FX SELECT selects the target for central FX 1-4/Filter buttons. Defaults A. | Filter resets that selected deck's filter, not all four filters. | `src/full/mod.rs:410`, `src/full/mod.rs:509` |
| Dry/wet fan-out | One hardware dry/wet knob emits three CCs. | Sets wet/dry for FX slots 1, 2, and 3 on the strip's target deck. | `src/full/mod.rs:487`, `src/full/catalog.rs:185` |
| Global Quantize fan-out | One button is sent to all four decks. | Toggles each deck separately; it is not an absolute "make all on" operation. Mixed initial states can remain mixed. | `src/full/mod.rs:509` |
| JOG/TURNTABLE local mode | JOG chooses nudge/CD behavior; TURNTABLE chooses touch scratching. Default is touch scratching. | Changes routing of touch/motion; neither button enables a motor. | `src/full/mod.rs:365`, `src/full/mod.rs:385` |
| Touch-owner routing | At touch-down, latch deck and scratch/seek/nudge destination. | Held touch continues controlling its original deck even if a deck selector changes. | `src/full/mod.rs:385`, `src/full/mod.rs:570` |
| Jog count recovery | Recover integer counts from normalized HID floats before converting movement. | Avoid delayed counts and reversal cancellation caused by float truncation. | `src/full/mod.rs:584` |
| GRID+jog | While GRID is held, accumulate 0.125 grid steps per recovered count and emit directional pulses. | Beatgrid editing replaces normal jog motion. | `src/full/mod.rs:585` |
| Relative MIDI encoding | Encode signed increments around CC value 64; split large deltas into chunks of up to 63. | No explicit maximum total jog speed is imposed here; per-message chunk size is not a total-motion cap. | `src/full/mod.rs:676` |
| Held-note ownership | Remember every MIDI destination owning a press. Release old notes before selector changes/new presses. | Deck/mode changes cannot strand a held Cue/pad/effect note; duplicate owners do not release each other. | `src/full/mod.rs:238`, `src/full/mod.rs:525` |
| Shutdown release | Release all owned MIDI notes when the bridge closes. | Prevent stuck held actions after Stop/Quit. | `src/full/mod.rs:183`, `src/full/mod.rs:559` |
| Front crossfader selectors | Cache physical assignments and curve until a raw playback output has arrived from Djay's mapping, then send the latest snapshot once. | After startup, send only physical changes. A playback packet is evidence of feedback, not an acknowledged input connection. Restart the bridge after mapping reconnect to initialize a new session; no silence-gap inference or periodic replay overrides UI choices. | `src/full/mod.rs`, `src/full/feedback.rs` |
| MIDI source/destination lifecycle | Create matching virtual input and feedback endpoints for the full profile. | Exposes `S4 MK3 MIDI Full`; does not claim the S4 audio device. | `src/full/mod.rs:131`, `src/full/feedback.rs:152` |
| Feedback decoding/cache | Parse received MIDI, including running status; retain supported software state. | Shifted/unsupported addresses cannot overwrite normal lamps; A/C and B/D sampler feedback is shared. | `src/full/feedback.rs:45`, `src/full/feedback.rs:111` |
| Local selector LEDs | Render deck, pad mode, jog mode, grid, and selected FX targets from bridge state. | These can light without a Djay feedback message. | `src/full/leds.rs:246` |
| Software-state lamps/pads | Translate supported returned software values into S4 palette/intensity bytes. | Unknown state remains dark. Bright mute indication describes the mute switch, not effective audibility under solo. | `src/full/leds.rs:309`, `src/full/leds.rs:377` |
| Momentary local indicators | REC, library controls, and Filter reset indicate held button state where persistent software state is unresolved. | A lit lamp here does not prove recording or playback is active. | `src/full/leds.rs:367` |
| Hotcue color interpretation | Decode Djay palette tokens or use explicitly selected fixed-slot colors. | Token-to-S4 palette correspondence is configurable; unrecognized tokens stay dark. | `src/full/leds.rs:470` |
| Jog-ring presentation | Choose selected deck color; fixed-period decorative chase only when raw playing state is true; green whole-ring firmware flash while looping. | A 2-second chase by default, not track position, physical platter motion, seek position, or BPM. | `src/full/output.rs:138` |
| Channel meter presentation | Convert normalized deck level into 14 S4 meter segments with brightness/gamma and freshness rules. | Fixed A-D meters; stale values go dark; no invented clipping signal. | `src/full/output.rs:168` |
| HID output pacing/cleanup | Coalesce unchanged reports; pace button, ring, and meter writes; clear outputs on shutdown. | Native reports only; no motor report `0x31`. | `src/full/output.rs:26` |
| Mapping generation | Expand the shared catalog into concrete deck/shift addresses and declared feedback. | Ensures emitter and generated mapping agree structurally; does not prove an action has the right musical meaning. | `src/full/mapping.rs:19` |
| LED preferences | Validate JSON and render from configured colors/levels/timing. | Presentation changes do not change input action assignments. | `src/full/leds.rs:158` |
| macOS app process management | Bundle the executable, install/open mapping, save/import/export settings, drain logs, and stop the child with SIGINT. | Runs without Rust on the laptop; closing Settings is not Stop, quitting the app is. | `macos/S4BridgeApp.swift` |

## Hardware-local, not bridge-managed audio

| Function | Owner | Current bridge behavior |
| --- | --- | --- |
| Master output level | S4 audio hardware | Do not map again to software master attenuation. |
| Booth output level | S4 audio hardware | Do not add duplicate software attenuation. |
| Headphone level and mix | S4 audio hardware | Decode where available, but do not assign duplicate software controls. |
| Master meter bars | S4 audio hardware | Preserve hardware behavior; separate master clip validation remains open. |
| Audio playback, sample recording, effects, Neural Mix | Djay and the selected CoreAudio device | HID/MIDI bridge controls operations but does not process, capture, or forward audio. |
| Live external-input routing | Djay/CoreAudio and S4 hardware source selection | Full-profile EXT selects per-deck THRU/Internal using feedback. DVS enable and input assignments remain manual; the bridge doesn't forward audio. |

## Not implemented

Playback-driven platter rotation, physical/playback-position ring tracking,
an on-controller settings menu, a latched Preview
mode, and automatic same-process mapping-reconnect detection are not present.
Whole-bank FX bypass and REC's end-to-end sample-recording behavior remain
unresolved.

App-managed screens, Info/STEMS legends, held-Preview encoder scrubbing, and
global Shift+Sync tempo-range cycling are implemented in current source.
See the [screen reference](screens.md) and [user guide](user-guide.md).
Startup channel-volume/EQ and crossfader assignment/curve initialization wait
for actual mapping feedback; gain, filter, crossfader position and tempo are
not included in the channel-volume/EQ push.
