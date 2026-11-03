# S4 MK3 Bridge user guide

Use this guide to DJ with the S4 MK3, understand the custom mappings, and
recognize what its lights and screens are telling you.

**Availability:** this guide covers the full-controller app, including
development-build features. EXT switching, motorized TT playback and its
tuning controls, red TT indicators, and multi-deck Instant FX are not in the
published download. Check the [download notes](../README.md#download-and-install)
for your package's features. Always use the app and its matching mapping
together, importing updates under a new name to preserve custom mappings.

The DJ laptop needs the standalone app, not Rust, Cargo or this repository.

## Contents

- [One-time setup](#one-time-setup)
- [Start and finish a set](#start-and-finish-a-set)
- [Deck focus and transport](#deck-focus-and-transport)
- [Browse, preview and load](#browse-preview-and-load)
- [Loops and beat jumps](#loops-and-beat-jumps)
- [Mixing, cueing and Neural Mix EQ](#mixing-cueing-and-neural-mix-eq)
- [Manual FX strips](#manual-fx-strips)
- [Mixer Instant FX](#mixer-instant-fx)
- [Pads and sample banks](#pads-and-sample-banks)
- [Jog wheels and feel](#jog-wheels-and-feel)
  - [Testing motorized playback](#testing-motorized-playback)
  - [Record a motor diagnostic capture](#record-a-motor-diagnostic-capture)
- [Screens and feedback](#screens-and-feedback)
- [Settings: what needs saving or a new mapping?](#settings-what-needs-saving-or-a-new-mapping)
- [Troubleshooting](#troubleshooting)
- [Intentional differences and limits](#intentional-differences-and-limits)

## One-time setup

You need an Apple Silicon Mac, a supported macOS/Djay Pro combination, Djay
PRO access for custom MIDI mappings, and the powered S4 MK3 connected by USB.
The menu-bar app is currently macOS-only.

1. [Install the app](../README.md#download-and-install), then open it.
   Quit any older bridge app or standalone bridge first: run one instance.
2. In Settings, select **Full controller + LED feedback** and turn off
   **Stop after a fixed duration** for normal DJ use.
3. Click **Start bridge**. For screens, enable **S4 screens** and follow the
   NI-driver prompt described below.
4. Choose **Install Djay mapping...**, save under a new filename, and accept
   Djay's import. Keep your custom mappings under their existing names.
5. In Djay, open **MIDI > Configure S4 MK3 MIDI Full**, enable the device,
   and select the imported **S4 MK3 Bridge** configuration. Its name may have
   a number appended.
6. Set Djay's audio device to the S4's native audio interface. Route **Main**
   to its speaker outputs and **Pre-cue** to its headphones. Audio does not
   pass through the bridge.
7. After selecting a different mapping, stop and start the bridge once.
   Confirm controls and headphone cue before using the speakers.

If Djay offers only a blank S4 configuration, restart Djay with the bridge
already running, then select the imported mapping.

### Let the app manage screen ownership

If NI's background screen driver is running, the app offers **Enable screens**,
**Continue without screens**, or **Cancel**. Enable screens temporarily stops
that driver. Optional automatic stopping and restoration are in the Screens
tab. Close Traktor, Maschine and Komplete Kontrol before handing over their
shared service; the bridge will not close those applications for you.

Continue without screens still runs controls, LEDs and saved tension. To enable
screens later, choose **Enable screens - stop NI driver** in the menu or
Screens tab. A red status icon can mean screens are blocked while controls
continue working. Use these recovery controls rather than stopping the driver
manually.
See the [screen-driver guide](screen-driver-handover.md).

## Start and finish a set

### Before playing

1. Power/connect the S4 and open Djay and the bridge app.
2. Put the channel-volume faders at your intended starting positions,
   normally down for tracks you are cueing.
3. Start the full-controller bridge with the matching mapping selected.
4. Check Djay's channel volumes and EQ. After mapping feedback arrives, the
   bridge applies the physical volume and high/mid/low EQ positions once.
   Do not assume that synchronization has happened before the mapping connects.
5. Check gain, filter, tempo and crossfader position separately: they are not
   included in that startup push. Crossfader assignment switches and curve
   have their own startup synchronization.
6. Cue a track through headphones, confirm the selected deck, then bring its
   channel volume into the mix.

Later mouse edits stay in place until you move the corresponding control.
Shared FX knobs use pickup instead of jumping to their physical positions.
See [startup and pickup details](controls-state-sync.md).

### When finished

**Turn TT off or Stop bridge before quitting Djay.** The bridge can retain
Djay's last reported playing state after Djay closes.

Pause playback or lower your outputs, then use **Stop bridge** or
**Quit S4 MK3 Bridge**. Stopping the bridge does not pause Djay.
Closing the settings window alone leaves the bridge running in the menu bar.

To return to Traktor, use **Restart Native Instruments screen driver** if needed
and accept **Stop bridge & restore NI driver** if prompted. Open Traktor after
restoration; restart it if it was already open. With automatic restoration
enabled, stopping the bridge restores a driver the app stopped. Turning only
the screens off doesn't return the driver during a running bridge session.

## Deck focus and transport

| Physical side | Available decks | Djay deck numbers |
| --- | --- | --- |
| Left | A or C | 1 or 3 |
| Right | B or D | 2 or 4 |

Choose A/C or B/D before loading or working a deck. Its transport, pads,
encoders and wheel follow that focus. The four mixer channels remain assigned
to their own decks; changing the left focus to C does not turn A's channel
fader into C's.

| Control | What it does |
| --- | --- |
| PLAY | Djay play/pause |
| Deck CUE | Djay's cue/jump action, dependent on whether the deck is playing or paused |
| Shift+CUE | Jump to the beginning of the song, not the saved cue point |
| SYNC | Sync the selected deck |
| Shift+SYNC | Cycle the tempo-fader range for **all decks** |
| MASTER | Choose that deck as Djay's sync master |
| REV, held | Reverse while held; release ends reverse |
| FLUX | Toggle Djay Slip independently |
| QUANT | Send a quantize toggle to all four decks |

REV does not automatically switch Slip on or restore an earlier Slip state.
Use FLUX explicitly when you want Slip. Shift+SYNC is a global range change,
not Traktor's per-deck tempo-fader lock.
QUANT toggles each deck; it does not force already-mixed quantize states into
one uniform state.

FLX and REV are dim red when off and bright red when active. They stay dark
until Djay reports their state. Their backlights use the palette brightness
setting, like JOG. Shift is dim white and brightens while held.

## Browse, preview and load

1. Select the destination deck with A/C or B/D.
2. Turn that side's **BROWSER** to browse the library.
3. Hold **Preview** to audition the selected track.
4. While still holding Preview, turn BROWSER to scrub the preview:
   clockwise forward, counterclockwise backward. **Shift is not required.**
5. Release Preview to stop auditioning.
6. Press BROWSER to load the selection into the deck chosen on that side.

Other library controls:

| Gesture | Action |
| --- | --- |
| Shift+BROWSER turn | Next/previous library source instead of track browsing |
| Shift+BROWSER press | Library Back |
| Star button | Djay's **Switch Library Table** action |
| Playlist button | Mark/unmark the selected song; not a rating or Queue toggle |
| Library View | Toggle library visibility |

Switch Library Table changes which library table you work in; changing source
is the separate Shift+BROWSER gesture. If Preview and Shift are both held,
**Preview wins** and BROWSER still scrubs.

If Preview was already playing or becomes out of sync after a mouse edit,
stop it in Djay before trying another hold.

## Loops and beat jumps

LOOP length and MOVE jump length are independent. Each logical deck keeps its
own values in Djay.

### Set a loop

1. Turn **LOOP** to choose the loop length in Djay.
2. Press LOOP to activate it; press again to leave it.
3. While active, turning LOOP changes its size.
4. Shift+LOOP turn moves the loop region. Shift+LOOP press uses **Reloop**
   to reactivate a stored loop range.

On the S4, LOOP shows the active length with green fill, or **OFF** when no
loop is active. It does not show the preselected inactive length; look at
Djay's selector while choosing that.

### Choose a jump length and move

1. Press **MOVE** to enter jump-size selection for this deck.
2. Turn MOVE to choose the jump length. The value changes immediately in Djay.
   The deck selector pulses; the screen shows **SELECT**.
3. Press MOVE again to leave selection.
4. Turn MOVE to jump by that length. With an active loop, beat jump moves the
   loop region by the jump length.

Selection stays active across deck switches until you exit it or restart the
bridge. Restarting clears selection mode, not Djay's jump length.

For example, choose an 8-beat jump, then turn MOVE normally for 8-beat moves.
Hold Shift while turning MOVE for 1-beat adjustments, including while SELECT
is active. The bridge restores the **last confirmed** jump length afterward.

If JUMP is `?`, the previous size is not known reliably. Shift+MOVE still
selects and jumps one beat, but cannot restore an unknown size. Establish a
confirmed size first. Manual loop-in/out is not mapped.

## Mixing, cueing and Neural Mix EQ

Channel volume, gain, high/mid/low EQ and filter control the corresponding
Djay channel. Use the **small channel headphone/CUE button** for pre-cue,
not the deck's large transport CUE button. Master, booth, headphone level and
headphone mix remain hardware-local controls.

Crossfader assignment switches choose a side or **THRU**. THRU means the channel
bypasses the crossfader, not that it is muted. Djay may show **Crossfader
assignment changed**; choose **Keep** to retain the physical assignments.
**Restore** changes them to Djay's defaults.

### Change a channel's EQ mode

1. Press the mixer **Filter** button. It flashes cyan, waiting for a deck.
2. Press that channel's **ON** button to toggle its normal/Neural Mix EQ mode.
3. To cancel without changing a deck, press Filter again.

This does not change the armed Instant FX decks. Outside this selection workflow,
channel ON independently arms/disarms that deck for Instant FX; it is not a
channel mute button.
Filter doesn't reset the software filter.

ON is white for normal EQ and cyan for Neural Mix EQ, brighter on each armed
Instant FX deck. An active Instant FX temporarily takes over its color.
Dark/unknown feedback is not proof the deck is muted.

### External audio through a deck

Djay can mix an S4 audio input through a deck in **DVS THRU** mode.

#### Required Djay DVS setup

**Turn on DVS before using EXT.** The bridge cannot enable it for you.

1. Open Djay **Settings > DVS** and check **Enable Digital Vinyl Control**.
2. Under **Input Sources**, select **Traktor Kontrol S4 MK3** for each deck
   you want to use with an external source, using the stereo pairs below.
3. Click **Apply** after changing the assignments. Leave DVS enabled, even
   when playing normal tracks in **Internal / INT** mode.

For all four channels, configure:

| S4 input | Available source types | USB input channels |
| --- | --- | --- |
| A | PHONO / LINE | 1-2 |
| B | PHONO / LINE | 3-4 |
| C | LINE / MIC C | 5-6 |
| D | LINE / MIC D | 7-8 |

![Djay DVS settings with Enable Digital Vinyl Control checked and S4 inputs 1-2, 3-4, 5-6 and 7-8 assigned to Decks 1-4](images/djay-dvs-settings.png)

These are audio input channels, not MIDI channels. Selecting another source
type on the same S4 input does not give it a separate USB pair.

For input D, including the front MIC D socket:

1. Select the appropriate S4 input type with **Shift+EXT**: LINE for rear
   input D or MIC D for the front microphone. Keep listening levels down
   while selecting or connecting a source.
2. Start the full-controller bridge with its matching mapping selected in
   Djay, after completing the DVS setup above.
3. Press **EXT 4** without Shift to switch Deck 4 from Internal to **Thru**.
   You can also select Thru in Deck 4's **DVS control-mode
   menu beside Play**. This is different from the crossfader's THRU assignment.
4. Set channel level and cue/mix normally. If the same input is also selected
   as Djay's global Microphone, leave that microphone route off to avoid
   mixing it through a second path.
5. Once Djay reports THRU, press **EXT 4** again to choose **Internal / INT**
   and return to normal track playback, or choose Internal in the same menu.

**EXT 1-4 always address decks 1-4**, regardless of A/C, B/D or FX selection.
Each press switches between Thru and Internal using Djay's reported mode.
If that mode is unknown, the button does nothing. Check the matching mapping,
then release EXT before trying again.

Holding **either Shift** leaves EXT for hardware input-type selection.
The EXT lamp lights only when Djay reports THRU; it doesn't confirm that DVS
is enabled or audio is arriving. If you hear nothing, check **Enable Digital
Vinyl Control** and the deck's input assignment in **Settings > DVS**.

Check levels, cueing and FX before using an external source live. Full mixer
behavior across all inputs, phono behavior and four simultaneous external
sources haven't all been verified.

## Manual FX strips

The two top FX strips each control three manual effect slots on one deck.
They are **control targets**, not Traktor-style shared FX audio buses.

1. Press a channel's **FX1** button to target it from the left FX strip, or
   **FX2** for the right strip.
2. Use strip buttons **1-3** for individual effects.
3. Hold Shift on the strip's physical side and turn parameter knob 1, 2 or 3
   clockwise for next effects or counterclockwise for previous effects in that
   slot. Movement is relative, not an absolute position in the effect list.
   Shift + an effect button still chooses its next effect as a shortcut.
4. Release Shift to adjust parameters with knobs 1-3. After browsing, move
   through the software parameter's pickup position before it responds.
5. Use the shared dry/wet knob for their wet/dry amounts.

Larger turns browse several effects. These knobs have end stops: release
Shift, reposition, then hold Shift again to continue browsing.
Repositioning does not select effects, but crossing the software parameter
value picks it up and resumes parameter adjustment. Dry/wet remains a wet/dry
control even with Shift held.

Defaults are A for the left strip and B for the right. Changing A/C or B/D
deck focus does not automatically change these targets.

### Pick up a software value without a jump

After startup, retargeting or an external software edit, a physical FX knob
can disagree with Djay. Its movement is held until it reaches or crosses
the software position.

- A blinking effect button means its parameter knob needs pickup.
- A blinking strip ON means dry/wet needs pickup.
- Move toward and through the value shown in Djay until the blink stops.
- Dry/wet reaches three software slots; each picks up independently. Its
  warning clears when all three have been picked up.
- An `FX SYNC` screen header identifies the affected deck and controls.
  A `?` there means feedback is missing, not a confirmed mismatch.

After pickup, ordinary knob movement does not flash a new warning while
Djay returns the updated values. A mouse edit in Djay or a different FX
target can require pickup again.

The strip's main **ON is not a reliable whole-bank bypass**: it duplicates
slot 1. Use the individual effect buttons or Djay's UI
when you need to turn the other slots off.

## Mixer Instant FX

These are separate from the two manual FX strips:

1. Press each channel **ON** without Filter selection armed to arm that deck.
   Press again to disarm it. Several decks can be armed together.
2. Hold a mixer FX button 1-4 to apply its assigned Djay Instant FX to all armed
   decks at once.
3. Release to end the effect on every deck where that press began.

Only A starts armed. For A and B together, press B's ON; press A's ON too if
you want B alone. You can disarm all four decks: an FX press then does nothing
until you release it and make a new press with a deck armed.

Changing the armed decks during a hold affects the next FX press, not the
effect already held. Bridge Stop also releases every held effect. The central
FX button lights when Djay reports that effect active on any armed deck;
each channel's ON can still show an active effect after that deck is disarmed.
These are per-deck effects, not routing into a shared effect bus.

## Pads and sample banks

| Mode | Pads | Shift |
| --- | --- | --- |
| HOTCUE | Trigger/set hotcues 1-8 | Clear the corresponding hotcue |
| SAMPLES | Trigger sampler players | Stop their playback |
| STEMS or MUTE | Pads 1-4 mute; pads 5-8 solo | Pads 1-4 exclusive solo; pads 5-8 stay ordinary solo |

A/C share sampler bank 1; B/D share bank 2. Selecting another deck on the same
physical side does not give you an independent new sample bank.

The STEMS columns are **Drums, Bass, Harmonic, Vocals**, in that order. The
lower four pads solo the same columns. Shift makes the upper row **ONLY**
so you can select one stem exclusively. MUTE is a mode-selection alias for
STEMS, not a held modifier and not a button that mutes the entire deck.

Neural Mix availability depends on the track and Djay support. The screen
reports actual mute/solo state; ON in a MUTE tile means that mute is active.

**REC is not the set recorder.** It uses Djay's Record Sample action, which
has a known bug. Use Djay's onscreen recording controls to record a mix.

## Jog wheels and feel

**JOG** toggles Vinyl: red enables top-touch scratching; blue keeps bend/nudge
behavior. Normal mode starts with Vinyl on, TT mode with Vinyl off. Each mode
remembers its last Vinyl choice during the session; separate startup options
are in Jog settings.

**TT** toggles motorized playback on each physical side. It starts off whenever
the bridge starts. Its lamp is red, bright when enabled and dim when disabled
(or dark if inactive lamps are set dark). See [motorized playback](#testing-motorized-playback).

Hold **Shift** and turn the jog to seek. In TT mode, hold Shift first, then
touch and turn the platter top; lifting your hand ends seeking even while the
wheel coasts. If you started a gesture before pressing Shift, release it first.
**GRID+jog** edits the beat grid instead of nudging.

### Change physical resistance

In **Jog > Jog wheel feel > Wheel tension**:

| Setting | Effect |
| --- | --- |
| Assist, -100 to -1 | Helps a wheel's movement while its metal top is touched |
| Neutral, 0 | No added tension |
| Brake, 1 to 100 | Resists movement, including after release |

Both wheels share the setting but respond independently. It isn't assigned to
a mixer fader or saved per deck. The app defaults to **-77**, toward Assist.
Use **Save preferences** for the next Start,
or **Save and restart** while running. No mapping export is needed.

### Change Djay's scratch/bend response

The Jog **Scratch and pitch bend mapping** section sets Djay's **Speed** and
**Reaction**. Defaults are **2.7% / 150%** for Scratch and **2.7% / 17%** for
pitch bend. **Touch-scratch direction** offers Normal or Reversed.

These change software response, not motor force. Save, use **Install Djay
mapping...**, then import and select the new mapping in Djay. Restarting alone
doesn't apply mapping changes. Haptic cue points aren't available.

### Backspin detection

In **Jog > Backspin**:

- **Detection** lets a sharp backward flick continue scratching after you lift
  your hand, following the wheel's real movement. It works during Scratch in
  Normal Vinyl mode and TT mode, including TT's low-speed Scratch with Vinyl off.
  Ordinary nudging, seeking and GRID don't trigger it.
- **Detection threshold** sets how sharp the flick must be. Raise it if
  ordinary scratches trigger backspins; lower it for easier detection. The
  default is **150000 counts/s^2**.
- **Extra scratch speed** adds speed during a detected backspin. The default
  is **5.2%**; **0%** keeps normal scratch travel. Keep normal Scratch speed
  matched to the mapping selected in Djay.
- **Motor assistance** helps the wheel during detected backspins in Normal
  mode only. It defaults on at **100% strength**. Lower the strength for less
  help, or use 0% or turn it off for none. TT uses its own playback assistance.

The screen shows **BACKSPIN** and the affected deck. After release, motor
assistance fades out within one second, but scratching can continue while the
wheel coasts. Retouching, stopping or reversing the wheel ends the backspin.
A top-touch backspin stays with its original deck if you change focus. A TT
Vinyl-off backspin cancels on a deck switch; otherwise Scratch can continue
until the wheel regains forward speed.

Apply changes with **Save and restart**, or save while stopped for the next
Start. No mapping export is needed. To compare motor strength, repeat the
same flick and release while keeping tension and Scratch speed increase fixed.
The controls don't change the running bridge until applied.

### Testing motorized playback

Motorized playback is experimental. Its feel differs from Traktor, and it
isn't fully verified for live use.

1. Install and select this app's matching mapping under a new filename. TT
   needs its Internal-mode feedback; keep your custom mappings intact.
2. Load a track and select **Internal / INT** in Djay, not THRU or DVS control.
3. Press **TT** on the desired side. Rotation follows playback on that side's
   selected deck. The default target is **33 RPM**.
4. Pause the deck or switch TT off to stop powered rotation. There is no
   automatic time limit.

**Turn TT off or Stop bridge before quitting Djay.** Closing Djay alone may
leave the last reported playing state active. Stop releases scratch holds
and clears motor force.

With Vinyl off, slowing the wheel bends the track down and speeding it up
nudges it forward. Small deviations within **2.25 RPM** of target produce no
pitch bend. The wheel must first settle near target speed before bending works.

The **Switch to scratching below** setting sets when slow rotation changes from
pitch bend to Scratch. Its default is **1% of target speed**, or **0.33 RPM** at
33 RPM.
This is an adjustable handoff point, not a verified minimum playback speed in
Djay. An abrupt stop can enter Scratch sooner. Once Scratch starts, keeping
your hand on the top retains it through forward and reverse strokes. After
release, it returns to pitch bend as the wheel recovers forward speed, at
2.33 RPM with the default settings. A fast forward nudge doesn't automatically
switch to Scratch; enabling Vinyl lets top touch select Scratch directly.

On a paused, loaded Internal deck, turning the platter moves the track in
Scratch even with Vinyl off and without top touch. The motor stays neutral.
You can also touch and turn a newly loaded Internal track before pressing Play.

The motor continues helping forward movement while you brake, hold or reverse
the wheel, easing off near target speed. Tension and backspin assistance don't
add extra force in TT mode.

In **Jog > Motorized platters**, start with **Base force** to adjust the
baseline motor drive. The default is **7,000 force units**. Move toward Weak
for a gentler push or Strong for a stronger push; speed-based drive and
corrections are added on top, up to the maximum force.

**Pitch bend response** has separate Braking and Nudge controls:

- **Strength** changes how strongly the track slows down or speeds up for the
  same platter movement. Defaults are **10x braking** and **3x nudge**.
- **Shape** changes the response to small movements. Lower values emphasize
  them; higher values make them gentler. Defaults are **1 for braking** and
  **1.6 for nudge**.

Tick **Advanced settings** for startup Vinyl choices, target speed, the
scratch handoff, maximum force and detailed motor tuning. Untick it to hide
those controls without changing their values. **Restore defaults** resets
the motorized platter controls, including response and advanced settings,
in the editor. It leaves wheel tension, backspin and mapping settings alone.

Save while stopped for the next Start, or use **Save and restart** while
running. These controls aren't live and don't need a new mapping.
Detailed motor tuning and limits are in the
[bridge function reference](bridge-function-reference.md#current-source-motorized-playback-testing).

#### See what braking is doing

Open **Diagnostics > Live platter diagnostics** with the full-controller
profile. The readings work at any log-detail setting.

- **Measured speed** is physical RPM, including while paused or scratching.
  Negative values mean reverse rotation.
- **Bend output** is the pitch bend rate sent to Djay. Negative means slowing;
  zero means no net pitch bend. It isn't a measurement of audible playback speed.
- **Mode** shows Bend, Scratch, Paused scratch, Seek, Grid, TT Off, or a waiting
  or blocked state. Waiting can mean the wheel hasn't settled near target yet.
  Manual scratch means a newly loaded track is ready for top-touch movement.
  Zero bend output during Scratch is normal.

Stale or stopped readings aren't current measurements. If TT is lit but the
wheel won't turn, check the track is loaded, playing and Internal, then confirm
the matching mapping and restart the bridge.

#### Adjust braking and nudge separately

**Jog > Testing: Motorized playback** has separate **Braking (slow down)** and
**Nudge (speed up)** controls:

| Setting | What it changes |
| --- | --- |
| Strength | Amount of pitch bend: 0.1-20x for braking, 0.1-10x for nudging |
| Shape | Response near target speed: above 1 is gentler, below 1 is stronger; range 0.25-4 |

Defaults are **2x braking / 1x nudge**, both Shape 1. The graphs show
physical speed deviation against pitch bend rate, not an audio-speed
percentage. Each graph scales independently; compare its number labels as
well as its shape. The shaded area is the 2.25 RPM quiet band. Below the saved
floor with Vinyl off, Scratch takes over from pitch bend.

To soften nudging, leave braking unchanged, lower only nudge Strength and use
**Save and restart**. Let the wheel settle into **Bend**, then repeat the same
gentle nudge. Change Shape separately to adjust the feel near target speed.
These controls don't change Scratch, motor force, the floor or normal jog
response, and don't need a new mapping.

Keep **Jog > Pitch bend > Speed / Reaction** fixed while comparing Strength
and Shape. Those separate settings change Djay's response and require a new
mapping. Judge the result by listening, not just by the diagnostic numbers.

#### Record a motor diagnostic capture

1. Select **Full controller + LED feedback** and set **Diagnostics > Log detail**
   to **Debug** before starting or restarting the bridge.
2. Wait for **Running**, then click **Start motor diagnostic capture** before
   performing the actions you want to record.
3. Repeat the problem or comparison, noting which deck and controls you used.
4. Click **Stop motor diagnostic capture**, then **Show file** to reveal the
   saved log in Finder.
5. Turn TT off or Stop bridge before closing Djay.

Captures are saved under
`~/Library/Application Support/S4 MK3 Bridge/diagnostics/`. They include
motor settings and timestamped platter activity, and continue if Settings is
closed. **Copy log** copies only the latest 60,000 characters of the displayed
log; the separate capture file isn't subject to that limit. Use **Normal** log
detail for everyday use and **Debug** for a requested capture.

For an ordered, hands-free speed comparison, follow the
[RPM baseline procedure](tt-rpm-baseline.md). The separate
[attended platter diagnostic](platter-diagnostic.md) is a developer tool, run
without the bridge; it doesn't control Djay.

## Screens and feedback

HOTCUE and SAMPLES show **JUMP / LOOP / KEY / RANGE**. STEMS and MUTE show the
mute/solo legend. The selected A/C or B/D deck supplies the corresponding screen.
There is no title, artwork, BPM, time or waveform display.

| Indication | Read it as |
| --- | --- |
| Pulsing deck selector / SELECT | MOVE is choosing jump size, not a connection fault |
| LOOP OFF | No active loop; not the inactive selected length |
| KEY `--` | Djay reports no musical key |
| KEY `?` | Missing/unrecognized key feedback |
| FX SYNC / blinking FX control | A shared knob needs pickup, or its feedback is missing |
| BACKSPIN A/B/C/D | That wheel is in a detected backspin owned by the named deck |
| Red running status icon | Requested screens are blocked/failed; check the Screens tab |

Normal-mode rings advance during playback, follow scratching and freeze when
paused and untouched. Playback animation resumes from the same position after
a scratch. Set its speed with **LEDs > Jog rings > Chase revolution**; it
doesn't show the track's actual position. TT rings follow physical wheel
rotation. Whole-ring loop flashing takes priority until the loop ends.

See the [screen reference](screens.md) and [LED reference](led-feedback.md)
for more detail.

## Settings: what needs saving or a new mapping?

| Change | How to apply it |
| --- | --- |
| LED colors, brightness, meter appearance | Save preferences while stopped, or Save and restart while running; no mapping export |
| Hardware jog tension | Same save/restart actions; no mapping export |
| Jog scratch/bend Speed, Reaction or touch-scratch direction | Save, export with Install Djay mapping..., import and select that new mapping in Djay |
| Backspin detection, threshold, speed increase or motor assistance | Save preferences while stopped, or Save and restart while running; no mapping export |
| TT startup Vinyl, braking/nudge response, floor or motor tuning | Same save/restart actions; no mapping export |
| Screens on/off and NI handover options | Screen controls save immediately; use their live actions |
| A build with changed MIDI bindings/feedback | Install/select its matching mapping under a new filename, then restart the bridge |

Mapping export uses the current Jog editor values. Saving or restarting alone
does not change Djay's selected mapping. Preserve custom maps when importing.

LED and Palette preferences can be imported/exported. These LED exports are
not a backup of every app preference or your custom Djay mapping.
Saved settings survive app updates. Technical files live under
`~/Library/Application Support/S4 MK3 Bridge/`; normal setup does not require
editing them.

## Troubleshooting

| Symptom | First action |
| --- | --- |
| No controls or LEDs | Check power/USB, full profile, running status, and S4 MK3 MIDI Full's selected mapping; rule out a second bridge instance |
| Changed mapping or restarted/reconnected Djay | Stop and start the bridge to reinitialize feedback-dependent startup state |
| Channel level/EQ did not match at startup | Confirm the matching mapping is connected and sending feedback, then restart; check the excluded controls separately |
| FX knob seems ignored or blinks | Check its target deck, then cross the software pickup value; missing feedback needs the matching mapping |
| Jumps are one beat after Shift+MOVE | Confirm JUMP feedback and the matching mapping before relying on size restoration |
| Preview stays playing | Stop Preview with the mouse to resynchronize the held workflow |
| Screens show logos, stay dark, or status is red | Use Settings > Screens and the app's NI handover/recovery controls; run only one bridge app |
| Return to Traktor after a crash | Use Restart Native Instruments screen driver; accept Stop bridge & restore NI driver if prompted |
| USB was disconnected | Reconnect and explicitly start the bridge again |

Brief input gaps suspend tension until fresh movement arrives. If the bridge
stops with an error, keep the Diagnostics log and report the exact message.

If Djay shows a key but the latest S4 build shows `?`, retain the track and
record the deck, title and Djay key before restarting. For a report, also
include app release/build, macOS/Djay versions and selected mapping name.
Copy relevant Diagnostics output and check it for private information.

## Intentional differences and limits

- LOOP and MOVE have independent lengths, rather than Traktor's shared length.
- Shift+SYNC changes global range; it does not lock one deck's tempo fader.
- REV and Slip are independent.
- Channel FX1/FX2 choose manual control targets, not shared audio buses.
- Filter then ON chooses an EQ-mode change; ordinary ON targets Instant FX.
- MUTE selects the STEMS layer rather than acting as a held modifier.
- The main FX ON and REC have the limitations described above.
- EXT switches fixed decks between THRU and Internal. DVS enable and input
  assignments remain manual; this isn't Traktor's factory input workflow.
- Djay may combine very rapid separate jump commands.

For every binding and lower-level detail, use the
[bridge function reference](bridge-function-reference.md). The
[Traktor comparison](traktor-djay-comparison.md) explains the factory
differences.
The [issue checklist](issues.md) tracks remaining work.
