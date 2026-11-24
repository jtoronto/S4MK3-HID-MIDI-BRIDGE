# Control synchronization and navigation validation

Validated October 23-24, 2026. This is an internal evidence record, not a setup
guide. User instructions are in [controls-state-sync.md](controls-state-sync.md).

## Scope and boundaries

Implemented physical startup channel-volume/EQ application, feedback-driven FX
pickup and indications, preservation of confirmed jump size, library-table
switching and shifted source navigation. Held Preview remains the scrub gesture.
At this validation stage, intermittent musical-key diagnostics were explicitly
deferred until the user was present. The subsequent
[captured key fix](key-feedback-order-validation.md) resolves that issue.

Native acceptance used a unique virtual MIDI endpoint, **S4 Controls QA 20261023**,
and freshly imported configurations. The existing custom S4 mapping was not
edited or replaced. No live bridge, HID interface, motor output or screen worker
was started. Screen evidence is the production raster and LED-report renderer,
not a claim about physical display appearance or lamp visibility.

The initial report-2 GET and additional fresh GET on full-profile analog
notifications still require attended physical timing/interaction acceptance.
Their framing follows the already-used numbered analog snapshot contract.
Minimal-profile behavior is unchanged.

## Harness and invocation

The temporary Rust child module drove the actual translator, generated mapping,
decoder and feedback cache through native CoreMIDI. It did not replace those
components with mocks. Its source was archived as non-executable evidence at
`.omo/evidence/controls-20261023/qa-driver-source.txt`; all compiled hooks and the
temporary `--controls-qa` argument were removed before final source checks.

Invocations during QA:

```sh
target/debug/s4mk3-hid-midi-bridge --controls-qa library
target/debug/s4mk3-hid-midi-bridge --controls-qa current
target/debug/s4mk3-hid-midi-bridge --controls-qa startup-first
target/debug/s4mk3-hid-midi-bridge --controls-qa final
```

These are historical harness invocations, not shipped CLI options. Input was
JSON on the owned PTY through `functions.bash_input`; feedback subscriptions were
armed before actions. Native controls were read through
`computer.window('43012').ax({maxDepth:2})` and captured with
`screencapture -x -l 43012 <artifact>`. Import confirmations and the configuration
popup were operated through native AX. Importing a new file alone did **not**
select it: the fresh name was explicitly selected in the MIDI editor.

Artifacts below are relative to `.omo/evidence/controls-20261023/`, retained
locally and excluded from commits.

## Startup: PASS

Feedback-first input used `kind:"seed"` with distinct normalized values:

| Control | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| Volume | 0 | 0.25 | 0.5 | 1 |
| High EQ | 0 | 0.25 | 0.75 | 1 |
| Mid EQ | 1 | 0.75 | 0.25 | 0 |
| Low EQ | 0 | 0.5 | 1 | 0.5 |

The binary emitted exactly channel-5 CC 4-19, including zero values. Native AX
and `startup-distinct-physical-values.png` showed the corresponding
0/25/50/75/100 percent controls on all four decks. Seeding shared FX positions
emitted no FX MIDI and changed no software FX value.

`--controls-qa startup-first` cached all sixteen original channel values before
processing initial raw playback feedback; they were emitted after readiness and
restored the native mixer. The deterministic regression additionally checks
latest pre-ready movement, duplicate observations, both launch orders and no
replay after a subsequent software edit.

## Jump preservation: PASS

Native reproduction before the fix: right Shift+MOVE emitted `99 05 7f`,
`89 05 00`; actual feedback changed from 32 beats to one beat. See
`jump-before-fix.png`.

Repeated fixed-action pulses were also insufficient: a four-detent batch moved
only one beat, about 56 waveform pixels. The selected-one-beat plus relative-CC
alternative moved one bar, about 222 pixels. The final runtime uses that measured
alternative rather than repeated fixed-action buttons.

For confirmed eight beats on B, a four-detent forward command emitted:

```text
91 43 7f   81 43 00   # select one beat
b1 02 44               # relative +4
91 46 7f   81 46 00   # restore eight
```

The backward command uses `b1 02 3c`. Both directions moved four beats while the
native GUI and feedback returned to eight. Evidence:
`jump-final-before-wave.png`, `jump-final-forward-four-wave.png`,
`jump-final-backward-confirmed.png`. Eight-beat selection must be established
by actual feedback before the action; an immediately preceding unacknowledged
raw selection is not a valid confirmed-state precondition.

The regression
`intermediate_one_beat_feedback_cannot_replace_an_inflight_restore` failed
before in-flight ownership was added: it restored Note 67 instead of Note 70.
It now passes, along with all ten sizes, four decks, signed multi-detent
transactions, zero delta and newer confirmed selection handling.

### Native burst limitation

An extreme zero-delay sequence coalesces in Djay independently of this change.
With native one-beat selection, the **unchanged normal MOVE** path received:

```json
{"kind":"encoder","name":"right_move_encoder","delta":4}
{"kind":"encoder","name":"right_move_encoder","delta":-4}
{"kind":"encoder","name":"right_move_encoder","delta":2}
{"kind":"encoder","name":"right_move_encoder","delta":-2}
```

The final pair was repeated four times in total. The bridge emitted all ten
CC 2 messages, but the paused native waveform ended two beats behind the origin.
`native-normal-move-burst.png` records this without any Shift-size restoration.
Separate positive/negative four-detent actions work. The native limitation is
tracked in [issues.md](issues.md); it is not reported as fixed.

## FX pickup and indication: PASS

Two native findings changed the implementation:

1. Embedded absolute feedback initially returned zeros despite actual Manual FX
   values of 50 percent on A and differing values on B-D. It did not track
   changes. It was replaced with root `control` outputs for
   `turntableN.fx1/2/3.parameterValue` and `.wetDryValue`, canonical CC 32-37.
2. Djay's pickup flag alone allowed the first retargeted movement to jump C from
   raw 45 to raw 83. A bridge-owned acquisition gate was added and tested through
   the native surface rather than relying on that flag.

The corrected root outputs returned A: 64/63/63 with wet 63, B: 63/63/46 with wet 0,
and C/D: 45/0/0 with wet 127. Changing A's first parameter to 75 percent returned 95
and matched native AX. These are actual model values, not interpreted initial
zeros.

Final retarget scenario:

```json
{"kind":"seed","values":[["fx1_knob_1",0.75],["fx1_knob_2",0.6],["fx1_knob_3",0.6],["fx1_dry_wet",0.5]]}
{"kind":"button","name":"mixer_ch3_fx1","pressed":true}
{"kind":"button","name":"mixer_ch3_fx1","pressed":false}
{"kind":"slider","name":"fx1_knob_1","value":0.65}
```

C remained raw 45, with **no FX CC emitted**. Moving to 0.2 crossed pickup,
emitted `b2 20 19`, and native feedback/AX became raw 25/20 percent. See
`fx-retarget-preserved.png` and `fx-crossed-pickup.png`.

A separate software change to 110 was confirmed through feedback and AX before
the next physical movement. Movement to 0.4 was blocked and the GUI remained
87 percent: `fx-external-edit-preserved.png`. A raw software edit and a physical
movement placed in the same unacknowledged PTY batch do not establish that
precondition; the bridge cannot act on feedback it has not received.

Dry/wet stayed at 127 while the physical knob approached from below. After
reaching 1.0, all three slots followed 0.4 and returned 51/40 percent.
Actual production LED frames at 0 and 300 ms then showed stable main ON with
only the three remaining parameter mismatches blinking. Deterministic tests
also cover partial/unknown feedback, independent wet slots, one-step tolerance,
retargeting, own-feedback lag and ordinary lamp restoration.

Production pages built from that native feedback were inspected:

- `fx-native-info-0.png`: C's `FX SYNC L123` and unchanged numeric Info fields.
- `fx-native-stems-0.png`: the same warning above all eight STEMS tiles.
- `fx-native-info-1.png` and `fx-native-stems-1.png`: opposite B unchanged.

## Library and Preview: PASS

Actual Note 11 changed focused native table from Playlist to Queue:
`library-before.png` and `library-table-after.png`. The generated target is
`musicLibrary.toggleLibrarySource`, not `focusQueue`.

Shift+BROWSER forward emitted shifted Note 25 and selected Apple Music.
Backward emitted Note 26 and restored My Collection:
`library-source-next.png` and `library-source-previous.png`.

Holding Preview with Shift emitted Preview toggle on the held deck and shifted
seek Notes23/24, not source notes. The native preview player was visible and
the source remained My Collection: `held-preview-with-shift.png`. Releasing
Preview closed the player. The deterministic route regression covers both
sides, all four decks, positive/negative/zero deltas and hold release.

## Source checks and self-review: PASS

The full implementation passed:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
swift test --package-path macos
bash -n macos/build-app.sh
plutil -lint macos/Info.plist
git diff --check
```

`final-checks.log` records exit 0, 320 Rust tests and 55 Swift tests, with no ignored
Rust tests or failed tests. Swift reported existing unhandled-resource warnings
for `macos/Assets/AppIcon.png` and `macos/build-jog-diagnostic.sh`; neither file
nor the Swift package declaration was changed.

The complete check set was repeated successfully before handoff; the final
receipt is `pre-handoff-checks.log`. Documentation links and `git diff --check`
also passed after the prose updates.

Regressions caught during the run, all corrected:

- Intermediate one-beat feedback replacing the selected restore size.
- Info model equality needing the new right-deck warning metadata.
- The offline preview export contract increasing from 36 to 44 images.
- Strict Clippy: small Copy warning passed by reference, function-size limits,
  a boolean simplification and a missing statement semicolon. No suppressions
  or test skips were added.

Self-review re-read production diffs, checked mapping/runtime addresses, selector
and held-owner boundaries, acquisition/feedback ordering and native artifacts.
Rust LSP reported no errors in its capped 50-file directory scan; strict Clippy
and the all-target build covered the complete target set. HEAVY tier held because
the work changes native MIDI integration and feedback state. No ulw-plan was
requested, so the reviewer gate was not triggered; this was a self-review.

The library, jump and startup increments were separately reconstructed from the
Git index and each passed formatting, strict Clippy, all Rust tests and the
all-target build before committing. Their logs are
`library-commit-checks.log`, `jump-commit-checks.log` and
`startup-commit-checks.log`. No partial feature dependencies were committed.

## Portable package: PASS

A separate source build was produced without replacing the existing app or
release artifacts:

```sh
S4_BRIDGE_OUTPUT_DIR="$PWD/dist/controls-20261024" \
  CARGO_PROFILE_RELEASE_STRIP=none bash macos/build-app.sh
codesign --verify --deep --strict "dist/controls-20261024/S4 MK3 Bridge.app"
unzip -tq "dist/controls-20261024/S4 MK3 Bridge-arm64.zip"
hdiutil verify "dist/controls-20261024/S4 MK3 Bridge-0.1.1-beta.3-arm64.dmg"
```

All exited 0. The bundled bridge itself exported 44 complete offline rasters;
all eight known/unknown, bright/dim Info/STEMS warning variants were visually
inspected in `package-rasters/`, including longest warning text, SELECT and
active-loop highlighting. No clipping, overlap or missing tile labels was seen.

The actual bundled mapping was read and parsed. `package-mapping-proof.json`
records 24 canonical FX outputs, 40 jump-size input addresses, four library-table
bindings and eight shifted source bindings, with endpoint **S4 MK3 MIDI Full**.

SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| ZIP | `47542ebd03573db494631366c6b7a0b067fba9efd0a316fb5eed9cce53670606` |
| DMG | `2e0440c7d7fa7134a9ad02c319030a3e51f9d709be1798feec780cd81fa1ea38` |

The local build retains the existing 0.1.1-beta.3 version identifier. It is not a
new published release. Signing remains ad-hoc; notarization was not performed.
The build reported the existing `hdiutil create` deprecation warning.

## Teardown and commits

Native channel/EQ values, probed FX values, paused loaded B, jump 32 and the
Instant FX view were restored; `native-restored-after-cleanup.png` captures the
result. The Preview player closed, all MIDI PTYs ended with exit 0, and the native
MIDI menu reported **No MIDI devices found**. The three exact owned QA mappings
were archived out of Djay's mapping directory, leaving
`S4 MK3 Bridge 1.djayMidiMapping` untouched.

The owned display assertion was killed with a process-absence receipt.
The obsolete child watch was removed. Temporary compiled QA hooks are absent.
The owned verification tree and run-root cache cleanup is recorded in the durable
notepad; no user release artifacts or shared caches are eligible for deletion.

Implementation commits:

- `38d4054` Add dedicated library source navigation.
- `5e5a371` Preserve confirmed jump size during one-beat moves.
- `8d344e8` Apply physical channel volume and EQ positions at startup.
- `c3ee67e` Add feedback-driven FX pickup and synchronization warnings.

No push, tag or release publication was requested or performed.

## Follow-up use

The user subsequently reported a successful hour-long set with the latest
build on the actual DJ laptop. The [set record](dj-laptop-set-validation.md)
captures that general acceptance without treating it as a focused check of
every startup, pickup or modifier case above.
