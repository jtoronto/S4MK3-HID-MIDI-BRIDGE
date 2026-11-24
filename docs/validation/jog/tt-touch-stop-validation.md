# TT top-touch Scratch retention and abrupt-stop entry

November 2, 2026. This follows the idle-UI comparison build. The user reported
two related problems: fast Scratch strokes repeatedly returned to Bend while
the top remained touched, and stopping from 33 RPM waited for the bend average
before entering Scratch.

## Runtime behavior

With TT and Vinyl off, the saved lower floor still selects speed-driven Scratch.
After entry, physical top contact prevents the normal speed-recovery exit.
Forward and reverse strokes therefore continue on raw relative CC5 without
releasing Scratch Note 7 or emitting CC4 bend commands.

Lifting the hand re-enables the existing speed-based exit. Releasing below exit
speed leaves Scratch active until recovery; releasing above it permits a single
release on the next fresh jog evaluation. Top touch alone at normal speed does
not enter Scratch. Explicit Vinyl-on Scratch retains its separate behavior.
Deck mismatch, playback permission, Shift/GRID, mode changes and stale-input
cancellation remain ahead of this touch hold.

A qualified abrupt stop from the normal-speed quiet band starts a separate
zero-travel confirmation using the existing validated sensor intervals.
At least 400,000 sensor ticks (4 ms) of fresh zero-count travel triggers entry,
without waiting for the 100 ms RPM estimate to fall through the saved floor.
The fast path also prevents that still-high estimate from immediately releasing
a confirmed stop. Nonzero movement and measurement resets clear the candidate;
duplicate reports cannot accumulate time.

The candidate starts only when the preceding averaged RPM is within 2.25 RPM
of target. This avoids treating low-speed encoder quantization during gradual
slowing as an abrupt stop. It does not remove the existing startup/recovery
qualification. The 100 ms RPM measurement, bend curves, MIDI addresses, mapping,
motor-control policy and saved preference schema are unchanged.

Four milliseconds is a sensor-evidence threshold, typically two fresh 2 ms
intervals, not a measured end-to-end audio delay. It is not a claim that software
can recognize a physical stop before the device reports it.

## Regression evidence

The real translator fixture covers both physical sides, floors 1/50/95, contact
before or after entry, and repeated forward/reverse strokes above the old exit
threshold. It checks raw CC5 in both directions, no bend or release during
contact, and one release followed by CC4 after above-threshold contact release.
Below-threshold release retains Scratch until speed recovery.

The new abrupt-stop regression first failed on the previous averaging-only
entry with no MIDI output where Scratch Note 7 was required. It now checks
stopped startup cannot bypass qualification, one zero interval plus a duplicate
cannot trigger entry, and the second fresh interval enters at 4 ms on both
sides at every tested floor, with or without top contact.

The touch-retention regression first failed on the previous release branch:
`side 0, floor 1: touched Scratch changed routes`.
Existing deck-switch and stale-input owner-release coverage now also runs
while top contact is held. A separate real-translator case checks gradual
deceleration to 1 RPM at a saved 0.33 RPM floor does not misclassify quantized
zero-count reports as a stop.
Removing the normal-speed candidate guard made that gradual-deceleration
regression fail by emitting Scratch instead of only CC4. The mutation was
restored before the final checks.

## Source and portable-package checks

All 508 Rust tests and 71 Swift tests passed, including the production runtime
handoff and motor-force fixtures. Formatting, strict all-target/all-feature
Clippy, all-target build, shell syntax, plist lint and diff whitespace checks
passed. Changed Rust files had no LSP errors.

The comparison app and ZIP/DMG are under `dist/tt-touch-stop-20261102/`.
Version remains `0.2.1-beta.1`, build 7. The separate output directory preserves
earlier packages. The build used `CARGO_PROFILE_RELEASE_STRIP=none`.
Deep/strict code-signature, ZIP integrity and DMG checksum checks passed.
Bundled mapping and LED/Jog/tension defaults match the accepted idle-UI build
byte-for-byte.

The actual packaged executable was replayed against the accepted idle-UI
executable using the built-in playback fixture and the same custom curve
configuration. All 12,915 timestamped motor reports match exactly.
At the fixture's abrupt 33-to-0 stop, both sides enter Scratch at host time
23,502,001 us instead of 23,598,001 us: 96 ms earlier, on the second fresh zero
interval. MIDI/routing records intentionally differ as a result. The fixture
has top contact during this stop; translator coverage separately exercises the
same entry without contact.

| Artifact | SHA-256 |
| --- | --- |
| Native executable | `c4335b5db4f3f2edacb5be8511e7dfa216ab43a6ada8a4da5335fd9758fd3e8e` |
| Rust executable | `b902de06fef6d91971039b53f7867b480024300d1ee881266c6b858e4e02e8ca` |
| ZIP | `fc695098a419ce476461b82203b9d107cfab9471d36cb7fe1c257f735d2d7517` |
| DMG | `085a9cc2083976607c1fa9422d276373f49f1a992f2f3d6fe1c1e1de2bddb55b` |

## Physical limits

No HID session, motor test, user-preference edit or Djay audio measurement was
performed for this change. The report does not establish scratching feel,
audible latency or physical confirmation of the top-touch signal. The separate
portable comparison build is for an attended user trial; no mapping reinstall
or preference reset is needed.
