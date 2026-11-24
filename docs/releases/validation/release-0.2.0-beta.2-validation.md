# Release preparation: 0.2.0-beta.2

October 26, 2026. Internal validation record. End-user release notes are in
[release-0.2.0-beta.2.md](release-0.2.0-beta.2.md).

## Scope and baseline

GitHub's authenticated release API confirms `v0.2.0-beta.1` was published
October 24, 2026. It is the baseline, superseding the earlier local record of
that release as a draft. Beta.2 adds backspin continuation, independent software
speed increase and optional motor assistance, relative Shift + FX knob effect
browsing, corrected fixed-color lamp intensity and stable acquired FX warnings.
Version metadata is `0.2.0-beta.2`, numeric `0.2.0`, build `6`.

The user accepted the backspin/motor checks, subsequently reported all checks
good on the actual DJ laptop, and accepted the FX browsing build with
"Works like a charm". Those are normal-use acceptance reports, not proof that
every fault path or target/modifier combination was exercised. Before release,
the user additionally selected a default 5.6% scratch speed increase and motor
assistance enabled at 100%. Explicitly saved values remain unchanged; fresh
settings and missing legacy fields receive the new defaults.

The user explicitly authorized committing, pushing and publishing beta.2 after
the requested default changes. Earlier packages remain preserved. The initial
pre-default package evidence below is superseded by the final build section.

## Initial source verification, before the default change

The final beta.2 source passed formatting, strict locked all-target/all-feature
Clippy, the locked all-target build, all 55 Swift tests, build-script syntax,
Info.plist lint and whitespace checks. Forty-six local documentation file links
in the release notes and updated guides resolve. Hosted links and anchors were
not checked by that file-link validator.

`cargo test --locked --no-fail-fast -- --test-threads=1` completed with 356
passing Rust tests and two existing diagnostic CLI failures:

- `explicit_free_play_startup_reaches_log_creation_before_hid`
- `wheel_selection_is_admitted_only_by_the_intended_profile`

The active FxKnobBrowse bridge was process 63379 during this run. The diagnostic
ownership check occurs before deliberate invalid-path capture creation, so the
tests cannot observe their expected log-creation rejection while the controller
bridge is running. The same mechanism was confirmed during the preceding FX
implementation run. No running app, test or ownership guard was modified.
The five new browsing regressions pass. This is an explained environment
limitation, not a clean full-suite result. Existing SwiftPM warnings for
`macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh` remain.

## Initial packages, superseded

Built with `CARGO_PROFILE_RELEASE_STRIP=none` and
`S4_BRIDGE_OUTPUT_DIR="$PWD/dist/Release-0.2.0-beta.2-20261026"`:

- `dist/Release-0.2.0-beta.2-20261026/S4 MK3 Bridge.app`
- `dist/Release-0.2.0-beta.2-20261026/S4 MK3 Bridge-arm64.zip`
- `dist/Release-0.2.0-beta.2-20261026/S4 MK3 Bridge-0.2.0-beta.2-arm64.dmg`

Build, deep strict signatures, ZIP integrity and DMG verification exited 0.
The app, ZIP-extracted app and read-only-mounted DMG app each passed release,
numeric and build-version checks, bundled CLI `--version`, mapping plist lint,
and byte comparisons for all three default configurations, font license and
generated mapping. The mapping also matches the accepted FxKnobBrowse package
byte-for-byte. The build script checked linked-library portability.

The temporary extraction and read-only mount were removed afterward. No app
was launched, no preferences were changed, and no HID/MIDI/motor/screen activity
was triggered by packaging verification.

```text
021775c8a472e102ffed7c2cd03b3197ea196633549d2d616c6dc49550fc02b0  S4 MK3 Bridge-arm64.zip
eaf3a989b9ef5dfef8a798da47b4ca03d089a4d24470fdff536edd2d18378011  S4 MK3 Bridge-0.2.0-beta.2-arm64.dmg
```

## Final defaults and source verification

Rust defaults, Swift model defaults and the bundled jog configuration now agree:
backspin detection enabled, speed increase 5.6%, motor assistance enabled and
assist strength 100%. Existing Rust/Swift regressions cover fresh initialization,
missing-field merging, explicit saved zero/off/non-default strength, and saved
round trips. The updated default-value regression failed against the original
configuration before production changes.

The final source passed all 358 Rust tests serially, all 55 Swift tests,
formatting, strict locked all-target/all-feature Clippy, the locked all-target
build, shell syntax, plist lint and whitespace checks. Unlike the earlier run,
the final run had no diagnostic ownership failures. No test or guard was
removed, skipped or weakened, and the agent did not stop the user's app.
Two old backspin byte expectations were updated for the requested gain:
100 raw counts now emit 307 accumulated event counts, and the ownership test's
10-count step emits 30 on the original scratch deck while the other wheel's
nudge remains unchanged. Existing additive-gain and normal-motion tests pass.
The existing SwiftPM two-file warnings remain. JSON language-server diagnostics
were unavailable because Biome is not installed; real Rust/Swift decoding tests
and bundled-resource equality checks validate the JSON instead.

## Final release packages

Built into `dist/Release-0.2.0-beta.2-final-20261026/`:

- `S4 MK3 Bridge.app`
- `S4 MK3 Bridge-arm64.zip`
- `S4 MK3 Bridge-0.2.0-beta.2-arm64.dmg`

Build, strict deep signatures, ZIP integrity and DMG verification passed.
The app, ZIP-extracted app and read-only-mounted DMG app each passed versions,
bundled CLI version, mapping lint, all three source-default comparisons,
font-license equality and mapping equality. Their jog defaults include the
final 5.6%/enabled/100% values. The mapping remains byte-identical to the accepted
FX browsing build because backspin options affect runtime, not mapping addresses.
Temporary extracted/mounted copies were cleaned up. No app was launched and
no preferences or hardware were changed.

```text
e3156371737ee2afc374b1a56c09c5c560fb1ed8551681feee3296975f036d30  S4 MK3 Bridge-arm64.zip
9231ba3b341fb5f6e320e156e516321dd459ce7474a4665fa3d46549a26bdaaa  S4 MK3 Bridge-0.2.0-beta.2-arm64.dmg
```
