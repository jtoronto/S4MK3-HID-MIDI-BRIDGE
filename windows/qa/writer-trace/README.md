# Output writer diagnostic

This separate package records who requests motor/ring output during one NI
screen acquisition and release. It uses the frozen **S4 Screen Motor
Diagnostics 3** app. It does not modify that app, mappings, saved tuning,
services or drivers. B acquires a session without publishing screen images;
blank screens are expected.

The portable Frida injector is included; Python, Rust and installation are not
required. Windows administrator access is required. The collector observes
write arguments and return values; it does not replace them or send test
reports to the controller. Instrumentation and stack collection can affect
timing.

## Prepare without starting anything

Extract the entire new package into its own folder. Preserve earlier packages
and recordings. Close any ordinary bridge normally. Leave NI services running.
Open **Windows PowerShell as Administrator**, change to the new folder, and run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\Start-WriterTrace.ps1 -PrepareOnly
```

The default base kit is:

```text
C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 3
```

For another location, supply `-DiagnosticKit 'absolute path'`. Preparation
verifies the package and frozen host/engine, compiles the collector, asks the
installed WPR to parse the profile, snapshots preferences/mapping and prepares
a new B app copy. `PREPARED_STOPPED` means no process was attached, recording
started or GUI opened. A later attended invocation prepares its own fresh run;
do not manually launch the preflight copy.

## One attended run

1. Pause **all Djay decks** and leave **both TT modes off**. Keep the original
   audio driver and saved tuning. No track reload, input-mode change or playback
   is needed. Keep both wheels untouched throughout.
2. After preparation is finished and the operator confirms they are watching,
   run this command in the elevated console:

   ```powershell
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\Start-WriterTrace.ps1
   ```

3. The console first prepares the fresh run, then warns that **either platter
   can move**. Type `YES` only when setup is complete and you are watching.
   This starts attachment to the identified, existing NI processes and arms
   USB ETW. It does not start or restart NI services.
4. The prepared GUI opens **stopped** only after the collectors are ready.
   Confirm it is stopped. Press Enter at the console prompt, then immediately
   click **Start bridge** in that GUI.
5. Wait for the current worker's acquired-case receipt. The console checks its
   identity and connection. Keep playback paused and TT off. The console holds
   the acquired session for 20 seconds; no image output is expected.
6. At the stop prompt, click **Stop bridge**, wait for neutral/release/stopped
   status, then quit that GUI normally. Press Enter in the console after GUI
   exit. Type `STOPPED` only when release and neutral are confirmed.
7. Leave the console and NI processes alone for the following **35-second
   post-release recording**. This is needed because the earlier extra output
   persisted after a successful release receipt. The launcher then removes
   hooks, waits for acknowledgements, saves ETW and writes `result.json`.

The interactive collection budget is 150 seconds. The agents and an
independent WPR deadline begin cleanup by 180 seconds; native command shutdown
has bounded grace. An expired deadline is incomplete evidence. Do not repeat
automatically or advance to A/C/D.

If readiness or collection fails, preserve the entire run and stop/quit the
bridge normally if it remains open. The launcher never force-kills the bridge,
NI service or other applications. It terminates only its own injector/command
processes when their cleanup fails. A failed WPR save attempts cancellation of
that exact owned instance and records both outcomes. If that also fails, the
receipt identifies the remaining uncertain session; do not run a global WPR
cancel command.

## Evidence and interpretation

The new package's `captures` directory contains a unique run with:

- Frozen app/config/source identity, preference and mapping snapshots.
- NI target names, PIDs, executable paths, creation times and SHA-256 hashes.
- `writer-*.stdout` JSON lines and raw stderr: exact canonical bytes, original
  buffer length/offset, API, handle, PID/TID, precise UTC, caller modules and
  accurate user-mode backtraces where available.
- Separate API entry/return records. A pending NT status is not completion;
  accepted calls do not establish successful USB transfer or applied torque.
- `usb.etl` and its stop/cancel receipt: USB provider data/IRP events, stacks
  and process/image metadata, subject to the installed provider's schema and
  trace losses.
- Manual lifecycle markers, bridge trace and final completeness result.

The extra motor signature is:

```text
31 01 e0 fe 00 00 01 e0 fe 00 00
```

`NtWriteFile` and `HidD_SetOutputReport` identify user-mode requests. IOCTL
buffer matches are **candidates**: an output buffer at entry may be uninitialized
receive memory. Nested API hooks can record the same request more than once;
do not sum their rows as USB traffic.
Backtraces are samples keyed by API, thread, handle, direct caller, IOCTL and
report layout. Later entries refer to that stream's sample. Each entry
retains its exact report bytes, PID/TID and direct caller address.

The bridge worker is not attached by this launcher; its existing native-write
trace is retained separately. An absence of matches in the attached NI
processes does not prove a kernel writer or exclude another process. ETW
logging-context PIDs or completion/DPC stacks are not automatically the
original writer. Attribute a USB request only after the payload/IRP and stack
relationship is demonstrated in the actual recording.
