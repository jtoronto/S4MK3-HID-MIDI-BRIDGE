# Windows Encdr screen integration validation - November 7, 2026

This record covers a development package, not a published release or full
Windows product acceptance.

## Software and package

- The Windows native bridge runner passed 110 shared tests, 257 engine tests,
  and 2 CLI tests, with none ignored. Its records and output are under
  `C:\Users\toron\AppData\Local\Temp\S4BridgeResetNative-f014aacace274c4daccb8609259badd6\results`.
- The native supervisor suite passed 12 tests, including bounded stalled
  setup, send and stop and process-tree containment. The native worker suite
  passed 14 tests; an unarmed worker exited without contacting the NI service.
  The Windows host suite passed 99 tests.
- The macOS source checks, Rust tests and build passed. Swift tests passed
  80 tests. One macOS Avalonia headless host test run had an intermittent
  disposal exception; a separate local run and the Windows host suite passed.
  The cause was not established.
- Development ZIP SHA-256:
  `d2cafae9f47d80fc7df0c8ec19d926de0f841ce458ddbacbd7c8c2adae634993`.
  ZIP integrity passed. Its engine SHA-256 is
  `72b4b6a04a035b7605272d361a65c2c908312ee2db5bf53915a0a219346bcd0d`;
  the supervisor is
  `a620af7b36efc9c90fd19d0e040b123c34959e8a6a09e7ad70567a36b56315a8`;
  and the Encdr worker is
  `c8b121879fa9948c4ab5ae850ad575088e1d94dfe0953de26a0e35bbc3287702`.
  The transferred Windows package and executable hashes matched.

## Attended controller check

The user confirmed the controller was connected, playback paused, and they
were watching before the bridge was started. With the full profile running,
the app showed `Bridge: ready | Screens: active`.

- The user confirmed independent pages with left HOTCUE/right STEMS and then
  left STEMS/right HOTCUE.
- Screens were switched Off and On; the app reported Off and then Active.
  After Retry, the user confirmed that both displays still responded
  independently. The test did not require the displays to blank when off.
- The bridge was stopped and started again. The app returned to Ready/Active,
  and the user confirmed both screens still followed their respective modes.
- The bridge was stopped and the host quit. No bridge, supervisor, or worker
  processes remained. NIHardwareService remained running. All five saved
  preference files matched their pre-test SHA-256 hashes. The previous
  `last-session.log` was restored after preserving the attended session log at
  `C:\Users\toron\AppData\Local\Temp\S4EncdrPreferenceBackup-b70c08c70b584789a1d470ffb56108e5\attended-session.log`.
  The preserved final-session log reports an accepted final neutral write and
  exit code 0; this is not a physical motor acceptance test.

## Limits

The helper's forced-failure and process containment evidence is from
hardware-free native tests, not an induced fault on the attached controller.
The attended check does not establish audio, normal motor behavior, sustained
load, every control, or every recovery path. It does not make this package a
published or generally accepted Windows release.
