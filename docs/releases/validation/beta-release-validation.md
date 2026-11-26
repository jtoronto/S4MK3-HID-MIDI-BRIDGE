# First beta packaging validation

Release: `0.1.0-beta.1`. Validated October 6, 2026 on Apple Silicon macOS.

## Identity and artifacts

- Cargo package and executable: `s4mk3-hid-midi-bridge`.
- App: `S4 MK3 Bridge.app`, bundle ID `com.s4mk3.bridge`.
- macOS numeric version: `0.1.0`; build: `1`.
- Complete release identifier: `S4BridgeReleaseVersion=0.1.0-beta.1`.
- DMG: `dist/S4 MK3 Bridge-0.1.0-beta.1-arm64.dmg`.
- Alternative archive: `dist/S4 MK3 Bridge-arm64.zip`.

The build reads the release version from Cargo and verifies the bundle metadata
before packaging. The existing settings location and custom mapping names are
unchanged. The approved icon is stored at `macos/Assets/AppIcon.png`; packaging
generates all ten standard icon representations in `AppIcon.icns`.

## Checks and installed-package evidence

All 81 Rust tests and 12 Swift tests passed. Formatting, strict Clippy, locked
all-target build, Swift diagnostics, shell syntax, plist lint, and whitespace
checks passed. The lockfile changed only the root package name/version.

The app passed deep strict signature verification; the ZIP and DMG passed
integrity verification. The DMG was mounted read-only and its Applications
shortcut resolved to `/Applications`. Its app was copied into an isolated
installation directory and its signature verified there.

The copied executable reported `s4mk3-hid-midi-bridge 0.1.0-beta.1`.
The production Swift controller loaded the copied bundle's calibrated LED
and jog defaults and exported a mapping through that real executable.
Export completed asynchronously with the source controller still stopped.
AppKit decoded all ten icon representations; native icon conversion confirmed
the expected dimensions from 16 through 1024 pixels.
Independent functional and visual icon reviews passed, covering all ten
representations against the approved artwork. The artwork's opaque white
outer background is retained from the approved source.

Saved LED preferences and the user's `S4 MK3 Bridge 2` mapping retained their
original hashes. The previous app/archive were preserved at
`/tmp/s4mk3-pre-beta.S1ymre`.

DMG SHA-256:

```text
34e549a643e531c2fb9752bf631e6a0baef27f3274aaab93011e7787f993313a
```

## Limits

Signing remains ad-hoc, without Developer ID signing or notarization.
Fresh-download quarantine/Gatekeeper behavior was not tested; the README
describes the expected system approval flow without claiming a verified
fresh-download pass. No GitHub release or tag was published.

Package QA did not open HID, import a mapping into Djay, or alter the running
user bridge. Previous live controller confirmations remain recorded in the
issue checklist; no additional physical behavior is established by packaging.
Known beta limitations are summarized in the README and `docs/issues.md`.
