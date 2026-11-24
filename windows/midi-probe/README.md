# Windows MIDI feasibility probe

This is a source-only, software-only diagnostic, not a Windows port of the
controller bridge. It does not open HID, audio or screen interfaces and cannot
send motor commands.

It tests app-local Windows MIDI Services API activation, a virtual MIDI device
named **S4 MK3 Windows Probe**, and communication with the legacy WinMM API.
The existing Rust bridge and macOS app are independent of this project.

## Build on macOS

Requirements:

- .NET 10 SDK for the Mac.
- Working Wine and Rosetta for Microsoft's Windows-only C#/WinRT projection
  generator.
- `curl`, `shasum` and network access to Microsoft NuGet and GitHub Releases.

From the repository root:

```sh
bash windows/midi-probe/build-probe.sh
```

Set `DOTNET` to the SDK's executable if it is not on PATH. The script downloads
the pinned Preview 11 MIDI API package, verifies its SHA-256, restores locked
dependencies and publishes a self-contained Windows x64 folder to
`dist/windows-midi-probe/`. Set `S4_PROBE_OUTPUT_DIR` to use another output
directory.

Wine uses an isolated prefix under the ignored `diagnostics/windows-tools/`
directory. The Windows computer does not need Wine, .NET, Rust, Visual Studio,
the Windows SDK or Microsoft's MIDI Tools application.

## Run on Windows

Requirements:

- Windows 11 25H2 or later, x64.
- The Windows MIDI Service and virtual MIDI transport available.
- The entire published folder, including the app-local MIDI API and .NET
  runtime files. Do not copy just the executable.

Open PowerShell in the extracted folder and run:

```powershell
.\S4MidiProbe.exe --detect
```

This checks API activation and service availability without creating a device.

```powershell
.\S4MidiProbe.exe --self-test
```

This subscribes to endpoint and MIDI 1 port events, creates the device, opens
its public ports through WinMM in a separate process, verifies exact messages
in both directions, closes the handles and waits for removal notifications.
Reported failures return a nonzero exit code and a JSON error. Native SDK
calls can themselves stall on affected Windows versions; event timeouts cannot
interrupt those calls. Run it again only after the first invocation has exited
to check reuse.

```powershell
.\S4MidiProbe.exe --listen 120
```

This publishes the device for a 120-second capture, logs incoming MIDI and
reports the processes connected to its public endpoint. It sends no MIDI.
The capture then requests cleanup. Durations from 1 to 600 seconds are accepted.
See the validation record for the observed Windows SDK cleanup stall.

Run only one probe instance at a time. Let it finish normally rather than
closing its process: Microsoft documents a service defect involving forcibly
terminated virtual-device applications on some Windows builds.

An available endpoint or a successful self-test does not prove that Djay
selected a mapping or returned feedback. The probe logs those observations
separately. It does not install or overwrite Djay mappings.

See [the validation record](../../docs/windows/validation/windows-midi-probe-validation.md) for
the tested system, results and remaining limits.
