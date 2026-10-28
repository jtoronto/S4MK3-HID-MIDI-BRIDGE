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
- Settings has Bridge, Screens, Jog, LEDs, Palette, and Diagnostics tabs. Status and Save/Apply
  actions remain fixed; each tab body owns one vertical scroll region.
- Bridge settings expose the actual CLI profile, bounded/continuous duration,
  raw logging, and log level. Mapping installation is a native save dialog that
  generates from the current jog settings through the bundled bridge executable.
- The full-controller profile includes the accepted info and STEMS/MUTE screens
  when screen support is enabled, alongside LEDs and saved jog tension.
  Minimal MIDI, HID probe, controller checks and mapping export never enable
  screens. A shared Enable S4 screens preference is available in the Screens
  tab and status menu. It defaults on to preserve existing screen behavior;
  automatic NI driver stopping defaults off and owned restoration defaults on.
  Disabled screens skip automatic NI driver checks. Explicit manual recovery
  remains available even when disabled and regardless of who stopped the agent.
- The Screens tab uses native GroupBox sections for screen status, automatic
  handover and manual NI driver recovery. Settings use the same 8/12/20 spacing,
  semantic text colors and native toggles/buttons as existing tabs.
  The enable toggle and automation preferences save immediately; screen toggles
  affect only the screen worker, never silently restart MIDI/LED/jog service.
- Agent confirmation uses the approved wording: "The Native Instruments screen
  driver software is running in the background and will prevent the screens
  from functioning in this app. We can temporarily stop it and restore it when
  you quit." Actions are Enable screens, Continue without screens and Cancel,
  with an explicit opt-in for future automatic stopping. Automatic restoration
  alone is not permission to stop the agent. Screen handover uses guarded SIGKILL
  for the verified NIHardwareAgent because its SIGTERM cleanup blanks the panels.
  Never disable login registration or terminate NI applications; applications
  using the service must be closed first.
- Record restoration responsibility before stopping NI's service. Restore only
  an agent this app stopped for automatic cleanup; manual restart has no such
  requirement. Wait for screen-release acknowledgement or whole-child exit
  before restoring. Keep NI stopped across Save and restart. Persist recovery
  responsibility across a crash so the user can restore manually next launch.
- Jog's Hardware group exposes the saved Jog Wheel Tension control for both
  physical wheels. Put its title and value above the label-hidden slider; the
  Assist, Neutral (0), and Brake legends share the slider's width. Neutral is
  the physical midpoint of the track, not the midpoint of a labeled control.
- Jog also exposes four mapping-time scratch/pitch-bend speed/reaction values.
  Those mapping changes affect the exported Djay mapping only: install the generated file
  under a new name and select it in Djay. Bridge restart is not required and
  does not apply mapping changes; seek behavior is unchanged. Hardware tension
  applies through Save and restart, independently of the mapping.
- LEDs and Palette expose all current JSON preferences. Invalid values must
  produce a visible error, not silently reset user configuration.
- Diagnostics contains the bundled release version and build number, a bounded
  selectable log, and the exact launch arguments.

## Tokens and primitives

- System window/control backgrounds, primary/secondary text, accent color.
- System body font, headline labels, and monospaced log/command text.
- Spacing: 8 points within rows, 12 between related controls, 20 around tab
  content and groups. Content starts at 760 by 680 points, minimum 760 by 540;
  native title-bar/toolbar chrome is additional, not deducted from that minimum.
  Six native toolbar tabs overflow entirely at 640 points on the tested macOS
  version. The verified minimum width keeps Screens directly discoverable;
  shorter windows still scroll vertically.
- Native Button, Toggle, Picker, Stepper, TextField, Slider, GroupBox, and TabView.
- Integer rows use the shared NumberRow (TextField plus Stepper) primitive.
  Floating-point percent rows use the shared DecimalRow primitive with the same
  8/12/20 spacing and trailing Stepper. No one-off field layouts.
- State is communicated in text and an SF Symbol, not color alone.
- Status menu uses a green SF Symbol while the bridge is running normally,
  red when requested screens are blocked/failed, and a template
  SF Symbol in other states that follows light/dark menu-bar appearance.
  The S4 label and accessible process-state description remain present.
  State text distinguishes "Bridge running" from "Screens blocked"; turning
  screens off removes screen-related red attention. A blocked menu exposes
  Enable screens / stop NI driver and the shared checked screen toggle.
  No custom animation in the app itself.
- Keyboard access, native file dialogs, standard window close behavior, and
  accessible text labels are required. Closing Settings does not stop the bridge.

## Jog preferences and mapping export

- `JogPreferences` mirrors `examples/jog-config.json`: mapping scratch/pitch-bend
  speeds and reactions, runtime backspin options, and `Testing: Motorized playback`
  parameters (`vinyl_enabled_at_startup`, `platter_target_rpm`, `platter_max_force`,
  `platter_base_force`, `platter_force_per_rpm`, `platter_kp`, `platter_ki`,
  `platter_force_rise_per_second`, `platter_startup_force_rise_per_second`,
  `platter_startup_ms`, `platter_overspeed_limit_rpm`). Speeds are
  finite and strictly positive; reactions are integers 0-150; motorized platter
  values are validated integers within their documented ranges. Unknown keys are
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
