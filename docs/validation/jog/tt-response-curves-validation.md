# Adjustable TT response curves

Date: November 2, 2026.

The existing motor/diagnostic work was first committed as `fd9853b`
(`Prevent false TT motor assistance during steady playback`). The subsequent
curve implementation was uncommitted at the time of this initial validation.
This is an unpublished comparison build,
not a release or publication.

## Behavior and compatibility

Jog settings now provide separate braking and nudge Strength/Shape controls
with live MIDI-rate graphs. Strength is 0.1-20 for braking and 0.1-10 for
nudging; each Shape exponent is 0.25-4, default 1.

Optional JSON keys `tt_brake_gain` and `tt_nudge_gain` override the combined
directional strength. Missing/null values inherit `tt_pitch_bend_gain`, doubled
for braking only. Thus a saved shared gain 5 remains effective braking 10 /
nudge 5, rather than being reset or doubled again. The two new
`tt_brake_curve_exponent` / `tt_nudge_curve_exponent` defaults are 1.
Older apps that reject unknown fields cannot read newly saved curve settings.

Outside the unchanged 2.25 RPM quiet band, let `e = measured_rpm - target`,
`a = clamp(abs(e) - 2.25, 0, 1)`, and `q = min(abs(e) / target, 1)`.
The selected strength `G` and exponent `P` produce relative MIDI count rate
`e * a*a*(3 - 2*a) * G * q^(P - 1) * 48`.
Shape 1 preserves the previous soft onset. Higher shapes soften small
deviations relative to deep movement; lower shapes strengthen them.
Above 100% deviation the shape factor stays 1 for linear continuation.

The graph shows that candidate bend rate before quantization and Scratch
routing, not measured Djay audio tempo. Its vertical
scale is independent for each curve. Values edit the graph immediately but
apply to the bridge through the existing Save/Start or Save and restart flow.
Mapping generation, motor regulation/assistance, raw Scratch, normal jog,
authorization, ownership, measurement timing and handoff thresholds are unchanged.

## Source checks and regression evidence

All required checks passed: formatting, strict all-target/all-feature Clippy,
503 Rust tests, all-target build, 70 Swift tests, shell syntax, plist lint and
diff whitespace. Swift tests passed again after the final numeric-field fix.
Rust LSP checks were clean. Swift SourceKit freshness timed out, and the
optional Biome LSP was unavailable; actual Swift compilation/tests and Rust
configuration parsing supplied the relevant checks.

New Rust coverage exercises real translation on both sides with independently
configured gains and exponents. A target of 25 RPM permits exact sensor-window
travel: a settled second at 20 RPM produces about -144 counts with braking
strength 3 / Shape 2; 30 RPM produces about +805 with nudge 1.5 / Shape 0.5.
At 75 RPM the response continues linearly rather than extrapolating the power.
Schema tests cover legacy inheritance, null overrides, serialization, valid
endpoints, invalid types/ranges and nonfinite values.

Existing normal-jog/paused-Scratch and mapping regressions now include custom
curve preferences. Directed mutations were restored after proving failures:

- Removing shape response changed the expected -144 counts to -720.
- Expanding the braking limit from 20 to 25 admitted an invalid preference.
- Leaking nudge strength into mapping sensitivity failed the native-mapping
  invariance test.

Swift coverage checks independent persistence/migration and the actual graph
calculation, including the quiet band, smooth onset, low target and linear
continuation. Its rate-factor mutation also failed before restoration.

## Exact executable comparison

Both development and actual packaged release executables were compared with
the preserved `dist/tt-handoff-20261102` bridge. For legacy configurations,
only the two newly emitted default Shape fields were removed from the
configuration record before comparison. Every other replay record matched:
MIDI, motor reports, timestamps, routing, local state and physical RPM.

| Floor | Legacy gain | Identical records |
| ---: | ---: | ---: |
| 1% | 1 | 17,486 |
| 1% | 5 | 17,996 |
| 50% | 5 | 17,898 |

A custom braking 10 / Shape 1 and nudge 1 / Shape 2 profile retained the
baseline's -13,632 braking counts while reducing positive counts from 24,644
to 4,024 in the synthetic playback fixture. All 12,915 motor reports,
non-bend MIDI, physical RPM, routing and local state remained identical.
These totals describe the fixture, not an audible slowdown percentage.

An isolated harness called the actual stopped app controller's Save action
and constructed a second controller from those saved files. Preferences
matched exactly, with no bridge child process. Feeding the resulting full
`jog-config.json` to the packaged Rust bridge produced the exact custom-profile
replay stream above. Actual user preferences were not read or changed by this
harness; its support directory is inside the local evidence folder.

## Native visual evidence and limits

The real SwiftUI component was rendered with `NSHostingView` at 640 and 1000
points, both light/dark, for legacy, custom, invalid-input and 1 RPM target
states: 16 captures, with 1280/2000 by 1000 pixel PNG output.
These are component bitmap renders, not desktop interaction captures or a
complete scrolled Settings-window test.

The initial dual read-only review found clipped huge numeric entries.
Scientific notation and wider fields resolved that defect; ordinary numeric
replacement parsing was also checked. A fresh reviewer passed all 16 updated
captures. Review round 1 took 102 seconds in parallel; round 2 took 105 seconds.
The current evidence is `review2-*.png`; earlier captures are retained only
as investigation evidence.

Mouse, keyboard and VoiceOver interaction were not tested: this session's
macOS Accessibility permission was denied. No desktop input was sent.
Source bindings, native component rendering, app-controller persistence and
packaged CLI behavior were checked separately, without claiming that they
prove real pointer/focus behavior. New curve feel and audio response still
require an attended controller comparison.

## Verified portable package

The app is `dist/tt-curves-20261102/S4 MK3 Bridge.app`, with ZIP and versioned
DMG alongside it. Version remains `0.2.1-beta.1`, numeric `0.2.1`, build `7`.
The separate bundle path identifies the comparison. The build used
`CARGO_PROFILE_RELEASE_STRIP=none` and the separate output directory.

App signatures, ZIP integrity and DMG checksum passed. ZIP-extracted and
read-only mounted DMG apps matched the local app byte-for-byte and independently
verified signatures. The disk image was detached and the owned QA directory
removed. The mapping, LED defaults and tension defaults match the preceding
package; bundled Jog defaults match the updated example configuration.
The local macOS tools emitted deprecation notices for image attach/detach,
but those operations and cleanup succeeded.

| Artifact | SHA-256 |
| --- | --- |
| Rust bridge | `1a7384b7b93ad59a4d2779f87810620d7f55322becb0e02f3b65cf33623b1169` |
| Native app executable | `9763b320da269394e7365dd5a46cf687bd9f9c86fd3411a5b8ff1c98f2305b3f` |
| ZIP | `2cb1cef2fda918196fe822604808c252ed9e53c198b2003b71a8e8015c03e238` |
| DMG | `e826f727786a70c598e416a40f54fdd43fb9682dfc633d1a994362d57150cb5c` |

During this initial validation, no hardware trial, actual GUI launch,
curve-feature commit, push or release was performed.
The previous bundles and unrelated Serato research remain
preserved. Local scripts, logs and captures are under
`.omo/evidence/tt-response-curves-20261102/`.
