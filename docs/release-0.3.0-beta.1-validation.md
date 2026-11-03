# 0.3.0-beta.1 draft preparation

## Scope

The user reported that testing was good and requested a commit, push and
GitHub draft release with end-user notes and screenshots. They selected
`0.3.0-beta.1` for the new motorized-playback milestone.

Implementation commit `bc40404` and the preceding local commits were pushed
to `main`. Unrelated `.gitignore` edits, `.senpi/` output and the untracked
Serato research document were excluded.

Cargo and the app now identify the release as `0.3.0-beta.1`, numeric version
`0.3.0`, build `8`. The earlier verified app/ZIP/DMG were preserved together
under `dist/previous-jog-build-20261103/` before rebuilding at the usual paths.

## Notes and screenshots

`docs/release-0.3.0-beta.1.md` is the end-user release body. It covers changes
since the published `v0.2.1-beta.1`: motorized playback and Vinyl operation,
Jog tuning/defaults, EXT switching and DVS setup, multi-deck Instant FX,
Diagnostics and native-app CPU improvements, and update instructions.

The Jog image at `docs/images/jog-settings-0.3.0-beta.1.png` is a genuine
native source-built UI capture, promoted from the isolated preview evidence
described in `docs/jog-settings-cleanup-validation.md`. No settings view code
changed between that capture and this release. The existing DVS screenshot
shows the required global enable and all four input assignments. Both images
were inspected against their captions and instructions.

The public README and guide distinguish the upcoming draft from published
download availability. Notes do not claim Traktor-equivalent feel, exact audio
speed, complete hardware coverage, or notarization.

## Verification

All required source checks passed: formatting, strict all-feature Clippy,
510 Rust tests, locked all-target build, 80 Swift tests, shell syntax, plist
lint and diff whitespace. SwiftPM retains the existing two-unhandled-files
warning; no new warning was suppressed.

The portable build completed with:

```sh
CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh
```

The app, ZIP and read-only-mounted DMG copies passed deep strict signature
verification. ZIP integrity and DMG checksum verification passed. All three
copies passed the real controller's fresh preference initialization/save and
mapping checks with isolated temporary support directories. Checks cover
release/numeric/build versions, bundled tuning, omitted retired preferences
and absolute Scratch routes, retained relative Scratch and Internal feedback.
The native and bridge executables match across archive copies. The bundled
bridge reports `0.3.0-beta.1` and regenerated a byte-identical mapping.

| Artifact | SHA-256 |
| --- | --- |
| `S4 MK3 Bridge-arm64.zip` | `b32e9695e43902612e1583e9da046e1bada3abc21d81d8b84b346c374ff757de` |
| `S4 MK3 Bridge-0.3.0-beta.1-arm64.dmg` | `afc5fa0a971cd4d9298c04694c07dad43b7d45264d78d93a382059c7553ec33d` |

The temporary mount was detached and extraction files removed. No production
app was launched, user preferences written, hardware started, or NI service
changed during packaging. No new hardware/audio comparison was performed.

## Draft gate

The draft must remain unpublished, be marked as a prerelease, target the pushed
release-preparation commit, and contain the exact release body above and both
verified archives. After upload, compare GitHub's asset digests with the hashes
above and verify that both screenshot URLs load.
