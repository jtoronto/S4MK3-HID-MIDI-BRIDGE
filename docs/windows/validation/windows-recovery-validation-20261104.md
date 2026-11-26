# Windows recovery validation, November 4, 2026

The recovery has produced landed Windows software, passing component checks
and native tests, and an unsigned portable integration candidate. A corrected
bounded hardware run now passes startup and physical crossfader control.
Windows support remains unaccepted and unpublished. Independent Play/Cue
note feedback now reaches the integrated bridge, and dedicated deck-A playback
feedback has been observed. Receiver-first startup has produced initial
feedback and physical output on both displays through the NI service, with
right-screen camera evidence and the user's left-screen confirmation.
The successor package also has bounded screen-toggle UI observations and
clean manual Stop and running Quit exits. Complete screen fields, both jog
wheels with normal motor settings, remaining recovery paths, audio coexistence
and a 60-minute attended run remain unverified. The corrected r2 package has
native evidence for both targeted UI fixes and a separate retained worker
Off/On protocol trial. The independent integration gate now approves the
bounded software/native integration milestone; 8/11 recovery plan milestones
are complete. Later r2 workflow observations and the successor launch-preview
package add focused evidence, not full native workflow or product acceptance.
The preview correction's separate independent gate is now approved within
that defect's scope. A later live-log scrolling defect has an accepted local
source fix, 94/94 tests and a successful build; successor packaging is
underway and native package QA hasn't passed. No running motor capture was
produced by the attended trial that exposed the defect.

This is the current recovery record. The earlier
[failed execution report](windows-bridge-execution-results-20261104.md)
is immutable historical evidence and hasn't been changed. Its closed-unmet
tasks don't become passes because later software exists. The
original recovery plan retained the original full
product obligations and all 133 parity rows.

The initial readiness question expired without confirmation, so no attended
hardware outputs began during that checkpoint. A later confirmed-watching
startup failed before controller initialization, as recorded below.
Preparation and offline validation aren't an attended trial.

## Subsequent attended startup

After the user confirmed watching, root immediately launched the prepared
controls check. At 18:28:27 UTC, the frozen EE worker opened native HID and
both MIDI directions, then exited 1 with `short S4 tension write: 95/11`.
Controller initialization remained false and feedback unknown; the intended
90-second controls exercise never began. Neutral cleanup logged the same
count error, and a subsequent process snapshot found no worker. Screens
were not requested. Physical lamp and jog observations remain pending.

The pinned Windows HID backend pads short reports to the collection's
95-byte output capacity and returns that padded count. The motor writer
compared it with the logical 11-byte report length. See
first attended attempt.
Earlier package and test evidence remains tied to its original snapshots.

The shared output boundary was then corrected without changing report bytes,
IDs or macOS counts. It requires the exact completed Windows padded count,
retaining rejection of genuine short writes. All eight source checks and the
fresh crossbuild exited 0; 77 shared, 256 engine and 2 CLI native tests passed
on Windows, all exit 0.

After renewed watching confirmation, the corrected worker completed its
90-second bound at 18:44:00 UTC with exit 0, controller initialization true
and final neutral write accepted. It recorded 100 input events, 100 MIDI
messages, 3 LED writes and zero feedback messages. The user confirmed that
the physical crossfader controlled Djay, LEDs were lit, and no platter movement
was noticed. This is focused control evidence, not normal motor operation or
complete LED acceptance. A GUI-driven silent-clip test received 308 feedback
messages with no outgoing control MIDI; managed Stop acknowledged request 1
and exited 0. A later announced Play/Pause comparison captured 274 normalized
messages, exclusively deck-A Play/Cue note pulses, with no dedicated playback
CC. The user reported that the Play LED appeared to work. These establish
basic genuine feedback, not the separate playback-state signal needed for
full jog/screen integration. See
correction and attended retry.

## Evidence locations and interpretation

### Dedicated playback property investigation

MIDI-only tests with the bridge stopped compared the selected baseline with
two separately named diagnostic mappings. In 120-second captures, the baseline,
an added root `controlType=control`, and an embedded `playing` binding produced
only blinking Play/Cue notes, with no dedicated playback CC.

The installed Windows Djay 5.6.7091.0 factory mappings instead use
`turntableN.isPlaying`. Changing only the deck-one root key from `playing`
to `isPlaying` produced channel 7 CC 4 values 127 and 0 for two independently
clicked GUI Play/Pause cycles. This isolates a working Windows property name;
it does not establish all-deck synchronization or integrated motor/screens.
The Windows-only production correction passed required source checks and
335 native Windows tests. Its subsequent attended run logged the real
deck-A CC 4 values 127/0 and reported feedback received. Package replacement
and all-deck acceptance remain pending. See
native property trial.

The same attended run failed NI screen registration with
`ShortWrite { expected: 20, actual: 0 }` before any accepted screen frames.
The user observed firmware splash screens on both displays and no platter
movement. Managed Stop request 1 was acknowledged; the worker exited 0 and
accepted its final neutral write. NIHardwareService remained at PID 4968,
and the subsequent process snapshot found no bridge worker. Screen
integration remains unmet. See
attended screen trial.

The bounded zero-write correction subsequently passed 342 native tests.
A user-provided webcam enabled right-side observation while the user was away;
the next minimum-force trial instead failed with `Disconnected`, stopped
cleanly, and left the right screen on the firmware splash. A bootstrap-only
discriminator then proved that posting the asynchronous reply read before
the registration write receives a complete 63-byte reply, while reading
after the same write returns zero bytes. That real reply also exposes a
trailing word parser mismatch. Those were the limits at that checkpoint. See
camera trial
and bootstrap discriminator.

### Physical right-screen delivery

Subsequent production captures proved the four-byte trailer varies, including
`WTra`; it is not fixed zero padding. The corrected parser preserves name and
length validation while accepting that opaque word. It then passed device
and instance registration but exposed another unsupported assumption: the
candidate waited for a reply after every setup message.

A separate setup-only diagnostic captured three distinct metadata query
responses and an independent control-granted notification. A fresh-session
single-frame comparison then received an exact four-byte TRUE and visibly
replaced the right firmware splash with a known two-band pattern. A minimal
comparison omitted five unused metadata queries, produced no setup replies,
and visibly reversed that pattern. These discriminate real delivery from a
retained image; they do not establish every possible service reply behavior.

Production now sends only the six configuration writes and requires the
independent control grant. It adds no optional-reply timeout. The queued
unsolicited-reply rejection remains, including a focused regression that
prevents a setup TRUE from authorizing the first frame. All required source
checks passed, followed by 92 shared, 256 engine and 2 CLI native tests.

Frozen worker `132ad3e54e3b3682753dead5095f8f2628e946701b6ceb0d2ffe915dad0fa317`
rendered the bridge layout on the physical right screen in camera-observed
run `ad22be21a2724e2e808b22d682b481ef`. Managed STOP 1 exited 0 at
22:52:14.5842290 UTC; final neutral output was accepted. The log records
zero input events, 170 LED writes, zero outgoing MIDI messages and 243
received MIDI messages. The final readiness display still said feedback
unknown: this run did not independently change Djay playback or establish
fresh mapped state. The sampled right jog remained at its tape baseline,
and visible telemetry reported both modes off and RPM zero.

The left screen was outside the webcam frame, and both-frame accepted counters
were not retained from stdout. Do not infer their physical acceptance from
the right image. A fresh process snapshot found no bridge, diagnostic or
Traktor process; NIHardwareService remained PID 4968. The complete stderr,
receipt and physical image are retained under
production trial.
The separate diagnostics and their hashes are recorded in
setup investigation.

An independent source assessment also identified that Mio can retain an
OS-pending bootstrap write after dropping the future. The subsequent
[cancellation validation](windows-ni-service-cancellation-validation-20261104.md)
records the scoped repair, 354 native application tests, actual pending-write
and runtime-drop cases, and negative controls. It supersedes this earlier
open cancellation issue without claiming general synchronous leak-free
destruction. Remaining physical and workflow obligations keep full Windows
support unaccepted.

### Numeric screen-state feedback

Subsequent receive-only tests with the bridge stopped establish genuine
Windows Djay output for loop length/activation, displayed key and tempo range.
GUI loop changes produced channel 13 CC 97/98 and note 6 transitions.
Changing the silent clip from key 5A to 12A and back produced channel 13
CC 112 values 51/41. Changing tempo range from 16% to 8% and back produced
channel 7 CC 8 values 10/20. Both 120-second receivers ended and closed
normally with exit 0; no worker or receiver remained.

These results support the existing property names, including the range
property absent from captured factory metadata. They also distinguish
missing initial state under these receiver connections from invalid names:
numeric output appeared after GUI changes, without an initial snapshot.
Jump-size GUI changes and production screen refresh were not exercised by
these receive-only captures. The silent clip was left paused at displayed
00:01.4, key 5A, loop length four active, range 16% and pitch 0%; exact loop
boundaries were not captured or proved restored. See
numeric native evidence.

A receiver-first startup discriminator then captured an initial state burst:
814 total packets, 346 CCs, all 33 numeric addresses on each of channels
13-16, playback-off state for all four decks, and tempo range settling to
16%. It spans cooperative Djay closure and relaunch, so some zero values can
also be output reset during closure. The new Djay window opened with empty
decks. This establishes the MIDI-only start order, not an integrated package
startup pass. The later
[integrated receiver-first run](windows-ni-service-cancellation-validation-20261104.md#integrated-startup-and-both-displays)
established initial feedback and output on both displays under its bounded
conditions. Windows startup guidance says to start the bridge before opening
Djay.

Djay restarted from PID 5936 to 6988; no service restart occurred, and
NIHardwareService remained PID 4968. The receiver exited 0 and a subsequent
snapshot found no bridge or diagnostic receiver. Djay did not retain its
loaded test deck or dynamic mixer positions. Root reloaded only the owned
silent clip and later observed it stopped at the beginning; the prior
00:01.4 position and loop/mixer state were not restored. Subsequent trials
must use the fresh baseline described in
receiver-first startup.

Main-workspace evidence is under
`.omo/evidence/windows-recovery/20261104/`, abbreviated `R/` below.
The two retained worktree evidence roots are:

- `F/`: `/Users/joey/Development/S4MK3-HID-MIDI-BRIDGE-wt/windows-bridge-foundations/.omo/evidence/windows-recovery/20261104/`
- `H/`: `/Users/joey/Development/S4MK3-HID-MIDI-BRIDGE-wt/windows-bridge-independent/.omo/evidence/windows-recovery/20261104/host/`

These are local evidence paths, not published downloads. Counts and verdicts
belong to their recorded source and binary snapshots. Later records supersede
specific pending or failed results without rewriting those earlier records.
The package has its own provenance and limited native checks; component tests
aren't a claim that every packaged workflow was exercised.

## Implemented software

The Rust engine now has Windows transport and normalized HID support, a
LoopBe mapping path, feedback routing, managed lifecycle handling and bounded
process inventory. The frozen engine increment received independent and parent
approval for its software and hardware-free native tests. Inventory capture
drains stdout and stderr concurrently, preserves causal errors, and bounds
cleanup of its owned child. The Windows LED fixture correction retained the
production validation assertions.

Evidence: engine parent acceptance,
`F/engine-review/review.md`, and
`F/engine-review/correction-verification.json`. Actual LoopBe/Djay behavior,
physical HID delivery and teardown remain outside that component approval.

The Avalonia tray host implements preferences and six settings areas:
Bridge, Screens, Jog, LEDs, Palette and Diagnostics. It includes mapping
export/import, validation, child-process controls and diagnostic capture.
V6 production changes and the later test-only corrections landed in main.
The final capture test uses a scoped reader with `FileShare.ReadWrite` and
stops capture before fixture deletion; its content assertion remains intact.
Neither test correction changed production code.

Evidence: V6 source landing,
`H/corrections-v6/handoff.json`, and `windows/bridge-app/DESIGN.md`.
The design's workflow coverage target isn't proof of complete native UI parity.

The NI-service screen route is integrated into the production Windows worker.
It binds the uniquely selected live HID collection's serial before initialized
screen input, then requires the service's matching identity and positive
control. It doesn't substitute a hardcoded serial or open MI_04 directly.
The shared renderer remains in use, with a separate Windows BGR565 service
serializer.

Screen setup and display exchanges run on the screen thread, with bounded
waits and cancellation on generation changes or shutdown. Active status
requires accepted full frames for both screens. Failure releases the session;
Off and shutdown don't send a blank image. Desired pages survive loss, and
reacquisition requires an explicit request. These are reviewed software
contracts, not observed installed-service behavior.

Evidence: `F/screens-implementation/integrated/integration.md`,
`receipt.json`, and `review/review.md`. The independent review approved this
source/build increment only. Its dependency correction also matters: the
lockfile upgrades transitive Tokio from 1.53.2 to the explicitly pinned
1.54.1, despite the integration prose saying existing dependencies weren't
upgraded.

## Recorded checks and native results

| Snapshot or check | Recorded result | Boundary |
| --- | --- | --- |
| Frozen engine, before NI integration | 60 shared, 255 engine and 2 CLI native tests passed, 317 total; exits 0 | Hardware-free component gate. The unchanged shared binary reused its matching prior native result. |
| Integrated NI source gate | 550 Rust and 80 Swift tests passed; formatting, strict all-target/all-feature Clippy, all-target build, shell syntax, plist lint and diff check passed | Source/software gate in the foundations worktree. |
| Integrated NI native suites | 77 shared, 255 engine and 2 CLI tests passed, 334 total; each exit 0 | Native execution of hardware-free tests, not a live NI service session. |
| Main source landing | Rust format, Clippy, tests and build, Swift tests, shell syntax, plist lint and diff check recorded exit 0 | See the initial landing receipt and successors for distinct failures and fixes. |
| Main portability repairs | Locked host restore and later Windows all-target cross-build passed; macOS all-target build and strict Clippy passed | Cross-building Windows examples doesn't establish native execution. |
| Final host tests | 88/88 on macOS and 88/88 on Windows, exit 0 | Corrected V6 test snapshot; no physical acceptance. |
| V6 host build and formatting | Both locked restores and whitespace checks passed; Release host build had zero warnings/errors | C# language-server diagnostics weren't available because `csharp-ls` wasn't installed. |
| Actual packaged V6 Rust generator | Baseline Windows mapping and unsaved Jog edits on all four decks passed, exit 0 | Offline generation with the earlier V6 package's worker. |
| Main candidate worker | Generated the expected baseline mapping on Windows, exit 0 | Exact main package check; no HID or MIDI access. |

Sources:

- Engine parent gate
  and `F/engine-review/correction-verification.json`.
- `F/screens-implementation/integrated/receipt.json`, `mac-gates.log`,
  `windows-build.log`, and `review/review.md`.
- Integrated native summaries.
- Initial main landing,
  continuation,
  and independent repairs.
- V6 landing checks,
  parent native host checks,
  and main package native checks.

The earlier main host run was 85/86. Initial Windows all-target compilation
failed in two examples using Unix-only MIDI APIs; later portability repairs
passed that cross-build while preserving their Unix logic. The host lockfile
was restored with one trailing newline difference and an equal parsed
dependency graph; locked restore then passed.

V6's first Windows run was 87/88 because an active capture prevented fixture
deletion. Fixing teardown exposed a reader-sharing exception earlier in the
same test, so the first failure wasn't proof that its visibility assertions
had passed. The final sharing-corrected archive,
`12371516563c80af1f927daff0c999e7fc7581f2ccae9a91907ea10d93f0b6ed`,
passed 88/88 with native exit 0. Earlier failure records remain preserved.

The integrated Swift run retained its pre-existing unhandled-file warning
for `AppIcon.png` and `build-jog-diagnostic.sh`. No warning was suppressed.
No broad source suite was rerun during main candidate packaging.

## Native GUI and mapping evidence

### Successor package: native screen and lifecycle checks

The successor integration ZIP is
`R/app-cancel-package/portable-final.zip`, SHA-256
`86cacacafe8a0a953e7ec34a60d833322cdb59359010548c4dcb0272b48f30c6`.
Root independently checked all 591 manifest payloads locally and again after
native extraction. Its self-contained native host test runner passed 88/88
with exit 0. These results belong to this archive, not a later UI correction.

In the actual packaged app, the parent saved and checked a bounded full
profile: 90 seconds, maximum platter force 1, base force and regulator gains
zero, backspin and assistance disabled, and tension zero. Startup reached
Bridge ready and Screens active. Toggling the actual Screens checkbox
produced off and active while the bridge stayed ready. That worker, PID
7720, reached its duration limit and exited 0 before a subsequent Stop click;
the disabled Stop action did nothing. This first run therefore proves bounded
exit, not manual Stop. The bounded debug log had already discarded the
screen transition lines when exported. The transition observations are native
UI evidence; they are not a retained protocol transcript or both-display
visual acceptance. The webcam recording covers only the right side.

Two subsequent launches used info logging and the same bounded motor values.
The actual Stop button stopped worker PID 13108, exit 0. A further launch
reached Screens active in the actual tray menu; selecting Quit drained worker
PID 10164, exit 0, and the tray host PID 13780 also exited 0. OS process
observers were subscribed before each launch and before host Quit. The
app-written `R/app-cancel-package/native-last-session.log` retains both
launches, screen-active acknowledgments, and managed stopping request 1.
These checks do not exercise working motors, playback feedback or audio.

Native inspection of this archive found two UI issues: successful NI access
information appeared in the error banner, and the bounded log rendered blank
despite its automation value and Copy log containing text. Both were
source-fixed, with 92/92 source host tests passing; the later r2 native checks
below establish the two targeted rendering corrections without changing this
archive's historical result. After Quit, all five original
preference files were restored byte-for-byte against their preserved SHA-256
inventory; the bounded trial versions were preserved separately. See
`R/app-cancel-package/profile-restoration-final.json`. The host and worker
were absent at restoration. No driver or service restart was used.

### Corrected r2 package: targeted UI and retained screen protocol

`R/app-ui-corrected-package/portable-r2.zip` has SHA-256
`ec243eba9708f6612d7700a52f77ecc1fc05d35781b5a201c52f6467d3561efc`.
The parent verified all 591 payloads locally and after native extraction.
The corrected native host suite passed 92/92, exit 0; the r2 notice correction
didn't change the tested host assembly.

Actual host 10020 visibly rendered the bundled worker's `--list` enumeration,
including one S4, no interfaces opened and `EXIT: 0`. With the bridge stopped,
the Screens checkbox Off/On prepared healthy access detail in its normal
section without a false error banner. Its status was pending, not active;
this UI check sent no screen traffic. Normal tray Quit exited 0, and all five
original preferences were restored exactly, with no host or worker remaining.
The parent native UI receipt
links these observations to the retained PNGs, session log and restoration
receipt. This establishes the two specific corrections, not complete UI
acceptance.

A separate CLI trial used the actual r2 worker with the host stopped and
retained full stdout: generation 1 OFF acknowledged `inactive`, then
generation 2 ON acknowledged `pending` and `active` within the same worker.
The protocol trial
establishes release/reacquisition through the production status contract.
It isn't a new app-checkbox protocol transcript. The 90-second timer ended
with exit 0 and accepted neutral debug writes; a later Stop attempt sent
nothing. Cleanup found no host/worker, all five original preference hashes
unchanged and NIHardwareService still Running, PID 4968. Force remained
maximum 1, base/gains zero, backspin/assistance off and tension zero.

The camera ended before ON, and its final image was dark with an overexposed
screen. This trial adds no physical ON-recovery or field-legibility evidence.
Normal motors, both jogs, audio, remaining recovery paths and endurance remain
unaccepted. The later
UI/package gate
and integration milestone gate
approve their bounded scopes. They supersede the earlier pending-review
wording without turning this trial into complete native workflow acceptance.

The failed `509bcbe4...` archive remains preserved and wasn't transferred for
native UI use. The r2 inventory retains 161 third-party Rust packages, including
the actual vendored Mio notice and socket2 notices. See
inventory provenance
for source attribution and the limits of retained compilation evidence.

### Focused r2 native workflows

The workflow record
extends the same r2 package's evidence. All six settings tabs were reachable.
Duration 0 visibly disabled Start and Save, and Error details showed the
nonempty range error. Restoring 3600 cleared it. Retrying screens while
stopped produced the genuine full-profile-start error without a worker.
Successful access detail no longer appearing as an error hasn't hidden this
real failure.

Native LED export wrote brightness 90. After an editor change to 45, Import
restored 90 and Reset restored 127. The exported JSON and
roundtrip receipt
record exported 90, saved 127, five unchanged original preference hashes and
zero workers. A separate Palette change saved Deck A cyan; after host 6448
quit with exit 0, fresh host PID 8008 visibly reloaded cyan. Revealing Jog
Advanced controls was editor evidence only, not applied motor behavior.

Minimal and Probe could be selected without launch. Minimal showed the
full-profile LED scope notice; Minimal with Debug kept capture disabled,
while Full with Debug enabled it. An editor-only capture started and stopped
without a worker. Its retained log has timestamps and an explicitly pending
launch/config header, with no genuine motor receipt or telemetry. Log detail
was disabled during capture. Show capture file opened a new Explorer window,
PID 13552, selecting the exact capture file; that window was then closed.

Closing Settings hid the window while OS process 8008 survived with
MainWindowHandle 0. A window-based helper's no-match result wasn't an app
exit. Tray Settings restored the same Diagnostics tab, Debug editor value
and capture path. Normal Quit exited 0. Before cleanup, the only semantic
preference difference was the deliberately saved cyan value. The
restoration receipt
records preservation of the trial profile, restoration of all five originals
by SHA-256 and absence of host/worker.

These checks produced no controller translation, screen, motor or audio
output. Running restart, genuine running capture, provider failures and the
remaining recovery matrix still belong to the open native workflow milestone.

### Successor package: launch-preview correction

The r2 workflow pass exposed a stopped preview using saved Full arguments
despite unsaved Minimal selection, and omitting the requested screen flag.
The failed r2 observation remains preserved. The successor source uses
editor values when no child is owned and retains the owned child's actual
launch command otherwise. Startup still validates and saves, prepares screen
access, then builds the runtime arguments from saved preferences.

The source verification
records a rejecting regression before the fix and 93/93 afterward. The
package verification
records ZIP SHA-256
`264be03947d43903b00a369a1c4edc8c0f424bb0746f4592553144c812e24d62`.
Its 591 payloads were checked locally and after native extraction, with the
per-file native hashes retained in
preview-native-payloads.json.
The frozen Rust worker remains
`a9dfbb3b05af134d9eb7f2435200d93487cd38d426b056a4f1efb09c60d84b36`;
the mapping, screen declaration and corrected 161-package Rust inventory
are unchanged from r2. These are package identity checks, not a fresh worker
build or complete build-input attestation.

The parent native preview record
and actual native output record 93/93 passing, exit 0. Retained screenshots
show stopped host 6232 displaying unsaved Minimal with 45 seconds and Raw on,
Probe without a MIDI argument, and Full with the requested screens and full
configuration paths. Saved preferences remained Full, unlimited duration
with 3600 stored, and Raw off. Owned-child preview retention is covered by
the native regression's supervised fixture child, not a new physical runtime.
Normal Quit exited 0; the cleanup receipt records all five original hashes
matching and no host/worker. No controller Start occurred.

The separate independent preview gate,
`st_01a1a9b0`, approves the preview defect unit. It supersedes the earlier
pending-review wording, without establishing native running-controller UI
or physical behavior. Neither package is a published or fully accepted
Windows product.

### NI workflow decision and focused attended follow-up

On November 5, 2026, the user selected **NI workflow: Keep service running
(Recommended)**. The parent ledger event `user-ni-workflow-decision` records
the answer at `2026-11-05T04:17:05.116Z`. The supported Windows route keeps
NIHardwareService running and uses an engine-managed screen session.
Legacy automatic service stop/restore and manual service stop/restart are
unsupported, not passing legacy lifecycle tests. The
NI applicability addendum
assesses the affected rows individually and retains applicable pending-state,
ownership, cancellation, screenless/cancel and recovery safety obligations.
The historical parity matrix isn't rewritten by this decision.

The canonical installed mapping is **S4 Corrected Playback Windows**, SHA-256
`dcd21f55908ff6ba88ce87529ea21c43a2a48c0d8525006244ab9584fbe2b172`.
The unsuffixed and plus1/plus2 mappings differ. LoopBe Mute caused the earlier
silent receive tests; after the user unchecked it, the
corrected receive-only capture
received exactly `0x00000097`, `0x007F0097`, `0x00000197`, `0x007F0197` for
left then right channel headphone cue off/on, then closed with exit 0.
That packet result alone wasn't physical-light evidence.

In the subsequent minimum-force run, the user reported both physical
screens working, both headphone cue lights following GUI changes and
neither platter moving. Native observation recorded host 9012, worker 10632
and `Bridge: ready | Screens: active` with full/screens/180-second settings.
No running motor capture was created: live log updates pulled the outer
Diagnostics page away from **Start motor capture**. The
restoration receipt
records app/worker absence and all five original preference hashes matching;
trial settings were preserved separately. This shutdown observation doesn't
establish numeric process exit codes.

The scrolling investigation
records the rejecting local 93/94 regression, corrected 94/94 result and
successful build with zero warnings/errors. The local source fix is accepted.
Packaging is underway under `st_01a1a9df`; native package QA hasn't passed
at this checkpoint. No accepted successor hash or running capture is inferred
from the local result. The
operator runbook
keeps live capture and subsequent attended stages behind that gate.

### Earlier host GUI and mapping checks

Earlier host GUI checks exercised duration validation, saving and reloading
3599 seconds, offline mapping export and cooperative quit. Settings were
restored to a full profile, unlimited duration, stored duration 3600, raw
reports off and info logging. These observations belong to that earlier
package, as recorded in
workflow-result.json.

The parent subsequently quit the stopped V3 host normally, confirmed its
process was gone and launched V6. Setting duration to 0 disabled both window
and tray Start actions and Save, with the expected range error. Duration was
restored to 3600. No Start action occurred.

Djay accepted a generated mapping and selected `S4 MK3 Bridge Windows` for
`LoopBe Internal MIDI`. The
mapping setup record
records that the learn editor was closed. Its advanced-options restoration
field is not a verified account of the final option state. Selection isn't
evidence of working control or genuine feedback.

For the exact main candidate, the parent quit the stopped V6 host and
confirmed its process had exited before launching the candidate. Settings
rendered with duration 3600 and no engine child. Its actual Install mapping
action staged a mapping outside Djay's mapping directory, showed a
pending-import notice and opened Djay's real confirmation dialog. After
confirmation, exactly one new file appeared:
`S4 MK3 Bridge Windows 2.djayMidiMapping`. All three existing mapping files
retained their hashes. The staging file remains in Generated Mappings.

Evidence: main package parent checks,
with captures and before/after inventories under `R/host-ui/22-*`,
`23-main-before-import.txt` and `24-main-after-import.txt`. These checks don't
establish Start/Stop with a controller, complete UI acceptance or live output.

The exact main candidate also retained its stopped tray process when Settings
was closed, then restored the same Diagnostics tab from its tray icon.
Root subscribed to its process exit before selecting normal tray Quit;
the host exited with code 0 and no controller worker remained. This proves
stopped-host hide/restore and quit, not running-worker Stop or drain.

## Main portable candidate and packaging limits

The candidate is `R/main-package/portable.zip`, built from the current main
working tree including uncommitted landed production sources. Checkout HEAD
at landing was `7ec18cf8a85a5450d2bb97043cd0bfb34555807d`; HEAD alone doesn't
represent the package's source.

| Artifact | SHA-256 |
| --- | --- |
| Candidate ZIP | `21dbb912d6f13e0c3987485a1c6494eeee3552a3b4f8e6bad157b1980dfe560e` |
| Packaged Windows worker | `7753b8e9e5c89160ecc4227e0545593e384779142439fc7c8f78cd6e5b154919` |
| Baseline Windows mapping | `adda9484d6c3c5b07d73d417f6a4c0cc480adfc8bd2cbd029cb58f74eaa96b2f` |
| Cargo.lock | `bbfb139ff0ca77fc106a0a813748337aa135a6406f422d5b10fdf94998a34352` |

The ZIP contains 588 files; its runtime manifest covers 587 payloads. Archive
integrity passed, and KENSUKE verified every manifest payload's size and
SHA-256. Parent verification matched 33 engine and five V6 production source
pins against main. Packaging compared 105 source snapshot files and recorded
no verification errors. The original Windows worker build exited 0.

The retained license inventories cover 159 Rust and 23 NuGet dependencies.
Provenance records the exact metadata, notice sources and fallback license
texts. These inventories aren't general redistribution approval.

Evidence: provenance.json,
verification log,
ZIP integrity log,
and parent native verification.

This is an unsigned integration candidate, not a published or accepted Windows
product. It has no `screen-access.json` approval declaration. NI driver/service
and LoopBe setup remain external. Source pins and timestamps support the
recorded worker identity; timestamps aren't cryptographic proof of the entire
build input tree. Earlier native component passes must retain their own binary
identities rather than being relabeled as an exhaustive test of this ZIP.

## Preservation and prerequisites

The successor corrected package is
`R/main-package-hid-fixed/portable.zip`, SHA-256
`484dd707089573a56172ed082eb62fb7532f7b999dd84abdd5441e40fe86c063`.
It contains the tested corrected worker
`7e513548b6639db74e130637df900af5ca7696843158314328635d3ed5328fe3`
and unchanged ADDA mapping. Its build and archive checks passed; KENSUKE
verified all 587 manifest payloads. It remains unsigned with no screen-access
declaration. The earlier package is preserved. See
successor provenance
and parent verification.

Source landing recorded 78 landed files with zero hash mismatches and checked
141 protected files with zero mismatches. Later repair receipts retained zero
protected and original-landed-source mismatches. V6 landing records its eight
source changes and the two test-only successors explicitly.

The NI integration preserved the frozen engine artifacts, 22 other
engine-manifest paths and eight renderer/text/macOS service files. Its
`screens.rs` difference was the new module declaration. Historical failed
reviews, logs and packages remain distinct from their successors.

The original Hercules mapping's preservation baseline was:
`f4bd84d5e400cc5a7d07a69ded1661c27b0b6476b9c5503c452fd0f46792dffb`.
The main candidate import preserved that file and both other existing
mappings, adding its own name instead of replacing user work.
See preservation baseline,
the source-landing receipts and the main package parent record. These bounded
checks don't claim that every temporary artifact or GUI option was restored.

The existing [Windows prerequisites](../windows-setup.md) and Windows notice in
the [user guide](../../user-guide.md) remain accurate. The supported provisioning
route has Traktor installed to supply the NI S4 MK3 driver and
NIHardwareService; Traktor itself needn't be running. Standalone provisioning
of the required service remains unverified. Keep the NI driver and service;
neither WinUSB replacement nor stopping NIHardwareService is an accepted
screen fix. The self-contained host is intended to avoid .NET, Rust and Cargo
installation on the DJ laptop, but that doesn't replace external prerequisites
or package acceptance.

## Remaining acceptance

The following evidence is still required before full Windows product readiness:

1. **Real controller and genuine feedback.** Extend the bounded crossfader,
   startup and feedback observations to the remaining physical controls and
   LEDs, deck/layer ownership, all-deck Djay feedback and outgoing-echo rejection
   in the integrated package. Disconnect and recovery still need acceptance.
   Mapping selection and generated keys don't establish this.
2. **Installed NI service and both screens.** Extend the established service
   startup and both-display output to all screen fields, content, color and
   orientation, control loss, retry and ownership handover. The r2 CLI trial
   retains generation-tagged release/reacquisition acknowledgments; the older
   app-checkbox trial remains UI-only. Physical ON recovery and field
   appearance weren't established by the new camera recording. Successful
   writes alone aren't physical display evidence.
3. **Both jog wheels and motors.** Attend normal and TT behavior on both
   sides, touch/scratch/bend ownership and neutral force on Stop, failure and
   disconnect, with concurrent controls and screens where applicable.
4. **Audio coexistence.** Verify main and headphone routing and controller
   audio while the integrated bridge is active, without claiming its audio
   interfaces for control translation.
5. **Native package workflows and recovery.** Complete the remaining settings,
   persistence, diagnostics, tray, lifecycle and failure/recovery workflows on
   the exact package. The targeted r2 UI/package and integration gates are
   approved, and the focused stopped workflows above pass. Running restart,
   genuine running capture and provider/fault paths remain open. The preview
   gate is approved for its defect unit; the live-scroll fix still needs
   native package QA. Bounded timer exit, manual Stop,
   running tray Quit and exact
   restoration of the five original preferences are established only for the
   recorded candidate and conditions above.
6. **Sustained attended use.** Complete the required 60-minute controller,
   both-screen, both-jog and audio acceptance, retaining focused per-control
   evidence and the original parity obligations. A successful set alone won't
   prove every fault or recovery path.

The user chose the NIHardwareService-running route. Legacy service
stop/restore is unsupported Windows product scope, not tested lifecycle
parity. The scoped NI addendum retains each applicable safety and recovery
obligation; it doesn't assign blanket N/A or pass status to composite rows.
The route decision doesn't establish physical recovery or full native
workflow acceptance. The recovery plan remains at 8/11 milestones complete, with
native workflows/recovery, attended controller/audio/load acceptance and
final delivery still open.

The first controls script's ValidateOnly run exited 0 and generated a mapping
offline without opening hardware, as recorded in
host-v6-parent-native.md.
Its preparation artifacts remain under `F/engine/first-control-prep/`.
The expired readiness question supplied no watching confirmation at that
checkpoint. The first confirmed-watching launch failed, then the corrected
retry established the focused startup/crossfader/LED observations above.
The complete physical acceptance items above remain open.
