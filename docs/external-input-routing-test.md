# External input to Djay THRU: attended test

Prepared October 27, 2026. This record contains the original diagnostic
procedure and a subsequent focused user confirmation, not acceptance of every
external input or a new EXT button mapping.

**Current status:** the subsequent user-led channel 4 test below confirms
working THRU mixing with the bridge active: channel fader, FX and mix/cue.
The earlier stopped-bridge headphone observation is retained as historical
evidence, not a current Djay mixing blocker. Physical EXT switching and
feedback-driven lamps were implemented and confirmed later in the separate
[EXT validation](ext-input-validation.md); they were not part of the original
routing test below.

## Question and evidence

Can Djay receive an S4 input and switch it into a deck's audio path without
an ordinary EXT press or a bridge-issued hardware routing command?

The first test deliberately leaves the bridge stopped and uses Djay's mouse
controls. Do not install a new MIDI mapping, start a diagnostic bridge, or
implement EXT switching before this baseline is established. No terminal
command, Rust installation, new app build or timecode record is required.

Two observations are necessary:

1. A known external source reaches Djay's assigned input.
2. Changing only the target deck's DVS mode between Internal and THRU removes
   and restores that source at the output.

A visible THRU label alone, an EXT lamp, MIDI feedback or input activity alone
does not establish the complete path. If the NI background driver remains
running, a pass establishes that this bridge needs no routing command under
that recorded driver state, not that no other host component can affect routing.

## Preparation before the attended run

1. Keep the bridge stopped. Check its status and confirm that no separate
   `s4mk3-hid-midi-bridge` process is running. Close Traktor if it is running.
   Record whether NIHardwareAgent is present; do not stop or restart it for
   this baseline. Do not open a second bridge.
2. Choose one source below. The preferred first case is a line source into
   rear input D, routed to empty Djay Deck 4. Keep the source paused.

   | Source | S4 connection | Hardware type | Djay deck | Input pair |
   | --- | --- | --- | --- | --- |
   | Stereo line source | Rear input D RCA L/R | LINE | 4 | 7-8 |
   | Microphone | Front MIC D | MIC D | 4 | 7-8 |
   | Phono-level turntable | Rear input A RCA L/R and ground | PHONO | 1 | 1-2 |

   Use a source independent of this Mac's Djay output. Do not cable an S4
   output back to its input. A turntable with an enabled line-output preamp
   is a LINE source, not a PHONO source; record which output it supplies.
3. Record the original physical source-type indicator, output levels and
   headphone CUE/MIX position.
   Connect the selected source while it is stopped. Do not press ordinary
   EXT at any stage. Source-type selection with Shift+EXT happens during
   the attended setup below.
4. Open Djay's settings without applying changes. Capture the existing
   Audio Devices and DVS settings, including mixer mode, output pairs,
   Microphone, Recording Input, DVS enablement and deck input assignments.
   Record the target deck's current DVS mode and whether it contains a track.
5. Finish the desktop setup, then tell the user:

   > Ready to test the external input with the bridge stopped. We will mute
   > the outputs, configure one DVS input, and compare Internal with THRU.
   > You will start the source and raise the listening level when asked.
   > No ordinary EXT press, bridge Start, motor or screen test is involved.

   Wait for fresh confirmation that the user is watching. Do not change
   routing, source type, playback or output levels before that confirmation.

If the settings cannot be opened or the source is not connected, preparation
is incomplete. Resolve that before announcing that the live run is ready.

## Attended setup

Execute in order after the watching confirmation. These steps use input D and
Deck 4; for the phono case substitute input A, Deck 1 and pair 1-2 throughout.

1. Have the user lower the physical S4 MASTER level and headphone volume
   before routing changes. Pause every Djay deck and stop any playing
   preview, sampler or looper. Leave the external source stopped.
2. Keep USB connected. Do not power-cycle into standalone mixer mode.
   On physical channel D, use Shift+EXT if necessary to select LINE for
   the RCA source or MIC D for the front microphone. Check the source-type
   indicator. Release Shift; do not press EXT by itself.
3. In **Settings > Audio Devices**, use **Internal** mixer mode. Retain the
   normal S4 Main and Pre-cue output pairs recorded during preparation.
   External mixer mode is separate deck-output routing, not this test.
4. Set the global **Microphone** input to **None** and keep recording off.
   Record, but do not repurpose, **Recording Input**. This excludes the
   microphone mix as a second audible route for the test source.
5. In **Settings > DVS**, enable **Digital Vinyl Control** if it was off.
   Assign **Deck 4** to **Traktor Kontrol S4 MK3**, input **ch 7-8**.
   Preserve the other decks' input assignments and keep their control modes
   Internal. If settings present an Apply button, apply the changes.
   Do not calibrate ordinary music or microphone audio as timecode.
6. Return to the deck view and locate Deck 4's **DVS control-mode menu**,
   then select **Internal / INT**. If the menu or THRU option is unavailable,
   record the exact choices and stop the run as blocked.

   The **THRU** assignment beside the crossfader is not this menu. In the
   four-deck layout, Djay can show that crossfader assignment even when DVS
   is disabled. It is not evidence of external audio routing.
7. Record the target deck's gain, EQ type, EQ, filter, FX, cue, channel fader
   and crossfader assignment. With the mouse, choose ordinary EQ, center the
   three EQ bands and filter, disable its effects, lower its channel fader
   to zero, and set its crossfader assignment to Through. Disable cue on
   the other decks, then enable **Precue on Deck 4**.
8. Have the user set the S4 headphone mix toward CUE and raise headphone
   volume only to a low listening level. Keep MASTER down. The bridge
   remains stopped: moving a physical channel fader will not operate
   Djay's software fader in this baseline.

## Execute the A/B routing test

1. **Internal baseline:** leave Deck 4 paused and in INT. Have the user play
   the external line source, or speak into the microphone.
   Check the DVS input display for a response to the source. For music or
   speech, a valid timecode circle or successful calibration is not expected.
   Record whether input activity is visible and whether the source is audible
   in cue. Raise only Deck 4's software channel fader to 50% and record again.
   The expected negative control is no external source in INT.
2. **THRU:** without touching the S4 EXT button, change Deck 4's DVS mode
   from INT to **THRU**. Have the user stop and restart the source, or
   speak and fall silent. Check that the headphone audio follows it.
   Record the mode, input activity, cue result and left/right behavior.
3. **Main output:** if input activity is established, test main even if cue
   was silent; these are separate observations. Have the user disable Deck 4
   Precue and turn headphone CUE/MIX toward MAIN to listen to the main mix
   without cue leakage. Keep headphone volume low. If speakers are connected,
   have the user raise MASTER to a low listening level while Deck 4's
   software fader remains at 50%.
   Verify that main audio follows the external source. Lower that software
   fader to zero, check whether main audio stops, then restore 50% and
   check whether it returns. Record whether listening used speakers or the
   headphone main mix. Lower MASTER again, return headphone CUE/MIX toward
   CUE and re-enable Deck 4 Precue to check its behavior independently.
4. **Software return:** keep the external source playing and change only
   the DVS mode back to **INT**. Check whether the external audio disappears
   from both main and cue. Do not press the deck's Play button yet.
5. **Repeatability:** select THRU once more and confirm the same external
   audio returns. Pause the source or stop speaking and confirm it becomes
   silent. Return to INT and lower the software fader and MASTER.
6. **Track playback:** stop the external source. Use the target deck's
   existing track, or load a user-selected track if it is empty; record
   that choice. In INT, cue and play it at a low listening level to verify
   normal internal playback remains available. Pause it afterward.

If input activity is absent or ambiguous, do not conclude that an extra
hardware EXT command is required. Stop and record the source, selected
preamp/type, cable, USB pair, DVS enablement and any macOS input-permission
message. A separate capture-input check may be needed to distinguish those
causes. Do not introduce an ordinary EXT press into this baseline as a guess.

## Restore before ending

1. Stop the external source and pause the target deck. Lower MASTER and
   headphone volume before restoration.
2. Restore the recorded DVS mode, input assignments and DVS enablement.
   Restore Audio Devices settings, including Microphone and any mixer-mode
   change; apply if required.
3. Restore the target deck's EQ type, EQ, filter, effects, cue, channel
   fader and crossfader assignment, plus other cue selections changed
   during setup. Preserve any previously loaded track; if the target was
   originally empty, eject only the test track loaded in this procedure.
4. Restore the original S4 source type using Shift+EXT while outputs are
   lowered. Restore headphone CUE/MIX and physical output levels last,
   with playback stopped.
5. Leave the bridge stopped and record the final software and hardware
   state. Do not start it merely to finish this test.

## Results to record

Mark each observation PASS, FAIL, BLOCKED or NOT RUN, with what was actually
heard or seen. Do not turn an expected result into a recorded result.

| Observation | Initial status |
| --- | --- |
| Source, hardware input type, USB pair and NI driver state recorded | NOT RUN |
| Input activity responds without ordinary EXT or bridge Start | NOT RUN |
| INT excludes the external source | NOT RUN |
| THRU sends the source to headphone cue | NOT RUN |
| THRU sends the source to main output | NOT RUN |
| Software channel fader controls main external audio | NOT RUN |
| INT removes external audio; a second THRU selection restores it | NOT RUN |
| INT permits normal track playback afterward | NOT RUN |
| Original settings restored; bridge remains stopped | NOT RUN |

A pass establishes only the tested source, channel, software controls and
recorded host state. It does not establish EXT MIDI actions, physical mixer
control under the bridge, gain/EQ/FX behavior, four simultaneous inputs,
state feedback or LED behavior.

After this baseline passes, repeat it for the other input pairs (A 1-2,
B 3-4, C 5-6, D 7-8). A separate simultaneous-source check is required for
four-channel acceptance; four successful individual checks are insufficient.

## Preparation evidence and current limits

Read-only preflight on October 27 found Djay and the bridge app open,
NIHardwareAgent present and no bridge backend process. The bridge app already
showed **Failed**, with "Bridge exited with status 1." That pre-existing error
was not diagnosed or changed for this test.

The actual Djay window was inspected. Its crossfader-assignment THRU labels
were visible; that is not DVS THRU acceptance. macOS refused both background
and foreground dispatch of the Settings menu, so no settings interaction,
routing change, bridge Start, source selection or audio test was completed.
Foreground-control access was released.

That initial preparation attempt did not complete the live setup. The attended
continuation below supersedes its pending status.

## Attended MIC D continuation

The user connected a microphone to front MIC D and confirmed "Watching"
before the routing changes. The following original settings were read from
Djay for restoration:

| Setting | Original value | Applied test value |
| --- | --- | --- |
| Mixer mode | Internal | Unchanged |
| Main output | S4 ch 1-2 | Unchanged |
| Pre-cue output | S4 ch 3-4 | Unchanged |
| Booth / Recording Input | None / None | Unchanged |
| Global Microphone | S4 ch 7-8 | None |
| Top-bar microphone toggle | User confirmed off during the test | Remained off; enabled only afterward as a separate test |
| Enable Digital Vinyl Control | Off | On |
| DVS inputs, Decks 1-3 | None | Unchanged |
| DVS input, Deck 4 | None | S4 ch 7-8 |
| Deck 4 DVS mode | INT | INT baseline, then user-selected THRU |
| Deck 4 low EQ | 0% | 50% |
| Deck 4 channel volume | 0% | 54% before the listening checks |

Deck 4 was empty; no test track was loaded. Its cue was already enabled,
other EQ bands and filter were centered, and its crossfader assignment was
already Through. The bridge was not started. No ordinary EXT press, NI driver
handover or bridge hardware-routing command was requested or issued.

Both Apply actions reported an accessibility error, but fresh inspection showed
the exact intended settings and disabled Apply/Revert controls afterward.
They were verified as applied rather than retried blindly.

The agent exposed Deck 4's DVS menu, which offered Internal, Relative,
Absolute and Thru. The attempted automated Thru selection left INT unchanged.
The user then took over UI operation; the agent stopped desktop input.

| Observation | Actual result |
| --- | --- |
| INT headphone baseline | PASS: user answered "No microphone audio" |
| THRU headphone listening | PASS: after the user-operated switch, user reported "Yes I can hear myself." |
| DVS input activity display | NOT RUN: no visual signal measurement recorded |
| THRU headphone main-mix and software-fader comparison | Audio audible, but fader-mute criterion FAILED: user selected "Still audible at zero" |
| Return to INT with fader at zero | PASS: user selected "Yes, INT silences it" |
| Second THRU selection and return to INT | PASS: user selected "Yes, both worked" |
| Internal track playback afterward | NOT RUN |
| Restoration of original settings | PASS, user-reported: "Original settings restored" after the supplied restoration steps |

The confirmed contrast establishes audible MIC D routing through the tested
Djay deck mode with the bridge stopped, including repeated software switching.
Headphones were instructed to monitor CUE for the first comparison and MIX
for the fader check. These are user listening observations, not independently
recorded audio or measurements of the physical headphone-control position.
The target deck's cue was enabled, and the actual main/fader listening
instruction did not explicitly turn it off. A cue contribution has not been
demonstrated, but it remains a confound to exclude with an isolated main-output
check. The fader-mute result is a failed criterion under this setup, not a
diagnosis of a Djay bug or proof of which software control should replace it.

The user clarified that the intended workflow is normal vinyl, with the
microphone serving as an available analog test source. Djay's 5.3 release
notes explicitly describe DVS THRU as playing regular vinyl; no timecode
record is required for that use. The remaining design question is how to
provide usable channel mixing for the external audio, not how to get an
INT/THRU source switch. The microphone result does not establish phono
gain/staging on inputs A/B, physical EXT mapping, other input pairs, four
simultaneous sources or gain/EQ/FX behavior.

Installed controller presets contain `mixer.externalLineVolume1..4`, including
`mixer.externalLineVolume4` in the Pioneer DDJ-1000SRT mapping. These are
external-mixer preset identifiers, not proof that they attenuate this
internal-mixer THRU path. No such MIDI action was sent during this baseline.

The user completed restoration using the supplied steps and confirmed
"Original settings restored." Those steps restore Deck 4 INT, channel volume
0% and low EQ 0%; clear Deck 4's DVS input and disable DVS; and restore the
global Microphone to S4 ch 7-8 while leaving the bridge stopped. This is
user-reported restoration, not an independent post-restoration UI inspection.
No test track was loaded or existing track replaced.

The baseline is complete with a source-switching pass and a failed
software-fader mute criterion. It is not acceptance of normal-vinyl mixing
or authorization to implement EXT routing. Physical source-type and
headphone-control positions remain user-managed; their original positions
were not independently captured by the agent.

The running Djay version was not independently captured during this test.
The old 5.6.9 action audit is not a test-version record; later read-only bundle
inspection found 5.6.10, build 67091. See the
[software comparison and unsent support draft](external-input-software-comparison.md)
for the cross-application findings, version limits and proposed isolated-main
follow-up.

## Subsequent bridge-active confirmation

The user later selected DVS THRU on channel 4 with the bridge active and
reported that the channel fader, FX and mix/cue all worked correctly.
This establishes a usable manually selected input-channel workflow for that
tested channel and supersedes the earlier concern that normal channel mixing
was unavailable.

The earlier session deliberately left the bridge stopped. The physical S4
channel fader therefore could not send MIDI through the bridge, but the
earlier failed criterion explicitly concerned the on-screen fader. Bridge
state alone does not establish why that headphone observation differed.
Its cause remains unconfirmed; no runtime fix was made.

This follow-up is user-reported physical acceptance, not an agent-run retest.
The exact global mic state, every gain/EQ control, other input pairs,
phono-stage behavior and four simultaneous external sources were not separately
reported. No physical EXT mapping or DVS mode feedback was implemented.

The [user guide](user-guide.md#external-audio-through-a-deck) now describes the
manual route. No support report alleging a channel-fader defect is warranted
by the superseded observation.

## User-verified USB input assignments

The user subsequently traced the S4 inputs to these stereo USB capture pairs:

| S4 input | Source types | USB input pair |
| --- | --- | --- |
| A | PHONO / LINE | 1-2 |
| B | PHONO / LINE | 3-4 |
| C | LINE / MIC C | 5-6 |
| D | LINE / MIC D | 7-8 |

This confirms the input address map rather than merely inferring it from
documentation. It does not by itself establish every channel's mixer-control
behavior, phono gain/staging or simultaneous four-source operation.

## Attended MIDI auto-enable check

A subsequent isolated Deck 4 MIDI test found that `controlModeThru` did
**not** enable the global DVS checkbox: the deck still showed INT while
DVS was disabled. After the user manually re-enabled DVS, the deck showed
THRU again. The cause of that remembered-mode behavior was not isolated.
The same run confirmed real THRU/Internal mode feedback over MIDI.

See [DVS MIDI validation](dvs-midi-validation.md) for the exact sequence,
captured messages, restoration and limits. This did not implement EXT.

## References

- [Algoriddim 5.3 release announcement](https://community.algoriddim.com/t/announcement-djay-version-5-3-for-macos-ios-android-and-djay-version-5-3-2752-0-for-windows/34745/)
  identifies DVS THRU as the regular-vinyl playback mode.
- [Algoriddim Mac DVS setup](https://help.algoriddim.com/user-manual/djay-pro-mac/hardware/dvs)
  documents DVS enablement, input assignments and per-deck mode controls.
- [Algoriddim audio-device settings](https://help.algoriddim.com/user-manual/djay-pro-mac/settings/audio-devices)
  separates internal mixing, external deck outputs, Microphone and Recording Input.
- [Algoriddim staff on analog audio through DVS THRU](https://community.algoriddim.com/t/analog-vinyl-effects-djay-pro/38038)
  establishes THRU as an external-audio candidate, not S4 acceptance.
- [NI S4 microphone routing](https://support.native-instruments.com/support/solutions/articles/69000879459-how-to-use-a-microphone-with-the-traktor-kontrol-s4)
  documents source selection and the shared MIC D / input D pair 7-8.
