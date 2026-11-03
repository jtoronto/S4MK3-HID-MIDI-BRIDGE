# Jog settings cleanup validation

November 3, 2026. This record covers the source implementation of the approved
Jog mockup, not a rebuilt distribution package or a new hardware-feel comparison.

## Defaults source

Read `jog-config.json` and `tension-config.json` from
`~/Library/Application Support/S4 MK3 Bridge/`. Neither saved file was edited.
The Jog file remained byte-identical through verification, with SHA-256
`ef2c34f2f44dc39e82fbe393381ffcd86989a9b4a7216926d1ec88fb867d1eea`.

The resulting app defaults include:

| Control | Default |
| --- | ---: |
| Base force | 7,000 force units |
| Braking strength / shape | 10x / 1 |
| Nudge strength / shape | 3x / 1.6 |
| Scratch handoff | 1% of target speed |
| Target speed | 33 RPM |
| Maximum force | 20,000 force units |
| Extra backspin scratch speed | 5.2% |
| Wheel tension | -77 |

The saved shared gain was 5, with no explicit braking override; its existing
2x braking factor gives effective strength 10. The nudge override's floating-point
serialization was `3.0000000000000004`, represented as 3 in the defaults.
Other saved Jog values are retained in `examples/jog-config.json`.
The standalone Rust tension loader still uses neutral when no tension file is
supplied; the app's bundled tension file supplies -77 explicitly.

## Implementation

- `macos/JogSettingsView.swift` puts Base force and both response editors first.
  Advanced settings is an initially unchecked, editor-only disclosure.
- Every setting has a nearby user explanation. Legacy migration copy and the
  developer footer paragraphs are removed; graphs say Pitch bend rate.
- Restore defaults changes only motorized settings in the editor. It does not
  save, restart, change wheel tension, or reset backspin/mapping preferences.
- Absolute platter sensitivity and generated deck-channel CC8 Scratch entries
  are removed. Existing numeric retired keys remain readable but are omitted
  on save. Other unknown preference keys remain invalid.
- Ring-only physical encoder phase tracking remains in `src/full/ring_position.rs`.
  Manual Scratch, relative TT correction, held-note ownership, and ring rendering
  retain their existing contracts.
- Swift defaults, Rust Jog defaults, bundled JSON, the approved HTML preview,
  and current guide/reference descriptions are aligned. Historical validation
  records retain the tuning used at their tested revisions.

## Automated evidence

All required commands finished with exit code 0:

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

Rust: 510 tests passed across the command's targets. Swift: 80 tests passed.
The preview draft guard passed, and all 30 local user-guide link destinations
exist.

The new Swift scoped-reset regression covers all motorized fields, preserved
unrelated editors, unchanged saved bytes, and stopped process state. Removing
the Base force reset assignment made it fail with 3,500 versus 7,000; restoring
the assignment returned the full suite to green.

Default-sensitive replay expectations now use 10,630 force units for full
holding assistance (`7000 + 110 * 33`). Fixed-gain quantization and 2x braking
regressions explicitly select their original baseline gains/shapes rather than
depending on changing user defaults. Their sign, count, timing, ownership and
force assertions remain in place.

## Native visual evidence and limits

Compiled all native sources against the macOS 12 deployment target with an
isolated preview harness and temporary resources/support directory. The bridge
was stopped throughout; no HID, motor, screen, MIDI or NI-driver action was
triggered. Inspected the actual native form at 760 and 1,000 points, including
the collapsed overview, expanded playback/motor settings, wheel tension,
Backspin and mapping sections in light/dark appearance. No clipped setting
labels, overlapping controls or horizontal form overflow were observed.

Local captures and the harness are under
`.omo/evidence/jog-settings-20261103/`. Expanded screenshots use an initialized
expanded state and scroll anchors. Desktop Accessibility/input permission was
unavailable, so automated native mouse/keyboard clicking was not performed.
Scoped reset behavior is established by the real controller regression above,
not by claiming a native button-click pass.

SwiftPM retains its pre-existing unhandled-file warning for
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`. Standalone
SourceKit file diagnostics cannot resolve the complete multi-file module;
the package tests and complete native compiler invocation succeeded.

The initial source-validation pass did not rebuild portable packages.
Hardware/audio equivalence to Traktor was not retested.

## Portable build

The subsequent requested build completed with exit code 0:

```sh
CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh
```

Outputs, all release `0.2.1-beta.1`, numeric version `0.2.1`, build `7`:

- `dist/S4 MK3 Bridge.app`
- `dist/S4 MK3 Bridge-arm64.zip`
- `dist/S4 MK3 Bridge-0.2.1-beta.1-arm64.dmg`

Deep strict code-signature verification, ZIP integrity, and DMG checksum
verification passed. The packaged Jog/tension JSON matches the source defaults
byte-for-byte. The bundled bridge regenerated a byte-identical mapping without
opening hardware.

The app and its extracted ZIP/read-only-mounted DMG copies all passed the real
preference loader's fresh initialization/save checks with isolated temporary
support directories. They contain the expected tuning, omit the retired
preference, omit deck-channel absolute Scratch CC8 bindings, and retain relative
Scratch CC5 and Internal-mode feedback. Both archive copies also passed deep
strict signature verification; their native and bridge binaries match the app
output byte-for-byte.

SHA-256:

| Artifact | Hash |
| --- | --- |
| ZIP | `00953d0658d6ef88a3452a7ac6abbd6fb1b08484e006294ca8008e005bd3f31a` |
| DMG | `a959fb2fb5aa2007c6844c28dba3eab88449bd2a952ede130f1c454e54bfc3d4` |

Saved user preferences remain unchanged. No app was installed, hardware started,
repository commit created, or release published. Signing remains ad-hoc.
