# Attended Windows Encdr screen-pattern validation

On November 7, 2026, an isolated Windows diagnostic used unmodified Encdr
commit `a5414658a69c4330d10b95f2359371dfc8550326` to alternate full-screen
patterns via NIHardwareService. It did not run the production bridge or
exercise its pages, audio, MIDI, LEDs, or motors.

The operator observed independent left red/green and right blue/white
updates in a 16-minute, 9-second run and again after restarting the diagnostic.
Encdr returned success for 1,934 sends in the initial run and 36 after
restart. The operator reported that both screens were still correct when last
checked; this was not continuous observation. The roughly 500 ms spacing
between sends was a diagnostic setting, not a production refresh guarantee.
Both Stop operations exited normally, with no diagnostic processes remaining
and NIHardwareService still running. The last colors remained displayed.
Normal exit does not establish service-side ownership restoration.

The SHA-256 digests of the original captures were
`6011c93470bddb134aad05f146cd87e96198e8a2d46ecad034b8f13672c021ad`
and `b544f39f7b38c34f9b7f8606405a382318430d5fd9da9262f66e87bb34c6a8ab`.
The preflight found one healthy candidate; upstream Encdr can fall back to a
different service-selected device, so strict serial binding was not proved.
The separate hardware-free Rust (4/4), portable supervisor (13/13), and
native JobObject (12/12) checks did not exercise the NI session.

See the [screen supervisor documentation](../windows/screen-supervisor/README.md)
for the screen-only helper protocol. The result does not qualify the ordinary
app's NI Bridge mode, blocked-call recovery, or concurrent hardware behavior.
