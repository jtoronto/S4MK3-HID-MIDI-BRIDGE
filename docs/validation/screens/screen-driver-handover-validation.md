# Screen handover software and package validation

October 22, 2026. Local development build, still 0.1.1-beta.3 / 0.1.1 / build 4.
No project commit, push or public release is part of this build.

The newest results are first. Later sections preserve earlier checkpoints;
their pending checks and package hashes describe those earlier builds.

## October 23 NI stop-method correction

Attended controls isolated the shutdown method rather than the renderer:

- The exact packaged bridge worked after NI was stopped before an S4 cold reset.
- With NI running and no bridge, the firmware logos stayed visible after boot.
- Stopping NI alone with SIGTERM darkened both panels before any bridge started.
- A later warm Start wrote complete frames but did not recover the panels.
- One explicitly approved, identity-checked SIGKILL preserved both logos.
- The unchanged packaged bridge then displayed both pages without another reset.
- The test bridge was stopped and the verified NI driver restored afterward.

Holding early redraws was not the fix: an otherwise identical unheld control
also worked when NI was absent before reset. The temporary screen gate and
one-cycle force-stop source probe have been removed.

The user selected guarded forced termination for screen handover. The production
backend retains its fresh PID/executable check, signed installation verification,
active-NI-app refusal, stopping consent, exit-event subscription and owned
restoration policy. Only the termination signal changes. No display-power bytes,
fixed startup delay, motor-law change or mapping change is introduced.

A new owned-process regression failed against the prior backend with termination
status 15 instead of 9. It subscribes before termination and checks the actual
process exit, without a sleep or polling loop. With the correction, all 55 Swift
tests and all 308 Rust tests passed. Required formatting, strict Clippy,
all-target build, shell syntax and plist checks passed. Changed production Swift
and native-test diagnostics are clean. The two pre-existing SwiftPM resource
warnings remain.

Final app, ZIP and DMG are in `dist/Screens NI Handover Fix-20261023/`, still
0.1.1-beta.3 / 0.1.1 / build 4. Earlier packages were preserved. All three app
copies passed deep strict signatures, arm64 checks, exact native/Rust executable
comparisons, version/default/license/mapping checks, and fresh isolated preference
initialization, save and reload using a rejecting fake NI agent. ZIP and DMG
integrity passed. No bridge or real NI action ran during this package QA.

The Rust executable is byte-identical to the previous verified candidate, so its
108 exact-raster comparisons remain applicable without another rendering run.
All five real JSON preference files retained their overnight pre-QA hashes.
The old stopped app completed a normal quit in 0.014 seconds; this is not an
attended active-bridge Quit check of the rebuilt app.

Final read-only cleanup checks found NIHardwareAgent restored and no bridge,
app or diagnostic helper running. All five preference hashes still matched
after the old app quit. The owned extraction directory and DMG mount were gone;
temporary diagnostic sources, scripts and executables were removed. Final
`git diff --check` passed.

| Final artifact | SHA-256 |
| --- | --- |
| App executable | `28f32accbd825a7e03c2dd1bbad59f3f886168ea4c52bc45160b3316df55d83b` |
| Bundled Rust bridge | `49ec1ea90170888faf4c9955eea2f8552d78b0b360dcc64cf76b98bfb2783d35` |
| ZIP | `6f67b67001257fb35fac26f7210bb8d78ff986711bbbec29c21ee7786111c788` |
| DMG | `ca33ccebb1cb4a27470566eeab127138eb735c54b2894fbc1f31b1c082987983` |

**Attended rebuilt-app acceptance passed on October 23.** The app executable
matched the final hash above. NI was running before Start; the app was stopped
with the full profile, saved tension and screen automation enabled. After the
user confirmed watching, normal Start stopped NI and launched the bundled bridge.
The user confirmed both screen pages were visible and stayed visible.

After a second watching confirmation, normal Stop ended the bridge and restored
NI. Both events were observed at 15:08:23 UTC; the app returned to Stopped and
`restorePending` became false. A normal application termination request then
completed in 0.011386792 seconds, measured by a process-exit subscription armed
before the request. The app lifetime monitor exited with code 0.

The final process check found NI running and neither the app nor bridge running.
All five real JSON preference files were byte-identical to their pre-test values.
This completes the reported screen-startup and slow-quit acceptance cycle.
The user then launched Traktor and confirmed its screens also worked properly,
verifying screen hand-back after NI restoration. The user subsequently tried the
corrected app on the actual DJ laptop and reported that it was working well.
Long-session endurance remains untested. No project commit, push or public
release was performed during validation.

One longer diagnostic run also exited on `wheel 0 sensor time reversed`.
`TENSION_FINAL_NEUTRAL: native write accepted` preceded teardown. This separate
sensor-clock finding is not changed by the NI stop-method correction.

## October 23 dark-screen and slow-quit follow-up

The user authorized controlled start/stop and NI handover diagnostics. On the
previous build, both panels became completely dark during saved-tension startup,
even though USB writes succeeded and the app reported screens active.
Attended isolation established:

- Static colour bars and labels display correctly.
- Normal HID/MIDI/LED service preserves those bars.
- Enabling bridge pages after normal service starts displays them correctly.
- Screen-enabled cold startup without tension displays bridge information.
- Saved-tension service without screen writes preserves the prior visible page.
- Enabling screens after tension service starts also displays bridge information.

The startup candidate therefore moves the first screen publication out of input
dispatch. Both runtimes publish after native LED output; the paced runtime also
requires an input snapshot before its explicit screen action. No fixed delay or
guessed backlight command was added. The standalone output path now submits its
initial publication rather than treating it as a refresh of an initialized worker.

The slow-quit path awaited NSWorkspace's background-app launch handshake after
the NI process had already appeared. A KVO-runningApplications candidate also
failed for a truly headless fixture and was removed. The corrected native backend
starts the verified installed executable with null stdio, checks the actual
process executable, discovers headless agent processes natively, and subscribes
to process-exit events. The existing signed identity and owned-restoration policy
remain in place. A real NI stop completed in 0.193 seconds and a real restoration
completed in 0.202 seconds with the expected process running afterward.

All 308 Rust tests and 54 Swift tests passed, along with required formatting,
strict Clippy, all-target build, shell syntax, plist lint and diff checks.
Production diagnostics are clean. The two existing SwiftPM unhandled-file
warnings remain. The new scheduler regression covers input gating, publication
after feedback, and failure before that barrier. Native regressions exercise
real AppKit and headless owned fixtures, executable mismatch refusal, discovery,
termination and exit observation.

Combined corrected deliverables are in
`dist/Screens Startup and Quit Fix-20261023/`. The app, ZIP copy and DMG copy
passed deep strict signatures, integrity, source version/resource comparisons,
isolated preference checks and 108 unchanged raster comparisons. Earlier
packages are preserved.

| Artifact | SHA-256 |
| --- | --- |
| App executable | `30fb6c2e9fca0b577f77d70b0c0df53560bdb462eabec237dcbd1cdb5236658d` |
| Bundled Rust bridge | `49ec1ea90170888faf4c9955eea2f8552d78b0b360dcc64cf76b98bfb2783d35` |
| ZIP | `8acc60602922e7077235a49c5c820e6d71883b0dfdb0474c2d140f9aadab576a` |
| DMG | `6b5dc25437c1e2d5ca2068c2a276e6468d2d3e6849c51415721c7e1c08518cf7` |

**Acceptance still pending:** the cold-reset prompt timed out before the new
startup could be observed on the physical panels. The combined packaged app
opened and exposed its stopped/ready menu, but macOS was behind the lock screen
and refused the final Start action. No attempt was made to bypass that input
restriction. Thus neither a visible cold start of the corrected renderer nor
the complete corrected app Start/Stop/Quit cycle is claimed verified here.
The real NI backend restoration timing above is verified independently.
The bridge and owned app were stopped, NIHardwareAgent was restored, and saved
screen automation preferences remained enabled with no restoration debt.

## October 23 startup crash correction

The original development app crashed when stopping NIHardwareAgent. Both supplied
local crash reports (`S4Bridge-2026-10-22-202825.ips` and
`S4Bridge-2026-10-22-203011.ips`) show `EXC_BAD_ACCESS` in
`NSRunningApplication.removeObserver` during KVO/application deallocation. Their
executable UUID matches the original packaged app.

The native termination stream retained its KVO token but not the application
being observed. That corrective build retained the application through observer
invalidation. A regression at the real native boundary launches only an owned
fixture app, drains the observer-registration autorelease pool, terminates the
fixture and awaits its exit event with a bounded timeout. The unfixed native
path reported an application deallocated with observers registered and exited
with signal 11; the corrected path passes. The final fixture runs an AppKit
event loop rather than relying on a sleeping process.

All 53 Swift tests and 307 Rust tests passed, together with the required source
checks. The same two pre-existing SwiftPM resource warnings remain. Production
Swift diagnostics are clean; standalone test SourceKit diagnostics timed out,
but the complete test target compiled and passed.

Corrected app, ZIP and DMG are under
`dist/Screens Handover Crash Fix-20261023/`; original development packages remain
untouched. All three app copies passed signatures, versions, resource/default
comparisons, isolated preferences checks and the existing raster comparisons.
ZIP and DMG integrity passed. The corrected packaged app opened successfully
with the real saved preferences, and its Screens tab showed `Ready to start`
and `NI screen driver is not running.` No bridge or real NI service was started
for this verification. The corrected app was left open, stopped, on Screens.
Actual controller startup and NI/Traktor hand-back still need an attended check.

Corrected SHA-256 hashes:

| Artifact | SHA-256 |
| --- | --- |
| App executable | `2a13f4307bd431856ce9f888b973d666436253c78358df680c2e392533844e48` |
| ZIP | `c2fc137c697a9aac5272faf9aac7d0387e414de5a6be3b831e9ee6bc9c879ea4` |
| DMG | `56f22434aac558bb08897f548752958d6892cd0521da83b736db1d813f48a111` |

The bundled Rust bridge remains byte-identical to the original handover build.

## Source checks

- All 307 Rust tests passed, without ignored or filtered tests in the full run.
- All 52 Swift tests passed. The existing SwiftPM warnings for
  `Assets/AppIcon.png` and `build-jog-diagnostic.sh` remain.
- Required formatting, strict locked all-target/all-feature Clippy, locked
  all-target build, shell syntax, plist lint and diff checks passed.
- Changed production Rust/Swift modules returned clean language-server
  diagnostics. Separate ignored QA entry-point files compile with the production
  sources under `APP_TESTS`; standalone SourceKit lacks that multi-file context.

The new deterministic regressions cover:

- Same-process live ON/OFF/RETRY and generation-aware release acknowledgements.
- No NI checks for disabled startup/menu refresh and non-full startup.
- Declining termination leaves the requested setting on and exposes recovery.
- Durable responsibility before termination; originally stopped agents are not
  automatically started; explicit manual recovery has no ownership requirement.
- Save and restart does not restore NI between child processes.
- Quit during NI termination does not launch a bridge and waits for restoration.
- Child exit during live OFF does not release app actions or finish quit before
  cleanup completes; concurrent restoration uses one native request.
- The bridge can exit while manual recovery confirmation is open without
  losing the subsequent restoration request.
- Actual NSMenu action dispatch changes the same saved master preference;
  manual recovery remains actionable with screens off and a non-full profile.
- Exact native bundle/executable identity, active-NI-app refusal, denied
  termination, observed exit, already-exited races, deadline and restore identity.

No fixed sleeps or polling delays are used to make these tests pass. The native
termination timeout test exercises its actual deadline with no exit event.

## Native layout evidence

The isolated preview compiles the production settings views, menu factory,
controller and handover coordinator. It injects a fake NI backend and an owned
shell child implementing the screen protocol, never the real hardware bridge.
Evidence is under `.omo/evidence/screen-handover-20261022/`.

48 native WindowServer screenshots cover all six tabs, light/dark appearance,
default 760 x 680 content and minimum 760 x 540 content, and scrolled minimum
views. Screens additionally covers active, disabled with recovery responsibility,
and blocked states. Fixed header/footer controls remain outside the scroll area.

Six additional inspected captures cover the NI stop confirmation in both
appearances, active/blocked/disabled native menus, and the stop-bridge/restore
confirmation: 54 final captures in total. The disabled menu leaves the master
unchecked and manual recovery enabled. Both confirmation dialogs show their
complete text and actions. The unbundled QA harness uses a generic alert icon;
the packaged app retains its bundled icon.

Early 640-point minimum captures revealed macOS collapsing all six tabs into an
overflow control. The minimum width was raised to the verified 760-point default;
the corrected captures retain direct access to all tabs. Earlier captures remain
in `before-minimum-width-fix.PisOgJ/`, not presented as passing evidence.

The display was initially inactive: both native capture and a direct
`CGGetActiveDisplayList` check reported no active displays. After the display was
woken, native screenshots succeeded. Temporary power assertions apply only while
the owned QA preview runs and are removed during cleanup.
All owned preview processes and their temporary power assertions were stopped
after inspection; the previously running app was left alone.

## Packages

Deliverables:

- `dist/Screens Handover Development-20261022/S4 MK3 Bridge.app`
- `dist/Screens Handover Development-20261022/S4 MK3 Bridge-arm64.zip`
- `dist/Screens Handover Development-20261022/S4 MK3 Bridge-0.1.1-beta.3-arm64.dmg`

Built with `CARGO_PROFILE_RELEASE_STRIP=none` and the script's
`S4_BRIDGE_OUTPUT_DIR` override. The older root-dist app/ZIP/DMG and running older
app are preserved. Intermediate owned checkpoints are kept under the new output
folder rather than overwritten or installed.

The app, independently extracted ZIP copy and read-only mounted DMG copy passed
deep strict ad-hoc signatures, arm64 checks, source versions/defaults/font-license
comparisons, mapping plist lint and native/bridge executable equality. ZIP
integrity and the DMG checksum passed. The build checks system-only absolute
dynamic-library dependencies.

Each copied bridge exported all 36 native page rasters, matching the accepted
smooth-font rasters byte-for-byte: 108 comparisons. The bundled mapping matches
the earlier app's default mapping, so the existing Info Screen mapping can stay
selected. No new MIDI action addresses were added for the handover feature.

Production `BridgeController` initialization/save/reload was checked against each
copy's real Resources using a fresh isolated support directory and a backend
that fails if any NI query/stop/restore is attempted. Screen preference defaults
and changed values round-trip; disabled full-profile previews keep the control
pipe but omit `--screens`, and minimal profiles omit both flags.

Final SHA-256 hashes:

| Artifact | SHA-256 |
| --- | --- |
| App executable | `b392cc98e57927696541b1b9c9e4dcd28ae189393de7c73a515c7007a1320ad1` |
| Bundled Rust bridge | `2147fa4e5d620acdda490d598555f802abdf5a4e3f100551dbde37d17d4f0ee4` |
| ZIP | `d4eefcd3db43c19f08f1f213fa6a06869524953ea02f0af647dd2d697b6d4bb0` |
| DMG | `c47d53898585544fccf316794d53327ae023e6f62adbd5b9a8fa7fe75b223a18` |

The production static-code verifier accepted the installed NIHardwareAgent's
bundle/executable/signature identity read-only. No real NI agent was stopped or
started. No HID, screen USB, motor or audio service was opened during this work.

## Physical acceptance still needed

Actual NI stop/restore and Traktor screen hand-back need an attended S4 check.
Likewise, live screen toggling during audio playback and saved tension operation
are not established by a fake transport, a fake driver, or native UI captures.
Use the [ordered user guide](screen-driver-handover.md); keep automatic stopping
off for the first attended handover until that behavior has been checked.
