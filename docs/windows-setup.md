# Windows setup for development builds

**Windows support is under development and has not been accepted for use.**
There is no published, ready-to-use Windows bridge download. These instructions
describe the current development app. Both displays have produced output in
bounded testing, but complete screen, control, motor, recovery and audio
acceptance remains open. The [implementation state](windows-current-state.md)
records the known software limitations.

## Native Instruments driver and background service

For the full Windows setup, install Native Instruments' **S4 MK3
driver** and **NIHardwareService**. The NI-service screen-access path
needs that background service as well as the controller's NI driver; an audio
driver alone is not evidence that the required service is present.

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

Install Windows MIDI Services and the **Preview 13 Transports** components
that provide **Basic Loopback (BLOOP)**. Use Microsoft's
[Preview 13 instructions](https://github.com/microsoft/MIDI/releases/tag/inbox-preview-13)
for the supported Windows version and component installation. The bridge
checks service and transport availability when starting. It does not install
drivers, restart MIDI Services or fall back to another MIDI backend.

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

For a screenless direct-HID run, open `S4Bridge.exe` directly and keep
**Enable S4 screens** off. This path does not use NIHardwareService; the
separate NI-decks launcher opts into the NI-managed path. Djay PRO is
required for the custom mapping.

1. Connect and power the S4. Close Traktor and any other bridge instance.
   If using the NI display path, leave NIHardwareService running.
2. Open the bridge. In Settings, select **full** and, if
   wanted, turn on **Enable S4 screens** in **Screens**.
3. Start the bridge and wait for **Bridge: ready**. In Djay, select the
   Windows mapping for **S4 MK3 Bridge BLOOP**. Djay can already be open,
   but may send its refresh before the bridge starts listening. Ready
   confirms the controller and MIDI handles, not feedback. Check playback and
   headphone cue lights against Djay; other values remain unknown until Djay
   sends them. If the initial feedback is missing, save the diagnostics and
   fully quit and reopen Djay while the bridge is ready. Switching
   mappings is not an established safe refresh.
4. Check **Screens: active** and the actual displays. The screen worker requests
   access through NIHardwareService; registration alone isn't enough. A
   pending, blocked or failed state isn't confirmation of working screens.

Screen preferences save immediately. **Enable S4 screens** can turn screens
off and on while a full-profile bridge is running. If access fails, read the
reported detail; **Retry verified access** retries screens on a running
full-profile bridge when the package declares access available. A package
without that declaration can offer to continue without screens. Don't stop
the NI service or replace its driver to bypass an access failure.

Current source builds use a separate screen helper so its failure can be
reported without stopping controller and MIDI handling. Turning screens off
or stopping the bridge can leave the last image on the displays; it does not
guarantee a blank screen. If release is reported as unconfirmed, check
Diagnostics and do not assume that the NI service has recovered screen
ownership. Keep NIHardwareService running.

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
**Review owned recovery** to read the diagnostic. The visible
**Restore an app-owned NI transition on stop** preference doesn't implement
Windows service restoration. Don't clear pending state or change a service
to bypass an error; Quit can refuse while that state remains unresolved.
Keep the error details for review.

## Diagnostics command preview

While the bridge is stopped, **Diagnostics > Command preview** follows your
current editor choices, including unsaved profile, duration and Raw HID
settings. For the full profile it also shows the requested screen setting.
This is a draft of the next launch, not confirmation that screens are available.
**Start bridge** validates and saves preferences, prepares screen access, then
creates the owned BLOOP endpoint before launching. Preparation can cancel
startup or change whether screens are included.

While the app owns a running or stopping child process, the preview shows that
process's actual launch command. Unsaved editor changes don't replace it.

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
