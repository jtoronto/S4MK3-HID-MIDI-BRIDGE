using Avalonia.Controls;
using S4Bridge.Core;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private Control BridgeTab()
    {
        var run = host.Store.Editor.Run;
        var profile = fields.Choice("Profile", run.Profile, ["full", "minimal", "probe"], v => run.Profile = v);
        profileControl = profile;
        var durationToggle = fields.Toggle("Stop after a fixed duration", run.LimitDuration, v => run.LimitDuration = v);
        var duration = fields.Number("Run duration (seconds)", () => run.DurationSeconds, v => run.DurationSeconds = (int)v!, 1, 604800, true);
        durationToggle.IsEnabled = duration.IsEnabled = !host.ExperimentalNiDecks;
        return UiTheme.BodyView(
            UiTheme.SectionView("Bridge operation",
                UiTheme.Text(host.ExperimentalNiDecks
                    ? "Experimental NI decks mode: full profile, BLOOP and continuous operation are fixed for this host session."
                    : "Choose the bridge profile and run duration.", true),
                profile,
                UiTheme.Text("Full: all controls and feedback. Minimal: Play, Cue and faders only. Probe: HID diagnostics, no MIDI.", true),
                durationToggle,
                duration,
                UiTheme.Text("Continuous mode runs until Stop or Quit. Closing Settings leaves the tray app running.", true)),
            UiTheme.SectionView("Djay mapping",
                UiTheme.Text($"With the bridge running, select this Windows mapping for {BloopEndpoint.PortName} in Djay. Existing mappings are never replaced."),
                UiTheme.Actions(Action("Install Djay mapping...", () => ExportMappingAsync(true)),
                    Action("Export mapping...", () => ExportMappingAsync(false)),
                    Action("Check controller", () => { host.Check(); return Task.CompletedTask; })),
                UiTheme.Text("The actual bundled Rust generator reads the current unsaved Jog editor. Restart alone does not apply mapping changes. Minimal mode uses different addresses; the full mapping is not a minimal mapping.", true)),
            UiTheme.SectionView("Files and applied values",
                UiTheme.Selectable(host.Store.Root),
                Action("Show preferences", RevealPreferences),
                UiTheme.Text("Editor changes are separate from saved JSON and the current launch. Save validates all files first, then replaces each file atomically. A partial I/O failure never restarts the bridge.", true),
                Action("Show saved / launched values", ShowValuesAsync),
                UiTheme.Text("The self-contained host needs no .NET, Rust or Cargo installation. Windows MIDI Services with Basic Loopback (Preview 13 transports) is required. Start creates the MIDI endpoint; Stop and Quit remove it after the worker exits. Audio stays routed directly through the S4.", true)));
    }
    private async Task ShowValuesAsync()
    {
        var saved = host.Store.Saved;
        var launched = host.LaunchedPreferences;
        var text = $"Saved profile: {saved.Run.Profile}, log: {saved.Run.LogLevel}, tension: {saved.Tension.Tension}, RPM: {saved.Jog.PlatterTargetRpm}\n" +
            (launched is null ? "No runtime launch recorded." : $"Last launched profile: {launched.Run.Profile}, log: {launched.Run.LogLevel}, tension: {launched.Tension.Tension}, RPM: {launched.Jog.PlatterTargetRpm}") +
            "\n\nThese are configuration snapshots, not proof of applied force. Motor receipts and actual aggregate status are separate diagnostics.";
        await NoticeAsync("Saved and launched values", text);
    }
}
