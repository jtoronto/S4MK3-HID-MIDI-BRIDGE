# S4 screen reference

This page describes screen support in the full-controller 0.2.0-beta.1 build.
For normal operation, start with the [user guide](../user-guide.md#screens-and-feedback).
Screen support is not included in 0.1.1-beta.3 or older downloads; see
[build availability](../../README.md#download-and-install).

## Start and stop screens

1. Use the full-controller profile and the mapping from the same build. Older
   mappings may lack numeric screen, FX pickup or jump-restoration feedback.
2. Enable **S4 screens** in Settings > Screens or the shared menu-bar toggle.
3. Start the bridge. If NI's background screen driver is running, use the app's
   handover prompt. It can temporarily stop that driver with your consent.
4. Use the screen toggle to release/re-enable screens during the same bridge
   session. MIDI, LEDs and tension continue when only screens are turned off.
5. Stop or quit the bridge when finished. NI restoration follows the saved
   preference and only applies automatically to a driver this app stopped.

Use the [screen-driver guide](screen-driver-handover.md) for consent choices,
blocked-screen recovery and returning to Traktor. No Terminal command or
separate prototype is needed on the DJ laptop.

## Info page: HOTCUE and SAMPLES

Both modes show four large readouts for the selected logical deck:

| Readout | Meaning |
| --- | --- |
| JUMP | Djay's selected beat-jump size. Press MOVE to enter/leave size selection. SELECT and an amber/dark background identify selection mode; the number stays visible. |
| LOOP | Active loop length, with green fill. OFF means no active loop; the inactive selected length is not shown. |
| KEY | Current pitch-adjusted numerical key, such as 11A. A reported no-key value shows --. Missing or unrecognized feedback shows ?. |
| RANGE | Djay's shared tempo-fader range. Shift+Sync cycles this globally, not just for the selected deck. |

Tempo pickup guidance is available in current macOS and Windows source
builds. On Windows, the detent LED works in both bridge modes, but the
display requires **NI Bridge** mode with screens active. Install and select
the mapping bundled with the build if your current mapping predates tempo
pickup feedback. Display-only changes do not require another import.

The tempo detent LED turns red for a persistent mismatch and blue when both
the physical and software faders are centered. Brief gaps while Djay
follows the physical fader do not trigger a warning. Moving away from where
a mismatch began replaces RANGE with a light-red PICKUP tile. The arrow
shows the physical fader and points toward the target tick. The large
percentage is the remaining fraction of full fader travel, not a BPM change.
Returning to your starting position restores RANGE but leaves the LED red
if pickup is still needed. Reaching the target clears both warnings. Djay
still controls pickup. The normal RANGE tile shows the configured setting,
not any temporary expansion Djay applies to a deck.

A/C selects the left screen's deck; B/D selects the right screen's deck.
Jump-selection mode is remembered independently for each deck until exited or
the bridge restarts. A hidden deck's selection does not flash its inactive
selector, and STEMS does not flash for a hidden selection latch.

The key follows its own feedback, not track-loading notifications. This prevents
new keys and the explicit no-key value from being erased during replacement
or eject. The [key validation record](../validation/feedback/key-feedback-order-validation.md) contains
the captured ordering and physical check.

Titles, artwork, time, BPM, waveforms and hotcue/sample pad grids are not shown.
The font is embedded; nothing needs installing on the DJ laptop.

## STEMS page

STEMS selects the Neural Mix layer. MUTE toggles global Neural Mix Mute FX
without changing the screen. The screen has four columns:
DRUM, BASS, HARM and VOX. Pads 1-4 control mute; pads 5-8 control solo in the
same order. With Shift held, the upper row becomes ONLY (exclusive solo);
the lower row remains ordinary SOLO.

Confirmed ON inverts its tile; OFF is dark. For MUTE, ON means the mute action
is active, not that the stem is audible. A thick outline shows a physical hold
independently of software state. A question mark means feedback is unknown.
Exclusive state is shown only when all four solo values are known and exactly
one is on. Mouse changes in Djay also refresh the display.

### Stem FX routing

Press **STEMS** again while in STEMS to open **STEM FX** on that side.
The STEMS button changes from green to purple. The screen shows the
selected deck and the Manual FX slot being edited:

| Pads | Action |
| --- | --- |
| 1–4 | Toggle the stem shown above that pad. Unavailable stems are blank. |
| 5–7 | Select FX 1, FX 2, or FX 3 to edit. |
| 8 | Route the selected FX slot to the whole deck. |

The screen and pads update from Djay feedback, whether the change came
from a pad or the Djay UI. Multiple stems can show ON. `?` means Djay has
not confirmed the route; if the layout is unknown, the route pads do not
send commands. Press **STEMS** again to return to mute/solo.

Install and select the new bundled Djay mapping to get the routing
commands and feedback. Windows **NI Bridge** mode can show the screen;
**Direct HID** provides the same pad controls and LEDs without a screen.

In either three-stem layout the screen labels remain **DRUM / HARM / VOX**.
When Djay is set to Drums / Bass / Melodic, **HARM** indicates Bass and
**VOX** indicates Melodic. The tested MIDI layout fields do not distinguish
those two groupings.

## FX pickup header

An optional **FX SYNC** header appears on Info and STEMS while shared FX knobs
need pickup. It follows the affected logical deck, not necessarily the deck on
the physical side holding that FX strip. **C FX SYNC L13W** means deck C needs
left-strip knobs 1, 3 and dry/wet picked up. R means right strip; a question
mark after a control means its feedback is missing.

The normal page remains underneath and returns to its usual layout when the
warning clears. See [pickup operation](controls-state-sync.md#shared-fx-controls).
This warning's question mark is separate from the musical KEY field.

## macOS ownership finding

NIHardwareAgent can own display interface 4 while the S4's audio interfaces
remain available to Djay. The app manages that verified background driver,
not the audio driver. It refuses the handover while Traktor, Maschine or
Komplete Kontrol is active; close those applications yourself first.

Screens requested but blocked/failed make the running status icon red. This
alone does not mean MIDI, LEDs or tension have stopped. Use **Enable screens -
stop NI driver** to retry without restarting the whole bridge. Use **Restart
Native Instruments screen driver** to return control to NI; accept the app's
stop-and-restore action before opening Traktor.

Turning screens off does not immediately restart NI alongside a still-running
bridge. Normal Stop/Quit completes release before restoration. Force Quit
cannot run normal cleanup; the next launch offers recovery for a recorded
restoration responsibility. See the [ordered recovery guide](screen-driver-handover.md#return-to-traktor-or-recover-after-a-crash).

## Feedback and implementation reference

These messages travel from Djay to the **S4 MK3 MIDI Full** destination on
macOS. Channels below are human, one-based channels. For Windows wire
addresses, see the [Djay action reference](../reference/djay-action-reference.md#generated-s4-mapping-shape).

| Channel | Message | Meaning |
| --- | --- | --- |
| 1-4, decks A-D | CC80-89 | Selected jump-size flags, 1/8 through 64 beats |
| 1-4, decks A-D | CC90-111 | Active loop-size flags |
| 1-4, decks A-D | CC112 | Calibrated current key; 0 means no key |
| 1-4, decks A-D | CC113 | Software tempo-fader position for pickup |
| 7 | CC8 | Global tempo range |

Deck-channel Note6 reports active looping. Missing/conflicting sizes remain
unknown; a cached size never overrides an inactive loop flag. Song loading
feedback does not reset the independent deck key.

The screen component uses 320 x 240 RGB565 frames on USB interface 4, endpoint
0x03. A separate latest-state worker handles rendering, selection flashes and
bounded transfers, outside HID/MIDI/motor processing. It starts only after
initialized input and a successful native LED tick. A screen failure disables
screen output rather than the rest of the controller. Normal shutdown displays
BRIDGE STOPPED.

For CLI users, screens remain opt-in with **--midi full --screens**. Do not run
a separate CLI bridge alongside the menu-bar app. Historical manual ownership
probes are in the [prototype validation record](../validation/screens/screen-prototype-validation.md#local-investigation-commands),
not the normal setup procedure.

## Offline previews

Use fresh output directories; exports refuse existing image paths:

~~~sh
target/debug/s4mk3-screen-diagnostic --output-dir /tmp/s4-static-preview
target/debug/s4mk3-hid-midi-bridge --generate-screen-previews /tmp/s4-page-previews
~~~

The diagnostic exports static patterns. The bridge exports its current matrix
of deck/mode/Shift pages, known and unknown states, held pads, populated Info
examples, FX warnings and stopped output. PPM files contain the pixels used
by the encoder. These export commands open neither USB nor MIDI.

## Validation history

- [Prototype, layout, font and early ownership records](../validation/screens/screen-prototype-validation.md)
- [Initial app integration](../validation/screens/app-screen-integration-validation.md)
- [Guarded NI handover and restoration](../validation/screens/screen-driver-handover-validation.md)
- [Numeric-feedback investigation](../research/screens/screen-feedback-investigation.md)
- [Musical-key load-order fix](../validation/feedback/key-feedback-order-validation.md)
- [Native rendering rules](../../src/screens/DESIGN.md)

## Protocol references

The implementation uses the simpler raw rectangle command from
[Mixxx's renderer](https://github.com/mixxxdj/mixxx/blob/300bd8967c4edaea38af9e32949be510f07b94f9/res/controllers/TraktorKontrolS4MK3Screens.qml):
16-byte `84 00 <screen> 21 ... <width> <height>` header, big-endian RGB565,
4-byte `40 00 <screen> 00` footer. One full frame is 153,620 bytes.
The [bulk descriptor](https://github.com/mixxxdj/mixxx/blob/300bd8967c4edaea38af9e32949be510f07b94f9/res/controllers/Traktor%20Kontrol%20S4%20MK3.bulk.xml)
specifies interface 4 / endpoint 0x03.

[Encdr's descriptor](https://github.com/RufusIbiza/Encdr/blob/1255c485eccbbd63c404bc6af2cecb2ee148593b/encdr/descriptors/ni_kontrol_s4_mk3.json)
independently confirms display dimensions and USB addresses, but uses a different
`0x60` framing variant. That framing must not be mixed with the raw `0x21` blit.
No upstream rendering implementation is copied into this bridge.
