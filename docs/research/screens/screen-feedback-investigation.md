# Djay screen-data investigation

October 21, 2026. Installed Djay Pro 5.6.9. The user approved reactive STEMS
first, then loop length, jump length, and musical key output investigation.
Tempo-slider range is also high priority: the useful value is the global
+/- percent range, not tempo-fader position or BPM.

## Evidence levels

An input action, associated model field, embedded output declaration, native
HID display field, and observed MIDI packet are different evidence levels.
Only a packet that follows known changes establishes a usable MIDI output.
The initial probe did not decode these candidates in the bridge. The October
22 info-page implementation now decodes the verified range and size flags,
plus the full current-key calibration described below.
An initial zero is especially weak evidence: guessed paths in the diagnostic
mapping also produced zero snapshots. The transitions below, not those
snapshots, establish the successful fields.

## Live results

After the user restored existing PRO access, the isolated MIDI-only capture
completed normally with 59 packets and no queue overflow. It opened no HID
interface. Playback stayed paused; no bridge, LED, or motor service ran.

| Display value | Result in this session |
| --- | --- |
| Global tempo-slider range | Confirmed for every available choice; original +/-4% restored |
| Selected beat-jump length | Confirmed independently of loop activity on deck A for 4/8/16 beats |
| Original/current musical key | Numeric feedback confirmed on deck A; October 22 follow-up calibrated all 24 labels |
| Selected loop length while OFF | Still no verified source; the tested size flags describe an active loop |

The current source generator and renderer now integrate these fields as
documented in [screen support](../../guides/screens.md). Software regressions cover all four
deck routes. Physical readability and live B/C/D acceptance are not established
by the deck-A probes.

## Tempo position follow-up — November 24, 2026

On macOS Djay Pro 5.6.11 (67174), a separate virtual MIDI mapping reported
Deck A's software tempo position through both the root
`turntable1.effects.tempo.currentValue` output and the embedded continuous
output on `turntable1.speed`. The two 7-bit streams matched for all 95 samples
each. In the first Sync trial, the user saw Deck A's software slider move
and both outputs changed from 127 to 64 without moving the physical fader.
After the synthetic fader was primed at 32, the user reported that repeating
Sync did not move the software slider. The capture contains no second
Sync-related tempo transition. A Sync press alone does not establish a
tempo change or a pickup mismatch.

Synthetic CC 0 input reached both endpoints, 0 and 127, and drove the
software slider directly. It did not exercise physical ghost-fader pickup.
These results do not establish whether the returned target stays fixed
through real pickup or whether a target beyond the physical range can be
distinguished from an endpoint. No HID was opened, and the temporary MIDI
mapping was removed after capture. Raw evidence was archived locally outside
the repository.

### Physical pickup through the bridge

A second attended capture used the real S4 left tempo fader and a temporary
full-profile macOS bridge, with platter-motor, ring and screen writes disabled.
The user moved the software slider to 127 while the physical fader was at 62.
The user confirmed that the on-screen software slider did not move until
the physical fader reached pickup. Both tempo outputs held 127 through
input 123, returned 124 at input 124, then tracked the remaining movement
through 127. The final capture recorded 416 messages without loss, including 314
physical tempo CCs and 49 matched values from each software-position output.

The bridge can calculate direction and distance from the reported software
position and its own fader input. The observed three-step acquisition margin
is from one approach to one endpoint, not a general pickup threshold. Neither
an explicit pickup flag nor out-of-range detection was verified. The temporary
mapping was removed and the production mapping was not modified. Raw evidence
is archived locally in `~/Documents/S4 Bridge Evidence/tempo-physical-2026-11-24/`.

On November 25, the S4 emitted both endpoints (0 and 127). In the repeated
215.3 BPM trial, the already-initialized fader acquired the reported target
at the upper endpoint. The user observed an effective range expansion and
a return to ±4% at the center detent; the global configured-range output
reported ±4% throughout that capture. A first movement after reconnect in
the earlier run immediately changed the target from 127 to 68. Its cause
was not established. The two captures recorded 2,736 and 1,731 packets
without loss. Raw evidence is archived in
`~/Documents/S4 Bridge Evidence/tempo-endpoint-2026-11-25/`.

### Warning presentation

The first hardware trial exposed two red LED false positives: Djay can
report center while the physical CC is still near the detent, and its
feedback can briefly trail the physical fader during a matched sweep. The
bridge now treats physical CC 61–66 as centered only when Djay reports
CC 63–64, and requires a new mismatch to persist for 75 ms. These are
display rules informed by the captures, not changes to Djay's pickup.

The same trial showed that the raw screen path displayed red as blue.
It now packs the logical RGB565 raster as big-endian BGR565 at the USB
boundary. The updated PICKUP tile replaces RANGE while guiding pickup,
and the arrow sits higher to avoid the bezel at an angle. The user
approved the bezel-adjusted macOS build. This does not independently
validate every deck, tempo range, or approach direction.

## Tempo-slider range

The user previously confirmed global range cycling via Shift+Sync, mapped to
`application.tempoSliderRangeNext`. Installed labels and factory mappings also
contain `application.tempoSliderRangeMinus` and `application.tempoSliderRangePlus`.
None of these actions has a metadata-declared range model value.

`Denon DJ SC6000.djayMidiMapping` has embedded `output` dictionaries on the
Minus/Plus Note controls (zero-based channel 0, notes 24/25, with modifier).
Those dictionaries set `midiMinValue=1`; this is not a numeric range output.
No percentage or index conversion is established by that declaration.

The installed executable also contains the property/selector strings
`tempoSliderRange`, `tempoSliderRangeIndex`, `tempoSliderRanges`,
`tempoRangeIs6Percent`, `tempoRangeIs10Percent`, `tempoRangeIs16Percent`,
`tempoRangeIs75Percent`, and the binding string `userDefaults.DJTempoSliderRange`.
Those names were leads, not proof of generic MIDI support.

The working root output is **`userDefaults.DJTempoSliderRange`**, with
`controlType=control`, `midiMessageType=3`, `midiMinValue=0`, and
`midiMaxValue=127`. The diagnostic assigned it CC 12 on human channel 16;
that address is not reserved in the production mapping.

| Actual General Settings range | Returned value |
| --- | --- |
| +/-4% | 5 |
| +/-6% | 8 |
| +/-8% | 10 |
| +/-10% | 13 |
| +/-16% | 20 |
| +/-25% | 32 |
| +/-50% | 64 |
| +/-75% | 95 |

For these integer choices, `round(value * 100 / 127)` recovers the percentage.
The sequence was 4 -> 8 -> 16 -> 75 -> 6 -> 10 -> 25 -> 50 -> 4; each changed
setting was checked in the actual UI and against returned MIDI. The final
`bf 0c 05` packet confirmed restoration. This is real global state, not a
count of Shift+Sync presses, and does not require reading another app's
preferences file.

## Loop and jump lengths

`turntable.autoLoopDurationRotary` and `turntable.skipDurationRotary` have only
`steppedRotary=true` metadata, not a numeric model value.

Discrete loop actions reference `turntable.autoLoop*BeatIntervalActive`;
SC6000 presets declare embedded feedback for several sizes. An earlier
project probe recorded these as active-loop indicators, not selected length
while looping is off, and found no duration feedback from an embedded
`autoLoopDurationRotary` output. This historical negative result must not be
advertised as an inactive-size solution.

The new probe independently reproduced that limitation: deck A's selected
length changed from 8 to 4 with looping OFF without positive size feedback.
Turning the paused deck's 4-beat loop ON emitted `bf 1a 7f` for
`turntable1.autoLoop4BeatIntervalActive`; turning it OFF emitted `bf 1a 00`
while the UI still selected 4 beats. Selecting 8 again while OFF did not
establish a selected-size output. The original 8-beat, inactive state was restored.

### Focused `autoLoopDuration` follow-up

A fresh mapping (`S4 Loop Duration Probe b61c`) was imported on October 21,
2026, with no subsequent edits to its loaded configuration. It assigned
`turntable1.autoLoopDuration` to CC40 with maximum 127 and CC41 with maximum
1, and `turntableSelected.autoLoopDuration` to CC42 with maximum 127, all on
human channel 16 with `controlType=control`.

All three candidates emitted initial zero only. None changed during inactive
loop selection 8 -> 4 -> 2 -> 4 -> 8 -> 16 -> 8, verified in the native UI
both empty and with a paused local track loaded. Positive controls worked:
jump 8 -> 4 -> 8 emitted CC31/32 transitions; enabling an 8-beat loop then
halving it emitted CC27 ON, CC27 OFF / CC26 ON. Disabling emitted CC26 OFF;
the UI returned to the selected 8-beat size. Duration candidates did not
change during the active-loop tests either.

These particular output configurations do not provide a usable selected
length in the installed version. This does not rule out every possible
Djay output route; no deeper reverse engineering was undertaken.

The tested fixture is [`loop-duration-20261021-b61c.plist`](../../fixtures/loop-duration-20261021-b61c.plist),
removed from Djay's user mappings folder and validated with plutil.
Capture `bash_56` / `mon_APSB380QQHKTY5E5` received 16 packets and was manually
stopped after testing; no overflow warning or playback ON packet was observed.
Deck A ended empty, loop 8 inactive, jump 8. Start playback was already OFF
and unchanged; Settings returned to Library and closed. Desktop control was
released. No production mapping, code, HID, LED, or motor behavior changed.

### Selected jump length

Jump-size actions such as `turntable.skipDuration1Beat` reference selection
states such as `turntable.skipping1BeatInterval`; the installed preset scan
did not find corresponding MIDI output assignments. The new root-output probe
did establish these states:

| Root output | Diagnostic CC, human channel 16 |
| --- | --- |
| `turntable1.skipping4BeatInterval` | 31 |
| `turntable1.skipping8BeatInterval` | 32 |
| `turntable1.skipping16BeatInterval` | 33 |

With no active loop, changing the actual native selection 8 -> 4 -> 8 -> 16 -> 8
returned zero for the old size and 127 for the new one. The original 8-beat
selection was restored. The names provide the beat values; no numeric index
or counted encoder detents are needed. Other declared sizes were not live-tested.

## Musical key

The previous inventory described factory key fields too broadly.
`turntableSelected.song.keyIndex` and `turntableSelected.displayKeyIndex` occur
in the Pioneer/AlphaTheta CDJ factory mappings as HID output elements
`originalKeyIndex` and `currentKeyIndex`, not generic MIDI outputs.
Those HID declarations alone did not establish independently routed A/B/C/D
MIDI values or a conversion to musical labels.

The separate MIDI probe now confirms root outputs
`turntable1.song.keyIndex` (original) and `turntable1.displayKeyIndex` (current).
A paused loaded track displayed `2 A` and both outputs returned 71. Pitch +1
changed the actual UI to `9 A` and only the current-key output changed to 81;
the original remained 71. Pitch -1 restored `2 A` and current value 71.
The selected-deck aliases tracked the same changes while A was selected.

These wire values are not MIDI note names; the initial two observations did
not establish a full label table. Independent live B/C/D ownership was not
checked in that probe. The test track was ejected
after restoring pitch, loop, and jump settings; all decks returned to empty.

### Complete numerical-key calibration, October 22

A separate fresh MIDI-only mapping captured current and original key. A paused
minor track starting at 2A was shifted up through all 12 pitch classes, with
each native UI label recorded beside its returned byte, then restored.
A locally generated C-major triad fixture was loaded without playback;
Djay analyzed it as 8B. Its corresponding 12-class sequence and restoration
were captured in the same way. Empty/ejected decks returned 0 and showed -.

| Label | MIDI value | Label | MIDI value |
| --- | --- | --- | --- |
| 8A | 10 | 8B | 5 |
| 3A | 20 | 3B | 15 |
| 10A | 30 | 10B | 25 |
| 5A | 41 | 5B | 36 |
| 12A | 51 | 12B | 46 |
| 7A | 61 | 7B | 56 |
| 2A | 71 | 2B | 66 |
| 9A | 81 | 9B | 76 |
| 4A | 91 | 4B | 86 |
| 11A | 102 | 11B | 97 |
| 6A | 112 | 6B | 107 |
| 1A | 122 | 1B | 117 |

The renderer uses those exact pairs and 0 -> --, not interpolation or MIDI
note naming. Other bytes remain unknown. Original output stayed fixed during
pitch changes; the screen uses current `displayKeyIndex`.
Local evidence is `.omo/evidence/screens/key-calibration-20261022-c7e4.*`
and `key-major-c7e4.wav`. Capture `bash_62` / `mon_N4RD75HDPW3YHGJC` was
manually stopped. Its mapping was removed from Djay, the generated fixture
ejected, pitch restored, Finder closed, and desktop control released.

## Diagnostic artifacts and restoration

The successful capture was `bash_48` / `mon_KSSR2MTZJX4EWGXG`; its complete
raw packet log is retained by the session. The exact active mapping is preserved
in [`fixtures/screen-data-probe-076e0732.plist`](../../fixtures/screen-data-probe-076e0732.plist).
Djay moved the original import into its user mapping folder; that owned test
configuration was moved out again after the probe. Existing custom mappings
were not edited.

Five additional raw/scaled aliases were briefly added on disk, but Djay kept
the already loaded mapping in memory. They were not observed in its output and
were removed from the retained fixture; their silence is not a negative result.
No production MIDI addresses or saved bridge preferences were changed.

The UI's +/-4% range, deck A's 8-beat jump selection, 8-beat inactive loop
selection, and unshifted pitch were restored. The settings tab was returned
to Library and the test-opened settings window closed. The MIDI-only process
ended normally and foreground control was released.

## Sources

- `/Applications/djay Pro.app/Contents/Resources/MidiModelMetadata.plist`
- `/Applications/djay Pro.app/Contents/Resources/MidiModelKeyPath.strings`
- `Contents/Resources/MIDI Mappings/Denon DJ SC6000.djayMidiMapping`
- `Contents/Resources/MIDI Mappings/AlphaTheta CDJ-3000X.djayMidiMapping`
- `/Applications/djay Pro.app/Contents/MacOS/djay Pro` (selector strings only)
- [Version-scoped catalog](../../reference/djay-action-reference.md), including its historical
  root-output union; that union includes HID fields and is not all MIDI.
- [Existing MIDI-only capture and historical progress probe](../../guides/led-feedback.md)

## Repeat this focused probe

From the repository root, use the checked-in diagnostic fixture, not the
release mapping. Keep this terminal open so its path variables survive.

1. Pause all decks and stop any previous `feedback-capture` process using
   `S4 Screen Data Probe 076e0732`. Do not run this during a set. In Djay's
   Settings > General, record the original slider range and Start playback
   setting. Keep Start playback off for the test. Record deck A's loop/jump
   selections and pitch before changing them; use a paused local test track.

2. Build the MIDI-only capture and import a uniquely named copy:

   ```sh
   cargo build --locked --example feedback-capture
   probe_dir="$(mktemp -d /tmp/s4-screen-data-recheck.XXXXXX)"
   probe_mapping="$probe_dir/Screen Data Recheck-$(uuidgen).djayMidiMapping"
   cp docs/fixtures/screen-data-probe-076e0732.plist "$probe_mapping"
   open -a "/Applications/djay Pro.app" "$probe_mapping"
   ```

   Accept installation of that new configuration, not replacement of a working
   mapping. Existing PRO access is required; do not start a trial for the probe.

3. Start the capture before making any test changes:

   ```sh
   target/debug/examples/feedback-capture \
     --port "S4 Screen Data Probe 076e0732" --seconds 600
   ```

   Expect `FEEDBACK_CAPTURE_READY`, then initial `DJAY_FEEDBACK` packets.
   `bf 0c VV` must correspond to the actual General Settings range, with
   `round(VV * 100 / 127)` recovering its percentage. A ready endpoint alone
   is not proof of working output.

4. In General Settings, select each range and compare that returned CC12
   value with the visible percentage, then restore the original range.
   Close Settings. On deck A, use Deck 1 > Beat Jump > Decrease/Increase Length
   to select 4, 8, and 16 beats without jumping playback. Expect CC31/32/33 to
   turn on at 127 for the selected size and off at 0 for the previous size.
   Restore the original jump selection.

5. Use Deck 1 > Looping > Loop Half/Double to select 4 beats with the loop OFF.
   The size flags should not be treated as selected-length feedback. On the
   paused track, toggle Auto Loop ON, then OFF: expect `bf 1a 7f`, then
   `bf 1a 00`, while the selected loop length remains 4. Restore the original
   inactive loop selection.

6. Note the original displayed key. Use Deck 1 > Key > Pitch + once: current
   key CC35 should change while original key CC34 stays unchanged. Pitch -
   restores the original value. Selected-deck aliases CC36/37 are not substitutes
   for independently routed A/B/C/D values; extend the fixture before testing
   other decks.

7. Restore every changed setting and eject a track loaded only for this test.
   Let the bounded capture finish; expect `FEEDBACK_CAPTURE_ENDED` and no
   overflow error. Djay moves the imported file into its mappings folder.
   Move only this uniquely named diagnostic back out:

   ```sh
   mv "$HOME/Music/djay/MIDI Mappings/$(basename "$probe_mapping")" "$probe_dir/"
   ```

   Keep the terminal's packet log with that fixture. Do not edit or remove
   existing custom mappings. No test configuration is installed by the renderer.
