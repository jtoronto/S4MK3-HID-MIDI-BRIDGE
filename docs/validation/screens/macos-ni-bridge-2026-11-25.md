# macOS NI Bridge software verification — 2026-11-25

The Apple Silicon app and bundled bridge built locally. The default release
strip setting caused the macOS loader to reject a stripped compiler plugin;
the previously documented `CARGO_PROFILE_RELEASE_STRIP=none` workaround was
used. `codesign --verify --deep --strict`, `unzip -tq`, and `hdiutil verify`
passed on the app, ZIP, and disk image respectively.

`cargo test --locked`, the all-target build, formatting and strict all-feature
Clippy passed. The native tests use isolated macOS message ports, not the NI
agent: both full-size display writes and deck/tension commands shared a single
acquired test instance. The fixtures also exercised wrong serial, control loss,
negative screen acknowledgment and cancellation while awaiting a reply.

All 82 Swift tests passed, including managed startup, live tension and stop
against a child process fixture. Desktop automation was denied Accessibility
and input access. The user instead tested the stopped app: switching modes
restored the Direct HID controls and retained the NI tension setting. Their
screenshots exposed a missing section: NI mode showed only tension, hiding
the shared jog settings. After the fix in `14e42fd`, the user confirmed the
rebuilt Jog tab showed all four Windows groups: NI tension, Vinyl at startup,
Backspin, and Scratch and pitch bend mapping. This was a settings check, not
a physical platter test.

**Not verified:** a session with NIHardwareAgent and a real S4, physical
screen updates, platter feel, or a full Djay set. The packaged CLI's argument
and config checks ran without hardware, but they did not reach NI acquisition:
the executable requires HID and CoreMIDI first and cannot redirect its NI
connection to the private fixture. The fixture results above therefore do not
prove end-to-end startup, missing-service behavior or shutdown of the packaged
process. For an attended check, stop
Direct HID, select NI Bridge, leave the NI service running, and start. Confirm
both screens update, both platters respond and the Jog tension slider changes
their feel. Stop and confirm both platters release before switching back to
Direct HID; check that the original motor tuning and screen settings remain.
