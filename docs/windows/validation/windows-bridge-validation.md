# Windows bridge validation

## Task 1: unchanged-source Mac baseline

Captured on November 4, 2026 (UTC) in the `windows-bridge-foundations`
worktree at `7ec18cf8a85a5450d2bb97043cd0bfb34555807d`, before production
source changes. The working tree already contained one modified tracked
document and 40 untracked files. Their paths and SHA-256 content hashes are
recorded in the task 1 source-content manifest. The existing dirty source was
preserved: its hashes and all 41 pre-existing status records were unchanged
after the baseline commands.

The supplied snapshot contains 42 source pairs. An independent SHA-256 check
matched all 42 source and destination hashes against the actual worktree files.
The snapshot and this baseline are distinct: the full manifest also covers
every tracked repository file and relevant dirty or untracked file, while
excluding generated build outputs and caches.

The required hardware-free commands all exited 0:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed |
| `cargo test --locked` | 510 passed; 0 failed; 0 ignored |
| `cargo build --locked --all-targets` | Passed |
| `swift test --package-path macos` | 80 passed; 0 failed |
| `bash -n macos/build-app.sh` | Passed |
| `plutil -lint macos/Info.plist` | Passed |
| `GIT_MASTER=1 git diff --check` | Passed |

SwiftPM emitted its existing warning that `macos/Assets/AppIcon.png` and
`macos/build-jog-diagnostic.sh` are unhandled files. The warning did not fail
the test command and was not changed here. The complete per-command logs,
timestamps, exit codes, and log hashes are in
`.omo/evidence/windows-bridge/20261104-01a1a4a4/task-1/logs/`.

After the successful all-target build, the actual
`target/debug/s4mk3-hid-midi-bridge --help` command exited 0. Its complete
stdout is preserved as `cli-help.stdout.txt`; its timestamp and SHA-256 are in
`cli-help.record.json`. This only verifies the CLI help surface and does not
open HID or MIDI.

Existing tests were run without modification. Rust unit, integration, runtime
replay, and CLI tests exercise their existing deterministic software
boundaries; Swift XCTest exercises the app model, preference handling, process
management, screen controls, diagnostics, and settings behavior. No new tests
were added. No Windows-native execution, Djay interaction, controller,
hardware, MIDI, HID, service, or driver behavior was exercised. The baseline
commands had no missing fixture prerequisites reported; platform-specific
Windows and physical acceptance remain unverified.

The machine and operating-system identity, source inventory, checks, CLI
artifact, dirty-tree preservation comparison, and adversarial review are
recorded in `baseline.json` and `source-content-manifest.json`. This record is
the unchanged-source baseline only; it does not complete task 1's separate
user-action parity coverage audit.
