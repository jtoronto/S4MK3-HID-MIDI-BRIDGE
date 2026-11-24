# Windows setup for development builds

**Windows support is under development and has not been accepted for use.**
There is no published, ready-to-use Windows bridge download. These instructions
describe the current development app. Both displays have produced output in
bounded testing, but complete screen, control, motor, recovery and audio
acceptance remains open. The [implementation state](windows-current-state.md)
records the known software limitations.

## Native Instruments driver and background service

**Direct HID** does not need **NIHardwareService**. Windows may still install
the controller's device driver; that is separate from the NI background service.
**NI Bridge** needs both the controller driver and **NIHardwareService**.
An audio driver alone does not establish that the background service is installed.

**Have Traktor installed first:** this is the current supported provisioning
route and the installation present on the tested Windows laptop. **Traktor
does not need to be running when using Djay.** The Traktor application and
NIHardwareService are separate: closing Traktor does not mean the required
background service should be stopped.

**Keep NIHardwareService running:** this is the supported Windows workflow.
The bridge requests screen access through that service and releases its own
screen session when screens are disabled or the bridge stops. Automatic NI
service stop/restore and manual service stop/restart are unsupported on
Windows. The macOS driver-handover instructions don't apply to this route.

This is not a claim that buying or licensing Traktor is the only way to obtain
NI's driver. NI also provides a separate
[Drivers & Other Files page](https://www.native-instruments.com/en/support/downloads/drivers-other-files/)
and [NI support](https://support.native-instruments.com/support/solutions)
for driver/setup information. Whether the standalone S4 MK3 installer supplies the
NIHardwareService required by this screen route has **not been
verified**. Until it is verified, a standalone audio-driver installation is
not a supported substitute for the current Traktor-provisioned setup.

Do not replace the NI display driver with WinUSB or stop NIHardwareService as
a presumed screen fix. Driver/service presence is a prerequisite, not proof
of successful screen access. The bridge must report its actual screen state;
unavailable, blocked or failed screens must not be presented as working.

The self-contained Windows host needs no .NET, Rust, Cargo or MIDI SDK
installation on the DJ laptop. Djay, Windows MIDI Services with the supported
Basic Loopback transport, and a separately accepted bridge package remain
distinct requirements.

## Windows MIDI Services and BLOOP

Both bridge modes need **Basic Loopback (BLOOP)**. If it is unavailable,
read Microsoft's [Preview 14 installation instructions](https://github.com/microsoft/MIDI/releases/tag/inbox-preview-14)
and use the [Preview 14 x64 Transports installer](https://github.com/microsoft/MIDI/releases/download/inbox-preview-14/Windows.MIDI.Services.Transports.1.0.52-preview.14-x64.exe).
Microsoft requires Windows 11 25H2 or later, removal of older preview
components as directed in the release notes, and closing all MIDI apps
before installing the transports. Open the bridge again after setup and
recheck Basic Loopback. The app does not install components or fall back
to another MIDI backend.

PowerShell can download the same official installer and open it without
using a browser. Follow the prerequisites above first. This is not an
installer-free or unattended installation:

```powershell
$url = 'https://github.com/microsoft/MIDI/releases/download/inbox-preview-14/Windows.MIDI.Services.Transports.1.0.52-preview.14-x64.exe'
$installer = Join-Path $env:TEMP ('MIDI-Transports-preview14-' + [guid]::NewGuid() + '.exe')
Invoke-WebRequest -Uri $url -OutFile $installer -UseBasicParsing -ErrorAction Stop
if ((Get-FileHash -Path $installer -Algorithm SHA256).Hash -ne '674b753ac21bd03535dc6f0cae9542d143a353804192284d5cd194f44be07242') {
    throw 'Installer hash does not match the Microsoft Preview 14 release.'
}
Start-Process -FilePath $installer -Verb RunAs -Wait -ErrorAction Stop
```

Normal app operation does not require LoopBe1 or loopMIDI, or a separately
running endpoint-creation utility. Start creates **S4 MK3 Bridge BLOOP**.
Stop and running Quit remove that app-owned connection after the actual
worker exits and its final output has been collected. Each new Start creates
a fresh connection with the same bridge-specific name.

## Put the bridge on your PC

Use either format when provided with a development build. Both contain the
same self-contained app and use the same prerequisites above.

- **Portable ZIP:** Extract the entire ZIP into a folder you choose, then
  open `S4Bridge.exe`. Keep its files together. To remove this copy, stop
  and quit the bridge, then delete the extracted folder.
- **Optional installer:** Run the `…-win-x64-Setup.exe` for your current
  Windows user. It puts the app under `%LOCALAPPDATA%\Programs\S4 MK3 Bridge`
  and adds **S4 MK3 Bridge** to Start. It does not need administrator
  privileges, run the bridge, or install Windows MIDI Services, NI software,
  or djay. To update, stop and quit the bridge from its tray menu, then
  run the newer installer. To remove it, use **Settings > Apps > Installed
  apps > S4 MK3 Bridge > Uninstall**.

Moving a ZIP copy on the same PC, or switching between ZIP and installed
copies, keeps your bridge preferences. Both use `%LOCALAPPDATA%\S4 MK3 Bridge`.
Removing either copy leaves preferences and djay mappings alone. The ZIP
does not carry your settings to another PC. Run only one copy at a time.

## Development package source and notices

For private builds, the engine/host inventory and the screen helper's source
and notice inventory are optional. The package records omitted inventories in
`PACKAGE-STATUS.txt` and `runtime-manifest.json`. If an inventory is supplied,
the packager validates it. Review the applicable source and license
requirements before sharing a package.

## Mapping setup

1. Install the prerequisites and open the bridge's Settings. In **Bridge**, choose the
   **full** profile for the full-controller mapping and screens.
2. Choose **Install Djay mapping...** and complete the import in Djay. Opening
   the generated file alone doesn't confirm that Djay imported it.
3. Start the bridge and wait for **Bridge: ready**. With the endpoint available,
   select the new Windows mapping for **S4 MK3 Bridge BLOOP** in Djay.
4. Keep custom mappings under their existing names. For a manual export, choose
   **Export mapping...** and a new filename; the bridge refuses to replace an
   existing mapping. Use the mapping generated by the matching Windows app,
   rather than the Mac mapping.

An older LoopBe mapping targets a different endpoint. Generate a new mapping
with the matching BLOOP app rather than changing or replacing a custom mapping.

Mapping generation uses the current Jog editor values, including unsaved
changes. Export and select a new mapping after changing mapping-related Jog
settings; restarting the bridge alone doesn't update Djay's mapping.

## Normal startup and screens

Djay PRO is required for the custom mapping. Open `S4Bridge.exe` and choose
a **Bridge mode** while stopped. Save the choice or start the bridge to
remember it for next time. No separate launcher is needed.

| Bridge mode | Screens | Platters |
| --- | --- | --- |
| **Direct HID** | Not available; no NIHardwareService required | Tunable in **Jog** with the full profile |
| **NI Bridge** | Available through NIHardwareService | Managed by NIHardwareService; motor tuning is not available |

The separate [attended trial runbook](../windows/qa/ni-bridge/ATTENDED-RUN.md)
and [validation record](windows-ni-decks-validation-20261123.md) describe
the earlier diagnostic packages and their limited hardware coverage.

1. Connect and power the S4. Close Traktor and any other bridge instance.
   If using the NI display path, leave NIHardwareService running.
2. In **Bridge**, select **Direct HID** or **NI Bridge**. Use **full**
   for normal Direct HID operation; NI Bridge always uses the full profile.
3. Start the bridge and wait for **Bridge: ready**. In Djay, select the
   Windows mapping for **S4 MK3 Bridge BLOOP**. Djay can already be open,
   but may send its refresh before the bridge starts listening. Ready
   confirms the controller and MIDI handles, not feedback. Check playback and
   headphone cue lights against Djay; other values remain unknown until Djay
   sends them. If the initial feedback is missing, save the diagnostics and
   fully quit and reopen Djay while the bridge is ready. Switching
   mappings is not an established safe refresh.
4. In **NI Bridge**, check **Screens: active** and the actual displays.
   Registration alone isn't enough. If the screens fail, check **Diagnostics**.
   Stop the bridge before changing modes.

In NI Bridge, the screens and platters use the same NI session. There is
no separate screen toggle for normal operation. The last display image can
remain after stopping. If release is unconfirmed, check Diagnostics and do
not assume that NI has recovered control. Don't stop the NI service or
replace its driver to bypass an access failure.

Use **Stop** to end the bridge or the tray's **Quit S4 MK3 Bridge** to stop it and exit the
app. Closing Settings leaves the tray app running. Stop the bridge before
returning to Traktor or another controller application.

If Stop is unconfirmed, the app keeps the worker and endpoint owned; it does
not force-kill the worker or claim a neutral controller state. If endpoint
removal fails after the worker exits, Start remains disabled and **Stop**
retries cleanup. Running Quit also waits for cleanup.

On the next Start, the app can recover its own abandoned connection only when
the saved ownership record matches and the former host and worker are no
longer running. It refuses an unowned name conflict or a saved endpoint
conflict. Read the error details and preserve `bloop-owner.json`; do not delete
the record or remove unrelated MIDI ports to bypass an ownership error.

If **NI recovery pending** appears, keep the saved state and use
**Review NI recovery** in the tray to read the diagnostic. The bridge does
not implement Windows service restoration. Don't clear pending state or
change a service to bypass an error; Quit can refuse while that state remains
unresolved.
Keep the error details for review.

## Diagnostics command preview

While the bridge is stopped, **Diagnostics > Command preview** follows your
current editor choices, including the selected bridge mode. Direct HID also
uses the selected profile and duration. This is a preview, not confirmation
that the controller or NI screens are available. **Start bridge** validates
and saves preferences before creating the owned BLOOP endpoint for MIDI.

While the app owns a running or stopping child process, the preview shows that
process's actual launch command. Unsaved editor changes don't replace it.

## Attended screen and motor diagnostics

For motor or ring problems that appear only with screens enabled, use the
separate [screen/motor diagnostic package instructions](../windows/qa/screen-motor/README.md).

## Optional temporary MIDI incident capture

Current development builds offer a separate **Diagnostics > MIDI diagnostic
capture** for investigating intermittent MIDI problems. It is optional and
applies to one full-profile bridge run. The checkbox isn't a saved preference;
enable it again for another capture. It doesn't change transport settings or
require Debug log detail.

1. With the bridge stopped, select the **full** profile and check
   **Capture MIDI on the next full-profile launch** in Diagnostics.
2. Start the bridge. Wait for **Bridge ready** and capture **Status: recording**
   before reproducing the issue. Follow the normal startup order above if
   opening Djay.
3. When you notice the problem, click **Mark issue**. Note the deck, physical
   control and symptom alongside the displayed marker number. The marker
   identifies when you noticed the problem, not its exact onset or cause.
4. Use bridge **Stop** to end the run and close the capture. Check the final
   capture status and its written, dropped and unwritten counts. A queued
   marker acknowledgement doesn't confirm that the marker was saved. Final
   counts report overall capture results; inspect the file to confirm an
   individual marker. Keep any error or **incomplete** status with the report.
5. Click **Show MIDI capture file** to reveal the `.jsonl` file in Explorer.
   Files are saved under `%LOCALAPPDATA%\S4 MK3 Bridge\Diagnostics\`. Retain the
   file with your symptom notes, app build, Djay version and mapping name.
   Review its contents for private information before sharing.

Capture continues while Settings is hidden. **Copy log** copies the bounded
display log, not this separate MIDI capture.

The file records incoming MIDI delivered to the bridge before its feedback
filtering, plus outgoing attempts before sending. An outgoing record doesn't
prove successful delivery or a change in Djay. Incoming records on the shared
BLOOP connection don't identify which application originated them, and the
capture can't reveal messages blocked inside the driver. Concurrent events
can be written out of sequence; don't assume the file's line order matches
their sequence numbers.

Capture files are limited to 64 MiB. When the recording limit is reached, the
app reports **Status: limited** and **truncated at recording limit**; further
events are dropped. A queued marker still isn't confirmed saved and may be
lost at the limit. Stop the bridge to close the capture and obtain final
written, dropped and unwritten counts, including limit drops. Queued limit
drops are also included in unwritten counts, so don't add those counts
together. Until final counts arrive, the app reports them as unavailable;
a missing final status means the capture may be incomplete.
