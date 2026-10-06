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
- Settings has Bridge, LEDs, Palette, and Diagnostics tabs. Status and Save/Apply
  actions remain fixed; each tab body owns one vertical scroll region.
- Bridge settings expose the actual CLI profile, bounded/continuous duration,
  raw logging, and log level. Mapping installation is a native save dialog.
- LEDs and Palette expose all current JSON preferences. Invalid values must
  produce a visible error, not silently reset user configuration.
- Diagnostics contains a bounded selectable log and the exact launch arguments.

## Tokens and primitives

- System window/control backgrounds, primary/secondary text, accent color.
- System body font, headline labels, and monospaced log/command text.
- Spacing: 8 points within rows, 12 between related controls, 20 around tab
  content and groups. Content starts at 760 by 680 points, minimum 640 by 540;
  native title-bar/toolbar chrome is additional, not deducted from that minimum.
- Native Button, Toggle, Picker, Stepper, TextField, GroupBox, and TabView.
- State is communicated in text and an SF Symbol, not color alone.
- Status menu is a template SF Symbol with an S4 label; it follows light/dark
  menu-bar appearance automatically. No custom animation in the app itself.
- Keyboard access, native file dialogs, standard window close behavior, and
  accessible text labels are required. Closing Settings does not stop the bridge.

## Verification surface

Capture every tab at default and minimum window sizes, light and dark.
Check bottom scroll positions for preference groups and long logs. Exercise
stopped, starting/running, stopping, invalid settings, and process failure.
Check the status menu separately. Native macOS has no phone/tablet surface.
Verify mapping bytes, saved preferences, subprocess arguments, and signal
shutdown independently of screenshot appearance.
