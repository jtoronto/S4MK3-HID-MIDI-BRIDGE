# Windows virtual MIDI shutdown: native investigation

November 3, 2026. Follow-up to
[the feasibility probe](windows-midi-probe-validation.md).

## Result

The shutdown hang is a reproducible self-deadlock in the installed Windows
MIDI service's virtual-device removal path. It is not explained by our endpoint
observers, async probe runner, message handler or WinMM peer: a synchronous
reference owner without any of those also hangs.

The owner waits for `MidiSrvDestroyClient` to return. Inside `midisrv.exe`, that
request holds the client-manager lock exclusively while shutting down a device
pipe. Virtual transport shutdown removes its paired public endpoint, which
calls back into the same client manager and requests shared access to the same
lock, on the same thread. That shared acquisition cannot complete while the
outer exclusive acquisition remains held.

This mechanism is established by native stacks, recovered registers and
disassembly of the installed service. It is more specific than the earlier
finding that an SDK disposal call blocks.

Microsoft's pinned source contains a servicing correction which collects pipes
under the manager lock and shuts them down after releasing it. This is a
service-side correction, not an application-side cleanup-order workaround.
The corrected path was not deployed or tested here. No claim is made that
Kensuke has that correction enabled, or that installing a newer app-local API
alone would fix its service.

## Environment and version correction

- Windows 11 Home 25H2, build 26200.9457.
- Djay 5.6.10 stayed open and responsive, PID 15608.
- NIHardwareService stayed running, PID 4772, start mode Auto.
- MIDI API: app-local Preview 11, package `0.99.92-preview.11`.
- Original probe and generated projection: unchanged.
- MIDI service before reproduction: PID 1332, start mode Manual.

The earlier report used the executable's string version, which is stale
relative to its numeric fixed version:

| Field from `System32\midisrv.exe` | Observed value |
| --- | --- |
| `FileVersion` string | `10.0.26100.7705 (WinBuild.160101.0800)` |
| `ProductVersion` string | `10.0.26100.7705` |
| Numeric fixed file version | `10.0.26100.9444` |
| Numeric fixed product version | `10.0.26100.9444` |

The native dump also identifies product build `10.0.26100.9444`. Do not use the
7705 string alone to decide which service binary or servicing fix is present.
The debugger displayed a different major/minor pair for its file-version
field; the numeric build/revision and product version agree on 26100.9444.

## Controlled reference case

A separate Mac-built executable reused the existing public C#/WinRT projection
and the same app-local native API from the probe folder. It created:

- The same `S4 MK3 Windows Probe` name and `S4PROBE20261103` product identity.
- One active, bidirectional function block on group 0.
- MIDI 1 legacy projection enabled.
- One device-side connection, with the virtual device attached as its plugin.

It installed no endpoint watchers, legacy-port watchers, application message
callbacks or client-in-use callbacks. It launched no WinMM peer and sent no
MIDI. Its main thread performed setup and shutdown synchronously.

A named event controlled shutdown. The event existed before device creation;
the operator signalled it only after an independent Windows PnP query found
both legacy interfaces. There were no synchronization sleeps.

Observed:

```text
REFERENCE_READY pid=2528 umpOnly=False
```

The independent query found all four present instances with status OK:

```text
SWD\MIDISRV\MIDIU_APPPUB_S4PROBE20261103
SWD\MIDISRV\MIDIU_APPDEV_S4PROBE20261103
SWD\MMDEVAPI\MIDIU_APPPUB_S4PROBE20261103_0_0
SWD\MMDEVAPI\MIDIU_APPPUB_S4PROBE20261103_1_0
```

After `Local\S4ProbeReferenceClose` was signalled:

```text
REFERENCE_DISCONNECT
```

Neither `REFERENCE_DISCONNECTED` nor `REFERENCE_CLOSED` followed. The owning
process remained inside `DisconnectEndpointConnection`.

This comparison rules out the removed application machinery as a necessary
trigger. It does not independently test a native C++ caller or another generated
projection; the server-side lock proof below does not depend on that distinction.
Djay remained open throughout, so this is not a test with all consumers closed.

## Native capture method

No Windows development environment or debugger installer was used. A temporary
self-contained .NET console reader was built on the Mac and deployed as files
under `%LOCALAPPDATA%\S4BridgeDiagnostics\shutdown-debug`.

It used Microsoft's `Microsoft.Debugging.Platform.DbgEng` and `SymSrv` NuGet
packages, both `20260319.1511.0`. The reader verified each live target's
executable before attaching: only the MIDI service or an owned probe/reference
under `S4BridgeDiagnostics` was allowed. Djay and NI service were not targets.

The reader used `DEBUG_ATTACH_NONINVASIVE`, waited for initialization, wrote
a full native dump using `.dump /ma`, and detached in cleanup. Target threads
were suspended only for capture. Symbol resolution and all further queries
ran offline against the dumps, with Microsoft's public symbol server and the
API's existing local PDB.

Captures:

| Snapshot | Target | Bytes |
| --- | --- | --- |
| `service-baseline.dmp` | Healthy service PID 1332 | 93,298,578 |
| `reference-blocked.dmp` | Blocked reference PID 2528 | 171,520,760 |
| `service-blocked.dmp` | Same service during that block | 97,559,621 |

The helper's setup validation initially failed because its header parser omitted
variadic COM methods. SDK verification corrected `IDebugControl::Execute` to
slot 66 and `WaitForEvent` to slot 93 before the successful captures. The native
debugger reported unavailable optional extensions (`sos`, `ntsdexts`, `uext`,
`exts`); capture and the built-in stack/disassembly commands completed without
those extensions. No managed-stack result is substituted for the native proof.

## Owner-side wait

The reference's main thread is waiting in the destruction RPC:

```text
Midi2_MidiSrvTransport!CMidi2MidiSrv::Shutdown::<lambda>
Midi2_MidiSrvTransport!CMidi2MidiSrv::Shutdown
Midi2_MidiSrvTransport!CMidi2MidiSrvBidi::Shutdown
Windows_Devices_Midi2!MidiEndpointConnection::DeactivateMidiStream
Windows_Devices_Midi2!MidiEndpointConnection::InternalClose
Windows_Devices_Midi2!MidiSession::DisconnectEndpointConnection
S4MidiProbe!ABI.Windows.Devices.Midi2.IMidiSessionMethods.DisconnectEndpointConnection
```

The matching transport source places `MidiSrvDestroyClient` inside that shutdown
lambda. Thus the owner is waiting for service cleanup, not for our removed
application callback or an unfinished peer process.

## Service-side lock proof

Service thread 5, system thread ID `0x40e8`, has this stack:

```text
ntdll!NtWaitForAlertByThreadId
ntdll!RtlpAcquireSRWLockSharedContended
ntdll!RtlAcquireSRWLockShared
MidiSrv!CMidiClientManager::OnDeviceRemoved+0x12d
MidiSrv!CMidiDeviceManager::DeactivateEndpoint+0x557
MidiSrv!CMidiDeviceManager::RemoveEndpoint+0x70
Midi2_VirtualMidiTransport!CMidi2VirtualMidiEndpointManager::DeleteClientEndpoint+0xe9
Midi2_VirtualMidiTransport!MidiEndpointTable::OnDeviceDisconnected+0x3a9
Midi2_VirtualMidiTransport!CMidi2VirtualMidiBidi::Shutdown+0x1a4
MidiSrv!CMidiDevicePipe::Shutdown+0x91
MidiSrv!CMidiClientManager::DestroyMidiClient+0x583
MidiSrv!MidiSrvDestroyClient+0x256
```

Offline disassembly and unwound nonvolatile registers establish the lock
identity and scope:

| Evidence | Value |
| --- | --- |
| Manager/lock pointer retained in outer `DestroyMidiClient`, RSI | `0x000001ee1566a530` |
| Manager/lock pointer used by re-entered `OnDeviceRemoved`, R14 | `0x000001ee1566a530` |
| Exclusive acquisition in `DestroyMidiClient` | `0x00007ff782cd6564` |
| Device shutdown call inside that scope | `0x00007ff782cd6906`, return at `+0x583` |
| Later exclusive-release instructions | `0x00007ff782cd6a26`, `0x00007ff782cd6ab8` |
| Re-entered shared acquisition | `0x00007ff782cdb926` |

The same thread is therefore trying to acquire shared access to the lock whose
exclusive scope surrounds its current device shutdown. Other destruction and
protocol-worker cleanup threads also wait behind that manager lock. That
explains why killing only the owner does not necessarily restore the service,
and why normal service stop can remain pending.

This is direct evidence of the service defect. It is not merely a similarity
to a bug report about forcibly terminating a device owner.

## Upstream correction and its limits

The pinned Microsoft source has a
`Feature_Servicing_MIDI2VirtualDeviceRemovalDeadlock` branch in
`CMidiClientManager::DestroyMidiClient`. That branch retains pipes while
updating the maps, leaves the exclusive-lock scope, and then calls their
`Shutdown` methods. The older branch calls device shutdown inside the scope.
`OnDeviceRemoved` takes the manager lock in shared mode.

That correction addresses the ordering demonstrated by this capture. It has
not been verified on Kensuke, and no undocumented feature override was applied.
A public-symbol lookup did not establish whether this service contains a
disabled copy of the correction; absence of a symbol is not proof of absence
of code.

The next useful implementation test is a Windows service/component combination
with this correction confirmed present and active, followed by two complete
probe runs without service recovery. Updating or replacing Windows components
is a separate action. No preview service package, driver, service configuration
or feature flag was changed during this investigation.

Changing application callback order, returning before cleanup, killing a child,
or restarting the shared MIDI service automatically is not a verified fix.
The latter would interrupt other MIDI clients and is retained only as an
explicit recovery action.

## Reproduction and inspection commands

The temporary reference source, project and native reader source are preserved
with the ignored local evidence. Its reference project points at the existing
`dist/windows-midi-probe/S4MidiProbe.dll` and `WinRT.Runtime.dll`, and targets
`net10.0-windows10.0.26100.0`.

On the Mac, from the repository root:

```sh
diagnostics/windows-tools/dotnet-10.0.403/dotnet publish \
  diagnostics/windows-tools/shutdown-debug/evidence/source/reference/ReferenceLifecycle.csproj \
  -c Release -r win-x64 --self-contained true --tl:off \
  "-p:S4ProbeBuildDir=$PWD/dist/windows-midi-probe"
```

The explicit property keeps assembly references correct after the temporary
project has been moved into the evidence directory. No project-source
compilation is required on Windows.

The five `ReferenceLifecycle` executable, assembly, PDB, dependency manifest and
runtime-configuration files from the publish output were copied alongside the
existing portable probe runtime in `midi-probe-20261103-v2`.

On Windows, the first PowerShell session ran:

```powershell
& "$env:LOCALAPPDATA\S4BridgeDiagnostics\midi-probe-20261103-v2\ReferenceLifecycle.exe"
```

In a second PowerShell session, after its `REFERENCE_READY` line:

```powershell
$devices = @(Get-PnpDevice -PresentOnly |
    Where-Object {$_.InstanceId -like '*S4PROBE20261103*'})
$devices | Select-Object Status, Class, InstanceId
$source = @($devices | Where-Object {
    $_.InstanceId -like '*APPPUB*S4PROBE20261103_0_0*'})
$destination = @($devices | Where-Object {
    $_.InstanceId -like '*APPPUB*S4PROBE20261103_1_0*'})
if ($source.Count -eq 0 -or $destination.Count -eq 0) {
    throw 'Both legacy ports must arrive before shutdown.'
}
$close = [System.Threading.EventWaitHandle]::OpenExisting(
    'Local\S4ProbeReferenceClose')
try { $close.Set() | Out-Null } finally { $close.Dispose() }
```

Native reader operations, before recovery:

```powershell
$dir = "$env:LOCALAPPDATA\S4BridgeDiagnostics\shutdown-debug"
$owner = Get-Process ReferenceLifecycle
$service = Get-CimInstance Win32_Service -Filter "Name='midisrv'"
& "$dir\NativeStacks.exe" capture $owner.Id "$dir\reference-blocked.dmp"
& "$dir\NativeStacks.exe" capture $service.ProcessId "$dir\service-blocked.dmp"
$symbols = "$env:LOCALAPPDATA\S4BridgeDiagnostics\midi-probe-20261103-v2;" +
    "srv*$dir\symbols*https://msdl.microsoft.com/download/symbols"
& "$dir\NativeStacks.exe" analyze "$dir\service-blocked.dmp" $symbols
```

The PIDs elsewhere in this record identify this capture only. These commands
resolve current processes; the reader also enforces its executable allowlist.
The temporary reader and reference files were removed from Windows after the
capture; redeployment is necessary before repeating this sequence.

Useful offline commands used after opening the service dump:

```text
~*kv 20
~5s
u MidiSrv!CMidiClientManager::DestroyMidiClient L50
u MidiSrv!CMidiClientManager::DestroyMidiClient+0x1c5 L45
u MidiSrv!CMidiClientManager::DestroyMidiClient+0x550 L30
u MidiSrv!CMidiClientManager::DestroyMidiClient+0x650 L50
u MidiSrv!CMidiClientManager::OnDeviceRemoved+0x100 L20
.frame /r 3
.frame /r a
dq 000001ee1566a530 L1
lmv m MidiSrv
```

Addresses and thread indices are specific to this dump. The symbol path used
the local API PDB directory followed by
`srv*<local symbol cache>*https://msdl.microsoft.com/download/symbols`.

## Recovery and verification boundaries

The owned reference process was stopped only after both blocked snapshots had
been saved. Normal MIDI service stop remained pending; recovery stopped the
verified `System32\midisrv.exe` process. Windows automatically restarted it.
The resulting service was Running as PID 8940, with Manual start mode preserved.
NIHardwareService remained PID 4772 and Djay remained responsive as PID 15608.
A fresh inventory found no probe owner or present probe endpoint.

After the evidence was copied to the ignored Mac directory, all temporary
Windows debugger files and the five added reference-executable files were
removed. The original probe directory and prior diagnostic files were
preserved. Final inventory:

```json
{"services":[{"Name":"midisrv","State":"Running","ProcessId":8940,"StartMode":"Manual"},{"Name":"NIHardwareService","State":"Running","ProcessId":4772,"StartMode":"Auto"}],"diagnosticProcesses":[],"probeEndpoints":[],"djay":[{"Id":15608,"Responding":true}]}
```

The native snapshots, stack logs and diagnostic source are deliberately retained
only under ignored `diagnostics/windows-tools/shutdown-debug/evidence/`.
Temporary package extractions and the original scratch projects were removed.

No production bridge source was changed, and no commits or pushes were made.
Both temporary C# tools compiled with warnings treated as errors. The existing
Rust/macOS checks were not rerun because their source was unchanged; their
previous results remain in the feasibility record.

There is no passing post-fix test: the installed service was recovered, not
patched. No normal-shutdown success, mapped Djay feedback or Windows controller
hardware behavior is claimed.

## Primary sources

- [Pinned Microsoft source revision](https://github.com/microsoft/MIDI/tree/fc8af83b876d065089b30d10fa515e5b4ab1ca04).
- [Transport destruction RPC](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Libs/MidiSrvTransportLib/MidiSrvTransport.cpp#L226).
- [Client-manager destruction and servicing correction](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Service/Exe/MidiClientManager.cpp#L1368).
- [Client-manager device-removal locking](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Service/Exe/MidiClientManager.cpp#L1672).
- [Virtual-device reference sample](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/samples/csharp-net/virtual-device-app-winui/MainWindow.xaml.cs).
- [Related Microsoft issue 1047](https://github.com/microsoft/MIDI/issues/1047).
- [Preview 11 release and compatibility limitation](https://github.com/microsoft/MIDI/releases/tag/inbox-preview-11).
- [Microsoft DbgEng package](https://www.nuget.org/packages/Microsoft.Debugging.Platform.DbgEng/20260319.1511.0).
