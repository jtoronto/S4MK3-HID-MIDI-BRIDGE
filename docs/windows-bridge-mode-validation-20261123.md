# Windows bridge mode integration

November 23, 2026. Local software validation, not Windows hardware
acceptance or a release.

The ordinary app saves **Direct HID** or **NI Bridge** in its run preferences.
Older run files without a mode open in Direct HID. Mode changes require
stopping the bridge. NI Bridge always launches the existing full, continuous
BLOOP route through the same endpoint owner as Direct HID. The former
attended launchers remain available for their separate diagnostic packages.

Direct HID does not request screens or start a screen worker. NI Bridge
uses a single NI session for screens, platter drive and jog rings. Its
screen status follows the running engine. Direct HID platter tuning is
retained for that mode; NI Bridge uses the engine defaults for TT
translation and does not use bridge motor assist. Vinyl startup, software
backspin and Scratch/Pitch Bend mapping settings remain available in both
modes. No motor command was added. Scratch beat haptics remain deferred.

The Bridge tab queries the Windows MIDI Services SDK without creating an
endpoint. The SDK may start the Windows MIDI service on demand. A negative
BLOOP result is displayed as unavailable, not proof that the package is
absent. The app links to Microsoft's Preview 14 x64 Transports installer
and instructions, then lets the user recheck. It does not install anything.
No WinGet manifest was verified for the specified Preview 14 Transports
asset. The [Windows development setup](windows-setup.md) provides an
optional PowerShell command that downloads, verifies and opens the same
installer, not an installer-free deployment.

## UI review

The approved layout keeps the compact **Bridge mode** picker with both
descriptions visible, the Basic Loopback status and recovery links, and
mode-specific Jog settings. The existing Vinyl startup switches stay
visible in both modes. NI jog tension is deferred; the Direct HID
Wheel tension control is unchanged.

Selected before/after Avalonia captures and their limits are retained
locally under `.omo/evidence/windows-bridge-mode-ui-20261123/`.

## Verification

- Portable host suite: 131 of 131 passed, including preference migration,
  saved NI launch and screen status, and BLOOP recovery UI. Run with the
  native Mac test apphost and pinned .NET runtime.
- Windows x64 desktop host and Rust engine cross-builds succeeded. The
  engine's Windows test executables were compiled but not run.
- Local Rust formatting, tests, strict Clippy and build, all 80 Swift
  tests, build-script syntax, plist lint and `git diff --check` passed.
- Headless Avalonia rendered both modes in light and dark at two window
  sizes and exercised the picker and dependency states. An actual Windows
  session is still needed to confirm the installer link, SDK discovery on a
  missing-transport machine, and physical screen and platter behavior.

No installer or bridge executable was run on Windows, and nothing was
deployed or published.
