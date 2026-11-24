> Historical prototype and integration record, not current app setup.
> Terms such as "current prototype" below refer to the revisions tested at the
> time. Use the [user guide](user-guide.md#screens-and-feedback) and
> [screen-driver guide](screen-driver-handover.md) for the maintained workflow;
> the old manual driver-shutdown procedure is not the normal app procedure.

# Experimental S4 screen support

October 22, 2026. The updated menu-bar app enables screens with its normal
full-controller Start action when **Enable S4 screens** is on, alongside saved
LED and tension preferences. Its new Screens tab and shared menu toggle manage
live screen support, consented NI driver handover and manual recovery; see the
[screen-driver guide](screen-driver-handover.md).
The standalone CLI remains opt-in with `--midi full --screens`. Older published
apps do not enable screens. The info page requires a mapping containing numeric
feedback; the prototype's Info Screen mapping already has it. Existing custom
mappings and saved preferences are not overwritten.
Use the [screen-driver guide](screen-driver-handover.md) for app setup and
recovery. The [integration validation record](app-screen-integration-validation.md)
preserves internal build and acceptance history.

## What is implemented

Current source builds add an optional `FX SYNC` header to both Info and STEMS.
It identifies the affected logical deck, left/right FX strip, parameter knobs
and dry/wet (`W`). Unknown values carry `?`. The underlying page reflows below
the header; it returns to its original layout when the warning clears.
See [pickup setup and behavior](controls-state-sync.md).

- Independent 320 x 240 left/right RGB565 rasters and bulk output on USB
  interface 4, endpoint 0x03. No audio or HID interface is claimed by this
  component. It does not reset the device or detach drivers.
- A full-screen STEMS legend with four instrument columns and mute/solo rows.
  DRUM, BASS, HARM, and VOX appear once; labels use a smooth 16-pixel bold
  sans-serif font. Confirmed ON inverts the tile, OFF leaves it dark,
  and ? distinguishes unknown feedback. A physical hold adds a thick outline
  independently of software state. MUTE remains a STEMS alias.
- The selected A/C or B/D deck owns its displayed feedback. Shift changes the
  upper action to ONLY (exclusive solo); that state requires all four normal
  solo outputs known and exactly one on. The lower row remains ordinary SOLO.
  Mouse changes refresh an existing screen worker without requiring a physical
  control event. Holding a pad through a deck or Shift change does not falsely
  mark the new action pressed.
- In every non-STEMS pad mode, four equal quadrants show JUMP (upper left),
  LOOP (upper right), KEY (lower left), and RANGE (lower right).
  Active loops have a green background and black text; inactive loops say OFF,
  never an old selected length. Unknown feedback says ?; a reported no-key
  value says --. Values use 32-48-pixel fonts selected by measured width,
  with 16-pixel labels. No deck/mode/Shift header, FX/jog row, numbered
  cue/sample legend, or encoder footer surrounds the data.
- Press MOVE to select jump size: SELECT stays visible while the JUMP
  background flashes amber/dark every 500 ms. Its number never disappears.
  Press MOVE again to leave selection. The existing per-deck selection flags
  survive deck changes; STEMS does not flash for a hidden selection latch.
- A separate worker with a latest-state mailbox. Rendering and bounded USB
  transfers and selection-flash timing do not run on the HID/MIDI/motor service
  thread. Unchanged steady pages are not retransmitted; flash phases redraw
  only the selecting info page. Screen failure disables screen output, not controls.
  The first initialized input starts screen service, after native HID/LED
  initialization; bridge construction itself does not claim the display.
- Graceful shutdown replaces active content with a large-text stopped page.
- Offline raster export and an isolated static test-pattern executable.
- Shared anti-aliased typography on info, STEMS/MUTE, stopped and diagnostic
  pages. Pre-rendered glyph coverage is embedded in the executable; no installed
  fonts, Python, or runtime rasterizer is required on the DJ laptop.
  Coverage blends against the actual dark, green, amber or inverted-tile fill.
  Text measurement and compositing remain on the screen worker, with no glyph
  allocation or font rendering added to HID/MIDI/motor processing.

The info page uses real global tempo-range feedback, selected jump-size flags,
active-loop size flags, and current (pitch-adjusted) key feedback.
All 24 numerical key labels were paired with MIDI values in a paused-track
calibration; the decoder does not interpolate unknown bytes. Selected loop
length while inactive remains unavailable through the tested routes and is
not displayed. Titles, artwork, time, BPM, and waveforms are not shown.
Tempo range is not a counter of Shift+Sync presses. See the
[data investigation](screen-feedback-investigation.md) and
[native-pixel layout rules](../src/screens/DESIGN.md). This pass does not add
new control layers, a settings-menu gesture, or powered platter rotation.

### Feedback addresses

All following messages are **from Djay to the `S4 MK3 MIDI Full` destination**:

| Human channel | Type and number | Value |
| --- | --- | --- |
| 1-4, decks A-D | CC80-89 | Selected jump-size flags, 1/8 through 64 beats |
| 1-4, decks A-D | CC90-111 | Active loop-size flags; exact interval names are shared by generator and cache |
| 1-4, decks A-D | CC112 | Current key, decoded from the calibrated wire table |
| 7 | CC8 | Global tempo range; eight verified percentage choices |

Existing deck-channel Note6 feedback reports whether looping is active.
Missing/conflicting size flags remain unknown. A cached loop size never
overrides an inactive loop flag. Legacy meter, loading and playback outputs
keep their addresses and meanings.

## macOS ownership finding

The first live probe failed with exclusive-access error `0xe00002c5`.
IORegistry showed `NIHardwareAgent` owning `Traktor Kontrol S4 MK3 BD@4`.
The separate audio interfaces 0-2 belonged to `usbaudiod`.

Temporarily quitting that agent allowed both static transfers to complete:
153,620 bytes per screen. S4 audio remained enumerated with 8 input and 4
output channels at 48 kHz. The agent was restored after the test. This
distinguishes an ownership conflict from a need to detach the audio driver.
The bridge never quits Native Instruments software automatically.

USB completion does not prove panel appearance. The user subsequently confirmed
the static bars/labels and both normal deck pages during the attended startup
check below. Exhaustive orientation/RGB, readability and audible dropout-free
coexistence checks are not claimed.

## Info-page software verification, October 22

- All 294 Rust tests and 18 Swift tests passed. Formatting, strict locked
  all-target/all-feature Clippy, locked all-target build, shell syntax, plist
  lint, and diff checks passed. The two existing SwiftPM unhandled-file
  warnings remain; no warning suppression was added.
- Real decoder/cache/projection regressions cover every declared interval and
  all four deck routes, unknown/conflicting states, global range, key labels,
  per-deck MOVE selection, and the separate steady STEMS page. The real worker
  advances selection flash phases without any new HID/MIDI publication.
- All 36 native exports have verified PNG signatures and 320 x 240 dimensions.
  Their 15 unique raster groups were visually inspected, including active green
  loops, inactive OFF, both selection phases, fractional lengths, long keys,
  widest range, unknown fields, STEMS states and stopped output. Evidence:
  `.omo/evidence/screens/info-20261022-84f2/`.
- A separate transferable prototype is under
  `dist/S4 Info Screen Development-20261022/`, with signed executables,
  matching generated mapping, and a double-click launcher. Release compilation
  used `CARGO_PROFILE_RELEASE_STRIP=none`. Signature verification, mapping plist
  lint and system-only dynamic-library inspection passed. The normal app,
  previous prototype and published archives were not replaced.
- This is software/render evidence, not a claim of physical readability,
  live all-deck acceptance or audible coexistence of the new layout.

## Smooth-font update, October 22

The user approved the anti-aliased sans-serif comparison and requested the same
font on STEMS/MUTE. S4 Screen Sans is an embedded coverage atlas derived from
the OFL-licensed Liberation Sans Bold 2.1.5, with measured proportional advances
and six native pixel sizes. The old enlarged 8 x 8 font dependency is removed.
Source, generator, font provenance and license are under `src/screens/text/`
and `tools/generate-screen-font.py`.

This is a typography-only change: quadrant/tile geometry, feedback routes,
green active-loop fill, held outlines and selection-flash timing are preserved.
The user subsequently accepted the updated font visually ("Looks good!").
That does not establish exhaustive all-deck feedback or audio-coexistence checks.

- All 299 Rust tests and 18 Swift tests passed, with no ignored or filtered
  Rust tests. Formatting, strict locked all-target/all-feature Clippy, locked
  all-target build, shell syntax, plist lint and diff checks passed.
  Rust diagnostics are clean. The existing two SwiftPM unhandled-file warnings
  remain. Python and shell language servers are unavailable; the generator
  executed successfully and the shell script passed `bash -n`.
- New regressions check embedded-mask metadata, proportional measurement and
  centering, RGB565 coverage endpoints/interpolation, clipping at every edge,
  and smoothing on dark, green, amber and inverted-tile fills.
- The real executables exported 36 page states and both diagnostic patterns.
  All 38 PNGs have native 320 x 240 dimensions; all 17 distinct raster groups
  were visually inspected, with no clipped labels, value overlap, changed
  gutters or lost hold outlines. Long fractions, 12B, plus/minus 75%, both
  selection phases, normal/shifted STEMS, unknown/ON/OFF, held tiles and stopped
  output are included. Evidence:
  `.omo/evidence/screens/smooth-20261022-6fd3/`.
- The pinned generator reproduces the compiled atlas source exactly. Normal
  builds consume static source only, not a font file or developer-installed
  typeface. The app packaging script retains the OFL notice in Resources;
  the normal app/ZIP/DMG were not rebuilt for this prototype.
- The separate `dist/S4 Smooth Screens Development-20261022/` prototype passed
  ad-hoc signature verification, arm64 and system-only library checks, launcher
  syntax and mapping plist lint. Both copied release executables reproduced all
  38 inspected rasters byte-for-byte. Its generated mapping is byte-identical
  to the existing installed `S4 Info Screen-20261022` mapping; no remapping is
  needed for this font update. Djay moved the original imported mapping out of
  the old prototype folder, so comparison used its installed Music/djay copy
  without modifying it. No HID, screen USB, motor or audio activity was started.

## Blank-screen startup correction

The first experimental full bridge replaced the firmware's Traktor / S4 MK3
logos with dark screens. The user confirmed the logos return on power-cycle
even without USB; restarting NIHardwareAgent is not a firmware-splash reset.

After power-cycle, the unchanged screen-only diagnostic displayed its color
bars and labels. Starting HID/MIDI/LED service without screen output preserved
those bars. Thus the existing raw protocol can render on this controller
without an additional backlight command.

The original full bridge started its asynchronous screen worker during bridge
construction, before native HID/LED initialization. Screen startup now waits
for the first processed input report, which follows native initialization in
both runtime paths. No fixed delay or guessed brightness command was added.
From a fresh firmware-logo baseline, the user confirmed both normal DECK A /
DECK B pages are visible with this change on October 21, 2026.

## Legibility-pass software verification

- All 282 Rust tests and 18 Swift tests passed, with no ignored or filtered
  tests. Required formatting, strict all-target/all-feature Clippy, locked
  all-target build, shell syntax, plist lint, and noninteractive diff checks
  passed. SwiftPM retains its existing two unhandled
  file warnings, listed in the historical record below.
- The final executable exported all 31 routing/feedback/press/stopped rasters.
  All PNG conversions have valid signatures and native 320 x 240 dimensions.
  Their 10 unique byte-identical groups match the visually inspected captures:
  no overlap, clipped label, or gutter overflow was found. Local final evidence
  is under `.omo/evidence/screens/legibility-20261021-f3557eb2/`.
- Deterministic regressions exercise real MIDI decoding into selected-deck
  pages, feedback-only state changes, independent sides, press/release ownership
  across deck and Shift changes, conservative exclusive-solo state, unknown/off
  rendering, quiet-page clearing, and feedback-before-HID startup protection.
  Existing worker tests still cover bounded event-gated coalescing and failure.
- These are software/raster passes, not physical playing-distance readability
  or audible-coexistence acceptance of the new layout. No motor trial or
  screen-protocol change was made.

## Historical verification before the legibility pass

- All 274 Rust tests and 18 Swift tests passed after the startup correction.
  The full suite was run with the live bridge stopped: one CLI ownership test
  had correctly refused to reach capture creation while the attended bridge
  was still running. No test was changed for that environment failure.
  Formatting, strict all-target
  Clippy, locked all-target build, shell syntax, plist lint, and diff checks
  passed. SwiftPM still reports the existing two unhandled files:
  `macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`.
- During the attended debug run's Ctrl-C shutdown, both stopped-page transfers
  completed, but the existing input path returned `request S4 input report 1`.
  That read/shutdown error was not changed as part of the screen-startup fix.
- The exact 27-image page matrix was generated from the real local-state
  adapter and visually inspected at 320 x 240. Deck/layer titles, full
  nine-character sample-bank labels, pad titles/actions, Shift, encoder roles,
  GRID/NUDGE, and stopped state fit without overlap. The separate two static
  test patterns were also inspected. Evidence is local under
  `.omo/evidence/screens/`; it is not a photograph of the hardware.
- With NIHardwareAgent temporarily quit, the normal full runtime wrote both
  initial pages and both stopped pages. Its clean Ctrl-C exit reported 180
  real Djay feedback messages, 9 outgoing MIDI messages and 6 LED writes.
  There were no physical control events during this unattended check.
- With NIHardwareAgent still owning interface 4, the runtime logged
  `SCREENS_DISABLED` while MIDI feedback and LED service continued. That
  initial two-second bounded probe exited through the existing no-input
  activity check; idle continuous service then exited cleanly with Ctrl-C.
- The signed portable bridge also wrote both initial/stopped pages while
  running the paced tension service with verified `{"tension":0}`. Ctrl-C
  returned exit 0, accepted final motor neutral, and reported 180 Djay feedback
  messages, 9 outgoing MIDI messages and 4 LED writes. This verifies neutral
  service coexistence, not active-force performance or audible audio quality.
- The portable experimental executables passed ad-hoc signature verification
  and contain no non-system absolute dynamic-library dependencies.
- The rebuilt startup-corrected portable bridge also sent initial/stopped
  pages with the neutral tension configuration, accepted final neutral and
  exited 0 on Ctrl-C. It received 180 Djay feedback messages and sent 9 MIDI
  messages / 4 LED writes. The previous prototype is preserved under
  `dist/previous-screen-startup-20261021.aDDLMi/`.

## Local investigation commands

Run these from the repository root. Stop the menu-bar bridge and any other
bridge CLI first. Keep the S4 powered and connected. Do not run Traktor or Mixxx
at the same time.

1. Build the bridge and screen diagnostic:

   ```sh
   cargo build --locked --all-targets
   ```

2. Identify an NI screen owner:

   ```sh
   pgrep -fl NIHardwareAgent
   ioreg -r -n 'Traktor Kontrol S4 MK3' -l -w 0
   ```

   Under `Traktor Kontrol S4 MK3 BD@4`, check `IOUserClientCreator`.
   If NIHardwareAgent is listed, quit it temporarily:

   ```sh
   pkill -TERM -x NIHardwareAgent
   ```

   This does not disable its login item. Do not kill `usbaudiod`.

   Confirm the agent is gone:

   ```sh
   pgrep -fl NIHardwareAgent
   ```

   Expect no output before continuing. Power-cycle the S4 now, with NI stopped,
   and wait for both firmware logos. October 23 testing showed that SIGTERM can
   leave the panels dark even after later valid frame transfers; resetting after
   NI has stopped clears that state. The packaged app instead uses the
   [guarded screen-preserving handover](screen-driver-handover.md), so its normal
   workflow does not require this manual reset.

3. Send the isolated test patterns:

   ```sh
   target/debug/s4mk3-screen-diagnostic --live
   ```

   Expect two `SCREEN_TRANSFER_OK` lines, one per side, with 153620 bytes.
   Physically check LEFT SCREEN on the left and RIGHT SCREEN on the right,
   top-left white markers, bottom-right yellow markers, and red/green/blue/white
   bars in that order. An error at interface 4 means access failed before pixels
   were sent.

4. Start the experimental full bridge without motor commands:

   ```sh
   target/debug/s4mk3-hid-midi-bridge --midi full --screens --until-stopped
   ```

   Expect `SCREEN_FRAME_WRITTEN` for both sides. If access fails, expect
   `SCREENS_DISABLED`; MIDI/LED service should remain running.
   Keep Djay audio routed directly through the S4.

5. In a second terminal, generate and install a uniquely named mapping:

   ```sh
   info_mapping="/tmp/S4 Info Screen-$(uuidgen).djayMidiMapping"
   target/debug/s4mk3-hid-midi-bridge --generate-mapping "$info_mapping"
   open -a "/Applications/djay Pro.app" "$info_mapping"
   ```

   Accept this new mapping, preserving old custom configurations. In Djay's
   MIDI configuration for **S4 MK3 MIDI Full**, select the new Info Screen
   mapping. Merely restarting the bridge with an older mapping does not
   provide the new numeric outputs.

   Select HOTCUE, then SAMPLES: both should show the same four-quadrant page.
   Change deck A's jump selection to 4/8/16 in Djay: JUMP follows the actual
   value without touching the controller. Press physical MOVE: expect SELECT
   and amber/dark flashing with a continuously readable number. Turn MOVE:
   the chosen size changes immediately. Press again: flashing stops.
   Give C a different jump size, then switch A/C and back; repeat B/D.

   With a paused track loaded, activate an 8-beat loop: upper-right should
   show 8 with green fill and black text. Halve the active loop: it shows 4.
   Exit the loop: it says OFF regardless of the still-selected inactive size.
   Pitch the key up one semitone in Djay: KEY follows the displayed numerical
   key; restore pitch afterward. Eject a test-only track: its old key must not
   remain. Change General Settings > Slider range, then restore the original:
   RANGE follows on both screens. Unknown fields indicate missing feedback,
   not a value guessed from encoder presses.

6. Load paused tracks with four-stem Neural Mix available on the decks being
   tested. Select STEMS, then MUTE on each physical side: both buttons should
   show the same four-column legend. In Djay, turn all four solos off.
   With the mouse, toggle Drum mute on/off. Expect only the relevant MUTE tile
   to invert and change ON/OFF, without touching the controller. If feedback
   has not arrived, expect ?, not a guessed OFF.

7. Hold the upper-left pad (Drum mute): expect its thick press outline.
   Release it: the outline clears while ON/OFF continues to follow Djay.
   Turn Drum solo on, then Bass solo on using the lower-left two pads.
   Both lower SOLO tiles should show ON. Hold Shift: the upper row reads ONLY,
   with no exclusive state on while two solos are active. Press the upper-left
   pad while Shift is held: when Djay confirms only Drum solo, upper Drum ONLY
   and lower Drum SOLO show ON. Release Shift: the upper row returns to actual
   mute states, not guessed audibility.

   Give A and C different mute states in Djay, then select A/C on the left.
   Repeat for B/D on the right; check independent confirmed states, not a deck
   header. Hold a pad while changing deck or Shift: the new action must not
   inherit the old press outline. Release the pad normally.
   Select HOTCUE and SAMPLES: both should return to the info page, with no pad grid.
   Return to STEMS: confirmed state should reappear. Exercise existing controls
   during playback and listen for dropouts; no control or MIDI assignment was
   changed by the layout pass.

8. Press Ctrl-C. Expect the bridge to stop normally and both displays to show
   the stopped page. For a separate tension-coexistence check, repeat steps
   4-8 with `--tension-config examples/tension-config.json` added to step 4.
   That bundled file is neutral; it does not change saved preferences.
   Active-force playing still requires a separate attended acceptance check.

9. Restore Native Instruments' agent when finished:

   ```sh
   open -g -a "/Library/Application Support/Native Instruments/Hardware/NIHardwareAgent.app"
   ```

   Confirm it appears in `pgrep -fl NIHardwareAgent`.

## Portable development binaries

The current smooth-font prototype is
`dist/S4 Smooth Screens Development-20261022/`. Stop the previous bridge,
then double-click its `Start Info Screens.command`. Keep the already-installed
`S4 Info Screen-20261022` mapping selected: this font-only update does not
change it. The folder includes a matching export for a new installation,
both standalone executables, instructions and the font's OFL notice.
No font installation or development toolchain is required on the DJ laptop.

The following paragraph describes the first prototype, not the current folder.
The mapping-only app/ZIP/DMG remain the verified `e1798d3` checkpoint.
The separate local `dist/S4 Screen Development/` directory contains ad-hoc-signed
experimental bridge and screen-diagnostic executables from before the
legibility pass; the updated source/debug binary is not yet in those portable
copies. The directory is not a release or
a menu-bar screen setting. It needs no Cargo on the destination Mac.

After stopping the normal bridge and temporarily quitting NIHardwareAgent as
above, run these from that directory:

```sh
./s4mk3-screen-diagnostic --live
./s4mk3-hid-midi-bridge --midi full --screens --until-stopped
```

Use Ctrl-C to stop the second command, then restore NIHardwareAgent using step 9.
Do not run both bridge executables together. These commands omit hardware tension;
they do not change any saved settings or install a Djay mapping.

## Offline previews

Use fresh output directories; exports refuse to overwrite existing images:

```sh
target/debug/s4mk3-screen-diagnostic --output-dir /tmp/s4-static-preview
target/debug/s4mk3-hid-midi-bridge --generate-screen-previews /tmp/s4-page-previews
```

The first exports both test patterns. The second exports 36 images: all four
decks, three pad modes and both Shift states, six confirmed/unknown/held STEMS
examples, five populated info-page examples, and the stopped page.
Identical unknown pages are expected because redundant deck/mode labels were
removed. PPM files contain the exact quantized pixels
used by the USB encoder; neither command opens USB or MIDI.

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
