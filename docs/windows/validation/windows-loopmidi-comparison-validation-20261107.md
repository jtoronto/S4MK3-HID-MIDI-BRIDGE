# Windows loopMIDI comparison preparation - November 7, 2026

This is an isolated test build, not a replacement for the existing LoopBe
bridge or a published Windows release.

## Scope and preparation

- The user installed loopMIDI. A separate `S4 MK3 loopMIDI Test` port was
  created; Windows reported its input and output device interfaces as OK.
  Exiting LoopBe's tray application did not remove its existing port.
- The comparison source was copied to
  `.omo/evidence/loopmidi-comparison-20261107/source`. In that snapshot, the
  existing shared-bus backend selects the exact test-port name for input and
  output, and the generated Djay mapping uses the same endpoint name.
  Internal CLI and status terminology still says `loopbe`.
- A separate mapping named `S4 MK3 Bridge loopMIDI Test.djayMidiMapping` was
  staged outside Djay. Compared with the previously generated LoopBe mapping,
  its parsed contents were identical apart from `endpointName`. No Djay mapping
  was imported, selected, overwritten, or removed during preparation.
- The comparison host's staged-import and suggested-export filename was also
  changed to that distinct name. Its generator retains a private temporary
  filename and rejects an existing export destination.
- MIDI incident capture and the Encdr screen-helper implementation were
  retained. Neither LoopBe nor loopMIDI feedback protection was changed.

## Evidence

- Isolated host tests passed 112/112 on macOS and 112/112 on Windows.
- The cross-built Windows engine's hardware-free native runner passed 110
  shared, 269 engine and 2 CLI tests, with none ignored.
- The exact staged engine also exported a mapping offline on Windows. The
  exported file was byte-identical to the bundled mapping and named
  `S4 MK3 loopMIDI Test` as its endpoint. The export exited 0 and emitted no
  HID-open or MIDI-ready markers. Its files are under
  `C:\Users\toron\AppData\Local\Temp\S4LoopMidiOfflineExport-019f1d8844e34f509893ef3100f56a9a`.
- The comparison ZIP passed `unzip -tq`; SHA-256:
  `16f5c3e7a51e593129c3b275e1bc65213dfc3619e04614f5cb4d4eefe84b0dae`.
  Windows transfer hashes matched the package, engine and mapping:
  `56446adf6dda0af83ec677afbbc1e5ac4a22c13c28d59068fb4dd5ddfd8d479b`
  for the engine and
  `b76956cc6b8110ecfa8ef193625c1cef18b429d83b5110c88a659b9babab8c8f`
  for the mapping.
- The staged app and mapping are under
  `C:\Users\toron\AppData\Local\Temp\S4LoopMidiComparisonV2-5c1c7fc433ab44a285aaefec57738959`.
  The engine and host test results are under
  `C:\Users\toron\AppData\Local\Temp\S4LoopMidiEngineNative-9d04e041853847deb15e16082d3cba3d\results`
  and
  `C:\Users\toron\AppData\Local\Temp\S4LoopMidiHostV2Native-b3007b1786ca4a419e5515cdc8d85aea`.
- An independent source review approved the isolated changes. With the user's
  permission, the stopped diagnostic host's 60,000-character live log and
  existing log and preferences were backed up before it was quit. The prior
  `last-session.log` was restored after preserving the post-quit log.
- The staged comparison host was opened without starting the bridge. Its
  window identified the loopMIDI comparison, named the test port, and reported
  the bridge stopped; the Diagnostics page showed capture idle and its
  session-only opt-in off. All five preference files and the original
  `last-session.log` matched their pre-switch hashes afterward. The live-log,
  log and preference backups are under
  `C:\Users\toron\AppData\Local\Temp\S4ComparisonPreserve-4046b400d80e42068519429b055f676c`.

## Limits

The tests and device enumeration do not prove MIDI delivery through loopMIDI,
Djay association, feedback, scratch behavior, or controller operation. No
test MIDI, HID, screen, or motor output was sent during preparation. A
separate attended comparison must verify those behaviors with the user
watching, starting from an agreed baseline. The user's earlier report that
screens stopped working and failed to recover remains open independently.
