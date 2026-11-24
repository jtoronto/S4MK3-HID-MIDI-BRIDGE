# Windows MIDI incident capture validation - November 7, 2026

This record covers a temporary development diagnostic for the reported
LoopBe mute. It does not establish the cause of that incident or full
Windows product acceptance.

## Verification

- The Rust formatting check, strict all-target/all-feature Clippy, full
  `cargo test --locked`, all-target build, and `git diff --check` passed on
  macOS after the capture-finalization correction.
- The Windows x64 engine payload was cross-built from the corrected source.
  Its hardware-free native runner passed 110 shared, 269 engine, and 2 CLI
  tests, with none ignored. The native host suite passed 111 tests. Test
  evidence is under
  `C:\Users\toron\AppData\Local\Temp\S4MidiCaptureEngineNative-e39b3e7b24ad4c35bde0dc8c4f0f8233`
  and
  `C:\Users\toron\AppData\Local\Temp\S4MidiCaptureHostNative-9038b7b3d01a42aaac5b0d7707333807`.
- Independent reviews approved the host controls and the recorder. A
  discovered cap-notification failure was fixed: the writer drains queued
  events and writes its final summary before reporting the notification
  error. Its deterministic regression was also reviewed.
- The development ZIP passed `unzip -tq` and has SHA-256
  `c7e33a28763a5c1b634f726820a156de30bdd8090f3e3bec40dc945cbe4a6161`.
  The bundled Windows engine has SHA-256
  `17de6070a014ce65e0779752ac81cf630519c48ea8e2a004f0a812641d8202cb`.
  Both matched after transfer to Windows.
- The staged app opened on Windows with the bridge stopped. The Diagnostics
  opt-in checkbox was toggled on and back off; the visible status changed to
  armed and no bridge or helper process was started. All five preference
  files and the prior session log matched their pre-check hashes.

## Limits

No attached-controller capture or physical reproduction was performed for
this diagnostic. The native host and engine tests cover their respective
software boundaries; they are not a live end-to-end MIDI capture. The app
was left open with the bridge stopped and capture unarmed.

An indefinitely stalled parent stdout pipe can block the capture writer's
status notification and delay its final join. The managed host normally drains
stdout independently and reports an unconfirmed stop after its timeout, but
the diagnostic does not guarantee bounded shutdown under a stalled pipe.
Capture producers do not perform file I/O or wait for queue capacity.
