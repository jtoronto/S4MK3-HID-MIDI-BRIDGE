# macOS Mute FX validation — November 24, 2026

Scope: the full macOS bridge and its generated Djay mapping. The existing
Windows MUTE-as-STEMS behavior was preserved; no Windows hardware run.

- The user reported that Mute FX worked on the S4 MK3. On an intermediate
  build that sent a cyan palette byte for on, they observed dim white off
  and bright white on. The [Mixxx S4 MK3 mapping](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js)
  independently treats MUTE as a single-color lamp.
- The final renderer uses white explicitly for both confirmed states and
  leaves both lamps dark if the state is unknown. Automated tests drive the
  MIDI decoder and assert both HID report bytes, including rejection of the
  command echo and unrelated deck feedback.
- The final arm64 app signature, ZIP integrity, and DMG checksum verified.
  Its bundled Djay mapping was byte-identical to the preceding color build,
  so that mapping does not need reinstalling.

The user accepted the white behavior. No separate physical capture of the
final white-palette build or systematic check of startup feedback and audible
echo on all four decks was recorded.
