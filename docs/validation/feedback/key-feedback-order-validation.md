# Musical key feedback ordering

October 24, 2026. Local source build; this fix is not in a published release.

## Captured failure

The user reproduced a `?` on the S4's Deck 2 screen while Djay showed
`11A` for **William Kiss - Jacker**. The actual bridge's existing debug
logger recorded:

| UTC time | Input endpoint/direction | MIDI address and value |
| --- | --- | --- |
| 14:52:01.190841 | S4 MK3 MIDI Full, Djay to bridge | Channel 2, CC 112, value 102: new key 11A |
| 14:52:01.190882 | S4 MK3 MIDI Full, Djay to bridge | Channel 7, Note 1, value 0: loading false |
| 14:52:01.224031 | S4 MK3 MIDI Full, Djay to bridge | Channel 7, Note 1, value 127: loaded |

No second key message followed. The new key was already valid, but
`FeedbackState::apply` called `info.clear_deck(1)` on loading false and
erased it. The screen then rendered its unknown marker.

The startup baseline had succeeded: loaded arrived before raw 71 (`2A`),
and the user confirmed `2A` on the physical S4. The failure was therefore
an ordering problem, not missing key analysis or an unrecognized key byte.

## Genuine eject

With Deck 2 paused, the user selected **Deck 2 > Eject** in Djay.
Production feedback was:

| UTC time | MIDI address and value |
| --- | --- |
| 14:59:49.893304 | Channel 2, CC 112, value 0: no key |
| 14:59:49.893341 | Channel 7, Note 1, value 0: loading false |

The old code erased the explicit no-key value too, and the user reported
`?` after eject. Raw 0 is the calibrated `--` marker.

## Fix and regression boundary

`turntableN.displayKeyIndex` is deck-owned feedback. The key cache now
changes only when that key output changes. A song's `loadingSuccess`
notification no longer clears it. The genuine-eject value 0 replaces the
previous key; there is no previous-track fallback, guessed label, timeout,
or delayed invalidation.

Other load-false clearing remains unchanged: active loop information, cue
feedback, meters, and playing state still clear.

The real Decoder -> FeedbackState -> screen-page regressions cover:

- Previous 2A, then new 11A before loading false/true, without a key resend.
- Genuine no-key feedback before or after loading false.
- Other-deck isolation and the existing calibrated/unknown key behavior.

The captured replacement regression failed before the fix with `None`
instead of `Some("11A")`. It passes after the fix.

## Software and package checks

All 322 Rust tests and 55 Swift tests passed. Required formatting, strict
Clippy, all-target build, shell syntax, plist and whitespace checks passed.
The three changed Rust files have no LSP errors.

The standalone app, ZIP and DMG were rebuilt separately under
`dist/key-feedback-20261024/`; existing applications and packages were not
overwritten. Version remains `0.1.1-beta.3`, numeric `0.1.1`, build `4`.
The package uses the normal app source, not the temporary logging hooks.
Deep/strict code-signature verification, ZIP integrity and DMG verification
passed; the bundled CLI reports `0.1.1-beta.3`.

SHA-256:

- ZIP: `4881377dbd7c87dbf2dcf0f68058a543a3c779e54b7b1ffc0b8141b6738756a9`
- DMG: `b1a785aeea165daec7aa043eb66a0ef1253d3a222da22a5fe5b4755879964396`

Existing build warnings: SwiftPM's two unhandled files and the macOS
`hdiutil create` deprecation.

## Evidence and diagnostic boundaries

Evidence is retained under `.omo/evidence/key-live-20261024/`:

- `diagnostic-app.log`: full actual-destination feedback and lifecycle log.
- `native-key-feedback.log`: initial independent output-only baseline.
- `native-failure-ax.json`: retained native Djay track/key observation.
- `captured-key-red.log`: failing captured-sequence regression.
- `key-fix-checks.log`, `key-fix-swift-tests.log`: required software checks.
- `key-fix-package-build.log`, `key-fix-package-verification.log`: packaging.

The evidence JSON passed syntax parsing. Its optional JSON LSP was unavailable
because Biome is not installed; no dependency was added for this artifact.

The temporary logger was an isolated app copy with its own identifier. It
enabled the existing Rust feedback logs and persisted the app's bounded
output; it did not change the running bridge logic. The user performed the
load/play/eject actions. No mapping replacement was needed.

At 15:05:52 the old diagnostic bridge exited with
`wheel 0 sensor time reversed`. Its log reports an accepted final-neutral
write, not independent physical force verification. NI's agent returned
(PID 69308), with `restorePending=false`. This is a separate sensor-safety
finding; no motor behavior was changed for this key fix.

## Updated physical acceptance

The user started the rebuilt normal app, loaded **William Kiss - Jacker**
on Deck 2, directly replaced it with **Drop Ha**, then selected
**Deck 2 > Eject**. They confirmed all three physical S4 values were correct:
`11A -> 2A -> --`.

This verifies the affected track-replacement and genuine-eject behavior on
the actual S4, separately from successful writes and software tests. The
existing Djay mapping was retained. No physical acceptance for other decks
or the independent sensor-timestamp guard is claimed.

The updated app remains running from the new build directory. The stopped
diagnostic copy and older app instance were closed; neither old bundle was
overwritten. Only the owned observer configuration and temporary diagnostic
resources are removed. Captures and this record are retained.

## Follow-up ordinary use

The user subsequently reported a successful hour-long set with the latest
build on the actual DJ laptop. See the
[set validation record](dj-laptop-set-validation.md) for the report and its
scope; it is separate from the focused key replacement/eject check above.
