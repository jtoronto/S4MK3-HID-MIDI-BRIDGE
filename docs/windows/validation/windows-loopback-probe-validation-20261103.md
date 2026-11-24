# Microsoft loopback feasibility test

November 3, 2026.

## Result

The installed Microsoft full loopback transport creates usable MIDI 1 port
interfaces and avoids the previous app-owned virtual-device shutdown hang.
It has **not** passed the bidirectional data test on Kensuke.

A-to-B Note On delivery succeeds. B-to-A delivery was not observed for the
tested CC, a Note On on the same return channel, or the identical known-valid
forward note. This persisted with SDK/WinMM and WinMM/WinMM clients and with
the return sender held open pending receipt acknowledgement.

This is a failed feasibility test for this configured pair, not proof that all
Microsoft loopback configurations or all machines are broken. The exact reason
for the missing return is unproven. No production backend or app integration
was implemented.

## Scope and environment

- Windows 11 Home 25H2, build 26200.9457.
- MIDI service PID 8940, Running, Manual, unchanged throughout data testing.
- NIHardwareService PID 4772, Running, Auto.
- Djay 5.6.10 PID 15608 remained open.
- Existing app-local Preview 11 API `0.99.92-preview.11`.
- No extra driver, Windows SDK, preview service package or system-wide .NET
  runtime was installed.
- No HID, audio, screen or motor traffic; no Djay mappings changed.

The earlier service version-string correction remains applicable: fixed
product version is 10.0.26100.9444, despite the executable's older 7705 string.
See [native shutdown investigation](../research/windows-midi-shutdown-investigation-20261103.md).

## Pair configuration and lifetime

The existing SDK exposed `MidiLoopbackManager.IsTransportAvailable = true`.
`GetActiveLoopbackEntries()` initially returned an empty list. The service's
built-in diagnostic endpoints were not altered.

`CreateTransientLoopback` succeeded with both endpoint definitions requesting
legacy MIDI 1 interfaces:

| Identity | Value |
| --- | --- |
| Association | `90491845-987f-44d7-b1cf-c89050384e58` |
| A name | `S4 Loopback Bridge` |
| A unique ID | `S4LOOP20261103A` |
| B name | `S4 MK3 Windows Probe LB` |
| B unique ID | `S4LOOP20261103B` |

```text
A: \\?\swd#midisrv#midiu_loop_a_s4loop20261103a#{e7cce071-3c03-423f-88d3-f1045d02552b}
B: \\?\swd#midisrv#midiu_loop_b_s4loop20261103b#{e7cce071-3c03-423f-88d3-f1045d02552b}
```

The creation process exited 0. Later processes opened the same association,
establishing that the pair outlives the creating application.

Here, "persistent" means independent of an application's connections.
`CreateTransientLoopback` does not save configuration to survive a service
restart or reboot. Reboot persistence was not tested.

SDK arrival events and actual WinMM enumeration agreed:

| Interface | Input index | Output index |
| --- | --- | --- |
| S4 Loopback Bridge | 0 | 1 |
| S4 MK3 Windows Probe LB | 1 | 2 |

Microsoft GS Wavetable Synth occupied output 0. Native code enumerated names
and indices in the consuming process rather than using SDK numbers blindly.

## Data and closure experiments

All receivers were registered before sends. Event waits were bounded; no fixed
sleep was used to synchronize data. The non-sending capture duration is an
observation window, not a synchronization delay.

| Case | Observed result |
| --- | --- |
| SDK A to original independent WinMM B | Forward note `0x00357791` received; reverse CC not received |
| Same, with raw SDK receive logging | No A receive callback of any UMP message type |
| SDK A / new WinMM B, sender held open for acknowledgement | Same missing return |
| WinMM A / WinMM B, sender held open | Same missing return |
| WinMM A/B, reverse status changed from CC `BF` to Note On `9F` | No return; channel and two data bytes unchanged |
| WinMM A/B, identical `0x00357791` note in both directions | Forward received; reverse not received |

The original forward was UMP `0x20917735`, or packed MIDI 1 `0x00357791`:
channel 2, note 119, velocity 53.

The original return was packed MIDI 1 `0x00486FBF`: channel 16, CC 111, value 72.
The original virtual-device probe had received this as UMP `0x20BF6F48`.
The loopback tests did not receive it.

Changing only the return status to `9F` produced `0x00486F9F`. The final
identical-message control used `0x00357791` in reverse. These were separate
diagnostic cases; neither was substituted for a passing CC feedback test.
The identical-message case does not independently rule out feedback protection.

Representative first-case output:

```json
{"stage":"winmm-received","data":{"packed":"0x00357791"}}
{"stage":"peer-passed","data":{"deviceToWinmm":"0x00357791"}}
{"stage":"connection-closing","data":{"AssociationId":"90491845-987f-44d7-b1cf-c89050384e58"}}
{"stage":"connection-closed","data":{"AssociationId":"90491845-987f-44d7-b1cf-c89050384e58"}}
```

That original peer's `peer-passed` meant it received the forward note and
submitted its return; it did not establish parent receipt. The parent timed
out and exited 1. The revised peer requires parent acknowledgement before
reporting success.

The SDK session's disposal returned normally in each SDK case. Pure WinMM
cases also returned through handle cleanup and process exit. Pair identity
remained available across subsequent invocations. No service restart or
recovery was used to progress between these cases.

There is no successful two-way run, and therefore no successful second complete
two-way run. Repeated connection reopening is a narrower result.

## Hypotheses and evidence limits

- MIDI 2 conversion alone does not explain the result: raw SDK callback logging
  saw nothing, and WinMM A also missed the return.
- Immediate sender shutdown is insufficient to explain it: holding handles open
  until receipt acknowledgement did not restore delivery.
- A CC-specific difference is insufficient: reverse Note On also failed.
- Wrong interface/group selection, a missing/stale transport callback, or other
  service-side routing remain possible. Display names were unique, but a
  driver-interface/group query was not added.

Read-only source review found that the pinned transport can return success
when its paired callback is absent; initialization/table timing is therefore
a candidate. No native breakpoint or transport callback-state capture was made
here, so this is not labelled a confirmed Windows defect or a verified fix.

The test leaves the original app-owned virtual-device deadlock diagnosis
unchanged. It also does not establish that the separate basic-loopback transport,
another pair creation, another SDK/service build or another machine behaves
the same way.

## Djay discovery and reuse observation

The final `--listen` invocation sent no MIDI and captured for 20 seconds.
Service reporting identified Djay PID 15608 holding both legacy input and
output interfaces for A, and both for B. These were separate
`Windows.Devices.Midi client session` entries; the harness's own A connection
was separately identified as PID 8056.

The capture ended with zero received messages. No mapping was imported or
selected and no GUI control was exercised, so this does not establish mapped
Djay feedback support or its absence.

```json
{"stage":"capture-ended","data":{"messages":0}}
{"stage":"connection-closed","data":{"AssociationId":"90491845-987f-44d7-b1cf-c89050384e58"}}
{"stage":"passed","data":{"pairRetained":true,"AssociationId":"90491845-987f-44d7-b1cf-c89050384e58"}}
```

That capture exited 0. Its `passed` applies to a non-sending capture and
connection lifecycle, not to the failed bidirectional data cases.

## Harness and reproducible commands

Source: `windows/loopback-probe/`. It is isolated from the existing probe,
Rust bridge and macOS application.

From the repository root, after building the original MIDI probe:

```sh
diagnostics/windows-tools/dotnet-10.0.403/dotnet publish \
  windows/loopback-probe/S4LoopbackProbe.csproj \
  -c Release -r win-x64 --self-contained true \
  -o dist/windows-loopback-probe --tl:off
```

The five `S4LoopbackProbe` executable, assembly, PDB, dependency manifest and
runtime-configuration files were copied alongside the existing portable native
API/runtime in `midi-probe-20261103-v2`. Both C# projects use warnings as errors.

The diagnostic operations, run individually after inspecting the previous
result, are:

```powershell
.\S4LoopbackProbe.exe --inspect
.\S4LoopbackProbe.exe --create
.\S4LoopbackProbe.exe --self-test
.\S4LoopbackProbe.exe --winmm-test
.\S4LoopbackProbe.exe --winmm-note-test
.\S4LoopbackProbe.exe --listen
.\S4LoopbackProbe.exe --remove
```

`--self-test` retains the SDK-A/native-B comparison. `--winmm-test` retains
the failing CC return. `--winmm-note-test` uses the identical note control.
The child modes are internal; no development environment is required on the PC.

The parent closes the child's redirected input on failure before waiting for
exit. The child's console read runs on a background task with a bounded wait.
This prevents an unacknowledged test return from leaving the native sender
waiting indefinitely. It is harness cleanup, not a data-delivery fix.

## Cleanup and checks

Supported removal of association `90491845-987f-44d7-b1cf-c89050384e58` returned
`Success=true` and exited 0, while Djay remained open. No service restart was
needed for creation, repeated connection tests, the listen run or removal.
The five temporary Windows executable/assembly/configuration/symbol files
were removed. Final inventory:

```json
{"stage":"loopback-cleanup","services":[{"Name":"midisrv","State":"Running","ProcessId":8940,"StartMode":"Manual"},{"Name":"NIHardwareService","State":"Running","ProcessId":4772,"StartMode":"Auto"}],"diagnostics":[],"testEndpoints":[],"djay":[{"Id":15608,"Responding":true}]}
```

The final source publish exited 0 with warnings treated as errors. No existing
Rust, Swift or original MIDI-probe source was changed, so their prior source
checks were not repeated. The failing data results remain failures; there is
no passing production-backend claim.

## Sources

- [Pinned loopback manager API](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Client/WinRT/core/MidiLoopbackManager.idl).
- [Creation configuration](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Client/WinRT/core/MidiLoopbackCreationConfig.idl).
- [Endpoint definition](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Client/WinRT/core/MidiLoopbackEndpointDefinition.idl).
- [Bidirectional transport forwarding](https://github.com/microsoft/MIDI/blob/fc8af83b876d065089b30d10fa515e5b4ab1ca04/src/in-box/Transport/LoopbackMidiTransport/Midi2.LoopbackMidiBidi.cpp).
- [Microsoft loopback lifecycle explanation](https://microsoft.github.io/MIDI/kb/virtual-loopback/).
