# LoopBe1 WinMM feasibility probe

This isolated diagnostic tests a separately installed LoopBe1 driver. It does
not access the S4 controller or create Microsoft virtual devices. Native MIDI
delivery uses WinMM, with no proprietary SDK. The optional
Djay observation uses the existing Microsoft API projection only to report
active sessions.

Build on the Mac after building `windows/midi-probe`:

```sh
diagnostics/windows-tools/dotnet-10.0.403/dotnet publish \
  windows/loopbe-probe/S4LoopBeProbe.csproj \
  -c Release -r win-x64 --self-contained true \
  -o dist/windows-loopbe-probe --tl:off
```

The five `S4LoopBeProbe` executable, assembly, symbols, dependency manifest and
runtime configuration can be staged beside the existing portable probe runtime
and native reporting API. No Windows development installation is needed.

Run one operation at a time and check its exit code:

```powershell
.\S4LoopBeProbe.exe --inspect
.\S4LoopBeProbe.exe --self-test
.\S4LoopBeProbe.exe --self-test
.\S4LoopBeProbe.exe --listen
.\S4LoopBeProbe.exe --listen-raw
```

`--self-test` opens the exact `LoopBe Internal MIDI` input/output, starts a
separate native WinMM peer and checks Note On `0x00357791` in one direction
and CC `0x00486FBF` in reverse. The parent requires the exact CC; receipt of
its own transmitted note cannot pass the return test. The child holds its
handles until the parent acknowledges receipt. A success requires peer
exit 0, normal handle closure and `passed` with `twoWay=true`.

`--listen` sends no MIDI, reports current service sessions and records incoming
short messages for 20 seconds. No received messages without a selected mapping
or GUI actions is not proof that Djay feedback is unsupported.

`--listen-raw` records WinMM packets for 120 seconds without querying Microsoft
MIDI Services sessions, allowing time for observed GUI actions. It requires
LoopBe1 but not the MIDI Services reporting class registration. It sends no
MIDI and accesses no controller hardware.

LoopBe1 is an installed driver dependency, not part of this portable executable.
Its personal/non-commercial and commercial/bundling terms must be considered
separately from SDK access.

## Isolated Djay routing proof

The supplied **Embedded Proof** mapping binds the crossfader to channel 5 CC 20
and explicitly sends continuous feedback on channel 8 CC 20. Import it under
its separate name, record the previous configuration and crossfader position,
and keep playback paused. The other **Proof** mapping is the earlier root-output
candidate; it did not produce feedback in the recorded test.

Additional operations:

```powershell
.\S4LoopBeProbe.exe --route-check
.\S4LoopBeProbe.exe --legacy-winrt-check
.\S4LoopBeProbe.exe --crossfade-right
.\S4LoopBeProbe.exe --crossfade-left
.\S4LoopBeProbe.exe --watch-crossfade-right
.\S4LoopBeProbe.exe --watch-crossfade-left
```

`--route-check` checks the actual cache's address boundary without opening MIDI.
`--legacy-winrt-check` sends a known packet through Windows' legacy MIDI API
and requires native WinMM reception; it is a transport comparison, not Djay
feedback evidence.

The `--crossfade-*` operations send a command and require its ignored echo
plus genuine feedback. The observed MIDI-driven commands moved Djay, but these
combined checks timed out waiting for feedback; their failures are retained.

The `--watch-crossfade-*` operations send nothing. Wait for `crossfader-ready`,
then move Djay's onscreen crossfader by mouse to the requested endpoint within
120 seconds. The successful independent GUI proof used the Embedded mapping.
Only channel 8 CC 20 can update the cache; control echoes remain ignored.
Afterward restore the original position/configuration and remove only the
separately imported test mappings.
