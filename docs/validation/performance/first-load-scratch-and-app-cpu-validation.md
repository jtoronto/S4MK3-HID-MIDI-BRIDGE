# First-load TT Scratch and native app CPU follow-up

November 2, 2026. The DJ-laptop report identified two problems after the
Shift-seek comparison: a newly loaded track could not Scratch until Play/Pause,
and the native app used excessive CPU while receiving control and screen logs.

## First-load failure and correction

The supplied trace has `loaded=Some(true)`, `internal=Some(true)` and
`playing=None` after track load. The previous manual TT path required confirmed
pause, so it did not permit the physical top-touch Scratch gesture. Play/Pause
provided `Some(false)`, making that existing path work.

The bridge now stores a separate manual top-touch permission for loaded,
confirmed Internal, non-THRU decks without known playing feedback. Confirmed
pause and motor authorization keep their original feedback requirements.
The fallback does not write a playing/paused value or create a synthetic
untouched paused-Scratch owner. With Vinyl off, touch and turn the top to
Scratch before the first Play/Pause. Telemetry shows `manual_scratch` when
ready and `scratch` while held; known pause retains `paused_scratch`.

The production output-service regression uses actual raw HID snapshots and MIDI
feedback decoding, not a copied routing function. It first failed with zero
Scratch entries where one entry was required. After the correction it checks
both physical sides, raw CC5 movement, exactly one Note 7 press/release per side,
and manual readiness after release. Every emitted motor report has zero force
for both wheels while playback remains unknown.

Unloaded, unknown/non-Internal and THRU cases do not receive this additional
permission. Existing confirmed-pause, cancellation and stale-input tests still
pass, including release of synthetic paused Scratch when playing becomes
unknown.

## Routine diagnostic traffic

The trace includes successful `SCREEN_FRAME_WRITTEN` messages roughly twice per
second and high-rate `SLIDER`/`ENCODER` events. These routine messages now require
Debug detail. Normal retains button/mode transitions and failures.
No input translation, MIDI output, frame scheduling or USB write is removed.
This log-level change alone is not proof that native app CPU is fixed.

## Native CPU cause, correction and comparison

When Diagnostics was mounted, each flushed output batch replaced the full
bounded SwiftUI `Text`. The whole log, up to 60,000 characters, was laid out
again. The native log panel now receives append/prefix-removal updates and edits
an `NSTextView`'s text storage incrementally. The bounded full string remains
available for Copy log and remounting the view.

The native worker compared the old and new production views with the same
synthetic 250-batch stream at 760/1000 points in light/dark appearance:

| Four-case comparison | CPU seconds |
| --- | ---: |
| Full-log SwiftUI replacement | 33.42 |
| Incremental native text storage | 3.82 |

This is 88.6% less CPU for that workload, about 8.7 times less. It is not a
measured percentage reduction on the DJ laptop, and the synthetic workload does
not reproduce the physical event cadence. The old unselected-Diagnostics control
used 1.14 CPU seconds for the workload; this supports a mounted-view hot path,
not a complete explanation of high CPU with Settings closed or another tab open.
Settings visibility during the DJ-laptop CPU spike remains unconfirmed.

The append/trim regression failed when prefix removal was disabled, then passed
after restoration. Parent coverage additionally applies emitted deltas to native
UTF-16 text storage with Unicode content and an incoming batch larger than the
whole retained log. Mutating UTF-16 removal to character-count removal made that
case fail; restoration passes. The native decoder accepts the machine-consumed
`manual_scratch` mode.

The parent rendered the final Diagnostics view after an incremental log append
at both widths and appearances. All four changed log panels were readable and
nonblank, without text overlap. Captures and the hardware-free source are under
`.omo/evidence/first-load-app-cpu-20261102/`. The standalone offscreen harness has
no app bundle version and does not reliably composite the native tab bar.
Those capture limits do not establish tab navigation, pointer/keyboard scrolling
or live hardware presentation.
Two independent read-only checks also passed all four changed log panels.

## Rust verification

All 510 Rust tests passed, including the production first-load regression and
the unchanged synthetic paused-Scratch cancellation tests. Formatting, strict
all-target/all-feature Clippy and the all-target build passed. The first
formatting check identified a long helper expression; it was wrapped before
the complete successful run. The test's right-wheel force assertion was also
corrected to bytes 9-10 after checking the actual wire layout; byte 8 is a
direction/mode field, not force.

## Native and portable-package verification

All 72 Swift tests passed after the UTF-16 mutation was restored. Shell syntax,
plist lint and diff whitespace checks passed. Production Swift diagnostics were
clean. SwiftPM retained its existing unhandled-file warnings for the app icon
and `build-jog-diagnostic.sh`; no warnings were suppressed.

The separate app, ZIP and DMG are in `dist/first-load-cpu-20261102/`.
Version remains `0.2.1-beta.1`, build 7; earlier packages are preserved.
Deep/strict signatures, ZIP integrity and DMG verification passed. Bundled
mapping and LED/Jog/tension defaults match the preceding Shift-seek build
byte-for-byte. No mapping reinstall or preference reset is needed.

| Artifact | SHA-256 |
| --- | --- |
| Native executable | `ad42739e02d280fdddd60288c1627d7ddb4f6bbedbca29f6b9334a6b31065a4e` |
| Rust executable | `1f6c3ff681563dbd88ccad6e70ffdbfab7015a862029c275e0f3b9dfe77dd30c` |
| ZIP | `a8abf096ef9c2bad78f80423f7c964c83ac5c3e9049742aaf572339ced77c82f` |
| DMG | `94a25a1593b9e402f60ad003a6fd09afbb0d34ab0881db4789ffba07670f6c51` |

Both actual packaged Rust executables were replayed with the same existing
playback fixture and custom curve configuration. All 17,068 structured runtime
records match, including 12,915 timestamped motor reports. The new first-load
case is covered separately at the production runtime boundary.

## Physical limits

No controller session, screen traffic, motor test or saved preference edit was
started for this work. Production replay establishes software routing and
zero-force output, not the DJ laptop's audible Scratch response or physical
motor behavior. Local performance comparisons must be distinguished from a
measured CPU improvement on the user's DJ laptop.
