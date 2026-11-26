using System.Diagnostics;
using S4Bridge.Models;

namespace S4Bridge.Core;

public sealed record LaunchSpec(string Executable, string[] Arguments,
    RunPreferences Run, bool Diagnostic = false, string MidiBackend = BloopEndpoint.Backend)
{
    public bool UsesNiBridge => !Diagnostic && Run.Mode == BridgeMode.NiBridge;
    public bool StartsScreens => UsesNiBridge || Arguments.Contains("--screens", StringComparer.Ordinal);
    public bool HasScreenControls => Arguments.Contains("--screen-control-stdin", StringComparer.Ordinal);
    public const string ExperimentalEnvironmentVariable = "S4_BRIDGE_EXPERIMENTAL_NI_DECKS";
    public static bool ExperimentalEnabled(string? value) => value == "1";
    public static LaunchSpec Experimental(string executable, PreferenceStore store, bool preview,
        string? midiCapturePath, string? screenMotorConfig)
    {
        if (midiCapturePath is not null || screenMotorConfig is not null)
            throw new InvalidOperationException("NI Bridge cannot run with MIDI capture or screen/motor diagnostics.");
        var run = (preview ? store.Editor.Run : store.Saved.Run) with
        { Mode = BridgeMode.NiBridge, Profile = "full", LimitDuration = false };
        var args = new List<string> { "--ni-decks", "--midi", "full", "--midi-backend", BloopEndpoint.Backend,
            "--managed-stdin", "--until-stopped" };
        foreach (var config in new[] { "led", "jog" })
            args.AddRange([$"--{config}-config", Path.Combine(store.Root, $"{config}-config.json")]);
        if (run.RawReports) args.Add("--raw");
        return new(executable, [.. args], run);
    }
    public static LaunchSpec Runtime(string executable, PreferenceStore store, bool screens, string? midiCapturePath = null) =>
        Runtime(executable, store, screens, midiCapturePath,
            Environment.GetEnvironmentVariable(ScreenMotorDiagnostics.EnvironmentVariable));
    public static LaunchSpec Runtime(string executable, PreferenceStore store, bool screens, string? midiCapturePath,
        string? screenMotorConfig) =>
        store.Saved.Run.Mode == BridgeMode.NiBridge && screenMotorConfig is null
            ? Experimental(executable, store, false, midiCapturePath, screenMotorConfig)
            : Build(executable, store, store.Saved.Run, screens, midiCapturePath, screenMotorConfig);
    public static LaunchSpec EditorPreview(string executable, PreferenceStore store) =>
        EditorPreview(executable, store, Environment.GetEnvironmentVariable(ScreenMotorDiagnostics.EnvironmentVariable));
    public static LaunchSpec EditorPreview(string executable, PreferenceStore store, string? screenMotorConfig) =>
        store.Editor.Run.Mode == BridgeMode.NiBridge && screenMotorConfig is null
            ? Experimental(executable, store, true, null, screenMotorConfig)
            : Build(executable, store, store.Editor.Run, false, null, screenMotorConfig, preview: true);
    private static LaunchSpec Build(string executable, PreferenceStore store, RunPreferences preferences, bool screens,
        string? midiCapturePath = null, string? screenMotorConfig = null, bool preview = false)
    {
        var run = preferences with { };
        var args = new List<string> { "--midi-backend", BloopEndpoint.Backend, "--managed-stdin" };
        if (screenMotorConfig is not null)
        {
            var diagnosticCase = ScreenMotorDiagnostics.Validate(screenMotorConfig, preview);
            if (!preview)
            {
                if (run.Profile != "full") throw new InvalidOperationException("Screen/motor diagnostics require the full controller profile.");
                if (midiCapturePath is not null) throw new InvalidOperationException("Screen/motor diagnostics cannot run with MIDI capture.");
            }
            screens = diagnosticCase != "off";
            run = run with { Mode = BridgeMode.DirectHid, RawReports = false, LogLevel = "info", LimitDuration = false };
            args.AddRange(["--screen-motor-diagnostic-config", screenMotorConfig,
                "--prepare-screen-motor-diagnostics"]);
        }
        if (midiCapturePath is not null)
        {
            if (!Path.IsPathFullyQualified(midiCapturePath)) throw new InvalidDataException("MIDI capture path must be absolute.");
            if (run.Profile != "full") throw new InvalidOperationException("MIDI capture requires the full controller profile.");
            args.AddRange(["--midi-capture", midiCapturePath]);
        }
        if (run.Profile != "probe") args.AddRange(["--midi", run.Profile]);
        if (run.Profile == "full")
        {
            if (screens || screenMotorConfig is not null) args.Add("--screen-control-stdin");
            if (screens) args.Add("--screens");
            foreach (var config in new[] { "led", "jog", "tension" })
                args.AddRange([$"--{config}-config", Path.Combine(store.Root, $"{config}-config.json")]);
        }
        args.AddRange(run.LimitDuration ? ["--seconds",run.DurationSeconds.ToString(
            System.Globalization.CultureInfo.InvariantCulture)] : ["--until-stopped"]);
        if (run.RawReports) args.Add("--raw");
        return new(executable, [.. args], run);
    }
    public ProcessStartInfo StartInfo()
    {
        var info = new ProcessStartInfo(Executable)
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true
        };
        foreach (var arg in Arguments) info.ArgumentList.Add(arg);
        info.Environment["RUST_LOG"] = Run.LogLevel;
        info.Environment["NO_COLOR"] = "1";
        return info;
    }
    // Display only. Execution always uses ArgumentList without a shell.
    public string Preview => string.Join(" ", new[] { Executable }.Concat(Arguments)
        .Select(s => "\"" + s.Replace("\"", "\\\"", StringComparison.Ordinal) + "\""));
}
