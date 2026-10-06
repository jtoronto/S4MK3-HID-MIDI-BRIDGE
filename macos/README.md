# Standalone macOS app

The app bundle packages the native status-bar app, the release bridge, its
default LED configuration, and a Djay MIDI mapping. It targets Apple Silicon
(arm64) and macOS 12.0 or later. Build it on an Apple Silicon Mac with Xcode
Command Line Tools, the Rust toolchain, and the locked Cargo dependencies
available:

```sh
bash macos/build-app.sh
```

The results are `dist/S4 MK3 Bridge.app` and the transfer archive
`dist/S4 MK3 Bridge-arm64.zip`. The script compiles
`macos/S4BridgeApp.swift` against the installed Apple macOS SDK, builds
`s4-connectivity-probe` with `cargo build --locked --release`, copies
`examples/led-config.json` into the app resources, and creates
`S4 MK3 Bridge.djayMidiMapping` by running that same release bridge with
`--generate-mapping`. The app bundle's `0.1.0` version follows the Cargo
package version; the script checks that these version values remain aligned.
If the app output already exists, move it aside before building again.

The bridge and then the app bundle receive ad-hoc signatures for local transfer
and launch. This is not Developer ID signing or notarization. Gatekeeper may
require the recipient to approve opening the transferred app; distribution to
unfamiliar Macs requires a separately provisioned Developer ID signing and
notarization process.

No package manager or project-specific runtime dependency needs to be installed
on the target laptop. The bundled bridge's linked libraries are checked during
the build; the script stops if it finds an absolute dependency outside
`/System/Library` or `/usr/lib`, instead of silently shipping a machine-local
library reference.

## Use on the DJ laptop

Transfer `dist/S4 MK3 Bridge-arm64.zip`, unzip it, and move the app to the
laptop's Applications folder, then open it.
The app lives in the menu bar and does not require Rust, Cargo, or this repository.
Its first launch opens Settings; closing that window leaves the menu-bar app
running.

1. Connect the S4 and its power adapter. Use **Check controller** if needed.
2. Choose the full-controller profile and click **Start bridge**.
3. Use **Install Djay mapping…**, save the file, and accept Djay's import.
4. Enable **S4 MK3 MIDI Full** in Djay and select **S4 MK3 Bridge** as its mapping.
   Djay may append a number to the imported configuration name on repeated imports.
   If only a blank S4 MK3 configuration appears, restart Djay with the bridge
   already running. Do not run a separate CLI bridge at the same time.
5. Keep audio routed directly through the S4 in Djay.

LED and run preferences are stored under
`~/Library/Application Support/S4 MK3 Bridge/`. LEDs and Palette cover every
current JSON setting; import/export accepts a full or partial LED JSON object.
**Save and restart** applies edits to an active bridge. Invalid settings remain
visible and do not silently replace saved preferences.

**Stop bridge** and **Quit S4 MK3 Bridge** send the bridge its normal Ctrl-C
signal and wait for it to exit. Quitting the settings window does not stop it.
Diagnostics shows the actual launch arguments, exit status, and a bounded log.
Raw/debug logging is intended for troubleshooting rather than normal DJ use.
Continuous mode permits an idle controller and stops cleanly without requiring
input activity. A fixed-duration run retains the CLI probe's activity checks.

See [configurable-controls.md](../docs/configurable-controls.md) for the
distinction between existing preferences, useful future input-sensitivity
settings, and hardware/protocol constants that should remain fixed.
The verified build/UI evidence and remaining limits are recorded in
[macos-app-qa.md](../docs/macos-app-qa.md).

## Developer checks

```sh
swift test --package-path macos
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
bash -n macos/build-app.sh
```
