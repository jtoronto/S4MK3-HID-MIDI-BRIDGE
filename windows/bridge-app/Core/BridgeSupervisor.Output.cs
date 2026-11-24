using System.Diagnostics;
using System.Text;
using S4Bridge.Diagnostics;

namespace S4Bridge.Core;

public sealed partial class BridgeSupervisor
{
    private async Task Drain(StreamReader reader, Process child, Guid id, bool stdout)
    {
        var buffer = new char[4096];
        var line = new StringBuilder();
        var discarded = false;
        try
        {
            while (await reader.ReadAsync(buffer).ConfigureAwait(false) is var count && count > 0)
            {
                for (var i = 0; i < count; i++)
                {
                    var c = buffer[i];
                    if (c == '\n')
                    {
                        var text = discarded ? "[oversized output line rejected]" :
                            line.ToString().TrimEnd('\r');
                        if (stdout && text == "S4_SCREEN_DIAG_PREPARED 1")
                        {
                            lock (gate) { if (Current(child, id)) prepared?.TrySetResult(); }
                        }
                        Ingest(child, id, text, stdout);
                        line.Clear(); discarded = false;
                    }
                    else if (!discarded)
                    {
                        if (line.Length == 65536) { line.Clear(); discarded = true; }
                        else line.Append(c);
                    }
                }
            }
            if (line.Length > 0) Ingest(child, id, line.ToString(), stdout);
            if (stdout)
            {
                lock (gate)
                {
                    if (Current(child, id))
                        prepared?.TrySetException(new IOException("Bridge stdout closed before diagnostic preparation."));
                }
            }
        }
        catch (Exception ex) when (ex is IOException or ObjectDisposedException)
        {
            lock (gate)
            {
                if (!Current(child, id)) return;
                Error = $"Process output read failed: {ex.Message}";
                State = "failed";
                if (stdout) prepared?.TrySetException(new IOException(Error, ex));
            }
            Changed?.Invoke();
        }
    }
    private void Ingest(Process child, Guid id, string text, bool stdout)
    {
        bool changed = false;
        lock (gate)
        {
            if (!Current(child, id)) return;
            if (stdout && text.StartsWith(ManagedStatus.Prefix, StringComparison.Ordinal))
            {
                try
                {
                    var status = ManagedStatus.Parse(text);
                    if (!launch!.Accept(status))
                        text = "Rejected stale/mismatched lifecycle status: " + text;
                    else
                    {
                        State = launch.State;
                        motorReadiness?.Accept(status);
                        if (launch.Status?.Error is { } failure) Error = failure;
                        changed = true;
                    }
                }
                catch (Exception ex) when (ex is System.Text.Json.JsonException or InvalidDataException
                         or KeyNotFoundException or InvalidOperationException or FormatException)
                { text = $"Rejected invalid lifecycle status ({ex.Message}): {text}"; }
            }
            motorReadiness?.Accept(text);
            if (stdout) Protocol?.Invoke(id, text);
            else StderrLine?.Invoke(id, text);
            // The live platter panel and diagnostic capture consume Protocol at every level.
            if (!stdout || ActiveSpec?.Run.LogLevel == "debug" ||
                !text.StartsWith(PlatterSnapshot.Prefix, StringComparison.Ordinal))
                Output?.Invoke(id, text);
        }
        if (changed) Changed?.Invoke();
    }
}
