# Microsoft loopback feasibility probe

This is a software-only diagnostic, not a Windows S4 bridge. It uses the
existing app-local MIDI projection from `windows/midi-probe`. Its independent
native WinMM peer runs as a separate instance of this harness and holds its
handles until receipt is acknowledged.

The measured result is incomplete: connection shutdown returns normally and
A-to-B traffic arrives, but the tested B-to-A return does not. Do not treat this
as a working Windows bridge backend. Detailed cases are recorded in
`docs/windows-loopback-probe-validation-20261103.md`.

The pair is configured separately from application connections. Closing a test
connection must leave both endpoints present. The API calls it a transient
loopback: this test does not save configuration across service restarts/reboots.

Build the existing portable MIDI probe first, then run from the repository root:

```sh
diagnostics/windows-tools/dotnet-10.0.403/dotnet publish \
  windows/loopback-probe/S4LoopbackProbe.csproj \
  -c Release -r win-x64 --self-contained true \
  -o dist/windows-loopback-probe --tl:off
```

Copy the five `S4LoopbackProbe` executable, assembly, PDB, dependency manifest
and runtime-configuration files into the existing Windows probe directory.
Its native API, runtime, WinMD and `S4MidiProbe.exe` must remain alongside them.
The diagnostic does not install a Windows development environment.

Run these commands in order in that Windows directory:

```powershell
.\S4LoopbackProbe.exe --inspect
.\S4LoopbackProbe.exe --create
.\S4LoopbackProbe.exe --self-test
.\S4LoopbackProbe.exe --winmm-test
.\S4LoopbackProbe.exe --winmm-note-test
.\S4LoopbackProbe.exe --listen
.\S4LoopbackProbe.exe --remove
```

These are the recorded diagnostic cases, not an unattended success script.
Check each exit code; the data cases currently fail. After a failure, establish
that the diagnostic processes exited before choosing the next case. Do not
start another owner to work around an operation that remains active.

- Creation must report success for only `S4 Loopback Bridge` and
  `S4 MK3 Windows Probe LB`.
- `--self-test` compares an SDK connection on A with WinMM on B.
- `--winmm-test` uses WinMM on both ends, preserving the original CC return.
- `--winmm-note-test` uses the identical known-valid note in both directions.
- A passing data case must report the exact round trip, normal connection
  closure and `passed` with the retained pair association. A timeout is a
  failure, even when handles subsequently close normally.
- Listen sends no MIDI, records clients on both endpoints, and captures for
  20 seconds. Port discovery alone does not establish mapped Djay feedback.
- Removal must report success. It only targets the names and unique product
  identities owned by this diagnostic, preserving other loopbacks.

Endpoint B retains the original probe prefix for comparison with the first
test. The current native peer selects the exact loopback name, enumerates
actual WinMM indices and checks the data rather than trusting SDK indices.

This harness has no HID, audio, screen or motor implementation and does not
install or modify Djay mappings.
