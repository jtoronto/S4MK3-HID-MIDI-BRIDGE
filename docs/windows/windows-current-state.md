# Windows implementation: current state

The source tree contains a Windows implementation and private development
packages. Windows support is **not
accepted or published as a ready-to-use download**. Historical test results
below apply to the particular builds in their records, not necessarily every
subsequent working-tree change. Dated records may name temporary artifacts
for provenance; those paths are not required to build or use the app.

## Architecture

- The shared Rust bridge handles HID, control translation, MIDI, feedback,
  LEDs and jog behavior. The Windows app now creates its own Basic Loopback
  (BLOOP) endpoint, **S4 MK3 Bridge BLOOP**, through Windows MIDI Services.
  The worker still uses WinMM. LoopBe remains an explicit CLI backend, while
  the normal app and generated Windows mapping use BLOOP. Feedback has separate
  address domains so outgoing control echoes are not treated as Djay state.
- `windows/bridge-app/` is a self-contained win-x64 .NET/Avalonia tray host.
  It stores preferences, generates mappings through the packaged engine,
  supervises the bridge process and its endpoint lease, and provides settings
  and diagnostics. The endpoint is receipted before publication, retained while
  the worker is live, and removed after actual exit and output drain. Failed
  cleanup blocks Start and can be retried with Stop. Recovery preserves
  unowned and saved conflicts rather than deleting by name.
  Mapping export preserves existing destinations. The optional MIDI incident
  capture records incoming messages before filtering and outgoing attempts;
  it cannot prove delivery or identify the sender on the shared bus.
- The normal app saves a choice between **Direct HID**, with native motor and
  ring output and no screens, and **NI Bridge**, with one NIHardwareService
  session for the screens, platters and rings. The modes do not share motor
  tuning. The NI-mode JOG tension slider sends one global unsigned value on
  the existing screen session; it does not enable native motor/ring writes.
  The [attended preset comparison](research/windows-ni-jog-tension-2026-11-24.md)
  verified the difference in feel and both screens on KENSUKE. That was the
  trial build, not a physical run of the integrated slider.
  The tested NI provisioning route assumes Traktor is installed for
  the driver and service, but Traktor need not run with Djay. Standalone
  service provisioning and the new app-level mode selection have not been
  qualified on Windows hardware. See the
  [NI trial validation](validation/windows-ni-decks-validation-20261123.md).
- The separate screen/motor diagnostic route uses a supervisor and
  Encdr-based worker. That worker pins Encdr revision `a5414658a69c4330d10b95f2359371dfc8550326`
  through Cargo. The main bridge retains its separate Encdr revision and a local
  patched Mio dependency. Offline screen builds need the pinned revision in
  Cargo's cache. Populate it with
  `cargo fetch --locked --manifest-path windows/encdr-screen-worker/Cargo.toml`
  before an offline build.

See [Windows development setup](windows-setup.md) for operational instructions, and
`windows/build-bridge.sh`, `windows/build-screen-worker.sh`, and
`windows/build-app.sh` for build entry points.

## Evidence and unresolved behavior

- **Earlier diagnostics:** the [November 7 Encdr run](validation/windows-encdr-pattern-validation-20261107.md)
  verified independent solid-color display updates, not production pages.
  The [November 9 wheel trace](validation/windows-motor-timing-validation-20261109.md)
  reproduced chatter on an older loopMIDI build without establishing its cause.
- **BLOOP source integration:** the SDK boundary and integrated Windows host
  cross-build, and the real Windows worker and its native test executables
  cross-build. Portable process/host and packager regressions pass. These are
  local software checks. Subsequent attended runs passed native SDK activation
  and endpoint ownership cases, 117 host tests and 382 Rust tests, actual UI
  export/readiness, and initial playback/cue feedback on two connections.
  Physical crossfader control reached Djay, and independent GUI playback
  changed the physical PLAY light. Actual app Stop, running Quit, stale owned
  recovery, visible conflict rejection and retry also passed. The test package
  and mapping were removed, six original preference files restored exactly,
  and twelve original mappings retained unchanged. Broader control/LED,
  screen, motor and audio acceptance remains open. Historical LoopBe
  or standalone BLOOP transport results do not establish this integrated app's
  behavior. See the
  [BLOOP integration validation record](validation/bloop-integration-validation-20261115.md)
  for retained checks, package identity and bounded acceptance evidence.
- **Software:** the November 7 MIDI-capture development build passed a
  hardware-free Windows native runner with 110 shared, 269 engine and two CLI
  tests, plus 111 host tests. An earlier Encdr integration build passed its
  Windows host, worker and supervisor suites. These are build-specific
  component checks, not end-to-end product acceptance. See
  [capture validation](validation/windows-midi-capture-validation-20261107.md) and
  [screen integration validation](validation/windows-encdr-screen-integration-validation-20261107.md).
- **Historical LoopBe and feedback:** a bounded isolated proof demonstrated Djay
  crossfader control, independent mouse-driven feedback and rejection of
  outgoing echoes. MIDI-driven changes did not return feedback in that proof.
  A later attended bridge run confirmed physical crossfader control and lit
  LEDs, and observed genuine feedback, including deck-A playback feedback.
  This does not prove every feedback family or control. See the
  [LoopBe proof](validation/windows-djay-loopbe1-proof-20261104.md) and
  [recovery record](validation/windows-recovery-validation-20261104.md).
- **Historical startup:** starting the LoopBe bridge before Djay has received startup feedback;
  opening the receiver after Djay can leave fields unknown until values
  change. Initial feedback remains an unresolved reliability boundary, not a
  guarantee that all fields synchronize on startup.
- **loopMIDI is a separate experiment**, not the normal BLOOP backend
  configuration. A separately named test port, mapping and private comparison
  build were prepared; its mapping differed in endpoint name. An attended
  test confirmed crossfader control and Djay-driven cue lights. Switching
  configurations brought the lights alive, but Djay became unresponsive;
  the user later observed recovery after a large temporary memory increase.
  A later repeat did not stall. The cause and FX-knob recovery after switching
  remain unknown; this is not an established safe workaround. See
  [comparison validation](validation/windows-loopmidi-comparison-validation-20261107.md).
- **Screens:** a bounded attended Encdr test showed independent left/right
  HOTCUE and STEMS pages, Off/On, Retry and stop/start, with clean process
  exit and NIHardwareService left running. Subsequently the user reported
  screens stopped updating during MIDI diagnosis and remained unavailable
  after bridge restart; the cause and recovery remain open. Registration or
  an active status alone does not prove physical output or complete fields.
  Source review also identified a possible startup race: a second screen-state
  initialization can overwrite an early active status. This has not been
  reproduced or linked to the reported screen failure.
- **Motors, audio and acceptance:** a screens-off, minimum-force control run
  reported no platter movement; an accepted final neutral write is not
  physical motor validation. Both jogs at normal settings, audio/pre-cue
  coexistence, complete control and screen coverage, fault recovery and
  sustained combined-load acceptance remain open.
- **Shutdown and retry review:** if writing a managed stop status fails, the
  current stop path can fail to signal the runtime after marking stop
  requested. Source review also found that a screen retry can proceed after
  an unconfirmed release. Neither failure has been reproduced on hardware;
  both need regression and recovery work before acceptance.

## Packaging and distribution

`windows/build-app.sh` can produce an unsigned, self-contained integration
package with the actual Windows engine and matching generated mapping, or a
host-only handoff without them. Screen-enabled integration additionally
requires its access declaration and both helper binaries. Djay, Windows MIDI
Services with Basic Loopback (provided by the Preview 14 Transports
installer), and the NI components for NI Bridge are separate dependencies;
the package does not install a driver.

Private builds can omit the engine/host dependency inventory and the screen
helper source/notice inventory. The packager records omissions in
`PACKAGE-STATUS.txt` and `runtime-manifest.json` and validates inventories
when supplied. Actual private no-inventory builds are not evidence of
distribution compliance: review applicable source and license obligations
before sharing. A packaged development build is not a published release or
proof of hardware readiness.

Use a new output path for every build, including its `.zip.sha256` sidecar:
the build script checks the directory and ZIP but can overwrite an existing
sidecar. Both host and packager restore from the local NuGet cache; the
packager then runs with `--no-restore`. Build intermediates and generated SDK
projection files live in a fresh directory under `TMPDIR`, removed when the
build script exits.

The optional per-user installer is built from the ordinary x64 portable ZIP,
not from a second publish. Keep the ZIP and its `.sha256` alongside each other.
With Python 3.11+ and Inno Setup 7's `ISCC.exe`:

```sh
python3 windows/build-installer.py \
  --archive /absolute/path/to/S4-MK3-Bridge-win-x64.zip \
  --output /absolute/path/to/existing/output-directory \
  --iscc /absolute/path/to/ISCC.exe \
  --version 0.3.0-beta.1
```

On macOS, use Wine and set `WINEPREFIX` to the isolated compiler prefix.
The installer is unsigned and remains a development artifact. A Windows
acceptance pass must verify user-level install, upgrade, Start menu launch
and uninstall on a supported PC before publishing.

The developer machine needs .NET SDK 10.0.403 and cached dependencies, including
Windows.Devices.Midi2 0.99.92-preview.11, CsWinRT 2.2.0 and Windows SDK metadata
10.0.26100.57. Microsoft's Preview 14 transport notes still specify the Preview 11 API;
the interim Tools DLL is not a build input. On macOS, Wine generates
the projection. Set `S4_BRIDGE_WINEPREFIX` to an existing isolated Wine prefix
and `DOTNET` to the retained SDK executable. The DJ laptop needs neither Wine
nor these developer tools. SDK notices and metadata are bundled with the app.
