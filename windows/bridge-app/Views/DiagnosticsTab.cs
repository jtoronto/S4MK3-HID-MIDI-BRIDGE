using Avalonia.Controls;
using Avalonia.Media;
using S4Bridge.Models;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private IncrementalLogView? log;
    private SelectableTextBlock? platter;
    private SelectableTextBlock? command;
    private SelectableTextBlock? capturePath;
    private Button? captureButton;
    private CheckBox? midiCaptureEnabled;
    private SelectableTextBlock? midiCaptureStatus;
    private SelectableTextBlock? midiCapturePath;
    private SelectableTextBlock? midiMarkerStatus;
    private Button? midiMarkerButton;
    private Button? midiShowButton;
    private bool midiMarkerPending;
    private ulong? midiPreviousMarker;
    private long logCursor;
    private Control DiagnosticsTab()
    {
        var run = host.Store.Editor.Run;
        var level = fields.Choice("Log detail", run.LogLevel, ["warn", "info", "debug"], v => run.LogLevel = v);
        captureLocked.Add(level);
        platter = UiTheme.Selectable("");
        command = UiTheme.Selectable("");
        capturePath = UiTheme.Selectable("");
        captureButton = Action("Start motor capture", () =>
        {
            if (host.Diagnostics.Capturing) host.Diagnostics.StopCapture();
            else host.Diagnostics.StartCapture(host.Store.Root, host.Supervisor.HasChild ?
                host.Supervisor.ActiveSpec!.Run : host.Store.Editor.Run);
            return Task.CompletedTask;
        });
        midiCaptureEnabled = new CheckBox
        {
            Content = "Capture MIDI on the next full-profile launch",
            IsChecked = host.MidiCapture.EnabledForNextLaunch
        };
        midiCaptureEnabled.IsCheckedChanged += (_, _) =>
        {
            var enabled = midiCaptureEnabled.IsChecked == true;
            if (host.MidiCapture.EnabledForNextLaunch != enabled)
                host.MidiCapture.EnabledForNextLaunch = enabled;
        };
        midiCaptureStatus = UiTheme.Selectable("");
        midiCapturePath = UiTheme.Selectable("");
        midiMarkerStatus = UiTheme.Selectable("");
        midiMarkerButton = Action("Mark issue", MarkMidiIssueAsync);
        midiMarkerButton.Classes.Add("accent");
        midiShowButton = Action("Show MIDI capture file", RevealMidiCaptureAsync);
        logCursor = 0;
        log = new IncrementalLogView
        {
            TextWrapping = TextWrapping.Wrap,
            Height = UiTheme.LogHeight,
            FontFamily = new FontFamily("Cascadia Mono,Consolas,monospace")
        };
        var controls = new List<Control>
        {
            UiTheme.SectionView("Version and log detail",
                UiTheme.Selectable("S4 MK3 Bridge 0.3.0-beta.1 (build 8) | .NET 10 | Avalonia 11.3.8"),
                level, fields.Toggle("Raw HID reports", run.RawReports, v => run.RawReports = v)),
            UiTheme.SectionView("Live platter diagnostics", platter,
                UiTheme.Text("Physical RPM and outgoing MIDI CC4 counts/s, not audible Djay playback speed. Diagnostics continue while Settings is hidden.", true))
        };
        if (host.SelectedMode == BridgeMode.DirectHid) controls.AddRange([
            UiTheme.SectionView("MIDI diagnostic capture",
                midiCaptureEnabled,
                UiTheme.Text("Session-only opt-in: applies once to the next bridge launch. The full profile is required; a minimal-profile launch is blocked while armed. Restart the bridge to begin; this does not change saved preferences.", true),
                UiTheme.Actions(midiMarkerButton, midiShowButton),
                midiCaptureStatus, midiCapturePath, midiMarkerStatus,
                UiTheme.Text("A marker acknowledgement means queued, not persisted. Final counts are reported when the capture ends; an individual marker is not confirmed persisted.", true)),
            UiTheme.SectionView("Motor diagnostic capture",
                UiTheme.Actions(captureButton, Action("Show capture file", RevealCaptureAsync)),
                capturePath,
                UiTheme.Text("Full profile and Debug logging are required. Captures record settings and motor requests from the current launch. Requested force is not measured torque.", true))
        ]);
        controls.AddRange([
            UiTheme.SectionView("Log",
                UiTheme.Actions(Action("Copy log", CopyLogAsync), Action("Show log folder", () =>
                {
                    var folder = System.IO.Path.Combine(host.Store.Root, "Logs");
                    Directory.CreateDirectory(folder); return OpenPath(folder);
                })),
                log, UiTheme.Text("Recent messages only. Captures continue while Settings is hidden.", true)),
            UiTheme.SectionView("Command preview", command)
        ]);
        return UiTheme.BodyView([.. controls]);
    }
    private Task RevealCaptureAsync()
    {
        var path = host.Diagnostics.CapturePath ?? throw new IOException("No capture has been created.");
        return RevealFile(path);
    }
    private Task RevealMidiCaptureAsync()
    {
        var path = host.MidiCapture.Path;
        if (string.IsNullOrEmpty(path)) throw new IOException("No MIDI capture has been created.");
        return RevealFile(path);
    }
    private static Task RevealFile(string path)
    {
        if (!File.Exists(path)) throw new IOException("The capture file is missing: " + path);
        // ArgumentList prevents shell interpretation of paths.
        var info = new System.Diagnostics.ProcessStartInfo("explorer.exe") { UseShellExecute = false };
        info.ArgumentList.Add("/select,"); info.ArgumentList.Add(path);
        using var child = System.Diagnostics.Process.Start(info);
        if (child is null) throw new IOException("Explorer could not reveal the capture.");
        return Task.CompletedTask;
    }
    private async Task MarkMidiIssueAsync()
    {
        midiPreviousMarker = host.MidiCapture.LastMarkerId;
        midiMarkerPending = true;
        try
        {
            var send = host.MarkMidiCaptureIssueAsync();
            UpdateDiagnosticsControls();
            await send;
        }
        catch { midiMarkerPending = false; throw; }
    }
    private async Task CopyLogAsync()
    {
        var clipboard = TopLevel.GetTopLevel(this)?.Clipboard ?? throw new IOException("Clipboard is unavailable.");
        await clipboard.SetTextAsync(host.Diagnostics.Log.Text);
    }
    private void SetDiagnosticsVisibility()
    {
        var visible = IsVisible && ownerVisible && !ownerMinimized && tabs.SelectedIndex == 5;
        if (visible) { RefreshDiagnostics(); refresh.Start(); }
        else if (refresh.IsEnabled) refresh.Stop();
    }
    private void RefreshDiagnostics()
    {
        if (log is null || platter is null || command is null) return;
        var running = host.Supervisor.State == "ready";
        host.Diagnostics.Refresh(true, running);
        platter.Text = host.Diagnostics.Presentation.Left + "\n" + host.Diagnostics.Presentation.Right;
        var delta = host.Diagnostics.Log.ReadSince(logCursor, log.Text?.Length ?? 0);
        log.Apply(delta);
        logCursor = delta.Cursor;
        command.Text = host.Supervisor.HasChild ? host.Supervisor.ActiveSpec!.Preview :
            "Next launch from editor: " +
            Core.LaunchSpec.EditorPreview(host.Executable, host.Store).Preview;
        UpdateDiagnosticsControls();
    }
    private void UpdateDiagnosticsControls()
    {
        if (captureButton is null) return;
        var run = host.Supervisor.HasChild ? host.Supervisor.ActiveSpec!.Run : host.Store.Editor.Run;
        captureButton.Content = host.Diagnostics.Capturing ? "Stop motor capture" : "Start motor capture";
        captureButton.IsEnabled = host.SelectedMode == BridgeMode.DirectHid &&
            (host.Diagnostics.Capturing || (run.Profile == "full" && run.LogLevel == "debug"));
        if (capturePath is not null) capturePath.Text = host.Diagnostics.CapturePath ?? "No capture created.";
        var midi = host.MidiCapture;
        if (midiCaptureEnabled is not null)
        {
            midiCaptureEnabled.IsChecked = midi.EnabledForNextLaunch;
            midiCaptureEnabled.IsEnabled = host.SelectedMode == BridgeMode.DirectHid && !host.Operating;
        }
        if (midiMarkerPending && (midi.LastMarkerId != midiPreviousMarker ||
            !string.IsNullOrEmpty(midi.MarkerError) || midi.State is not ("recording" or "limited")))
            midiMarkerPending = false;
        if (midiMarkerButton is not null)
        {
            midiMarkerButton.IsVisible = midi.State == "recording";
            midiMarkerButton.IsEnabled = host.CanMarkMidiCaptureIssue &&
                host.Supervisor.State == "ready" && !midiMarkerPending;
        }
        if (midiShowButton is not null)
            midiShowButton.IsEnabled = !string.IsNullOrEmpty(midi.Path) && File.Exists(midi.Path);
        if (midiCapturePath is not null)
            midiCapturePath.Text = string.IsNullOrEmpty(midi.Path) ? "No MIDI capture file." : "File: " + midi.Path;
        if (midiCaptureStatus is not null)
            midiCaptureStatus.Text = $"Status: {midi.State}" +
                (midi.EnabledForNextLaunch ? " | armed for next launch" : "") +
                (midi.MaxFileBytes is { } max ? $" | file limit: {max} bytes" : "") +
                (midi.Truncated ? " | truncated at recording limit" : "") +
                (midi.HasFinalCounts
                    ? $" | attempted: {midi.AttemptedCount}, queued: {midi.QueuedCount}, written: {midi.RecordCount}, dropped: {midi.DroppedCount} (limit: {midi.DroppedLimitCount}), unwritten: {midi.UnwrittenCount} (includes {midi.DroppedLimitQueuedCount} limit drops)"
                    : midi.State is "failed" or "incomplete" or "limited"
                        ? " | Final loss and written counts unavailable."
                        : " | Loss and written counts available after capture ends.") +
                (string.IsNullOrEmpty(midi.Error) ? "" : "\nError: " + midi.Error);
        if (midiMarkerStatus is not null)
            midiMarkerStatus.Text = !string.IsNullOrEmpty(midi.MarkerError) ? midi.MarkerError :
                midiMarkerPending ? "Mark issue: awaiting queue acknowledgement; not confirmed persisted." :
                midi.LastMarkerId is { } id && midi.LastMarkerAccepted == true
                    ? $"Marker {id} queued; not confirmed persisted." : "No acknowledged marker.";
    }
}
