# TT RPM diagnostic validation

Date: November 2, 2026.

Status: diagnostic instrumentation, offline executable comparisons and portable
package verification complete. The attended hands-free baseline remains for the
user to perform using [the ordered procedure](tt-rpm-baseline.md).

## Scope and artifact

The unpublished app is `dist/rpm-diagnostic-20261102/S4 MK3 Bridge.app`, with ZIP
and DMG alongside it. Version remains `0.2.1-beta.1`, numeric `0.2.1`, build `7`;
identify this diagnostic by its bundle path and hashes, not version alone.
The previous `dist/tt-response-20261102/` package is preserved.

No motor, assistance, bend, mapping or preference values were changed.
No physical controller trial, GUI launch, audio recording or installed-preference
write was performed during preparation. The existing user app/bridge was left
running. No source commit, push or publication was performed.

Debug-only `MOTOR_REGULATOR data={...}` samples provide per-side raw encoder
endpoints, input timing/status, contact/authorization, exact motor window travel
and timing, motor RPM, control mode, requested force and active PI components.
Samples are limited to one per 20 ms per side except mode changes. Inactive PI
state is null. `S4_PLATTER_STATUS` adds the common-clock `host_us` for its
independent approximately 100 ms speed estimate and sent bend rate.

Requested force is the sample-arbitrated request, not applied torque or proof
that a USB write succeeded. Sampling can miss sub-20 ms behavior. Debug logging's
effect on real hardware timing remains unmeasured.

## Source and regression evidence

- `cargo fmt --all -- --check`, strict all-target/all-feature Clippy, all-target
  build, shell syntax, plist lint and diff whitespace checks passed.
- All 68 Swift tests passed, including saved-capture retention of the complete
  new event payload beyond the bounded display buffer.
- The full Rust run with `--no-fail-fast` completed with 494 passing tests and
  two existing environment-dependent failures. A subsequent final
  `cargo test --locked --test tension_cli` passed all three tests, including the
  newly added offline emitter regression. Together these account for 495
  distinct passing tests and the same two environment-blocked tests.
- The two failures are
  `explicit_free_play_startup_reaches_log_creation_before_hid` and
  `wheel_selection_is_admitted_only_by_the_intended_profile`. Both expect an
  invalid output path to fail at capture creation. The running prior bridge
  (PID 86660 during verification) instead caused the unchanged ownership guard
  to refuse the command earlier. A direct invocation with that invalid path
  confirmed the competing-process error before capture/HID access. Neither
  test was edited, skipped or weakened, and the user process was not stopped.
- Rust LSP checks reported no errors on the changed runtime and test files.
  Swift LSP freshness timed out; the full Swift test build and standalone
  `swiftc` app build passed. The optional Biome LSP was unavailable for local
  evidence JSON/JavaScript; no dependency was installed.

The production-payload regression uses real decoded motion to verify window
counts/ticks, motor RPM, PI components, assistance transfer and clearing of
inactive PI state. Temporarily exposing old PI state during assistance made
the test fail at its inactive-state assertion; the mutation was restored.

The real executable's offline regression verifies all four emitted control
modes, sampling cadence, sensor-clock values and common bend timestamps without
opening HID or MIDI. Disconnecting the emitter made its mode-coverage assertion
fail. A zeroed bend timestamp made the existing telemetry test fail. Removing
the app's capture-filter entry made the saved-event assertion fail. All
mutations were restored before final checks.

An initial thread-local tracing fixture lost all events under the parallel
library harness. Its logging dependency was replaced with deterministic payload
assertions and the actual executable boundary above, not sleeps, disabled
parallelism or a skipped test.

## Unchanged-output comparison

Both the development executable and actual packaged release executable were
compared with the preserved prior packaged bridge using the production offline
playback fixture. Trials used a 1% floor, base force 7000 and gains 1 and 5;
remaining configuration fields used the same defaults in each compared pair.

After removing only the added `status.host_us` field, complete replay records
matched: motor reports and their order/timing, MIDI, routing/local state,
configuration, physical RPM and bend output.

| Gain | Identical records | Motor reports | MIDI messages | Regulator samples | Timestamped bend frames |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 17,486 | 12,915 | 4,231 | 2,504 | 251 |
| 5 | 17,996 | 12,915 | 4,741 | 2,504 | 251 |

The candidate ran with Debug enabled while the prior executable ran at Info.
Every emitted measured motor RPM agreed with its exact count/tick window.
Active PI components summed to the unclamped request; the reported regulator
force matched the sample request. All four modes were present and unchanged-mode
sample intervals were at least 20 ms.

This proves unchanged output for those fixtures, not physical RPM stability,
real-time USB timing or audio behavior.

## Portable package

The build used `S4_BRIDGE_OUTPUT_DIR="$PWD/dist/rpm-diagnostic-20261102"` and
`CARGO_PROFILE_RELEASE_STRIP=none`. App signature verification, ZIP integrity
and DMG checksum verification passed. Extracted ZIP and read-only mounted DMG
apps matched the local app byte-for-byte and independently verified signatures.
The temporary extraction directory was removed and the disk image detached.
The generated mapping matched the preceding package byte-for-byte.

| Artifact | SHA-256 |
| --- | --- |
| Bundled Rust bridge | `0052afdb61aa07ec5415cb535c508dc1de1bd989b14c3b58cbef008db77f8fe6` |
| Native app executable | `e579a8cc32226cf7871bd80d7b16a7062300d12c0cb93571bac1a99004c7a289` |
| ZIP | `cd1a90f8911571d17b158dba44e12fb33c16998c4111c69eeec321b66b4880f3` |
| DMG | `12ef08d2e9f16064ec42cb095e7a878104657cafb6de0d6e6a62212d43759f93` |

Local evidence is retained under `.omo/evidence/rpm-diagnostic-20261102/`:
source/full-suite and final diagnostic checks, Swift checks, mutation failure
logs, development/packaged comparisons, the comparison script/configurations,
and package build/integrity/hash output. That local evidence directory is not
a substitute for this repository record or the user-facing baseline procedure.
