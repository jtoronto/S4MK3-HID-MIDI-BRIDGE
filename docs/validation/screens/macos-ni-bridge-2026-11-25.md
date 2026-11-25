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

## Offline managed CLI process check

Run on macOS:

```sh
cargo test --offline --locked --bin s4mk3-hid-midi-bridge ni_process_tests -- --nocapture
```

The macOS Rust test executable launches itself as a child, parses the normal
NI CLI arguments, and runs the real managed runtime. Only the HID/Encdr and
CoreMIDI boundaries are substituted. The native transport uses a unique
private CFMessagePort fixture. No fixture bootstrap or I/O switch is compiled
into the shipping executable.

The child emitted managed `ready` after the fixture acquired the selected
instance and acknowledged the left and right frames. The fixture observed
startup tension `107`, a live stdin update to `173`, then both motor Pause
commands and both ring clears after `S4_BRIDGE_CONTROL 42 STOP`. The child
acknowledged stop ID `42` and exited successfully; both private notification
ports were gone. Rejecting the right frame prevented readiness. An absent
private bootstrap emitted `S4_SCREEN_STATUS 0 failed` and a terminal
`S4_BRIDGE_STATUS` containing `NI message port unavailable`, without readiness
or a raw HID/USB fallback write.

**Not verified:** the packaged, non-test executable's acquisition against
NIHardwareAgent, a real S4, physical screen updates, platter feel, or a full
Djay set. The offline suite excluded the two pre-existing real CoreMIDI
integration tests. For an attended check, stop
Direct HID, select NI Bridge, leave the NI service running, and start. Confirm
both screens update, both platters respond and the Jog tension slider changes
their feel. Stop and confirm both platters release before switching back to
Direct HID; check that the original motor tuning and screen settings remain.
