# Windows implementation: current state

As of November 7, 2026, the source tree contains a Windows implementation
and private development packages. Windows support is **not
accepted or published as a ready-to-use download**. Historical test results
below apply to the particular builds in their records, not necessarily every
subsequent working-tree change. Some detailed records link to local
`.omo/evidence` artifacts that are not tracked in Git.

## Architecture

- The shared Rust bridge handles HID, control translation, MIDI, feedback,
  LEDs and jog behavior. The Windows full-profile mapping and WinMM transport
  use the separately installed LoopBe1 shared MIDI bus. Feedback has separate
  address domains so outgoing control echoes are not treated as Djay state.
- `windows/bridge-app/` is a self-contained win-x64 .NET/Avalonia tray host.
  It stores preferences, generates mappings through the packaged engine,
  supervises the bridge process, and provides settings and diagnostics.
  Mapping export preserves existing destinations. The optional MIDI incident
  capture records incoming messages before filtering and outgoing attempts;
  it cannot prove delivery or identify the sender on the shared bus.
- The engine-managed screen route uses a separate supervisor and Encdr-based
  worker to request access through NIHardwareService. Screen failure is
  reported separately from controller/MIDI operation. The supported Windows
  workflow keeps the service running; automatic service stop/restore is not
  supported. The tested provisioning route assumes Traktor is installed for
  the NI driver and service, but Traktor need not run with Djay; standalone
  service provisioning is unverified.
- The screen worker pins Encdr revision `a5414658a69c4330d10b95f2359371dfc8550326`
  through Cargo. The main bridge retains its separate Encdr revision and a local
  patched Mio dependency. Offline screen builds need the pinned revision in
  Cargo's cache. Populate it with
  `cargo fetch --locked --manifest-path windows/encdr-screen-worker/Cargo.toml`
  before an offline build.

See [Windows setup](windows-setup.md) and the
[user guide](user-guide.md) for operational instructions, and
`windows/build-bridge.sh`, `windows/build-screen-worker.sh`, and
`windows/build-app.sh` for build entry points.

## Evidence and unresolved behavior

- **Software:** the November 7 MIDI-capture development build passed a
  hardware-free Windows native runner with 110 shared, 269 engine and two CLI
  tests, plus 111 host tests. An earlier Encdr integration build passed its
  Windows host, worker and supervisor suites. These are build-specific
  component checks, not end-to-end product acceptance. See
  [capture validation](windows-midi-capture-validation-20261107.md) and
  [screen integration validation](windows-encdr-screen-integration-validation-20261107.md).
- **LoopBe and feedback:** a bounded isolated proof demonstrated Djay
  crossfader control, independent mouse-driven feedback and rejection of
  outgoing echoes. MIDI-driven changes did not return feedback in that proof.
  A later attended bridge run confirmed physical crossfader control and lit
  LEDs, and observed genuine feedback, including deck-A playback feedback.
  This does not prove every feedback family or control. See the
  [LoopBe proof](windows-djay-loopbe1-proof-20261104.md) and
  [recovery record](windows-recovery-validation-20261104.md).
- **Startup:** starting the bridge before Djay has received startup feedback;
  opening the receiver after Djay can leave fields unknown until values
  change. Initial feedback remains an unresolved reliability boundary, not a
  guarantee that all fields synchronize on startup.
- **loopMIDI is a separate experiment**, not the current LoopBe1 backend
  configuration. A separately named test port, mapping and private comparison
  build were prepared; its mapping differed in endpoint name. An attended
  test confirmed crossfader control and Djay-driven cue lights. Switching
  configurations brought the lights alive, but Djay became unresponsive;
  the user later observed recovery after a large temporary memory increase.
  A later repeat did not stall. The cause and FX-knob recovery after switching
  remain unknown; this is not an established safe workaround. See
  [comparison validation](windows-loopmidi-comparison-validation-20261107.md).
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
requires its access declaration and both helper binaries. Djay, LoopBe1 and
the NI components are separate dependencies; the package does not install a
driver.

Private builds can omit the engine/host dependency inventory and the screen
helper source/notice inventory. The packager records omissions in
`PACKAGE-STATUS.txt` and `runtime-manifest.json` and validates inventories
when supplied. Actual private no-inventory builds are not evidence of
distribution compliance: review applicable source and license obligations
before sharing. A packaged development build is not a published release or
proof of hardware readiness.

Use a new output path for every build, including its `.zip.sha256` sidecar:
the build script checks the directory and ZIP but can overwrite an existing
sidecar. The host restore uses a local NuGet source, but the packager's
separate `dotnet run` has not been established as offline-only.
