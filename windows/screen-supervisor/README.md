# Windows screen supervisor

`s4-screen-supervisor.exe --serial SERIAL` starts the sibling
`s4-encdr-screen-worker.exe --serial SERIAL`. SERIAL is 1-64 ASCII letters or
digits. The executable requires Windows and is published with its own .NET
runtime. It has no Encdr dependency and performs no NI, HID, MIDI, motor, driver
or service operations.

The binding interface is
`.omo/evidence/encdr-public-session-a5414658-20261105/production-helper-contract.md`.
The bridge must launch this supervisor inside its outer kill-on-close job,
assigning the supervisor before allowing it to execute. The supervisor creates
an unnamed non-inheritable kill-on-close job for its worker, retains the job and
process handles, and sends ARM (`A5`) only after assignment succeeds. No
breakaway flags are used; the worker job nests inside the bridge job.

Bridge stdin contains versionless records: tag byte, unsigned little-endian
32-bit length, then payload. Tag 1 requires exactly 307,200 bytes (left then
right RGB565 little-endian 320 x 240 images). Tag 2 requires zero bytes and
requests Stop without clearing the displays. Headers and payloads are forwarded
unchanged. The bridge must wait for READY, then wait for FRAME_OK before
submitting another frame. An unsolicited, duplicate or out-of-order worker
response fails the session; acknowledgements are never stored for later frames.

Stdout is only newline-terminated ASCII `READY`, `FRAME_OK`, `STOPPED`, or
`ERROR <detail>`. Worker errors preserve their single-line ASCII detail.
Worker stdout lines are limited to 16,384 characters, with LF or CRLF endings;
incomplete lines, embedded CR, other controls and non-ASCII bytes fail.
Worker stderr is continuously drained and discarded without buffering logs or
writing to protocol stdout. Exit status is zero only after cooperative STOPPED,
worker exit zero, and bounded reap/drain. Errors return status 1.

Connect (including ARM write and READY) has an 8-second deadline. A frame write
and FRAME_OK share 3 seconds. Partial incoming records have 3 seconds after the
first byte; waiting for the first byte while idle has no timeout. Graceful Stop
has 500 milliseconds for the write, STOPPED and worker exit. Stop during a
connect or frame operation aborts the session and returns ERROR; it never queues
a Stop behind a blocked pipe write. Bridge stdout writes have a 3-second
deadline. Each cleanup reap/drain wait is at most 2 seconds, followed by an error
write of at most 250 milliseconds. The deadline waits do not require blocked
pipe calls to honor cancellation.

EOF, loss of the retained parent process handle, worker failure, malformed input
and deadlines terminate only the owned worker job. Assignment failure kills and
reaps the exact unarmed process handle. Forced termination returns ERROR and
does not claim service-side ownership restoration. The bridge's outer job
contains a supervisor that itself stops responding.

## Build and test

Use .NET SDK 10.0.403 with runtime packages 10.0.14 available in the NuGet cache:

```sh
dotnet build windows/screen-supervisor/S4ScreenSupervisor.csproj
dotnet run --project windows/screen-supervisor/Tests/S4ScreenSupervisor.Tests.csproj
dotnet publish windows/screen-supervisor/S4ScreenSupervisor.csproj \
  -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true \
  -o windows/screen-supervisor/publish/supervisor
dotnet publish windows/screen-supervisor/Tests/S4ScreenSupervisor.Tests.csproj \
  -c Release -r win-x64 --self-contained true \
  -o windows/screen-supervisor/publish/tests
```

On Windows, with no controller connected, run the published test executable:

```powershell
.\windows\screen-supervisor\publish\tests\S4ScreenSupervisor.Tests.exe --windows
```

For an external bridge test using `process-wrap`, the published supervisor itself
has a hardware-free mode:

```powershell
.\s4-screen-supervisor.exe --self-test connect-stall --worker-pid-file C:\new-test-dir\worker.pid
```

Modes are `normal`, `connect-stall`, `send-stall`, `stop-stall`, `blocked-write`
and `flood`. This starts the supervisor executable itself as a fake worker, never
the Encdr worker or NI code. The supervisor still uses its production parent
watch, worker job, ARM gate, protocol and deadlines. After assignment and ARM,
the fake worker atomically creates the **new** PID file containing its decimal
PID and LF. The directory must exist and the marker path must not exist.
Subscribe to creation before launching; retain handles to both the supervisor
and marker PID before killing the outer bridge owner. `connect-stall` blocks
before READY (the PID marker is the readiness signal); `blocked-write` emits
READY then stops reading stdin. The other modes use the normal binary protocol.
After killing the owner, require both retained handles to signal within the
bridge test's deadline. The 8-second connect watchdog is still active, so kill
the owner promptly after the marker appears.

The Windows suite additionally launches the actual production supervisor in
this mode and verifies outer-owner death kills both it and the nested worker.
Pass `--windows --supervisor C:\path\s4-screen-supervisor.exe` when the supervisor
is not in the `publish/supervisor` sibling directory. Its optional
`--assignment-gate` self-test argument waits for one `A5` byte before creating any
worker; this permits the native suite to establish outer assignment using
`Process.Start` without racing child creation. `process-wrap` suspended launch
does not need this argument or extra byte.

The portable suite exercises the real framing, reply admission and session
loops over fragmented byte streams, with manually fired deadline timers.
Windows fixtures exercise real jobs, blocked anonymous pipes, stderr pressure,
assignment failure, pre-ARM refusal, parent handles, nested membership and
owner death with armed and unarmed workers. They only launch the test executable.
The nesting fixture gates inner worker creation until outer assignment; the
bridge's suspended process launch remains the bridge integration owner's check.
There are no fixed sleeps; the fixture switches cannot select an NI worker.

Publish the production executable alongside the separately built worker.
The self-contained publish copies the selected runtime's notices as
`DOTNET-LICENSE.txt` and `DOTNET-THIRD-PARTY-NOTICES.txt`; distribute these
alongside the executable and the worker's notices.
Publishing this project does not package or validate the worker.
Native fixture execution and physical display acceptance are separate checks.

Local source and publish checksum manifests can identify a particular build.
They are generated artifacts and are not tracked with the source.
