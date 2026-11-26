# 0.4.0-beta.1 release preparation — 2026-11-25

This is a local preparation record, not a publication or hardware acceptance
record. The Mac bundle identifies itself as 0.4.0, build 9, with release
identifier `0.4.0-beta.1`.

## Apple Silicon Mac

Built with `CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh` and
`S4_BRIDGE_OUTPUT_DIR` pointing to `dist/release-0.4.0-beta.1/macos`.
The app's ad-hoc signature (`codesign --verify --deep --strict`), ZIP integrity
(`unzip -tq`) and disk image (`hdiutil verify`) passed. This is not Developer
ID signing or notarization. Formatting, strict all-feature/all-target Clippy,
the all-target Rust build, 82 Swift tests, shell syntax and plist checks passed.
The offline Rust library suite passed 138 tests, and all four managed NI
process tests passed. No live controller or Windows runtime check was made.

| Download | SHA-256 |
| --- | --- |
| `S4 MK3 Bridge-0.4.0-beta.1-arm64.dmg` | `514353a75025f2a04d8c1ddf7814c9f68e7ca0956233d89c471adf1d98610117` |
| `S4 MK3 Bridge-arm64.zip` | `ad6534e914528c9e62d6b7d66258ad943444b93d07460ec012281e38d837c631` |

## Windows x64 — private review candidate

The portable candidate and its dependency records are under
`dist/windows-candidates-0.4.0-beta.1/inventory-review/`, outside the release
downloads. The ZIP's SHA-256 is
`3007e6499e4c074f1f98795fc5bf7d8bd7ee8e4880a9db77582c44e0e5daff8e`.
All 622 files passed archive and manifest checks. Installer input validation
passed; no Setup executable was built or run.

The Windows setup uses Preview 14 **Transports** for Basic Loopback. The
app uses `Windows.Devices.Midi2` version `0.99.92-preview.11`, as directed
by Microsoft's [Preview 14 API notes](https://github.com/microsoft/MIDI/releases/tag/inbox-preview-14#api).
No change of API package is needed.

Do not publish this ZIP. `REVIEW-GAPS.md` alongside it records the third-party
debug symbols, corresponding-source delivery, and dependency notices still
requiring review. This package has not been run on Windows.

## Final local Windows artifacts — 2026-11-26

Use `dist/release-0.4.0-beta.1/windows/`, not the earlier private review
directory. The portable was rebuilt from `dce87c1` with a packaging-only
change to omit `Windows.Devices.Midi2.pdb` before generating the file
manifest. All 621 portable package files passed verification. Inno Setup
7.1.1 compiled the per-user installer from that exact portable payload.

| File | SHA-256 |
| --- | --- |
| `S4-MK3-Bridge-0.4.0-beta.1-win-x64-Setup.exe` | `7d61a60021ed26d71bab567ea869ce4dd8414d05e6934a85592e270bc3942687` |
| `S4-MK3-Bridge-0.4.0-beta.1-win-x64.zip` | `c83635dccacb98d51b16d9a7cfa0ac8333c1788729304d072812c11205b5226b` |
| `S4-MK3-Bridge-0.4.0-beta.1-win-x64-source.tar.gz` | `b8f25727ed0a90a6669a01f9a18b2f76c5193046b851f4bdbf4563dae859ca2e` |

An independent extractor (innounp 2.71.4) unpacked the final installer.
All 620 files listed in the portable manifest matched, as did the manifest
itself. No payload files were missing or extra. The installer is unsigned.

All three checksum sidecars verified locally. The source archive retains
the packaging patch, locked inputs, build instructions and notices. Read
its `MAINTAINER-REVIEW.md` before publishing. No native Windows install,
upgrade, uninstall, or controller session was exercised with these files.
