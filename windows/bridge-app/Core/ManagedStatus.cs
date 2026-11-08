using System.Globalization;
using System.Text.Json;

namespace S4Bridge.Core;

public sealed record Readiness(bool Hid, bool? MidiInput, bool? MidiOutput,
    bool ControllerInitialized, string Feedback);
public sealed record ManagedStatus(string State, string Profile, string Backend,
    ulong? RequestId, Readiness Readiness, string? Error)
{
    public const string Prefix = "S4_BRIDGE_STATUS ";
    public static ManagedStatus Parse(string line)
    {
        if (!line.StartsWith(Prefix, StringComparison.Ordinal))
            throw new InvalidDataException("Not an aggregate status line.");
        using var doc = JsonDocument.Parse(line[Prefix.Length..]);
        var root = doc.RootElement;
        if (root.ValueKind != JsonValueKind.Object ||
           new[] { "version", "state", "profile", "backend", "request_id", "readiness", "error" }
               .Any(key => !root.TryGetProperty(key, out _)))
            throw new InvalidDataException("Aggregate status is missing required fields.");
        if (!root.GetProperty("version").TryGetInt32(out var version) || version != 1)
            throw new InvalidDataException("Unsupported status version.");
        var state = Text(root, "state", ["starting", "ready", "stopping", "failed"]);
        var profile = Text(root, "profile", ["full", "minimal", "probe"]);
        var backend = Text(root, "backend", ["loopbe", "core-midi"]);
        var request = root.GetProperty("request_id");
        ulong? id = request.ValueKind == JsonValueKind.Null ? null : ParseId(request.GetString()!);
        if (id is not null && state != "stopping")
            throw new InvalidDataException("Request ID is only valid for stopping.");
        var r = root.GetProperty("readiness");
        var ready = new Readiness(r.GetProperty("hid").GetBoolean(),
            NullableBool(r.GetProperty("midi_input")), NullableBool(r.GetProperty("midi_output")),
            r.GetProperty("controller_initialized").GetBoolean(),
            Text(r, "feedback", ["unknown", "received", "not_applicable"]));
        var errorNode = root.GetProperty("error");
        string? error = null;
        if (state == "failed")
        {
            var stage = errorNode.GetProperty("stage").GetString();
            var message = errorNode.GetProperty("message").GetString();
            if (string.IsNullOrWhiteSpace(stage) || string.IsNullOrWhiteSpace(message))
                throw new InvalidDataException("Failed status requires a causal error.");
            error = $"{stage}: {message}";
        }
        else if (errorNode.ValueKind != JsonValueKind.Null)
            throw new InvalidDataException("Nonfailed status has an error.");
        if ((profile != "full" && (ready.MidiInput is not null || ready.Feedback != "not_applicable"))
            || (profile == "probe" && ready.MidiOutput is not null)
            || (profile == "full" && (ready.MidiInput is null || ready.MidiOutput is null
                                      || ready.Feedback == "not_applicable"))
            || (profile == "minimal" && ready.MidiOutput is null))
            throw new InvalidDataException("Readiness does not match the profile.");
        if (state == "ready" && (!ready.Hid || !ready.ControllerInitialized
            || (profile != "probe" && ready.MidiOutput != true)
            || (profile == "full" && ready.MidiInput != true)))
            throw new InvalidDataException("Ready status lacks required milestones.");
        return new(state, profile, backend, id, ready, error);
    }
    public static ulong ParseId(string value)
    {
        if (string.IsNullOrEmpty(value) || !value.All(char.IsAsciiDigit)
            || !ulong.TryParse(value, NumberStyles.None, CultureInfo.InvariantCulture, out var id))
            throw new InvalidDataException("Invalid ASCII u64 request ID.");
        return id;
    }
    private static bool? NullableBool(JsonElement value) =>
        value.ValueKind == JsonValueKind.Null ? null : value.GetBoolean();
    private static string Text(JsonElement root, string name, string[] allowed)
    {
        var value = root.GetProperty(name).GetString();
        return value is not null && allowed.Contains(value, StringComparer.Ordinal) ? value
            : throw new InvalidDataException($"Invalid status {name}.");
    }
}

// The UUID and process identity are local, never accepted from child text.
public sealed class LaunchState(Guid id, int processId, string profile)
{
    public Guid Id { get; } = id;
    public int ProcessId { get; } = processId;
    public string Profile { get; } = profile;
    public string State { get; private set; } = "starting";
    public ulong? StopRequest { get; private set; }
    public ManagedStatus? Status { get; private set; }
    public bool StopUnconfirmed { get; private set; }
    public bool IsReady => State == "ready" && !StopUnconfirmed;
    public bool Accept(ManagedStatus status)
    {
        if (status.Profile != Profile || status.Backend != "loopbe" || StopUnconfirmed) return false;
        if (status.RequestId is { } request && StopRequest != request) return false;
        if (State == "stopping" && status.State == "stopping"
            && StopRequest is { } expected && status.RequestId != expected) return false;
        var allowed = State switch
        {
            "starting" => status.State is "starting" or "ready" or "stopping" or "failed",
            "ready" => status.State is "ready" or "stopping" or "failed",
            "stopping" => status.State is "stopping" or "failed",
            "failed" => false,
            _ => false
        };
        if (!allowed) return false;
        State = status.State;
        Status = status;
        return true;
    }
    public void RequestStop(ulong request)
    {
        StopRequest ??= request;
        if (State != "failed") State = "stopping";
    }
    public void Unconfirmed() { StopUnconfirmed = true; State = "failed"; }
}
