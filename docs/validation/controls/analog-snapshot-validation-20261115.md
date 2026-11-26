# Regular analog snapshots - November 15, 2026

## Scope and mechanism

The full-profile input batch now requests report 2 every time, after its
existing button/touch and jog snapshots and bounded interrupt read. It no
longer waits for a report-2 notification to refresh faders and knobs.
The interrupt read still uses its existing 2 ms timeout, and the separate
reader retains its existing minimum admission interval and queue policy.
This is one analog GET per batch, with no catch-up loop or new timer.

The offline [capture analysis](../../windows/research/windows-input-screen-latency-analysis-20261115.md)
found a raw crossfader jump after 268 ms while other input reports continued.
That identified the notification-dependent analog path as a correction target.
The correction removes that dependency; it does not establish why the
controller's notifications were sparse or prove a physical latency improvement.

The batch transport lives in `src/input/batch.rs`, with the existing entry
points retained in `src/input.rs`. Its private device interface separates the
native HID operations from the batch selection and framing logic. Production
and tests execute the same batch function; tests do not open HID or MIDI.
The diagnostic's unobserved initial analog read reuses the existing startup
helper instead of duplicating its call.

Startup parser seeding, MIDI translation, change detection, FX pickup,
addresses and control ownership are unchanged. Minimal/probe mode still uses
interrupt data without GET snapshots. No mapping export or preference change
is required. The later Windows build and deployment are recorded below.

## Deterministic regression

`input::batch::tests::full_batches_refresh_analog_without_analog_notifications`
feeds the real batch selector a device with current analog values and either:

- No interrupt.
- An old button report.
- An old jog report.
- An old analog report.

Each case changes the physical snapshot from 2695 to 3006 and checks complete
button, jog and analog framing and the current analog bytes. It protects the
transport's selection of current state, without claiming a real USB cadence.

With the original notification gate retained, the test failed as intended:

```text
assertion `left == right` failed: interrupt ID 0
  left: [23, 60, 0]
 right: [23, 60, 79]
```

After removing the gate, both batch tests passed in the main bridge and
diagnostic binary. The second test verifies that minimal batches return the
original interrupt bytes, including timeouts, and make no snapshot requests.
Existing framing, observation, decoder and translator tests remain separate
owners of their respective contracts.

## Source checks

The following checks passed in the isolated BLOOP worktree on macOS:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed |
| `cargo test --locked` | 590 tests passed across library, binaries and integration targets |
| `cargo build --locked --all-targets` | Passed |
| `swift test --package-path macos` | 80 tests passed |
| `bash -n macos/build-app.sh` | Passed |
| `plutil -lint macos/Info.plist` | Passed |
| `git diff --check` and new-file whitespace check | Passed |

Language-server diagnostics were clean for the changed Rust files. The
source review keeps framing at the HID boundary and the existing public
entry points; no unsafe code, lint suppression or dependency was added.
The existing large diagnostic reader only loses duplicated call lines.

Swift Package Manager emitted an existing resource warning for
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`. Neither file
nor the Swift package manifest changed; all Swift tests passed.

## Windows build and deployment

Following the user's build/deploy request, the corrected source was built
with the retained Rust 1.88.0 cross-build tools and .NET SDK 10.0.403. Both
build scripts exited 0. The existing screen helpers were reused unchanged.
The package contains 515 files including its runtime manifest.

Native Windows software suites passed: 110 shared, 272 engine and two CLI
tests, with zero failures or ignored tests. Both new analog transport
regressions passed in the real Windows engine test binary. This run did not
open the controller or start the app.

The corrected app and ZIP were deployed to
`C:\Users\toron\Desktop\S4 MK3 Bridge BLOOP`. All 514 installed manifest
payloads and the ZIP matched their expected hashes. The existing shortcut
targets the deployed `App\S4Bridge.exe`. Eleven preference/generated-mapping
files retained their baseline hashes; the BLOOP mapping itself is unchanged.
The previous App, ZIP, mapping and instructions remain under
`Previous builds\20261115-before-analog-tOYf7I` in that desktop folder.

```text
ZIP SHA-256:
b07be50f194c01657f739aa20d250ff11d5aea40f9d3fea0e3e78bec8a351fec
Engine SHA-256:
7ea2212bf88b8f261cb1ee300751f7a6a76d60084e380b3b06e4cf347d29d783
```

Build inputs, logs, native results, deployment verification and cleanup
receipts are retained in the original checkout under
`.omo/evidence/s4-analog-deploy-20261115-tOYf7I/`.

## Verification limits

Physical crossfader response, native Windows request cost and behavior under
sustained load remain unverified. The app is left stopped for the user's
test. Avoid continuously printed raw reports when assessing timing.

No screen/motor diagnostic, tuning change or commit is part of this correction.
The original checkout's unfinished platter work is untouched.
