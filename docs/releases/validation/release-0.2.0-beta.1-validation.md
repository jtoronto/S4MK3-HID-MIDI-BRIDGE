# Release preparation: 0.2.0-beta.1

October 24, 2026. Internal validation record. User-facing notes are in
[release-0.2.0-beta.1.md](release-0.2.0-beta.1.md).

## Scope and baseline

The previous published release is `v0.1.1-beta.3`, at `d1eb4db`.
The new notes describe the net functionality since that release, not the
intermediate screen, driver or key investigations. The user selected a minor
version milestone: `0.2.0-beta.1`, numeric bundle version `0.2.0`, build `5`.

Feature work through `1141e1c` and the maintained-guide commit `244fb9d`
were pushed to `main` before final release preparation. The final preparation
changes version metadata, release notes and availability documentation only.
The portable packages include the same controller behavior as the successful
[hour-long DJ-laptop set](dj-laptop-set-validation.md).

This preparation targets an unpublished GitHub draft with DMG and ZIP assets.
Publication is a separate action. The README continues to identify
`0.1.1-beta.3` as the available published download until then.

## Source checks

The final `0.2.0-beta.1` working tree passed:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
swift test --package-path macos
bash -n macos/build-app.sh
plutil -lint macos/Info.plist
git diff --check
```

All 322 Rust tests and 55 Swift tests passed, with no test failures or ignored
Rust tests. The existing SwiftPM unhandled-file warnings for
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh` remain.
The updated user/release documentation passed 76 local-link/anchor checks.
Hosted links were not checked by that validator.

## Package build and verification

Built with:

```sh
CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh
```

Outputs:

- `dist/S4 MK3 Bridge.app`
- `dist/S4 MK3 Bridge-arm64.zip`
- `dist/S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg`

The app, ZIP-extracted app and read-only-mounted DMG app each passed strict
deep signature verification, plist/mapping lint, release/numeric/build checks
and the bundled CLI's version check. All three default configurations and the
screen-font license matched the source resources byte-for-byte. The generated
mapping matched across all three copies. The build's linked-library check
found no non-system absolute dependency in either executable.

The production `BridgeController`, compiled with the app sources and
`APP_TESTS`, initialized from each copy's actual resources, saved to a unique
temporary support directory and reloaded matching LED, jog, tension and
screen preferences. No child bridge was started. The first checker invocation
omitted its production source arguments; the corrected invocation compiled
successfully and the same checker verified the final minor-version packages.
The temporary support directories were removed.

ZIP integrity and `hdiutil verify` passed. No HID/MIDI ports were opened,
no motor or screen commands were sent, and saved user preferences were not
changed. This was packaging verification, not a new physical acceptance test.
Focused hardware/recovery limits remain in their existing validation records.

Previous packages are preserved in
`dist/previous-before-beta4-20261024.0dF0Mz/`. The unpublished initial beta.4
packages were retained in
`dist/unpublished-beta4-before-020-20261024.9JU8rH/`; they were never released.

## SHA-256

```text
8086cf4cff32b2b38cfe9522c9e87f1a12592af8c8cb70aed436c5587366fde3  S4 MK3 Bridge-arm64.zip
01b5cf14826ba9fef2c41aa0473547d68fa10d119ceb2ce4bb32984f1ea9a634  S4 MK3 Bridge-0.2.0-beta.1-arm64.dmg
```
