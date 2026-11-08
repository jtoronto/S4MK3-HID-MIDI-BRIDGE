using System.Globalization;
using System.Text.Json;

namespace S4Bridge.Diagnostics;

public sealed class MidiCapture
{
    private const string Prefix = "S4_MIDI_CAPTURE_STATUS ";
    private readonly object gate = new();
    private Guid? launch;
    private ulong markerId;
    private readonly HashSet<ulong> pendingMarkers = [];
    private bool enabled;
    public event Action? Changed;
    public bool EnabledForNextLaunch
    {
        get { lock (gate) return enabled; }
        set { lock (gate) enabled = value; Changed?.Invoke(); }
    }
    public string State { get; private set; } = "idle";
    public string Path { get; private set; } = "";
    public string Error { get; private set; } = "";
    public ulong RecordCount { get; private set; }
    public ulong DroppedCount { get; private set; }
    public ulong AttemptedCount { get; private set; }
    public ulong QueuedCount { get; private set; }
    public ulong UnwrittenCount { get; private set; }
    public ulong DroppedLimitCount { get; private set; }
    public ulong DroppedLimitQueuedCount { get; private set; }
    public ulong? MaxFileBytes { get; private set; }
    public bool Truncated { get; private set; }
    public bool HasFinalCounts { get; private set; }
    public ulong? LastMarkerId { get; private set; }
    public bool? LastMarkerAccepted { get; private set; }
    public ulong? LastMarkerSequence { get; private set; }
    public string LastMarkerElapsedUs { get; private set; } = "";
    public string MarkerError { get; private set; } = "";

    public string? PrepareLaunch(string root, string profile)
    {
        lock (gate)
        {
            if (!enabled) return null;
            if (profile != "full") throw new InvalidOperationException("MIDI capture requires the full controller profile.");
            var directory = System.IO.Path.GetFullPath(System.IO.Path.Combine(root, "Diagnostics"));
            Directory.CreateDirectory(directory);
            Path = System.IO.Path.Combine(directory, $"midi-{DateTime.UtcNow:yyyyMMdd-HHmmss}-{Guid.NewGuid():N}.jsonl");
            enabled = false;
            launch = null;
            pendingMarkers.Clear();
            markerId = 0;
            State = "pending";
            Error = MarkerError = "";
            RecordCount = DroppedCount = AttemptedCount = QueuedCount = UnwrittenCount = 0;
            DroppedLimitCount = DroppedLimitQueuedCount = 0;
            MaxFileBytes = null;
            Truncated = HasFinalCounts = false;
            LastMarkerId = null;
            LastMarkerAccepted = null;
            LastMarkerSequence = null;
            LastMarkerElapsedUs = "";
        }
        Changed?.Invoke();
        return Path;
    }

    public void Launched(Guid id)
    {
        lock (gate)
        {
            if (State != "pending") return;
            launch = id;
            State = "starting";
        }
        Changed?.Invoke();
    }

    public void LaunchFailed(Exception error)
    {
        lock (gate)
        {
            if (State != "pending") return;
            State = "failed";
            Error = "MIDI capture launch failed: " + error.Message;
        }
        Changed?.Invoke();
    }

    public ulong NextMarker()
    {
        ulong id;
        lock (gate)
        {
            if (State != "recording") throw new InvalidOperationException("MIDI capture is not recording.");
            id = checked(++markerId);
            pendingMarkers.Add(id);
            MarkerError = "";
        }
        Changed?.Invoke();
        return id;
    }

    public void MarkerSendFailed(ulong id, Exception error)
    {
        lock (gate)
        {
            pendingMarkers.Remove(id);
            MarkerError = "MIDI capture marker send failed: " + error.Message;
        }
        Changed?.Invoke();
    }

    public void Accept(Guid id, string line)
    {
        if (!line.StartsWith(Prefix, StringComparison.Ordinal)) return;
        lock (gate)
        {
            if (launch != id) return;
            try
            {
                using var document = JsonDocument.Parse(line[Prefix.Length..]);
                var status = document.RootElement;
                if (status.GetProperty("version").GetInt32() != 1) throw new InvalidDataException("Unsupported capture status version.");
                var state = status.GetProperty("state").GetString();
                switch (state)
                {
                    case "started":
                        if (State != "starting" || status.GetProperty("path").GetString() != Path)
                            throw new InvalidDataException("Unexpected capture start or path.");
                        MaxFileBytes = status.TryGetProperty("max_file_bytes", out var maxFileBytes)
                            ? maxFileBytes.GetUInt64() : null;
                        State = "recording";
                        break;
                    case "limited":
                        if (State != "recording" ||
                            (status.TryGetProperty("path", out var limitedPath) && limitedPath.GetString() != Path))
                            throw new InvalidDataException("Unexpected capture limit or path.");
                        Truncated = true;
                        State = "limited";
                        break;
                    case "marker":
                        if (State is not ("recording" or "limited")) throw new InvalidDataException("Marker outside capture.");
                        var request = ulong.Parse(status.GetProperty("request_id").GetString()!, CultureInfo.InvariantCulture);
                        if (!pendingMarkers.Contains(request)) throw new InvalidDataException("Unknown marker receipt.");
                        var accepted = status.GetProperty("accepted").GetBoolean();
                        var sequence = status.GetProperty("sequence");
                        ulong? markerSequence = sequence.ValueKind == JsonValueKind.Null ? null : sequence.GetUInt64();
                        var elapsed = status.GetProperty("elapsed_us");
                        if (elapsed.ValueKind is not (JsonValueKind.Null or JsonValueKind.Number))
                            throw new InvalidDataException("Invalid marker elapsed time.");
                        var markerError = !accepted
                            ? "MIDI capture marker rejected: " + status.GetProperty("reason").GetString() : "";
                        pendingMarkers.Remove(request);
                        LastMarkerId = request;
                        LastMarkerAccepted = accepted;
                        LastMarkerSequence = markerSequence;
                        LastMarkerElapsedUs = elapsed.ValueKind == JsonValueKind.Null ? "" : elapsed.GetRawText();
                        MarkerError = markerError;
                        break;
                    case "finished":
                    case "failed":
                        if (State is not ("recording" or "limited") || status.GetProperty("path").GetString() != Path)
                            throw new InvalidDataException("Unexpected capture completion or path.");
                        var counts = status.GetProperty("counts");
                        var attempted = counts.GetProperty("attempted").GetUInt64();
                        var queued = counts.GetProperty("queued").GetUInt64();
                        var written = counts.GetProperty("written").GetUInt64();
                        var droppedLimit = counts.TryGetProperty("dropped_limit", out var limit)
                            ? limit.GetUInt64() : 0;
                        var droppedLimitQueued = counts.TryGetProperty("dropped_limit_queued", out var limitQueued)
                            ? limitQueued.GetUInt64() : 0;
                        var dropped = checked(counts.GetProperty("dropped_full").GetUInt64()
                            + counts.GetProperty("dropped_oversized").GetUInt64()
                            + counts.GetProperty("dropped_disconnected").GetUInt64() + droppedLimit);
                        var unwritten = counts.GetProperty("unwritten").GetUInt64();
                        if (droppedLimitQueued > droppedLimit || droppedLimitQueued > unwritten)
                            throw new InvalidDataException("Invalid queued limit-drop count.");
                        _ = checked(dropped - droppedLimitQueued + unwritten);
                        var truncated = counts.TryGetProperty("truncated", out var truncatedValue)
                            && truncatedValue.GetBoolean();
                        var failure = state == "failed" ? status.GetProperty("error").GetString() : null;
                        AttemptedCount = attempted;
                        QueuedCount = queued;
                        RecordCount = written;
                        DroppedCount = dropped;
                        DroppedLimitCount = droppedLimit;
                        DroppedLimitQueuedCount = droppedLimitQueued;
                        UnwrittenCount = unwritten;
                        Truncated |= truncated || droppedLimit != 0;
                        HasFinalCounts = true;
                        State = state == "failed" ? "failed" : Truncated ? "incomplete" : "finished";
                        if (state == "failed") AddError("MIDI capture failed: " + failure);
                        if (pendingMarkers.Count != 0) MarkerError = "MIDI capture ended before marker acknowledgement.";
                        pendingMarkers.Clear();
                        break;
                    default: throw new InvalidDataException("Unknown capture status.");
                }
            }
            catch (Exception ex) when (ex is JsonException or InvalidDataException or KeyNotFoundException
                or InvalidOperationException or FormatException or OverflowException or ArgumentNullException)
            {
                AddError("Invalid MIDI capture status: " + ex.Message);
            }
        }
        Changed?.Invoke();
    }

    public void Exited(Guid id)
    {
        lock (gate)
        {
            if (launch != id) return;
            if (State is "starting" or "recording" or "limited")
            {
                State = "incomplete";
                AddError("Bridge exited without a final MIDI capture status; capture may be incomplete.");
            }
            if (pendingMarkers.Count != 0) MarkerError = "Bridge exited before marker acknowledgement.";
            pendingMarkers.Clear();
            launch = null;
        }
        Changed?.Invoke();
    }

    private void AddError(string message) =>
        Error = string.IsNullOrEmpty(Error) ? message : Error + "\n" + message;
}
