# S4 MK3 Bridge 0.2.1-beta.1

## What's changed since 0.2.0-beta.1

### Backspins continue after you let go

Previously, lifting your hand ended scratching even when the platter was still
spinning backward. A detected backspin now keeps following the wheel's real
movement after release, so the sound can follow the remaining spin instead of
cutting straight back to normal playback.

Use **Turntable mode** and give the platter a sharp backward flick, then release.
Ordinary scratch releases still return to nudging. Touching the platter again,
stopping it or reversing direction ends the backspin. If you change decks while
it coasts, the backspin stays with the deck that started it.

In **Settings > Jog > Backspin**:

- **Detection** is enabled by default. Raise **Acceleration threshold** if
  ordinary scratches trigger it too easily; lower it if your flicks are missed.
- **Scratch speed increase** adds extra software response only during a
  detected backspin. The default is **5.6%**, giving backspins more travel
  without changing ordinary scratch response. Reduce it if you prefer less.
- **Motor assistance** helps the wheel carry a detected backspin. It is
  experimental, **on by default at 100% strength**. Set **Motor assist strength**
  from **0-100%**, or switch assistance off, to suit your feel. This is independent
  of normal Jog Wheel Tension and software scratch speed. After release,
  assistance fades out within one second; catching or stopping the wheel
  ends it earlier.

Choose **Save and restart** to apply changes while running, or save them before
the next Start. These backspin settings do not need a new mapping export.
The screen shows **BACKSPIN** and its deck while detection is active, helping
you tune the threshold. This is gesture-driven assistance, not a platter that
rotates continuously with playback.

### Browse effects with Shift + knob

Stepping through a long effect list one button press at a time was tedious.
You can now hold **Shift on the FX strip's side** and turn parameter knob
**1, 2 or 3** to browse that slot's effects: **clockwise for next,
counterclockwise for previous**. A larger turn scrolls several effects.
The strip still controls the deck chosen with the channel's FX1 or FX2 button.

The knobs have end stops. Release Shift, reposition the knob, then hold Shift
again to keep browsing. Without Shift, the knob returns to parameter control
and waits until you reach or cross Djay's parameter value before taking over.
Crossing that value resumes parameter adjustment, so repositioning is not a
dedicated silent reset gesture. Dry/wet works as before, and **Shift + effect
button** remains available for a single next-effect step.

### Clearer lights and fewer false FX warnings

REV and FLX no longer stay almost equally bright when off and on. Their idle
backlight now follows JOG's idle intensity, making active reverse or Slip
easier to recognize. Shift is dim at rest and bright while held.

FX pickup warnings also stop briefly reappearing during ordinary knob turns
just because Djay has not returned the updated value yet. A genuine mismatch
after changing targets or editing a value with the mouse still asks you to
move through the software value before taking control.

## Updating

1. Stop the bridge and quit the old app.
2. Open **S4 MK3 Bridge-0.2.1-beta.1-arm64.dmg** and replace the app in
   **Applications**, or use the ZIP. Your saved preferences are preserved.
   Diagnostics should show **0.2.1-beta.1 (build 7)**.
   The new defaults apply to fresh settings and fields absent from an older
   configuration; explicitly saved values, including assistance off, are kept.
3. Use **Install Djay mapping...** in the new app. Save under a new name to
   preserve custom mappings, then select that configuration in Djay.
4. Start the bridge. If you changed mappings while it was running, stop and
   start it again.

**The new mapping is required for Shift + FX knob browsing.** The older mapping
does not include those controls.

See the [user guide](https://github.com/jtoronto/S4MK3-HID-MIDI-BRIDGE/blob/main/docs/user-guide.md)
for the full workflows and settings.

Requires an Apple Silicon Mac, macOS 12 or later, Djay Pro with PRO access,
and a powered S4 MK3. The app remains ad-hoc signed rather than notarized;
follow the [first-launch instructions](https://github.com/jtoronto/S4MK3-HID-MIDI-BRIDGE/blob/main/README.md#download-and-install)
if macOS blocks it.

## Still a beta

The main FX ON button still duplicates slot 1 rather than bypassing the whole
bank. Screens do not show artwork, waveforms or track titles. Playback-driven
platter rotation remains unsupported. A wheel sensor-time error or USB
disconnect can still stop the bridge; automatic recovery from brief input
gaps does not cover those faults.
