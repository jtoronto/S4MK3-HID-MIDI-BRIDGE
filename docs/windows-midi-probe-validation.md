# Windows MIDI feasibility probe validation

November 3, 2026.

## Conclusion

The [native shutdown follow-up](windows-midi-shutdown-investigation-20261103.md)
now identifies a self-deadlock inside Windows MIDI Service during virtual-device
removal. It also corrects the service-version interpretation: the 7705 string
version differs from the installed binary's numeric fixed version 26100.9444.

The separate [loopback trial](windows-loopback-probe-validation-20261103.md)
avoids that shutdown hang and is discovered by Djay, but has not passed reverse
data delivery on the tested pair. It is not a working backend claim.

The subsequent [LoopBe1 trial](windows-loopbe1-probe-validation-20261104.md)
passed exact two-way native WinMM delivery and normal shutdown in two fresh
invocations without service recovery. Djay opened both directions. This is a
working transport candidate with an installed-driver dependency and shared-bus
echo; mapped Djay feedback and full bridge integration are not yet verified.

The portable Mac-built Windows probe works for API activation, virtual-device
creation and exact bidirectional MIDI transfer to an independent WinMM process.
Installed Djay 5.6.10 also opens both public MIDI directions.

The tested Preview 11 API / in-box service combination is not ready for a
production bridge: normal SDK cleanup reproducibly stalls. Individual
disconnect, explicit responder removal and session-owned disposal were tested.
Recovery required stopping the owned probe and restarting Windows MIDI Service.
There is no automatic service-restart workaround in the probe.

No Djay mapping was installed or selected. The non-sending capture received
zero MIDI messages, so actual mapped Djay feedback and control behavior remain
unverified. A graceful second invocation without service recovery is also not
established. The result supports proceeding with a Windows port, but not
shipping this MIDI backend before resolving its teardown behavior or selecting
another virtual-MIDI implementation.

## Scope

Build a portable Windows MIDI diagnostic on the Apple Silicon Mac and run it
on Kensuke without installing a development environment there. This is not
a Windows controller bridge implementation.

The probe has no HID, USB, audio, screen or motor implementation. The Rust
bridge, its dependency pin, the macOS host and existing Djay mappings are not
changed. Deployment uses a separate user-local diagnostic directory.

## Tested environment and dependencies

- Windows 11 Home 25H2 x64, build 26200.9457.
- Windows MIDI Service running; its EXE's string version is 10.0.26100.7705.
  The follow-up native investigation records numeric fixed version
  10.0.26100.9444; the string alone does not identify the serviced binary.
- Installed virtual and loopback transport DLLs are version 1.0.15.0.
- Djay 5.6.10 is running in the interactive desktop session.
- NIHardwareService 2.8.2 is running. No S4 is presently enumerated.
- No Rust, .NET, C++ or MIDI Tools executable was found on Windows PATH.
- Mac SDK: .NET 10.0.403 arm64, runtime 10.0.14, unpacked into an ignored
  diagnostic directory rather than installed system-wide.
- Wine 11.0 with Rosetta executes the C#/WinRT 2.2.0 projection generator.
- Target framework: `net10.0-windows10.0.26100.0`, Windows SDK metadata
  `10.0.26100.57`, self-contained `win-x64`, without Native AOT.
- Windows MIDI API: `Windows.Devices.Midi2` `0.99.92-preview.11`.
  Downloaded package SHA-256:
  `13afc6e6da2adac309ab8c174352f46f1c0ebe2d79e65475bd6274defb73df74`.

The Mac SDK archive was verified against Microsoft's release metadata SHA-512:
`5893c66b48ea3674c7da18501059d606e96f3672fabe4d5e41762e692c7c83df3ee0b2b9dc5a0786191da2dc413d7335158101c14c935ab217f31ac2529770f3`.

## Evidence path

| Claim | Decisive evidence |
| --- | --- |
| Mac can build a portable Windows executable | Successful self-contained publish and Windows execution with no installed .NET runtime |
| API works app-locally | `--detect` reports `systemProvided=false` and `serviceAvailable=true` |
| Virtual device exposes MIDI 1 ports | Endpoint and legacy-port arrival events plus WinMM port-name verification |
| Controller messages reach legacy applications | Device sends UMP `0x20917735`; independent WinMM callback receives packed bytes `0x00357791` |
| Application feedback reaches the device | WinMM sends packed bytes `0x00486FBF`; device callback receives UMP `0x20BF6F48` |
| Normal shutdown removes the device | Endpoint and both legacy-port removal events, subscribed before creation |
| Fresh invocation works | A second standalone process repeats the self-test after the first exits |
| Djay connects | Service reporting identifies Djay's process on the probe's client endpoint |
| Djay returns feedback | Actual captured channel-voice messages after Djay mapping selection or user interaction; not inferred from port presence |

Receivers are subscribed before sends. Event waits use bounded timeouts; the
self-test has no fixed sleeps or polling delays. The listen mode's duration is
the requested capture window, not a synchronization workaround.

## Build and deployment

From the repository root, with .NET 10 on PATH:

```sh
bash windows/midi-probe/build-probe.sh
```

For the isolated SDK used here:

```sh
DOTNET="$PWD/diagnostics/windows-tools/dotnet-10.0.403/dotnet" \
  bash windows/midi-probe/build-probe.sh
```

The script verifies the API package, performs locked restore, translates the
projection-generator paths through Wine's `Z:` filesystem mapping, and
publishes to `dist/windows-midi-probe/`. Passing Unix absolute paths directly
to `cswinrt.exe` failed because that tool interpreted `/Users/...` as an
option; the dedicated wrapper supplies Windows-form paths instead.

Initial deployment:

`C:\Users\toron\AppData\Local\S4BridgeDiagnostics\midi-probe-20261103\`

The folder contains the executable, app-local API DLL/PRI/WinMD, C#/WinRT
projection and self-contained .NET runtime. Existing system DLLs, service
settings, NI drivers and Djay configuration are not replaced.

## Results

Initial API activation, before any virtual-device creation:

```json
{"stage":"api","data":{"systemProvided":false,"serviceAvailable":true,"apiVersion":"1.0.0.0","os":"Microsoft Windows NT 10.0.26200.0"}}
```

This establishes successful Mac-built Windows execution and app-local API
activation.

### First endpoint and WinMM attempt

The first self-test created the private device endpoint and observed the
client-visible endpoint plus both MIDI 1 ports:

- Name: `S4 MK3 Windows Probe`.
- Private instance: `SWD\MIDISRV\MIDIU_APPDEV_S4PROBE20261103`.
- Public instance: `SWD\MIDISRV\MIDIU_APPPUB_S4PROBE20261103`.
- SDK-reported MIDI 1 source number: 0; destination number: 1.

The round trip did not run successfully. A snapshot of the owned process heap
recovered the original exception:

```text
System.InvalidOperationException: inspect WinMM input: WinMM error 2
S4MidiProbe.WinMmPeer.Check
S4MidiProbe.WinMmPeer.Open
S4MidiProbe.Probe.RunAsync
```

WinMM error 2 is `MMSYSERR_BADDEVICEID`. The original probe passed the SDK's
reported port numbers directly to WinMM capability queries without independently
enumerating the consuming process's WinMM port list. The corrected implementation
calls `midiInGetNumDevs` / `midiOutGetNumDevs`, enumerates actual WinMM names and
indices, and uses a separate WinMM consumer process. This is compiled but has not
yet been retested against a recovered Windows session.

The first process entered cleanup before its exception reached Main. A real
managed-stack capture established that it was stalled in:

```text
Windows.Devices.Midi2.MidiSession.DisconnectEndpointConnection
S4MidiProbe.VirtualController.Dispose
S4MidiProbe.Probe.RunAsync
```

The 180-second SSH command deadline ended the local SSH command, not the remote
probe. Read-only inspection confirmed probe PID 2968 remained alive and that
only its private APPDEV endpoint remained present. Do not equate the local
command timeout with successful Windows cleanup.

Portable stack and snapshot readers were built on the Mac and copied into a
separate user-local diagnostic directory. No system-wide runtime, SDK or
developer environment was installed on Windows. The snapshot reader inspected
only the verified owned probe executable and disposed its snapshot afterward.
The temporary stack-tool host initially failed to resolve CoreCLR; a properly
published self-contained diagnostic host worked.

At this checkpoint, Windows MIDI Service PID 10148 and Djay PID 15608 remained
running; Djay reported responsive. No MIDI service restart, NI service change
or controller opening has been performed. Recovery of the hung probe and any
MIDI service restart require the user's decision before another device run.

### Initial checkpoint, before the corrected round trip

Established: Mac-hosted Windows build, self-contained execution, app-local API
activation, virtual-device creation and MIDI 1 port notifications.

Unproven: actual MIDI delivery in either direction, clean endpoint removal,
second-run reliability, Djay endpoint selection and Djay feedback. Successful
enumeration is not a substitute for these checks.

### Corrected round trip and authorized recovery

The user subsequently authorized stopping the hung probe and recovering Windows
MIDI Service. The verified probe process was stopped. A normal service stop
remained pending; the verified `System32\midisrv.exe` process was force-stopped
and the service returned to Running. Its Manual start mode was preserved.
Djay remained open and responsive.

The corrected executable was deployed separately under
`S4BridgeDiagnostics\midi-probe-20261103-v2\`. A passive invocation first
reported no WinMM inputs and only Microsoft GS Wavetable Synth as output,
then exited 1 with the expected absent-probe error, without creating a device.

After service recovery, the full self-test successfully enumerated and opened
the actual native WinMM ports from an independent child process:

```json
{"stage":"winmm-ports","data":{"sources":[{"Number":0,"Name":"S4 MK3 Windows Probe"}],"destinations":[{"Number":0,"Name":"Microsoft GS Wavetable Synth"},{"Number":1,"Name":"S4 MK3 Windows Probe"}]}}
{"stage":"winmm-received","data":{"packed":"0x00357791"}}
{"stage":"device-received","data":{"ump":"0x20BF6F48"}}
{"stage":"peer-passed","data":{"deviceToWinmm":"0x00357791"}}
{"stage":"round-trip","data":{"deviceToWinmm":"0x00357791","winmmToDevice":"0x20BF6F48","passed":true}}
```

This establishes actual bidirectional delivery between the virtual controller
and a separate WinMM application. The child closed and exited before the owner
began disconnecting.

The owner again stalled during SDK disconnect. Thus the original invalid-index
failure and the cleanup stall are separate findings; passing the round trip
does not establish successful teardown. The second hung owner was stopped
under the user's recovery authorization. On the next recovery Windows
automatically restarted the service after its process was stopped, so a wait
for the intermediate Stopped state timed out; fresh inspection established
Running with a new PID and unchanged Manual start mode.

The explicit responder-removal check passed its MIDI round trip and logged
`responder-detached`, then stalled in disconnect. The session-owned disposal
variant also passed its MIDI round trip, detached the responder, logged
`session-closing`, and stalled. The retained source uses session-owned disposal
and emits each diagnostic stage; it does not conceal or bypass the stall.
No broad preview service or transport package has been installed.

### Actual Djay discovery and feedback observation

A separate `--listen 20` run sent no MIDI. Service reporting identified the
already-running `djayApp.exe`, PID 15608, against the probe's public endpoint.
At capture start it held the MIDI 1 source interface; at capture end it held
both source and destination interfaces:

| Process | Interface suffix | Direction |
| --- | --- | --- |
| `djayApp.exe`, PID 15608 | `MIDIU_APPPUB_S4PROBE20261103_0_0` | Input to Djay |
| `djayApp.exe`, PID 15608 | `MIDIU_APPPUB_S4PROBE20261103_1_0` | Output from Djay |

Both sessions were named `Windows.Devices.Midi client session`. This directly
establishes Djay endpoint discovery and opening, rather than inferring it from
a generic endpoint list. The Windows MIDI protocol manager also had its own
connection; it was not mistaken for Djay.

The capture ended with:

```json
{"stage":"capture-ended","data":{"messages":0}}
```

No mapping import or GUI interaction was performed. Zero messages in this
unmapped, non-sending capture do not prove that Djay feedback is unsupported.
The capture owner encountered the same session-disposal stall and was stopped
under the user's recovery authorization.

### Final Windows state

After the final authorized recovery, a fresh inventory reported:

```json
{"services":[{"Name":"midisrv","State":"Running","ProcessId":1332,"StartMode":"Manual"},{"Name":"NIHardwareService","State":"Running","ProcessId":4772,"StartMode":"Auto"}],"ownedProcesses":[],"probeEndpoints":[],"djay":[{"Id":15608,"Responding":true}]}
```

No probe, stack-inspector or heap-inspector process remained. No present PnP
instance containing the probe's product identity or friendly name remained.
This verifies absence after recovery, not successful normal SDK teardown.
NIHardwareService retained its original PID and start mode; Djay remained open.

The portable probe and diagnostic files remain in their separate user-local
folders for reproducibility. No system-wide development tools, preview service
packages or virtual-MIDI drivers were installed. No controller traffic was sent.

## Remaining Windows integration work

1. Resolve normal teardown with the supported SDK/service combination, or
   evaluate another virtual-MIDI implementation. A production app must not
   require MIDI-service restarts on Stop or Quit.
2. Repeat two complete, graceful invocations without service recovery.
3. Import a new, separately named Djay test mapping and verify actual input
   actions and returned state; preserve existing mappings.
4. Only then integrate the Rust platform boundaries and Windows tray/settings
   host. HID, NI screen-service routing, audio coexistence and motor timing
   remain separate Windows hardware tests.

### Source checks

The required Rust formatting, strict Clippy, locked tests and all-target build
passed. Swift passed 80 tests; its package emitted an existing warning about two
unhandled files. macOS build-script syntax, plist lint, probe-script syntax and
Git whitespace checks passed. Git's pager held the final check open until it
was quit normally; the command then exited 0.

The corrected C# project publishes with warnings treated as errors. No Rust or
Swift source was modified.

The NuGet package's GUID optimizer initially attempted to execute a Windows
apphost directly on macOS. The build now invokes its managed
`IIDOptimizer.dll` through the Mac .NET SDK. It reports
`0 IID calculations/fetches patched`; the tool's documented source returns -1
for that normal no-change outcome, which the official MSBuild target handles
without replacing the assembly. An initially added strict zero-only exit check
was incorrect and was removed; the optimizer is executed, not disabled.

## Sources and known service limitation

- [Supported Preview 11 API release and deployment terms](https://github.com/microsoft/MIDI/releases/tag/inbox-preview-11).
- [Pinned C# console sample project](https://github.com/microsoft/MIDI/blob/inbox-preview-11/samples/csharp-net/basics/client-basics-cs.csproj).
- [Pinned virtual-device sample](https://github.com/microsoft/MIDI/blob/inbox-preview-11/samples/csharp-net/virtual-device-app-winui/MainWindow.xaml.cs).
- [Pinned endpoint watcher sample](https://github.com/microsoft/MIDI/blob/inbox-preview-11/samples/csharp-net/watch-endpoints/Program.cs).
- [Pinned legacy-port watcher sample](https://github.com/microsoft/MIDI/blob/inbox-preview-11/samples/csharp-net/watch-midi1-ports/Program.cs).
- [Microsoft virtual-device issue 1047](https://github.com/microsoft/MIDI/issues/1047).
- [Microsoft .NET 10 release metadata](https://builds.dotnet.microsoft.com/dotnet/release-metadata/10.0/releases.json).
- [C#/WinRT optimizer exit-code source](https://github.com/microsoft/CsWinRT/blob/master/src/Perf/IIDOptimizer/Program.cs).

Issue 1047 concerns forcibly terminating an app which owns a virtual device.
This probe tests normal shutdown; forced termination was used only to recover
already-stalled test owners, with user authorization. The exact native cause
of this observed cleanup stall has not been established, so it is not claimed
to be the same defect. The transport file version alone does not establish
whether a particular servicing fix is installed.

That statement describes the initial probe's evidence boundary. The subsequent
[native investigation](windows-midi-shutdown-investigation-20261103.md)
establishes the concrete service lock re-entry and matches it to Microsoft's
servicing correction, without claiming that the correction has been tested on
this PC.
