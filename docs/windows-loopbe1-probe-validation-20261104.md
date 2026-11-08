# LoopBe1 Windows MIDI feasibility

## Result

**The native transport test passed twice.** Separate processes exchanged the
exact note and CC, closed normally and reopened successfully without service
recovery. Djay discovered and opened both LoopBe1 directions while remaining
running throughout.

The subsequent [isolated Djay proof](windows-djay-loopbe1-proof-20261104.md)
established mapped control, independent GUI-driven feedback and echo rejection
for one crossfader. Full controller operation remains untested. LoopBe1 is
an installed dependency and echoes each sender's messages back to its input;
those constraints must be accounted for before production integration.

## Scope and baseline

This November 4, 2026 session tests the separately installed LoopBe1 WDM driver,
without purchasing or using a proprietary SDK. Kensuke's recorded local clock
was still November 3 during initial setup. This is a software-only transport
test, not a Windows bridge implementation or S4 hardware acceptance.

Before installation:

- MIDI service: PID 8940, Running, Manual.
- NIHardwareService: PID 4772, Running, Auto.
- Djay 5.6.10: PID 15608, responsive.
- No LoopBe devices or installed-program entries.
- No active earlier diagnostic processes.

Existing Rust/macOS code, earlier MIDI probes and Djay mappings were preserved.
No controller HID, audio, motor or screen traffic was used.

## Official installer and installation

The installer came from
`https://www.nerds.de/data/setuploopbe1.exe`, linked by the vendor's download
page. Its size was 1,016,128 bytes; SHA-256:

```text
0958b39c23e02f86ba63f520cfc333842e3f7c31e197416c12f2718756ecb9be
```

Windows `Get-AuthenticodeSignature` returned Valid. Signer:
`E=contact@nerds.de, CN=Daniel Schmitt, O=Daniel Schmitt, C=DE`;
issuer GlobalSign ObjectSign CA. The SSH token had administrator rights, but
the user offered to perform installation, so agent installation work paused.
The user subsequently reported installation finished without a reboot.

Offline 7-Zip inspection identified an NSIS 2 installer with separate driver
payloads dated April 2011 and a monitor executable. The age alone was not used
to infer incompatibility.

Post-install inventory:

| Device | Instance |
| --- | --- |
| nerds.de LoopBe1 - Internal Midi Port (WDM), MEDIA, OK | `ROOT\*LOOPBEMIDI1\0000` |
| Same friendly name, MidiEndpoint, OK | `SWD\MIDISRV\MIDIU_KSA_10008119704918736402` |
| LoopBe Internal MIDI, SoftwareDevice, OK | `SWD\MMDEVAPI\MIDIU_KSA_10008119704918736402_0_0` |
| LoopBe Internal MIDI, SoftwareDevice, OK | `SWD\MMDEVAPI\MIDIU_KSA_10008119704918736402_1_0` |

LoopBe monitor PID 10076 was responsive. MIDI service, NI service and Djay
retained their original PIDs and state.

The native WinMM probe enumerated:

| Flow | Index | Name |
| --- | --- | --- |
| Input | 0 | LoopBe Internal MIDI |
| Output | 0 | Microsoft GS Wavetable Synth |
| Output | 1 | LoopBe Internal MIDI |

Enumeration exited 0. The diagnostic chooses the exact LoopBe input/output
names and requires one of each, rather than copying SDK port indices.

## Independent two-way MIDI

`windows/loopbe-probe/` is an isolated C# harness using native WinMM calls.
It was built on the Mac and run with the existing self-contained .NET runtime
on Windows. No Windows development environment or system-wide runtime was
installed. Microsoft API references are used only by the optional session
reporting mode, not for MIDI delivery or creating virtual endpoints.

The parent opens LoopBe input/output, starts another instance as its peer and
waits for peer-ready. Both callbacks exist before the parent sends. The peer
must receive the exact note before sending the exact CC back. Parent receipt
is acknowledged through redirected stdin before the peer closes its handles.
Every asynchronous wait has a bounded deadline.

| Direction | Packed MIDI 1 | Meaning |
| --- | --- | --- |
| Parent to peer | `0x00357791` | Channel 2, Note On 119, velocity 53 |
| Peer to parent | `0x00486FBF` | Channel 16, CC 111, value 72 |

The parent's own note is echoed to its input, but only the distinct CC can
complete its return task. Thus self-echo cannot produce a false round-trip
pass. Child stdout includes its actual PID separately from the parent's.

First run: parent PID 8832, peer PID 9112. Both exact messages arrived at the
other process. The peer acknowledged success, closed, exited 0, and the parent
closed normally and exited 0:

```json
{"stage":"received","pid":9112,"data":{"packed":"0x00357791"}}
{"stage":"received","pid":8832,"data":{"packed":"0x00486FBF"}}
{"stage":"peer-passed","pid":9112,"data":{"acknowledged":true}}
{"stage":"peer-closed","pid":9112,"data":{}}
{"stage":"round-trip","pid":8832,"data":{"forward":"0x00357791","reverse":"0x00486FBF","peerPid":9112}}
{"stage":"connection-closed","pid":8832,"data":{}}
{"stage":"passed","pid":8832,"data":{"twoWay":true}}
```

The second invocation used fresh parent PID 15472 and peer PID 12216. It again
received both exact messages, acknowledged the return, closed both processes
normally and exited 0. Post-run inventory found no `S4LoopBeProbe` processes.
MIDI service remained PID 8940 Running/Manual and NI service remained PID 4772
Running/Auto. No restart, recovery or reboot occurred between runs.

## Djay discovery and available feedback

The non-sending observation used PID 10460 and ran for 20 seconds. Service
reporting showed Djay PID 15608 with two `Windows.Devices.Midi client session`
entries:

```text
Input:  \\?\swd#mmdevapi#midiu_ksa_10008119704918736402_0_0#{504be32c-ccf6-4d2c-b73f-6f8b3747e22b}
Output: \\?\swd#mmdevapi#midiu_ksa_10008119704918736402_1_0#{6dc23320-ab33-4ce4-80d4-bbb3ebbf2814}
```

The probe's own `WinMM client` entry held these same interfaces, separately
from Djay. Both Djay connections were present at the start and end of capture.
The capture received zero messages and closed normally, exiting 0.

No Djay mapping was imported, selected or edited, and no GUI controls were
exercised. Therefore this establishes discovery and open interfaces, not mapped
control or software-driven feedback. Zero messages in this observation does
not establish that feedback is unsupported.

## Reproduction

Build the original `windows/midi-probe` first to provide the existing reporting
projection, then build this diagnostic from the repository root:

```sh
diagnostics/windows-tools/dotnet-10.0.403/dotnet publish \
  windows/loopbe-probe/S4LoopBeProbe.csproj \
  -c Release -r win-x64 --self-contained true \
  -o dist/windows-loopbe-probe --tl:off
```

The build exited 0, with warnings treated as errors.

Five `S4LoopBeProbe` files (EXE, DLL, PDB, deps JSON, runtimeconfig JSON) were
staged alongside the existing portable runtime/native reporting API:

```text
%LOCALAPPDATA%\S4BridgeDiagnostics\midi-probe-20261103-v2
```

Run individually and check exit codes:

```powershell
.\S4LoopBeProbe.exe --inspect
.\S4LoopBeProbe.exe --self-test
.\S4LoopBeProbe.exe --self-test
.\S4LoopBeProbe.exe --listen
```

The hidden `--peer` mode is started by the parent. The diagnostic does not
create/delete LoopBe endpoints or configure the monitor's mute state.

## Limits for a Windows product

LoopBe1 is an installed driver dependency; a portable host executable would
still require that one-time installation. It is not an SDK embedded into the
bridge. The vendor's free personal/non-commercial terms differ from commercial
use and bundling terms.

This is a shared MIDI bus: the bridge sees its own output as well as messages
sent by other clients. Production input/feedback routing must account for that.
These few messages do not establish jog-rate throughput, latency, long-session
stability, feedback-loop protection behavior or hot-plug/recovery behavior.
No production Rust integration, Windows host UI or generated mapping changes
were made.

## Final cleanup and verification

The temporary five-file probe overlay and staged installer were removed from
Windows after checking that no diagnostic process remained. The installed
LoopBe1 driver, its four present PnP records and monitor were deliberately
retained. They were not uninstalled.

Final inventory:

- MIDI service PID 8940, Running/Manual, unchanged.
- NIHardwareService PID 4772, Running/Auto, unchanged.
- Djay PID 15608 and LoopBe monitor PID 10076 responsive.
- No `S4LoopBeProbe`, `S4MidiProbe` or `S4LoopbackProbe` processes.
- All four LoopBe PnP records present with Status OK.
- No temporary `S4LoopBeProbe` files in the staging directory.

No reboot or service restart was performed. The new C# build passed with
warnings treated as errors, native enumeration exited 0, both two-way
invocations exited 0 and the observation/cleanup commands exited 0.
The final source review covered native callback lifetime, exact packet
matching, independent child acknowledgement and cleanup.

Existing Rust/macOS code and prior probe sources were unchanged; their prior
checks were not repeated. No project commits or pushes were made.

## Sources and related tests

- [LoopBe1 product and licensing](https://www.nerds.de/en/loopbe1.html).
- [Official downloads](https://www.nerds.de/en/download.html).
- [Earlier full Microsoft loopback trial](windows-loopback-probe-validation-20261103.md).
- [Earlier app-owned virtual-device shutdown diagnosis](windows-midi-shutdown-investigation-20261103.md).
