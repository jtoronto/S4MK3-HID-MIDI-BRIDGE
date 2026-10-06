# S4 MK3 Bridge native UI

## Reference and scope

This is a macOS utility, not a web dashboard. Use native AppKit status menus
and SwiftUI controls, system typography, semantic colors, and system focus
rings. The Apple/minimalist reference contributes quiet utility chrome and
clear hierarchy; its marketing imagery, web fonts, and hero layouts do not
apply. No React, browser runtime, or downloaded assets are needed.

## Structure

- A persistent status-bar item opens Start/Stop, Settings, Check Controller,
  Install Mapping, and Quit actions. The first menu row states actual process
  status. No automatic bridge startup or forced restart.
- Settings has Bridge, Jog, LEDs, Palette, and Diagnostics tabs. Status and Save/Apply
  actions remain fixed; each tab body owns one vertical scroll region.
- Bridge settings expose the actual CLI profile, bounded/continuous duration,
  raw logging, and log level. Mapping installation is a native save dialog that
  generates from the current jog settings through the bundled bridge executable.
- Jog exposes the four mapping-time scratch/pitch-bend speed/reaction values.
  Jog changes affect the exported Djay mapping only: install the generated file
  under a new name and select it in Djay. Bridge restart is not required and
  does not apply jog changes; seek behavior is unchanged. No backspin, motor,
  or screen behavior is changed from this tab.
- LEDs and Palette expose all current JSON preferences. Invalid values must
  produce a visible error, not silently reset user configuration.
- Diagnostics contains a bounded selectable log and the exact launch arguments.

## Tokens and primitives

- System window/control backgrounds, primary/secondary text, accent color.
- System body font, headline labels, and monospaced log/command text.
- Spacing: 8 points within rows, 12 between related controls, 20 around tab
  content and groups. Content starts at 760 by 680 points, minimum 640 by 540;
  native title-bar/toolbar chrome is additional, not deducted from that minimum.
- Native Button, Toggle, Picker, Stepper, TextField, Slider, GroupBox, and TabView.
- Integer rows use the shared NumberRow (TextField plus Stepper) primitive.
  Floating-point percent rows use the shared DecimalRow primitive with the same
  8/12/20 spacing and trailing Stepper. No one-off field layouts.
- State is communicated in text and an SF Symbol, not color alone.
- Status menu is a template SF Symbol with an S4 label; it follows light/dark
  menu-bar appearance automatically. No custom animation in the app itself.
- Keyboard access, native file dialogs, standard window close behavior, and
  accessible text labels are required. Closing Settings does not stop the bridge.

## Jog preferences and mapping export

- `JogPreferences` mirrors `examples/jog-config.json`: `scratch_speed`,
  `scratch_reaction`, `pitch_bend_speed`, `pitch_bend_reaction`. Speeds are
  finite and strictly positive; reactions are integers 0-150. Unknown keys are
  rejected; saved files merge over bundled defaults like LED preferences.
- Bundled defaults live at `Resources/default-jog-config.json`, copied by
  `build-app.sh` from `examples/jog-config.json`. User preferences live at
  `supportURL/jog-config.json`. Missing user files fall back to bundled
  defaults; invalid saved files set `requiresReset` and are preserved on disk
  until explicit reset, matching LED behavior. Reset restores run, LED, and
  jog editors to bundled defaults without touching partially calibrated values
  until Save.
- Mapping export calls `exportMapping(to:)` with a user-confirmed destination.
  The helper writes the current jog settings to an owned temporary config,
  runs the bundled executable with
  `--generate-mapping <fresh-temp-output> --jog-config <temp-config>`
  asynchronously (no `waitUntilExit` on the main actor), then installs the
  fresh output to the requested destination. The destination may already exist
  after the save-panel overwrite confirmation; the temporary output path is
  always fresh because the Rust generator refuses to overwrite. Temporary
  files are cleaned up. Export does not open HID, may run while the bridge is
  running, guards duplicate exports with `isExporting`, reports failures as
  text errors, and never disturbs the source bridge `process` lifecycle.

## Verification surface

Capture every tab at default and minimum window sizes, light and dark.
Check bottom scroll positions for preference groups and long logs. Exercise
stopped, starting/running, stopping, invalid settings, and process failure.
Check the status menu separately. Native macOS has no phone/tablet surface.
Verify mapping bytes, saved preferences, subprocess arguments, and signal
shutdown independently of screenshot appearance.
