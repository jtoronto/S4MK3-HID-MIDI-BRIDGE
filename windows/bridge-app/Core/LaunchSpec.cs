using System.Diagnostics;
using S4Bridge.Models;

namespace S4Bridge.Core;

public sealed record LaunchSpec(string Executable, string[] Arguments,
    RunPreferences Run, bool Diagnostic = false, string MidiBackend = BloopEndpoint.Backend)
{
    public static LaunchSpec Runtime(string executable, PreferenceStore store, bool screens, string? midiCapturePath = null) =>
        Build(executable, store, store.Saved.Run, screens, midiCapturePath);
    public static LaunchSpec EditorPreview(string executable, PreferenceStore store) =>
        Build(executable, store, store.Editor.Run, store.Editor.Screen.Enabled);
    private static LaunchSpec Build(string executable, PreferenceStore store, RunPreferences preferences, bool screens, string? midiCapturePath = null)
    {
        var run = preferences with { };
        var args = new List<string> { "--midi-backend", BloopEndpoint.Backend, "--managed-stdin" };
        if (midiCapturePath is not null)
        {
            if (!Path.IsPathFullyQualified(midiCapturePath)) throw new InvalidDataException("MIDI capture path must be absolute.");
            if (run.Profile != "full") throw new InvalidOperationException("MIDI capture requires the full controller profile.");
            args.AddRange(["--midi-capture", midiCapturePath]);
        }
        if (run.Profile != "probe") args.AddRange(["--midi", run.Profile]);
        if (run.Profile == "full")
        {
            args.Add("--screen-control-stdin");
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
