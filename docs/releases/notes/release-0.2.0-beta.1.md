# S4 MK3 Bridge 0.2.0-beta.1

## What's changed since 0.1.1-beta.3

### Live S4 screens

Both screens now follow the selected deck. HOTCUE and SAMPLES show **jump
size, active loop length, musical key and tempo range**. STEMS/MUTE shows
the four stem mute/solo states; hold Shift for the exclusive-solo legend.
Jump-size selection flashes without hiding the number.

Enable or disable screens from the menu bar or **Settings > Screens**.
If Native Instruments' background driver is using them, the app offers to
temporarily stop it and can restore it afterward. Close Traktor, Maschine
and Komplete Kontrol before enabling this handover. You can continue without
screens and keep using controls, LEDs and jog tension.

### Smoother mixing and FX handoffs

- **Startup positions:** channel volumes and high/mid/low EQ now match the
  physical controls once Djay's mapping connects. Set your faders before
  starting. Gain, filter, tempo and crossfader position are not included.
- **FX pickup:** after startup, target changes or software edits, a mismatched
  knob waits for pickup instead of jumping Djay's value when moved. Move it
  through the software position to pick it up. Blinking FX buttons and an
  **FX SYNC** screen header show which controls need attention.
- **One-beat moves:** Shift+MOVE uses the full encoder movement and restores
  your last confirmed jump size afterward. If the size is unknown, it still
  jumps one beat but cannot restore the previous size.

### Updated browsing and cue controls

- **Hold Preview** to audition; turn BROWSER while held to scrub, then release
  to stop. No Shift needed.
- **Shift+BROWSER turn** changes library source; Shift+BROWSER press goes back.
- **Star** switches the library table. **Playlist** marks/unmarks the selection.
- **Shift+CUE** jumps to the song's beginning; normal CUE keeps its cue action.

A new [user guide](https://github.com/jtoronto/S4MK3-HID-MIDI-BRIDGE/blob/main/docs/user-guide.md)
explains these workflows, mapping choices and settings in one place.

## Updating

Quit the old bridge, open **S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg**, and replace
the app in **Applications**. A ZIP is also available. Your saved preferences
are preserved; Diagnostics should show **0.2.0-beta.1 (build 5)**.

**Install and select the mapping exported by the new app.** Save it under a
new filename so your custom mappings remain intact, then stop/start the
bridge after selecting it in Djay. The 0.1.1-beta.3 mapping lacks the new controls
and feedback.

Requires an Apple Silicon Mac running macOS 12 or later, a compatible Djay Pro
version with PRO access, and a powered S4 MK3. The app remains ad-hoc signed,
not notarized; see the
[first-launch instructions](https://github.com/jtoronto/S4MK3-HID-MIDI-BRIDGE/blob/main/README.md#download-and-install)
if macOS blocks it.

## Still a beta

Screens do not show artwork, waveforms or track titles. Powered platter
rotation and playback-position tracking remain unsupported; audio backspin
still ends on touch release. The main FX ON button still duplicates slot 1
rather than bypassing the whole bank. A wheel sensor-time error or USB
disconnect can still stop the bridge; these are separate from automatic
recovery after brief input gaps.
