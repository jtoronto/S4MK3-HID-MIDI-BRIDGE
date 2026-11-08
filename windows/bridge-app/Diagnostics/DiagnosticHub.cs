using System.Text.Json;
using S4Bridge.Models;

namespace S4Bridge.Diagnostics;

public sealed record DiagnosticPresentation(string Left, string Right);
public sealed class DiagnosticHub(TimeProvider? clock = null) : IDisposable
{
    private readonly object gate = new();
    private readonly TimeProvider time = clock ?? TimeProvider.System;
    private Guid? launch;
    private PlatterSnapshot? latest;
    private DateTimeOffset received;
    private string? receipt;
    private StreamWriter? capture;
    public BoundedLog Log { get; } = new();
    public string? CapturePath { get; private set; }
    public bool Capturing { get { lock (gate) return capture is not null; } }
    public string Error { get; private set; } = "";
    public DiagnosticPresentation Presentation { get; private set; } = new("Decks A / C | Stopped", "Decks B / D | Stopped");
    private static readonly string[] Events = [
        "MOTOR_JOG_BEND","MOTOR_JOG_POSITION","MOTOR_SCRATCH_HANDOFF","MOTOR_REGULATOR",
        "MOTOR_TT_SELECTION","MOTOR_PLAYBACK_GATE","MOTOR_TT_STATE","S4_PLATTER_STATUS"];
    private static readonly string[] StderrEvents = [
        "MOTOR_JOG_BEND","MOTOR_JOG_POSITION","MOTOR_SCRATCH_HANDOFF","MOTOR_REGULATOR",
        "MOTOR_TT_SELECTION","MOTOR_PLAYBACK_GATE","MOTOR_TT_STATE"];

    public void BeginLaunch(Guid id)
    {
        lock (gate)
        {
            launch = id; latest = null; receipt = null;
            WriteCapture($"# S4_MOTOR_CONFIG launchID={id} pending");
        }
    }
    public void Accept(Guid id, string line)
    {
        lock (gate)
        {
            if (id != launch) return;
            if (line.StartsWith(PlatterSnapshot.Prefix, StringComparison.Ordinal))
            {
                try { latest = PlatterSnapshot.Parse(line); received = time.GetUtcNow(); }
                catch (Exception ex) when (ex is JsonException or InvalidDataException or KeyNotFoundException
                    or InvalidOperationException or FormatException)
                { Log.Append("Invalid platter telemetry: " + ex.Message); }
            }
            if (line.StartsWith("S4_MOTOR_CONFIG ", StringComparison.Ordinal))
            {
                try
                {
                    using var doc = JsonDocument.Parse(line["S4_MOTOR_CONFIG ".Length..]);
                    if (doc.RootElement.GetProperty("jog").ValueKind != JsonValueKind.Object)
                        throw new InvalidDataException("Motor receipt requires jog object.");
                    receipt = line;
                    WriteCapture($"# {receipt} launchID={id}");
                }
                catch (Exception ex) when (ex is JsonException or InvalidDataException or KeyNotFoundException or InvalidOperationException)
                { Log.Append("Invalid motor receipt: " + ex.Message); }
            }
            if (Events.Any(e => line.Contains(e, StringComparison.Ordinal))) WriteCapture(line);
        }
    }
    public void AcceptStderr(Guid id, string line)
    {
        lock (gate)
        {
            if (id != launch) return;
            if (StderrEvents.Any(e => line.Contains(e, StringComparison.Ordinal))) WriteCapture(line);
        }
    }
    public bool Refresh(bool visible, bool running)
    {
        lock (gate)
        {
            if (!visible) return false;
            var stale = time.GetUtcNow() - received > TimeSpan.FromSeconds(0.5);
            var next = !running ? new DiagnosticPresentation("Decks A / C | Stopped", "Decks B / D | Stopped")
                : latest is null ? new("Decks A / C | Waiting", "Decks B / D | Waiting")
                : new(latest.Left.Present(stale), latest.Right.Present(stale));
            if (next == Presentation) return false;
            Presentation = next;
            return true;
        }
    }
    public void StartCapture(string root, RunPreferences activeOrEditor)
    {
        lock (gate)
        {
            if (capture is not null) throw new InvalidOperationException("Capture already running.");
            if (activeOrEditor.Profile != "full" || activeOrEditor.LogLevel != "debug")
                throw new InvalidOperationException("Select Full controller and Debug, then start or restart before capturing.");
            Directory.CreateDirectory(Path.Combine(root, "Diagnostics"));
            var path = Path.Combine(root, "Diagnostics", $"motor-{time.GetUtcNow():yyyyMMdd-HHmmss}-{Guid.NewGuid():N}.log");
            var writer = new StreamWriter(new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read));
            try
            {
                writer.WriteLine("# S4 MK3 Bridge motor diagnostic");
                writer.WriteLine($"# Started: {time.GetUtcNow():O}");
                writer.WriteLine($"# launchID={launch?.ToString() ?? "pending"} {receipt ?? "S4_MOTOR_CONFIG pending"}");
                writer.WriteLine("# Captured events: " + string.Join(", ", Events));
                writer.Flush();
            }
            catch { writer.Dispose(); throw; }
            capture = writer; CapturePath = path;
        }
    }
    public void StopCapture()
    {
        lock (gate)
        {
            var writer = capture;
            if (writer is null) return;
            capture = null;
            try { writer.WriteLine($"# Stopped: {time.GetUtcNow():O}"); writer.Flush(); }
            finally { writer.Dispose(); }
        }
    }
    private void WriteCapture(string line)
    {
        if (capture is null) return;
        try { capture.WriteLine(line); capture.Flush(); }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            Error = "Motor capture write failed: " + ex.Message;
            var writer = capture; capture = null;
            try { writer.Dispose(); }
            catch (IOException close) { Error += "; close failed: " + close.Message; }
            Log.Append(Error);
        }
    }
    public void Dispose() => StopCapture();
}
