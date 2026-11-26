# Screen/motor diagnostic build validation

## Scope and result

The user confirmed that the screen/motor problem still occurs on the corrected
analog build and requested implementation without another full attended
reproduction check. The diagnostic build is implemented and delivered stopped.
No controller, motor or screen diagnostic was started.

Work is in `.slim/worktrees/bloop-integration`, based on
`60df4353335f597d644ccb2aba82e2f743407c3c` plus uncommitted changes.
The existing analog correction and BLOOP transport are included.
All 12 protected unfinished platter-work files in the original checkout
matched their retained baseline hashes. No repository commit or push was made.

## Delivered package

The separate Windows desktop folder is:

`C:\Users\toron\Desktop\S4 Screen Motor Diagnostics`

It contains the self-contained App, numbered launchers, preparation/run
scripts, ordered README, START HERE instructions, BUILD-IDENTITY,
source snapshot and a complete portable ZIP. The diagnostic GUI was never
launched. The existing accepted BLOOP GUI remained open as PID 13984,
with no bridge worker or screen helper running at delivery.

Delivery verified all 514 runtime-manifest payloads and all 529 portable
archive entries. Three accepted-build files and five saved preference files
retained their baseline SHA-256 hashes. The generated mapping matches the
accepted BLOOP mapping. No mapping installation or service change occurred.

| Artifact | SHA-256 |
| --- | --- |
| Worker | `2b1c74bd0fc524d1da9251cc6d5fe35fb995a040ca46f73ebe0225488deb2a2b` |
| Mapping | `b7834821c10c2814a46c15273ae2bb50b132a0dc386319004f9cb513f41fb94e` |
| App-only build ZIP | `dba7119e1adb1ac56edab2f02cfb1fa64df58f595826196f9ee2a7bc9c347b41` |
| Delivered portable diagnostic ZIP | `e6b55b4568647cd3e31a059e0c8df37821db5207011a3f068c12169e40071b1b` |
| Unchanged screen supervisor | `a620af7b36efc9c90fd19d0e040b123c34959e8a6a09e7ad70567a36b56315a8` |
| Unchanged screen worker | `c8b121879fa9948c4ab5ae850ad575088e1d94dfe0953de26a0e35bbc3287702` |
| Retained helper source archive | `299b1e9dc239e5b0289989604683607f4978984f97a562731c9371b4513b1c3d` |

The packaging step verified the retained helper/source/license inventory.
The source snapshot records the changed build inputs against the base revision;
documentation closeout follows that snapshot.

## Software verification

All commands completed successfully:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked`: 615 tests passed across the repository suites.
- `cargo build --locked --all-targets`
- `swift test --package-path macos`: 80 tests passed.
- Shell syntax checks for the macOS and Windows build scripts.
- `plutil -lint macos/Info.plist`
- `git diff --check`
- macOS host suite: 120/120 passed.
- Windows engine cross-build, native-test cross-build and self-contained
  Windows host/test packaging.
- PowerShell launcher parsing on macOS and native Windows PowerShell.

Native Windows checks used the exact packaged worker, verified by hash:

| Suite | Result |
| --- | --- |
| Shared Rust tests | 128 passed |
| Production worker tests | 277 passed |
| Worker CLI integration tests | 2 passed |
| Native host software tests | 120/120 passed |

These suites use fake input/output and explicit supervisor fixture modes.
They do not open the controller or establish hardware acceptance.
Focused regressions cover capture-disabled/enabled output equivalence,
capture overflow/export failure, first Stop, native results, sensor and
freshness records, screen cases and failed screen transactions.
The static-frame deadline regression passed in the final suites.

The Swift package retains its pre-existing warnings about two unhandled files:
`Assets/AppIcon.png` and `build-jog-diagnostic.sh`. C# LSP was unavailable;
the warning-as-error compiler and both platform host suites passed.
Two parallel Rust LSP requests were cancelled; completed Rust diagnostics
and the strict all-target compiler checks were clean.

## Evidence and limits

Useful build logs, native stdout/stderr and receipts, deployment receipt,
runtime/helper manifests, dirty-source patch and source snapshot are under:

`.omo/evidence/s4-screen-motor-20261116-RERCLK/`

Windows desktop MCP access timed out during baseline collection. The existing
verified SSH host pin provided the working software-test and delivery route.
This did not require any service, controller or driver recovery.

The capture is bounded and exported after teardown. Loss, missing footer,
failed setup or unconfirmed release prevents a complete-timeline claim.
READY/FRAME_OK are software receipts, not device display acknowledgements;
native output acceptance does not measure torque. Observer overhead and the
physical screen/motor boundary remain for the user's later attended run.
The cause of the reported problem remains unproven.

The [operator runbook](../../../windows/qa/screen-motor/README.md) gives exact
commands, readiness checks, physical actions and Stop/export steps in order.
No full attended reproduction recheck is required before those diagnostics.

## Temporary output disposition

The task-owned macOS build workspace and Windows software-test staging folder
are removed after preserving the receipts. The permanent desktop diagnostic
folder and its ZIP are intentionally retained for the later attended run.
The accepted BLOOP app, previous builds, mappings, preferences and existing
captures are preserved.
