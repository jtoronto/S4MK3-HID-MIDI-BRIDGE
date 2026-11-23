using Avalonia.Controls;
using S4Bridge.Core;
using S4Bridge.Models;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private Control? bloopNotice;
    private TextBlock? bloopState;
    private TextBlock? bloopDetail;
    private Button? bloopCheck;
    private Button? bloopDownload;
    private Button? bloopMore;
    private TextBlock? bloopInstallHint;
    private Control BridgeTab()
    {
        var run = host.Store.Editor.Run;
        modeControl = fields.Choice("Bridge mode", ModeName(host.SelectedMode),
            ["Direct HID", "NI Bridge"], ChangeMode);
        var operation = new List<Control>
        {
            modeControl,
            UiTheme.Text("Direct HID: no screens. Does not need NIHardwareService. Platter force and wheel tension are customizable.", true),
            UiTheme.Text("NI Bridge: supports screens and needs NIHardwareService. Platters are managed by NI and cannot be tuned in this app.", true),
            UiTheme.Text("Stop the bridge before changing mode. Closing Settings leaves the tray app running.", true)
        };
        if (host.SelectedMode == BridgeMode.DirectHid)
        {
            profileControl = fields.Choice("Profile", run.Profile, ["full", "minimal", "probe"], v => run.Profile = v);
            operation.AddRange([
                profileControl,
                UiTheme.Text("Full: all controls and feedback. Minimal: Play, Cue and faders only. Probe: HID diagnostics, no MIDI.", true),
                fields.Toggle("Stop after a fixed duration", run.LimitDuration, v => run.LimitDuration = v),
                fields.Number("Run duration (seconds)", () => run.DurationSeconds,
                    v => run.DurationSeconds = (int)v!, 1, 604800, true)
            ]);
        }
        else operation.Add(UiTheme.Text("NI Bridge uses all controls and runs until Stop or Quit.", true));
        bloopState = UiTheme.Text("");
        bloopDetail = UiTheme.Text("", true);
        bloopCheck = Action("Check again", host.RefreshBloopAsync);
        bloopDownload = Action("Download from Microsoft", () => OpenPath(BloopEndpoint.Preview14InstallerUrl));
        bloopMore = Action("Release information", () => OpenPath(BloopEndpoint.Preview14ReleaseUrl));
        bloopInstallHint = UiTheme.Text("Download opens Microsoft's installer link; it does not install anything. Run the installer yourself, then check again. Probe diagnostics do not need Basic Loopback.", true);
        bloopNotice = UiTheme.SectionView("MIDI connection",
            bloopState, bloopDetail,
            UiTheme.Actions(bloopDownload, bloopMore, bloopCheck),
            bloopInstallHint);
        return UiTheme.BodyView(
            bloopNotice,
            UiTheme.SectionView("Bridge operation", [.. operation]),
            UiTheme.SectionView("Djay mapping",
                UiTheme.Text($"With the bridge running, select this Windows mapping for {BloopEndpoint.PortName} in Djay. Existing mappings are never replaced."),
                UiTheme.Actions(Action("Install Djay mapping...", () => ExportMappingAsync(true)),
                    Action("Export mapping...", () => ExportMappingAsync(false)),
                    Action("Check controller", () => { host.Check(); return Task.CompletedTask; })),
                UiTheme.Text("Mapping export includes your unsaved Jog mapping settings. Import and select the new mapping in Djay to apply them."
                    + (host.SelectedMode == BridgeMode.DirectHid ? " The full mapping does not work with the Minimal profile." : ""), true)),
            UiTheme.SectionView("Files and applied values",
                UiTheme.Selectable(host.Store.Root),
                Action("Show preferences", RevealPreferences),
                UiTheme.Text("Save applies your settings and restarts a running bridge. Mapping changes also need a new mapping import in Djay.", true),
                Action("Show saved / launched values", ShowValuesAsync),
                UiTheme.Text("Windows MIDI Services with Basic Loopback is required. Start connects MIDI; Stop and Quit disconnect it. Audio stays routed directly through the S4.", true)));
    }
    private static string ModeName(BridgeMode mode) => mode == BridgeMode.NiBridge ? "NI Bridge" : "Direct HID";
    private void ChangeMode(string value)
    {
        var mode = value == "NI Bridge" ? BridgeMode.NiBridge : BridgeMode.DirectHid;
        if (mode == host.SelectedMode) return;
        // Rebuilding discards numeric drafts, so only a valid editor may change mode.
        if (!fields.Valid || !host.FieldsValid || !host.CanChangeMode) return;
        host.SelectMode(mode);
        BuildTabs();
    }
    private void UpdateBloop()
    {
        if (bloopNotice is null || bloopState is null || bloopDetail is null || bloopCheck is null) return;
        var unavailable = host.BloopStatus is { Status: not BloopDependencyStatus.Available };
        bloopState.Text = host.CheckingBloop ? "Checking Basic Loopback..." :
            host.BloopStatus is null ? "Basic Loopback has not been checked." :
            unavailable ? "Basic Loopback unavailable" : "Basic Loopback ready";
        bloopDetail.Text = host.BloopStatus?.Status switch
        {
            BloopDependencyStatus.ServiceUnavailable =>
                "Windows MIDI Services is unavailable. Basic Loopback is needed to connect the bridge to Djay.",
            BloopDependencyStatus.TransportUnavailable =>
                "Windows MIDI Services is available, but Basic Loopback is unavailable. Install Microsoft's transport package to connect the bridge to Djay.",
            BloopDependencyStatus.CheckFailed =>
                "The Basic Loopback check could not finish. Check again or open Diagnostics for more information.",
            _ => ""
        };
        bloopDetail.IsVisible = !host.CheckingBloop && !string.IsNullOrEmpty(bloopDetail.Text);
        if (bloopDownload is not null) bloopDownload.IsVisible = unavailable;
        if (bloopMore is not null) bloopMore.IsVisible = unavailable;
        if (bloopInstallHint is not null) bloopInstallHint.IsVisible = unavailable;
        bloopCheck.Content = host.CheckingBloop ? "Checking..." : "Check again";
        bloopCheck.IsEnabled = !host.CheckingBloop;
    }
    private async Task ShowValuesAsync()
    {
        var saved = host.Store.Saved;
        var launched = host.LaunchedPreferences;
        static string Values(Preferences preferences) =>
            $"{ModeName(preferences.Run.Mode)}, profile: {(preferences.Run.Mode == BridgeMode.NiBridge ? "full" : preferences.Run.Profile)}, log: {preferences.Run.LogLevel}" +
            (preferences.Run.Mode == BridgeMode.DirectHid
                ? $", wheel tension: {preferences.Tension.Tension}, target RPM: {preferences.Jog.PlatterTargetRpm}"
                : ", platters managed by NI");
        var text = $"Saved: {Values(saved)}\n" +
            (launched is null ? "The bridge has not been started." : $"Last launched: {Values(launched)}") +
            "\n\nThese are settings, not measurements of platter force. See Diagnostics for runtime status.";
        await NoticeAsync("Saved and launched values", text);
    }
}
