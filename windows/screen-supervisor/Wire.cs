using System.Buffers.Binary;
using System.Text;

namespace S4ScreenSupervisor;

internal static class Wire
{
    internal const int FrameLength = 307_200;
    internal static readonly byte[] Stop = [2, 0, 0, 0, 0];

    // Keep the original header and pixels: no re-encoding or protocol version.
    internal static async Task<byte[]?> ReadRecord(Stream input, TimeProvider clock,
        TimeSpan assemblyBudget, CancellationToken cancel)
    {
        var header = new byte[5];
        if (await input.ReadAsync(header.AsMemory(0, 1), cancel) == 0) return null;
        async Task<byte[]> Finish()
        {
            await input.ReadExactlyAsync(header.AsMemory(1), cancel);
            var length = BinaryPrimitives.ReadUInt32LittleEndian(header.AsSpan(1));
            if (!((header[0] == 1 && length == FrameLength) || (header[0] == 2 && length == 0)))
                throw new InvalidDataException("invalid record tag or length");
            var record = new byte[5 + (int)length];
            header.CopyTo(record, 0);
            await input.ReadExactlyAsync(record.AsMemory(5), cancel);
            return record;
        }
        return await Finish().WaitAsync(assemblyBudget, clock, cancel);
    }

    internal static async Task ReadReplies(Stream input, Action<string> receive, CancellationToken cancel)
    {
        var buffer = new byte[4096];
        var line = new StringBuilder();
        int count;
        while ((count = await input.ReadAsync(buffer, cancel)) != 0)
        {
            for (int i = 0; i < count; i++)
            {
                var c = buffer[i];
                if (c == '\n')
                {
                    var text = line.ToString();
                    if (text.EndsWith('\r')) text = text[..^1];
                    if (text.Contains('\r')) throw new InvalidDataException("embedded CR in worker response");
                    receive(text);
                    line.Clear();
                }
                else
                {
                    if ((c < 32 && c != '\r') || c > 126 || line.Length == 16_384)
                        throw new InvalidDataException("invalid or oversized worker response");
                    line.Append((char)c);
                }
            }
        }
        if (line.Length != 0) throw new EndOfStreamException("incomplete worker response");
    }
}

// Receipt, admission and completion use the same lock. ACKs are never queued for
// a later request, including multiple lines returned by one pipe read.
internal sealed class Replies
{
    private readonly object gate = new();
    private string? expected;
    private TaskCompletionSource? pending;
    private bool idle;
    private bool stopped;
    internal bool Stopped { get { lock (gate) return stopped; } }
    internal bool Idle { get { lock (gate) return idle; } }
    internal TaskCompletionSource<string> Failure { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);

    internal Task Expect(string reply)
    {
        lock (gate)
        {
            if (expected is not null || stopped || Failure.Task.IsCompleted)
                throw new InvalidDataException("request while worker is busy or terminal");
            if (reply != "READY" && !idle)
                throw new InvalidDataException("request before READY");
            idle = false;
            expected = reply;
            pending = new(TaskCreationOptions.RunContinuationsAsynchronously);
            return pending.Task;
        }
    }

    internal void Receive(string line)
    {
        lock (gate)
        {
            if (Failure.Task.IsCompleted) return;
            if (line.StartsWith("ERROR ", StringComparison.Ordinal) && line.Length > 6)
            {
                Failure.TrySetResult(line[6..]);
                return;
            }
            if (line != expected)
            {
                Failure.TrySetResult("unexpected worker response: " + line);
                return;
            }
            stopped = line == "STOPPED";
            idle = !stopped;
            expected = null;
            pending!.TrySetResult();
        }
    }
}
