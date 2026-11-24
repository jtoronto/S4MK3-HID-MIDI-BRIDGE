# Output writer trace preparation

## Purpose and scope

The diagnostic attributes exact motor/ring report arguments to a user-mode
PID/thread and caller context during one NI acquisition/release. USB ETW
provides additional payload/IRP and stack evidence, without assuming that its
logging-context PID or completion stack is the original writer.

Preparation changes only the new `windows/qa/writer-trace/` tooling, its tests,
this record and owned delivery/evidence files. It does not change the bridge,
motor law, NI protocol messages, mapping, preferences, service or driver.
No NI process attachment, controller opening, MIDI publication, USB recording,
bridge GUI launch or hardware test is part of preparation.

The operator procedure is `windows/qa/writer-trace/README.md`. It uses the frozen
kit 3 host/engine and one B/acquired session, without image publication.
All decks remain paused and both TT modes remain off; there is no track reload
or audio/input-driver change. Keep both wheels untouched. A new watching
confirmation is required before attachment/recording. After normal Stop and
GUI exit, NI collection continues for 35 seconds.

## Attribution boundaries

The agent observes `NtWriteFile`, `NtDeviceIoControlFile` and dynamically loaded
`HidD_SetOutputReport`. It preserves canonical report bytes, original buffer
length/offset, API, handle, precise UTC, PID/TID, direct caller and sampled
accurate backtraces. Sampling keys include API, thread, handle, direct caller
and exact report bytes; later entries reference that stream sample.

NT status pending is not completion. IOCTL buffer matches are candidates,
especially output buffers examined at entry. Nested API hooks can describe
one request repeatedly. No row count is automatically a USB rate or physical
torque measurement.

An empty trace in the selected NI processes does not establish a kernel
writer or rule out another process. The actual USB payload/IRP/origin join
must be demonstrated after an attended recording.

## Tool and package identities

Frida is a portable, unmodified official Windows x64 injector, version 17.27.4.
The official release API was checked independently of the research summary.
Its compressed asset SHA-256 is:

```text
68474826ac22767b8c5711fde83cb2f468deddcfdacaffd1b2fce547419e0a01
```

Uncompressed `frida-inject.exe` SHA-256:

```text
3415601f709ffa1aefa8e5aa8825ec91ea36f94ee5c61e1e8f1441a13f384edc
```

The bundle includes its upstream license and a hash manifest. It requires no
Python/Rust installation or product rebuild. The host DLL remains
`538ab82f98042f46ff461c931b72792a21e9c965a16c2c5777b0457babc466d6`;
the engine remains
`75bf26f55c1cfbe8bde7e25bf7e8e325922f6d044e8905235b4a5bdf8747618c`.

Corrected archive: `dist/S4 Output Writer Trace-20261122-9fbe057e-r2.zip`,
38,943,768 bytes, SHA-256:

```text
9f782e570ee10eccf93f493ba0c49ad0647f1b49f16e798e58fc7ec4c633b21c
```

## Software evidence

- The exact packaged agent passed two real Frida/native ABI fixture scenarios
  with 19 assertions: complete/padded/embedded report bytes, separate API
  layers, PID/thread/caller identity, accepted/failed/pending returns, explicit
  STOP and deadline removal. The native fixture remained usable afterward.
  It invoked harmless exported stubs, not a controller or HID API.
- Changing the temporary bundle's motor report length from eleven to ten
  produced the intended observed-payload failure. The source was unchanged.
- The managed collector's subprocess fixtures passed readiness/PID binding,
  EOF/error/deadline rejection, exact UTF-8/binary evidence preservation,
  stopped acknowledgements, native exit failures and bounded owned cleanup.
- The independent WPR deadline executed an ordinary-file command while the
  foreground waited, and preserved a failed-stop/scoped-cancel outcome.
- The real PowerShell ETW wrapper passed owned save, foreign-instance refusal
  and failed-start cleanup against an ordinary-file CLI fixture.
- Every shipped PowerShell entrypoint parsed. The profile parsed as XML.
  All source files passed whitespace checks and the source-size review.

The macOS injector requires terminal stdin; an initial redirected-pipe fixture
failed `tcgetattr`. The native harness uses a PTY. The Windows injector has a
separate cooked-pipe branch. The agent uses the injector's explicitly flushed
stdout channel for readiness and stop acknowledgements.

The 150-second interactive budget leaves cleanup time. Agents detach hooks at
180 seconds. A C# timer independently initiates cleanup of the exact WPR
instance, even while PowerShell waits for input. Failed saves attempt scoped
cancel and preserve errors; no global WPR stop/cancel is used.

## Delivery and native Windows checks

The corrected ZIP reached the Windows desktop with the exact archive SHA-256
above. The complete folder is
`C:\Users\toron\Desktop\S4 Output Writer Trace-20261122-9fbe057e-ready`.
The native fixture verified its complete file manifest before running.

The original synchronous delivery left a partial extraction after an RPC
failure. An asynchronous worker subsequently completed extraction but its
completion callback expired. After the user reconnected the laptop, no owned
worker remained. The result was inspected rather than replayed blindly.

Actual Windows checks:

- The official injector reported 17.27.4 and exited zero.
- The collector compiled on Windows .NET Framework.
- Installed WPR 10.0.26100 accepted/listed the profile, exit zero. No trace
  session was started.
- The full launcher correctly refused the non-elevated session at its
  administrator gate. No elevation or bypass was attempted. The complete
  elevated `-PrepareOnly` run is therefore unverified.
- An independent ordinary-file fixture captured the exact motor signatures,
  original eleven/95-byte buffer lengths, PID/TID and KERNELBASE caller. It
  opened no device and called no HID API.
- That fixture initially exposed a real collector defect: after clean STOP
  and injector exit zero, stdout/stderr did not reach EOF. A separate native
  probe found open, empty pipes after both injector and target exited; the
  remaining writer's identity was not established.
- Windows draining now observes producer exit before checking queued bytes,
  reads available bytes directly without managed read-ahead, and finishes once
  the exited producer's queue is empty. It does not require unrelated pipe
  writers to close.
- The same file fixture then passed, including clean hook removal, zero
  outstanding/errors/drops, normal target exit and no cleanup error. The full
  managed subprocess regression suite passed on Windows, exit zero, and
  again on macOS after the drain change.

All 24 original preference/log files matched the pre-check hashes. Apps,
mappings and previous recordings were never mutation targets. No NI process
was attached, bridge GUI launched, controller opened or USB recording started.
Service, driver and registry configuration was not changed.

Cleanup receipts in the evidence directory account for owned temporary
staging and the incomplete/superseded first delivery. The corrected complete
folder and ZIP are intentionally retained as stopped deliverables.

## Evidence and primary sources

Compact evidence:
`.omo/evidence/s4-writer-trace-20261122-9fbe057e/`.
It contains official asset metadata, package identity, native/managed/deadline
test logs, the meaningful payload mutation and preparation/cleanup receipts.
The portable package is a deliverable, not an evidence build tree.

Primary references:

- `https://api.github.com/repos/frida/frida/releases/latest`
- `https://github.com/frida/frida/releases/tag/17.27.4`
- `https://frida.re/docs/javascript-api/`
- `https://github.com/frida/frida-core/blob/17.27.4/inject/inject.vala`
- `https://learn.microsoft.com/windows-hardware/drivers/hid/sending-hid-reports`
- `https://learn.microsoft.com/windows-hardware/drivers/usbcon/how-to-capture-a-usb-event-trace`
- `https://learn.microsoft.com/windows-hardware/test/wpt/eventprovider`
- `https://learn.microsoft.com/windows-hardware/test/wpt/wpr-command-line-options`
- `https://learn.microsoft.com/windows/win32/api/namedpipeapi/nf-namedpipeapi-peeknamedpipe`
- `https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-readfile`
