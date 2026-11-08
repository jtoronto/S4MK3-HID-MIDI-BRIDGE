using S4Bridge.Diagnostics;
using S4Bridge.Models;
using S4Bridge.Platform;

namespace S4Bridge.Core;

public sealed class HostController
{
    public PreferenceStore Store { get; }
    public BridgeSupervisor Supervisor { get; } = new();
    public DiagnosticHub Diagnostics { get; } = new();
    public MidiCapture MidiCapture { get; } = new();
    public ScreenCoordinator Screens { get; }
    public MappingExporter Exporter { get; }
    public string Executable { get; }
    public Preferences? LaunchedPreferences { get; private set; }
    public string Message { get; private set; } = "";
    public string Error { get; private set; } = "";
    private readonly Dictionary<string, string> fieldErrors = [];
    public bool Operating { get; private set; }
    public event Action? Changed;
    public bool FieldsValid => fieldErrors.Count == 0;
    public string FieldError => string.Join("\n", fieldErrors.Values);
    public bool CanStart => !Operating && !Supervisor.HasChild && !Store.RequiresReset && FieldsValid && File.Exists(Executable);
    public bool CanStop => !Operating && Supervisor.HasChild && Supervisor.State != "stopping";
    public bool CanMarkMidiCaptureIssue => MidiCapture.State == "recording" && Supervisor.Launch?.IsReady == true;
    public void SetFieldError(string field, string? error)
    {
        if (string.IsNullOrWhiteSpace(error)) fieldErrors.Remove(field);
        else fieldErrors[field] = error;
    }
    public void ClearFieldErrors() => fieldErrors.Clear();
    public string FormatErrorDetail(string fieldError = "") =>
        string.Join("\n", new[] { Error, Supervisor.Error,
            Screens.State is "blocked" or "failed" ? Screens.Detail : "", Diagnostics.Error, MidiCapture.Error,
            string.IsNullOrWhiteSpace(fieldError) ? FieldError : fieldError }
            .Where(s => !string.IsNullOrWhiteSpace(s)));
    public Func<ScreenAccess, Task<ScreenConsent>> Consent { get; set; } =
        _ => Task.FromResult(ScreenConsent.Cancel);

    public HostController(string resources, string root)
    {
        Store = new(resources, root);
        Executable = Path.Combine(resources, "s4mk3-hid-midi-bridge.exe");
        Screens = new(Store, new NIServiceControl(resources));
        Exporter = new(Executable, Path.Combine(root, "Temp"));
        Error = Store.LoadError;
        if (!File.Exists(Executable))
            Error += "Engine not packaged. Settings are available, but Start, Check and mapping export require the actual Rust executable.";
        Supervisor.Launched += id =>
        {
            Diagnostics.BeginLaunch(id);
            MidiCapture.Launched(id);
            Screens.Started(id, false);
        };
        Supervisor.Output += (_, line) => Diagnostics.Log.Append(line);
        Supervisor.Protocol += (id, line) => { Diagnostics.Accept(id, line); Screens.Accept(id, line); MidiCapture.Accept(id, line); };
        Supervisor.StderrLine += (id, line) => Diagnostics.AcceptStderr(id, line);
        Supervisor.Exited += (id, _) => { Screens.Exited(id); MidiCapture.Exited(id); };
        Supervisor.Changed += () => Changed?.Invoke();
        Screens.Changed += () => Changed?.Invoke();
        MidiCapture.Changed += () => Changed?.Invoke();
        Exporter.Output += line => Diagnostics.Log.Append(line);
    }
    public async Task StartAsync()
    {
        if (!FieldsValid) throw new InvalidDataException(FieldError);
        if (!CanStart) throw new InvalidOperationException("Bridge unavailable, busy, already running, or preferences require recovery.");
        Operating = true; Changed?.Invoke();
        try
        {
            Store.Save();
            var screens = await Screens.PrepareAsync(Consent);
            if (screens is null) { Message = "Start cancelled. No process launched."; return; }
            LaunchedPreferences = Store.Saved.Copy();
            var capturePath = MidiCapture.PrepareLaunch(Store.Root, Store.Saved.Run.Profile);
            try
            {
                var spec = LaunchSpec.Runtime(Executable, Store, screens.Value, capturePath);
                Diagnostics.Log.Append("LAUNCH: " + spec.Preview);
                Supervisor.Start(spec);
            }
            catch (Exception ex) { MidiCapture.LaunchFailed(ex); throw; }
            if (Supervisor.Launch is { } launch) Screens.Started(launch.Id, screens.Value);
            Error = "";
        }
        finally { Operating = false; Changed?.Invoke(); }
    }
    public async Task MarkMidiCaptureIssueAsync()
    {
        if (!CanMarkMidiCaptureIssue) throw new InvalidOperationException("Wait for the bridge and MIDI capture to be ready.");
        var id = MidiCapture.NextMarker();
        try { await Supervisor.SendMidiCaptureMarkerAsync(id); }
        catch (Exception ex) { MidiCapture.MarkerSendFailed(id, ex); throw; }
    }
    public async Task ApplyAsync()
    {
        if (!FieldsValid) throw new InvalidDataException(FieldError);
        if (Operating || Supervisor.State is "stopping" or "checking" || Screens.Busy)
            throw new InvalidOperationException("Wait for the current operation before saving.");
        // A failed validation/write must never stop a working bridge.
        Store.Save();
        Message = "Preferences saved.";
        if (Supervisor.HasChild)
        {
            await Supervisor.StopAsync();
            if (Supervisor.State != "stopped")
                throw new IOException("Prior bridge did not confirm normal exit. Restart cancelled.");
            await StartAsync();
        }
        Changed?.Invoke();
    }
    public void Check()
    {
        if (Operating || Supervisor.HasChild) throw new InvalidOperationException("Stop the bridge before checking controllers.");
        Supervisor.Start(new(Executable, ["--list"], Store.Editor.Run with { }, true));
    }
    public void Report(Exception error)
    {
        Error = error.Message;
        Diagnostics.Log.Append("APP_ERROR: " + error.Message);
        Changed?.Invoke();
    }
    public void Edited() { Message = "Editor changes are not applied until saved; mapping edits require a new export."; Changed?.Invoke(); }
    public void Reset()
    {
        Store.Reset(); Error = ""; Message = "Defaults restored in the editor. Save to apply."; Changed?.Invoke();
    }
    public async Task QuitAsync()
    {
        if (Operating || Screens.Busy) throw new InvalidOperationException("Wait for the current operation before quitting.");
        await Supervisor.StopAsync();
        if (Supervisor.HasChild) throw new IOException("Bridge still owned. Quit cancelled.");
        if (Store.Saved.Screen.RestorePending) Screens.RequireOwnedRecovery();
        Diagnostics.StopCapture();
        PreferenceStore.AtomicWrite(Path.Combine(Store.Root, "Logs", "last-session.log"), Diagnostics.Log.Text);
    }
}
