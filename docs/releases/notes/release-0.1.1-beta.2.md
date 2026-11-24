# 0.1.1-beta.2 release notes

Release dated October 20, 2026. Cargo release `0.1.1-beta.2`; macOS numeric
version `0.1.1`, build `3`. GitHub release tag: `v0.1.1-beta.2`.

## Changes since 0.1.1-beta.1

- Saved Jog Wheel Tension in Jog > Hardware: Assist / Neutral / Brake for both
  wheels, with independent motion/touch response. Neutral (0) is the
  first-install default. Save and restart applies it without a mapping export.
- Mixer Filter is a one-shot Neural Mix EQ deck selector. Press Filter, then
  a channel's ON to toggle that deck's EQ mode without changing the Instant FX
  target. Press Filter again to cancel.
- ON indicates actual EQ mode in white/cyan and the Instant FX target by
  brightness; an active effect's color takes priority. Idle Filter is dim cyan.
- Aligned tension labels, a green running menu-bar icon, and app release/build
  information in Diagnostics.
- Refreshed README Jog screenshot and current feature/issue documentation.

## Updating

Install and select the mapping exported by the new app under a new filename.
Keep existing custom mappings. The updated mapping is required for Neural Mix
EQ actions and mode feedback; hardware tension alone needs no mapping change.
Saved LED, jog, run, and tension preferences remain in Application Support.

## Evidence and remaining checks

The feature build passed 248 Rust and 18 Swift tests, strict Clippy, builds,
and app/ZIP/DMG integrity checks before this version-only increment.
After the version bump, all 248 Rust and 18 Swift tests passed again, along
with formatting, strict all-target/all-feature Clippy, the locked all-target
build, shell/plist checks, documentation links, and version consistency.
The debug CLI reports `s4mk3-hid-midi-bridge 0.1.1-beta.2`. Existing SwiftPM
unhandled-file warnings for the icon and diagnostic build script remain.

The portable packages were rebuilt and verified on October 20, 2026 using
`CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh`:

- `dist/S4 MK3 Bridge.app`
- `dist/S4 MK3 Bridge-arm64.zip`
- `dist/S4 MK3 Bridge-0.1.1-beta.2-arm64.dmg`

The build checked both executables for non-system absolute library dependencies.
Strict deep signature verification passed for the app and the copies extracted
from the ZIP and mounted read-only DMG. ZIP and DMG integrity checks passed.
Each copy reports release `0.1.1-beta.2`, numeric version `0.1.1`, build `3`;
its bundled bridge reports `s4mk3-hid-midi-bridge 0.1.1-beta.2`.

The production Swift controller loaded each copy's bundled LED, jog, and tension
defaults with fresh temporary preferences, saved, and reloaded them unchanged.
Tension initialized at Neutral (0). All three defaults and the mapping matched
across archives; mapping bytes also matched the previous feature build.
These checks opened no HID or MIDI ports, sent no motor commands, and did not
change saved user preferences. Temporary extractions and the verification
mount were removed after the checks.

The prior app, ZIP, and beta.1 DMG are preserved in
`dist/previous-beta2-rebuild-20261020.xlFLYJ/`.

SHA-256:

```text
090630a09959301795e8abad471b8774dbed54fa11a524f0e38d5097423b7838  S4 MK3 Bridge-arm64.zip
d031a0975e8bb0ca23670add195156e2e268041fe5455a5129ca4543d1154ab1  S4 MK3 Bridge-0.1.1-beta.2-arm64.dmg
```

Focused physical tension checks passed, including both-wheel independence,
Djay/MIDI/LED coexistence, Stop/Quit, persistence, and explicit restart after
USB reconnect. Live EQ feedback was captured independently for all four decks;
the user accepted the physical workflow and dim-cyan idle refinement.
See [tension integration](jog-tension-integration.md) and
[EQ validation](neural-mix-eq-check.md).

The new Jog image is a capture of the production native view at defaults,
with an isolated controller and no hardware bridge started. It does not
constitute a full UI regression sweep.

Broader actual-laptop testing and exhaustive EQ edge cases remain open.
The initial laptop launch error for `default-tension-config.json` stopped
occurring without changes; its cause is not confirmed. Backspin continuation,
playback-driven platter rotation, screens, and jog-ring touch/position polish
are not part of this release. See [current issues](issues.md).
