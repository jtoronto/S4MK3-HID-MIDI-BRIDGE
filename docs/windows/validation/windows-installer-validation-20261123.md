# Windows ZIP and installer validation

November 23, 2026. This is a local packaging check, not a release or a
live Windows acceptance run.

The portable ZIP remains the canonical build. `windows/build-installer.py`
checks its SHA-256 sidecar and runtime manifest, then packages the same
payload using Inno Setup. Setup installs for the current user, adds one
Start menu shortcut and registers an uninstaller. It does not run the
bridge, install prerequisites or write to the bridge preferences or djay
mapping directories.

The x64 bridge engine and app were cross-built. The ZIP passed integrity and
checksum checks. The installer compiler was Inno Setup 7.1.1, verified
against the upstream release asset SHA-256. The build wrapper rejected ZIPs
with an invalid hash, a parent-directory path or an incomplete payload,
and refused to replace an existing output.

The exact installer rejected Wine's Windows build 22000, which is below
the Windows 11 25H2 requirement. An otherwise identical test installer
with only the minimum OS build lowered was exercised in a disposable Wine
prefix. It refused an unowned install folder, installed all 239 runtime
files byte-for-byte plus the manifest, registered an HKCU uninstall entry,
and created the Start menu shortcut. Reinstall repaired a removed payload
file. Uninstall removed the app, shortcut and registration and preserved
both fixture preferences and a fixture djay mapping. The app was not started.

Still to verify on Windows 11 25H2 or later: the exact installer under a
standard account, Start menu launch, running-app refusal, a version-to-version
upgrade, and uninstall. No driver, MIDI service, controller, or djay
configuration was touched. The private artifacts are unsigned and omit
the optional reviewed dependency inventory; they are not distribution-ready.

The exact candidates and their SHA-256 sidecars are retained locally in
`dist/Windows Installer Review-20261123/` for that Windows pass. The build
and isolated lifecycle receipts are in `.omo/evidence/windows-installer-20261123/`.

## Kensuke smoke result and icon follow-up

The user reported that the original installer worked on Kensuke. That
report does not separately establish standard-user, upgrade, or uninstall
coverage.

The branded rebuild uses the same artwork as the tray, without its live
status badge. Both the published `S4Bridge.exe` and Setup contain all nine
matching icon sizes (16–256 px). The Start menu shortcut and Installed Apps
entry both reference `S4Bridge.exe` for their icon. The ZIP and Setup
checksums passed; the bundled engine and djay mapping are unchanged.
The branded candidates are in `dist/Windows Installer Icon Review-20261123/`.
The rebuilt Setup was copied to Kensuke's desktop and its SHA-256 matched
the retained build (`efe24fa3e467c75e94805fa7631052e9d3928f23933719be305da501fa75ffe1`).
The user reported that it tested successfully on Kensuke. The report did
not specify which icon locations or installer lifecycle steps were checked.
