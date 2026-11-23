using System.Globalization;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace S4Bridge.Core;

// This opt-in diagnostic consumes the pinned worker's existing gate verdict.
// It never authorizes a motor or substitutes for missing Djay feedback.
internal sealed class DiagnosticMotorReadiness
{
    private sealed record Wheel(int Deck, bool TtEnabled, bool? Loaded, bool? Playing,
        bool? Internal, bool? Thru, bool Authorized);
    private sealed record Snapshot(int Version, Guid LaunchId, int Pid, string Case,
        string State, bool ReceiverReady, bool CaseReady, Wheel? Right, bool Ready, string? Error);
    private static readonly JsonSerializerOptions Json = new() { PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower };
    private readonly string path;
    private Snapshot current;

    private DiagnosticMotorReadiness(string output, string diagnosticCase, Guid launch)
    {
        path = Path.ChangeExtension(output, ".readiness.json");
        current = new(1, launch, 0, diagnosticCase, "starting", false, false, null, false, null);
        using var file = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read);
        JsonSerializer.Serialize(file, current, Json);
        file.Flush(true);
    }

    internal static DiagnosticMotorReadiness? Prepare(LaunchSpec spec, Guid launch)
    {
        var index = Array.IndexOf(spec.Arguments, "--screen-motor-diagnostic-config");
        if (index < 0) return null;
        if (index + 1 == spec.Arguments.Length)
            throw new InvalidDataException("Diagnostic config argument is missing.");
        var config = spec.Arguments[index + 1];
        var diagnosticCase = ScreenMotorDiagnostics.Validate(config);
        using var document = JsonDocument.Parse(File.ReadAllText(config));
        var output = document.RootElement.GetProperty("capture_path").GetString()
            ?? throw new InvalidDataException("Diagnostic output is missing.");
        return new(output, diagnosticCase, launch);
    }

    internal void Bind(int pid) => Publish(current with { Pid = pid });

    internal void Accept(ManagedStatus status) => Publish(current with
    {
        State = status.State,
        ReceiverReady = status.State == "ready" && status.Readiness.MidiInput == true,
        Error = status.Error
    });

    internal void Stop() => Publish(current with { State = "stopped", ReceiverReady = false });

    internal void Accept(string line)
    {
        if (line == "S4_SCREEN_DIAG_READY " + current.Case)
        {
            Publish(current with { CaseReady = true });
            return;
        }
        if (!line.Contains("MOTOR_PLAYBACK_GATE", StringComparison.Ordinal)) return;
        try
        {
            var side = Field(line, "side");
            if (side != "right") return;
            var deck = int.Parse(Field(line, "deck"), NumberStyles.None, CultureInfo.InvariantCulture);
            if (deck is not (2 or 4)) throw new InvalidDataException("Invalid right motor deck.");
            var wheel = new Wheel(deck, Boolean(Field(line, "tt_enabled")),
                Flag(Field(line, "loaded")), Flag(Field(line, "playing")),
                Flag(Field(line, "internal")), Flag(Field(line, "thru")),
                Boolean(Field(line, "playback_gate_open")));
            Publish(current with { Right = wheel, Error = null });
        }
        catch (Exception error) when (error is InvalidDataException or FormatException or OverflowException)
        {
            Publish(current with { Right = null, Error = error.Message });
        }
    }

    private void Publish(Snapshot next)
    {
        next = next with
        {
            Ready = next.State == "ready" && next.ReceiverReady && next.CaseReady &&
                next.Right is { Deck: 2, TtEnabled: true, Loaded: true, Playing: true,
                    Internal: true, Thru: not true, Authorized: true }
        };
        if (next == current) return;
        PreferenceStore.AtomicWrite(path, JsonSerializer.Serialize(next, Json));
        current = next;
    }

    private static string Field(string line, string name)
    {
        var match = Regex.Match(line, @"(?:^|\s)" + Regex.Escape(name) + @"=(""[^""]*""|[^\s]+)",
            RegexOptions.CultureInvariant, TimeSpan.FromMilliseconds(100));
        if (!match.Success) throw new InvalidDataException("Motor gate is missing " + name + ".");
        return match.Groups[1].Value.Trim('"');
    }

    private static bool Boolean(string value) => value switch
    {
        "true" => true,
        "false" => false,
        _ => throw new InvalidDataException("Invalid motor gate boolean.")
    };

    private static bool? Flag(string value) => value switch
    {
        "Some(true)" => true,
        "Some(false)" => false,
        "None" => null,
        _ => throw new InvalidDataException("Invalid motor gate feedback flag.")
    };
}
