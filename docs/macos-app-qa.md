# macOS app verification

Verified on the Apple Silicon build Mac, 2026-10-06. The app declares macOS 12+
and has not yet been exercised on the separate DJ laptop.

| Check | Result | Evidence |
| --- | --- | --- |
| Rust behavior and continuous-mode validation | PASS | 64 tests; strict all-target/all-feature Clippy; formatting |
| Native preferences and process management | PASS | 7 Swift tests, including real pipe drain and SIGINT acknowledgement; no fixed sleeps |
| Bundle integrity | PASS | arm64 executables, minimum OS 12.0, deep/strict code-sign verification |
| Runtime dependencies | PASS | bridge links only system frameworks/libraries; clean-PATH enumeration found S4 without opening interfaces |
| Transfer archive | PASS | `unzip -tq` verified `dist/S4 MK3 Bridge-arm64.zip` |
| Native appearance | PASS | inspected Bridge, LEDs, Palette, and Diagnostics; paired light/dark views and bottom scroll positions |
| Minimum window size | PASS | corrected `contentMinSize`; palette footer no longer clips with native toolbar chrome included |
| Status-menu enablement | PASS | Start and Check disabled while running; Stop enabled; manual menu enablement overrides macOS auto-validation |
| Start and idle Stop | PASS | actual bundled bridge reached Running, then Stopped; no owned bridge remained |
| Quit with bridge running | PASS | app and owned bridge exited; no orphan |
| Close Settings | PASS | menu-bar app and bridge remained running |
| Mapping installation | PASS | native installer completed; Djay imported numbered mapping; SHA-256 equals bundled mapping |
| LED JSON export/import/save | PASS | actual native actions completed; exported and saved JSON SHA-256 identical after round trip |
| Invalid preference handling | PASS | native tests reject unknown fields, malformed arrays/ranges, and preserve invalid saved files until explicit reset |

Native screenshots are local evidence under `.omo/evidence/macos-app/`.
The original dark system appearance was restored. Earlier failing minimum-size
captures are diagnostic history, not the final layout.

The live Stop test originally exposed probe-only activity validation in
continuous mode. A failing regression reproduced it; the fix allows idle
continuous service use while keeping bounded probe activity checks unchanged.
The first menu pass also exposed AppKit's automatic item validation; manual
enablement now matches real process state.

The app is ad-hoc signed for local transfer. Developer ID signing and
notarization are not included. Full physical cue-palette calibration and
master-clip verification remain the bridge's previously documented limits;
the app does not claim new motor/screen/audio or end-warning support.
