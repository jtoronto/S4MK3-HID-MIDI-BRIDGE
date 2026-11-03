# Jog settings proposal

Approved design preview. The native implementation is in `macos/JogSettingsView.swift`.

## Brief

- Primary action: Save and restart, matching the running bridge's existing action.
- Priorities: base motor force, pitch bend response, optional detailed motor tuning.
- Keep the other existing Jog controls, using the bundled defaults for the preview.
- Omit absolute platter sensitivity and its abandoned position-based mapping.
- Restore defaults resets only the Motorized platters group, including its
  advanced controls. It leaves other Jog groups and the expansion checkbox alone.
- The advanced checkbox is a disclosure control, not a motor-enable preference.
- Remove preference-migration history, low-level message names, testing history,
  and the developer paragraphs at the bottom. Keep necessary apply instructions
  beside the affected mapping controls.
- Use the native utility's system typography, semantic colors, grouped settings,
  checkbox, sliders and number fields. No new app design system or runtime.

Layout: Jog settings - fixed window controls and actions around a long settings
form -> scroll-body-shell
(https://raw.githubusercontent.com/changeroa/StyleGallery/main/patterns/viewport-shell/scroll-body-shell.md);
scroll owner: `.settings-scroll`; anti-patterns: no scrolling window actions,
no nested scrolling inside advanced settings, and no horizontal form overflow.

## Motorized platters inventory

Base force is the main slider outside Advanced settings. Its range is
0-25,000 force units in steps of 50, default 7,000, labeled Weak to Strong.
Braking Strength 10x / Shape 1 and Nudge Strength 3x / Shape 1.6, with response
plots, also remain visible outside Advanced settings.

Advanced settings retain:

- Normal and TT Vinyl startup choices, default on/off.
- Target speed, 33 RPM, and scratch handoff floor, 1%.
- Maximum force 20,000, speed-based force 110, immediate correction 4,000 and
  sustained correction 3,000.
- Running force rise 200,000 units/s, startup duration 200 ms,
  startup force rise 0 and overspeed limit 0.

Descriptions follow `src/jog/platter.rs`, `macos/TTResponseCurveView.swift`
and `examples/jog-config.json`. In particular, speed-based force follows the
target speed, not measured speed; the running rise limit's zero means no
increase, while the startup rise limit's zero means an immediate step.

Other groups retain wheel tension, backspin settings, manual scratch,
untouched pitch bend and the mapping-install shortcut.
Wheel tension defaults to -77 and extra backspin scratch speed to 5.2%.

## Preview behavior

`jog-settings.html` is standalone and has no network or bridge connection.
Advanced disclosure works without JavaScript. With scripting enabled, slider
readouts, response previews and the scoped Restore defaults button work.
The Save and restart and mapping-install buttons are illustrative; they cannot
control the app.

The page follows the host's theme and also supports system light/dark appearance.
Narrow preview reflow is for convenient review, not a proposed change to the
native app's 760-point minimum width.
