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
    private readonly bool trialLauncher;
    public BridgeMode SelectedMode => trialLauncher ? BridgeMode.NiBridge :
        CanUseLegacyScreenControls ? BridgeMode.DirectHid : Store.Editor.Run.Mode;
    public bool CanChangeMode => !trialLauncher && !CanUseLegacyScreenControls && !Operating &&
        !Supervisor.HasOwnedResources && !Screens.Busy && !Diagnostics.Capturing && FieldsValid;
    public bool CanUseLegacyScreenControls => !trialLauncher &&
        Environment.GetEnvironmentVariable(ScreenMotorDiagnostics.EnvironmentVariable) is not null;
    public string ScreenStatus => CanUseLegacyScreenControls ? Screens.State :
        SelectedMode != BridgeMode.NiBridge ? "unavailable in Direct HID" :
        Screens.State == "ready" ? "not running" : Screens.State;
    public Preferences? LaunchedPreferences { get; private set; }
    public string Message { get; private set; } = "";
    public string Error { get; private set; } = "";
    public BloopDependencyProbeResult? BloopStatus { get; private set; }
    public bool CheckingBloop { get; private set; }
    private readonly Dictionary<string, string> fieldErrors = [];
    private readonly IBloopEndpointOwner? endpointOwner;
    private readonly Func<BloopDependencyProbeResult>? checkBloop;
    public bool Operating { get; private set; }
    public event Action? Changed;
    public bool FieldsValid => fieldErrors.Count == 0;
    public string FieldError => string.Join("\n", fieldErrors.Values);
    public bool CanStart => !Operating && !Supervisor.HasOwnedResources && !Store.RequiresReset && FieldsValid && File.Exists(Executable);
    public bool CanStop => !Operating && Supervisor.HasOwnedResources && Supervisor.State != "stopping";
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
    public HostController(string resources, string root, IBloopEndpointOwner? endpointOwner = null,
        bool experimentalNiDecks = false, Func<BloopDependencyProbeResult>? checkBloop = null)
    {
        this.endpointOwner = endpointOwner;
        this.checkBloop = checkBloop;
        trialLauncher = experimentalNiDecks;
        Store = new(resources, root, trialLauncher ? BridgeMode.NiBridge : BridgeMode.DirectHid);
        Executable = Path.Combine(resources, "s4mk3-hid-midi-bridge.exe");
        Screens = new(Store, new NIServiceControl(resources));
        Exporter = new(Executable, Path.Combine(root, "Temp"));
        Error = Store.LoadError;
        if (!File.Exists(Executable))
            Error += "The bridge engine is missing. Start, controller check and mapping export are unavailable.";
        Supervisor.Launched += id =>
        {
            Diagnostics.BeginLaunch(id);
            MidiCapture.Launched(id);
            var spec = Supervisor.ActiveSpec;
            if (spec is { Diagnostic: false } && (spec.UsesNiBridge || spec.HasScreenControls))
                Screens.Started(id, spec.StartsScreens, spec.UsesNiBridge);
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
    public async Task RefreshBloopAsync()
    {
        if (checkBloop is null || CheckingBloop) return;
        CheckingBloop = true;
        Changed?.Invoke();
        try
        {
            var result = await Task.Run(checkBloop);
            BloopStatus = result;
            if (result.Detail is { } detail)
                Diagnostics.Log.Append("BLOOP_CHECK: " + detail);
        }
        finally { CheckingBloop = false; Changed?.Invoke(); }
    }
    public void SelectMode(BridgeMode mode)
    {
        if (!Enum.IsDefined(mode)) throw new ArgumentOutOfRangeException(nameof(mode));
        if (mode == SelectedMode) return;
        if (!CanChangeMode) throw new InvalidOperationException("Stop the bridge before changing mode.");
        Store.Editor.Run.Mode = mode;
        if (mode == BridgeMode.NiBridge) MidiCapture.EnabledForNextLaunch = false;
        Edited();
    }

    public async Task StartAsync()
    {
        if (!FieldsValid) throw new InvalidDataException(FieldError);
        if (!CanStart) throw new InvalidOperationException("Bridge unavailable, busy, already running, or preferences require recovery.");
        Operating = true; Changed?.Invoke();
        try
        {
            Store.Save();
            var niBridge = SelectedMode == BridgeMode.NiBridge;
            LaunchedPreferences = Store.Saved.Copy();
            if (niBridge && (MidiCapture.EnabledForNextLaunch || Diagnostics.Capturing))
                throw new InvalidOperationException("NI Bridge cannot run with diagnostic capture.");
            var capturePath = MidiCapture.PrepareLaunch(Store.Root, Store.Saved.Run.Profile);
            try
            {
                var spec = trialLauncher
                    ? LaunchSpec.Experimental(Executable, Store, false, capturePath,
                        Environment.GetEnvironmentVariable(ScreenMotorDiagnostics.EnvironmentVariable))
                    : LaunchSpec.Runtime(Executable, Store, false, capturePath);
                LaunchedPreferences = LaunchedPreferences with { Run = spec.Run };
                Diagnostics.Log.Append("LAUNCH: " + spec.Preview);
                if (spec.Run.Profile == "probe") Supervisor.Start(spec);
                else await Supervisor.StartAsync(spec, endpointOwner
                    ?? throw new InvalidOperationException("BLOOP endpoint management is unavailable."));
            }
            catch (Exception ex) { MidiCapture.LaunchFailed(ex); throw; }
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
        if (Supervisor.HasOwnedResources)
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
        if (Operating || Supervisor.HasOwnedResources) throw new InvalidOperationException("Stop the bridge before checking controllers.");
        Supervisor.Start(new(Executable, ["--list"], Store.Editor.Run with { }, true));
    }
    public void Report(Exception error)
    {
        Error = error.Message;
        Diagnostics.Log.Append("APP_ERROR: " + error.Message);
        Changed?.Invoke();
    }
    public void Edited() { Message = "Unsaved changes"; Changed?.Invoke(); }
    public void Reset()
    {
        var mode = Store.Editor.Run.Mode;
        Store.Reset();
        if (Supervisor.HasOwnedResources) Store.Editor.Run.Mode = mode;
        Error = ""; Message = "Defaults restored. Save to keep them."; Changed?.Invoke();
    }
    public async Task QuitAsync()
    {
        if (Operating || Screens.Busy) throw new InvalidOperationException("Wait for the current operation before quitting.");
        await Supervisor.StopAsync();
        if (Supervisor.HasOwnedResources) throw new IOException("Bridge resources still owned. Quit cancelled.");
        if (Store.Saved.Screen.RestorePending) Screens.RequireOwnedRecovery();
        Diagnostics.StopCapture();
        PreferenceStore.AtomicWrite(Path.Combine(Store.Root, "Logs", "last-session.log"), Diagnostics.Log.Text);
    }
}
