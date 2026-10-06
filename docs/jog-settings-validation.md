# Configurable jog settings validation

Validated on Apple Silicon macOS on 2026-10-06.

The generated mapping defaults to scratch Speed 2.7%, Reaction 150%, and
pitch bend Speed 2.7%, Reaction 17% on all four decks. The native Jog pane
edits these four mapping parameters separately from LED preferences.
Seek and runtime HID movement counts are unchanged.

## Software and package evidence

- All 78 Rust tests and 12 Swift tests passed.
- Rust formatting, strict Clippy, all-target build, shell syntax, app plist
  lint, and diff whitespace checks passed.
- The rebuilt standalone app passed deep strict code-signature verification,
  and its ZIP passed archive integrity verification.
- The bundled executable regenerated a byte-identical bundled mapping.
  All eight scratch/pitch-bend entries contain the requested defaults.
- Hashes of the existing `S4 MK3 Bridge 2` mapping and saved LED preferences
  were unchanged before and after packaging.

## Native use evidence

An isolated native window used the production settings view and controller
with separate QA preferences. Editing and saving scratch 3.4% / 90% and
pitch bend 4.2% / 31%, then exporting through the real Rust generator,
produced those values in all eight mapping entries. Reloading the QA window
retained the saved values.

All five settings tabs were captured at normal and minimum window widths,
in light and dark appearances. Captures and the harness are local evidence
under `.omo/evidence/jog-settings/`, not shipped assets.

The functional review passed. The visual review found no Jog-specific product
defect but requested complete minimum-size evidence. Replacement full-window
light/dark PNGs are 1280 by 1184 pixels and include the status header, all four
fields, mapping guidance, and Reset/Save footer.
The visual reviewer cleared the evidence blocker after inspecting both
replacement captures.

Keyboard editing and saving were exercised. Full keyboard navigation,
prolonged export while a source process runs, and reduced-motion behavior
were not manually verified. Existing app-wide window titles and minimum-size
scroll boundaries in sibling panes were outside this change.

QA did not open the S4, import a mapping into Djay, or change the running
user bridge. Physical playback with the rebuilt mapping remains untested.
Released-spin routing and platter tension remain open issues; these settings
do not establish a fix for either.

The previous app and archive were preserved at
`/tmp/s4mk3-pre-jog-prefs.N4Y0Ye`.
