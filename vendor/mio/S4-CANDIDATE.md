# S4 pending-write cancellation candidate

This is an unaccepted local candidate, not a new upstream Mio release. Native
Windows tests and physical screen behavior must be established by the operator.
No Windows, NI service, hardware, or SSH execution was performed to prepare it.

## Pristine source and intentional changes

The import is Mio 1.2.4 from the installed crates.io source. Its upstream revision
is `79d1417c94e3a45c8bdd796e5422e22e8c488086`, recorded in the preserved
`.cargo_vcs_info.json`. The original MIT license is retained in `LICENSE`.
The downloaded `mio-1.2.4.crate` SHA-256 matches the application's original lock:

```text
1788edb87fdc09c7e26304471e2f5be8cdefb1b6930d6e3985fc02ff53bf86ee
```

The source was imported with standard `cargo vendor` through a temporary
generated Cargo package depending only on exact Mio:

```sh
cargo new --lib --vcs none --name mio_vendor_import /tmp/mio-import
cargo add --manifest-path /tmp/mio-import/Cargo.toml mio@=1.2.4 --no-default-features
cargo vendor --manifest-path /tmp/mio-import/Cargo.toml --locked --versioned-dirs /tmp/mio-pristine
```

Only the generated `mio-1.2.4` directory was imported. `Cargo.toml.orig` and
`.cargo_vcs_info.json` were copied from the installed package as provenance.
All 71 upstream files, excluding Cargo's `.cargo-ok` installation marker,
matched the installed source byte-for-byte before local changes.
`.cargo-checksum.json` remains the Cargo-generated pristine checksum inventory;
this modified package is a **path patch**, not a directory-source replacement.
Its checksum inventory must not be presented as a checksum of the modifications.

The intentional production changes are the nondefault
`s4-cancel-pipe-writes-on-drop` feature and one feature-gated cancellation call
for `State::Pending` writes in `NamedPipe::drop`. Application `Cargo.toml`
patches Mio to this path and enables the feature through a Windows-only direct
dependency. Main-crate `unsafe_code = "forbid"` is unchanged.
The editor's automatic formatter also reordered imports in `named_pipe.rs`.

The other additions are the private `named_pipe_drop_tests.rs` module, this
record, and the external test watchdog. `Cargo.toml.orig` is unmodified.
No closing flag, callback algorithm, buffer state, reference counting,
OVERLAPPED representation, selector, or IOCP implementation was changed.

## Source ownership proof and limits

The relevant files are under `src/sys/windows/`:

- `named_pipe.rs`, `Inner::maybe_schedule_write`: a successfully submitted
  native write stores its owned buffer and retains an `Arc<Inner>`.
  `State::Pending` is distinct from immediate `State::Ok`.
- `NamedPipe::drop`: cancellation runs while holding the existing I/O mutex.
  It does not take or clear the buffer, change state, release the completion's
  retained Arc, or wait for native completion.
- `write_done`: the actual dequeued completion reconstructs its retained Arc,
  interprets the native result, and releases ownership through existing code.
- `Io::cp` and `Inner::handle`: the retained owner keeps both pipe and port
  alive even if the selector has already been destroyed.
- `selector.rs`, `SelectorInner::drop`: zero-timeout draining dispatches ready
  named-pipe callbacks. It does not await every outstanding native operation.

This mirrors existing read-drop cancellation. If a completion arrives after
the last selector drain, production has no extra drainer: its retained owner
and port can remain unreclaimed. The late-completion tests deliberately retain
a **test-only** port clone so they can verify that storage and handle survive,
dispatch actual native packets, and prove eventual release. Their extra drain
is not a production cleanup mechanism or proof of synchronous leak-free Drop.

The private pipe-then-selector tests use the production destruction order and
report whether late ownership was observed. The read-only case is the existing
policy's comparison control. These exercise Mio's actual reactor teardown;
they do not claim to execute Tokio's entire current-thread runtime destructor.
Tokio 1.54.1 `PollEvented::drop` deregisters then drops its Mio source, and the
screen worker destroys its session before its current-thread runtime.

Cancellation abandons upstream's deliberate graceful flushing of pending
writes. It does not recall bytes already delivered or guarantee cancellation
beats successful completion. A successful short completion can still take
Mio's existing suffix-resubmission branch after Drop. The current bootstrap
uses local message pipes in `PIPE_WAIT`; this candidate does not promise a
general no-resubmission policy for arbitrary byte-stream pipes.

The added Windows FFI is confined to native test fixtures in this dependency.
It creates local-only pipes, transfers unique handles once, and dispatches only
real packets carrying Mio's existing completion references. Miri cannot
establish these native Windows/IOCP behaviors; root must run the native suites
and, for a memory-safety claim, native heap diagnostics.

## Native gates

The private tests first submit an 8 MiB write to a non-reading synchronous
message peer with a 4 KiB input buffer. They require both `State::Pending`
and native `GetOverlappedResult(..., FALSE) == ERROR_IO_INCOMPLETE`. Thus a
synchronously completed operation merely awaiting its IOCP callback cannot
stand in for OS-pending I/O.

Feature-on tests cover:

- cancellation and reclamation without any peer read;
- completion winning cancellation while its callback is still queued,
  including `ERROR_NOT_FOUND` from targeted cancellation;
- forced late completions after selector teardown;
- actual pipe-then-selector teardown with late ownership retained safely.

The read-only teardown control runs with either feature configuration.
The feature-off test proves the full pending message remains drainable after
Drop, preserving upstream graceful behavior.

Application `bootstrap_windows_tests.rs` retains its original immediate-close,
read-order, cancellation, deadline, and drop coverage. Three added 8 MiB
transport cases wait for a real request prefix before cancelling, expiring the
original absolute deadline, or dropping the future. Peer closure is checked
with tiny writes in the opposite direction, without draining the pending
request and accidentally allowing the original Mio implementation to finish.
Their prefix is a wire-level trigger; the private tests own the OS-pending
proof.

Each suite must run as its own process under the supplied external watchdog.
Synchronous peer reads and runtime destructors cannot be interrupted by a
timeout future inside the same current-thread runtime. No test uses sleeps.

## Local source and cross-compilation results

The preparation host is Apple Silicon with Rust/Cargo 1.88.0. These checks
completed without executing Windows code:

- Both feature-on and feature-off private native test configurations passed
  `cargo check --target x86_64-pc-windows-gnu --tests` with their feature sets
  shown below.
- Scoped application formatting, private-test `rustfmt --check`, and manifest
  whitespace checks passed.
- Cargo's target-specific feature graph contains this feature on Windows and
  no active Mio dependency on macOS.
- Comparison against the pristine import confirmed the write callback,
  read/connect callbacks, write-submission ownership, and selector unchanged.
  Main manifest/lock changes exactly preserve their pre-existing content apart
  from the intended path patch, direct dependency, and Mio source/checksum move.

Two checks could not establish a clean result:

- Vendored native-test Clippy reports five unchanged upstream findings:
  `unnecessary_cast` and `zero_ptr` in `iocp.rs`, two `needless_lifetimes` in
  `named_pipe.rs`, and `needless_borrow` in `event_done`. The identical five
  findings were reproduced against the unmodified installed Mio 1.2.4 source.
  They were not suppressed or changed. The feature-off Clippy command was not
  reached after the feature-on command failed.
- Application Windows cross-compilation stops in the unchanged `hidapi` build
  because `x86_64-w64-mingw32-gcc` is not installed. Consequently the new
  application Windows tests still require native compilation by root.

Native execution, mutation runs, heap diagnostics, and physical screen
acceptance remain unperformed. The PowerShell watchdog was not executed on the
preparation host.

## Exact native commands

Run the following PowerShell commands from the repository root on Windows.
They compile first, extract the actual test executable from Cargo's JSON, and
then start only that executable under the watchdog. Stop on a failed command.

```powershell
$ErrorActionPreference = "Stop"

$on = cargo test --manifest-path vendor/mio/Cargo.toml --locked --target-dir target/mio-drop-on --no-default-features --features os-poll,os-ext,s4-cancel-pipe-writes-on-drop --lib --no-run --message-format=json |
    ForEach-Object { $_ | ConvertFrom-Json } |
    Where-Object { $_.reason -eq "compiler-artifact" -and $_.profile.test -and $_.executable } |
    Select-Object -Last 1
if ($LASTEXITCODE -ne 0 -or -not $on) { throw "Feature-on native compilation failed." }
& .\vendor\mio\run-native-tests.ps1 -Executable $on.executable -Filter drop_tests
if ($LASTEXITCODE -ne 0) { throw "Feature-on native tests failed." }

$off = cargo test --manifest-path vendor/mio/Cargo.toml --locked --target-dir target/mio-drop-off --no-default-features --features os-poll,os-ext --lib --no-run --message-format=json |
    ForEach-Object { $_ | ConvertFrom-Json } |
    Where-Object { $_.reason -eq "compiler-artifact" -and $_.profile.test -and $_.executable } |
    Select-Object -Last 1
if ($LASTEXITCODE -ne 0 -or -not $off) { throw "Feature-off native compilation failed." }
& .\vendor\mio\run-native-tests.ps1 -Executable $off.executable -Filter drop_tests
if ($LASTEXITCODE -ne 0) { throw "Feature-off native tests failed." }

$app = cargo test --locked --bin s4mk3-hid-midi-bridge --no-run --message-format=json |
    ForEach-Object { $_ | ConvertFrom-Json } |
    Where-Object { $_.reason -eq "compiler-artifact" -and $_.profile.test -and $_.executable } |
    Select-Object -Last 1
if ($LASTEXITCODE -ne 0 -or -not $app) { throw "Application native compilation failed." }
& .\vendor\mio\run-native-tests.ps1 -Executable $app.executable -Filter bootstrap
if ($LASTEXITCODE -ne 0) { throw "Application bootstrap tests failed." }
```

Use `-Filter ""` with the application executable for the full main-binary suite.
The feature-off graceful control is not a substitute for mutation testing:
root should also temporarily remove only the new Drop cancellation branch in
an isolated copy, keep the feature enabled, and require the pending-write
reclamation and three oversized application cases to fail under the watchdog.
Restore the branch before the accepted run.

The existing physical right-screen result at production `132ad3e5` is prior
operator evidence, not acceptance of this candidate. Root must preserve the
immediate-reply-and-close regression and perform any fresh physical acceptance.
