using System.Threading.Channels;

namespace S4ScreenSupervisor.Tests;

// A byte stream, not a protocol mock: production framing and reply parsers run
// unchanged over independently fragmented data in both directions.
internal sealed class BytePipe : Stream
{
    private readonly Channel<byte[]> chunks = Channel.CreateUnbounded<byte[]>();
    private byte[] current = [];
    private int position;
    internal void End() => chunks.Writer.TryComplete();
    public override async ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken cancellationToken = default)
    {
        while (position == current.Length)
        {
            if (!await chunks.Reader.WaitToReadAsync(cancellationToken)) return 0;
            if (!chunks.Reader.TryRead(out var next)) continue;
            current = next;
            position = 0;
        }
        int count = Math.Min(buffer.Length, current.Length - position);
        current.AsMemory(position, count).CopyTo(buffer);
        position += count;
        return count;
    }
    public override ValueTask WriteAsync(ReadOnlyMemory<byte> buffer, CancellationToken cancellationToken = default)
    {
        if (!chunks.Writer.TryWrite(buffer.ToArray())) throw new IOException("closed test pipe");
        return ValueTask.CompletedTask;
    }
    public override Task FlushAsync(CancellationToken cancellationToken) => Task.CompletedTask;
    public override void Flush() { }
    public override bool CanRead => true;
    public override bool CanWrite => true;
    public override bool CanSeek => false;
    public override long Length => throw new NotSupportedException();
    public override long Position { get => throw new NotSupportedException(); set => throw new NotSupportedException(); }
    public override int Read(byte[] buffer, int offset, int count) => ReadAsync(buffer.AsMemory(offset, count)).AsTask().GetAwaiter().GetResult();
    public override void Write(byte[] buffer, int offset, int count) => WriteAsync(buffer.AsMemory(offset, count)).GetAwaiter().GetResult();
    public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
    public override void SetLength(long value) => throw new NotSupportedException();
    protected override void Dispose(bool disposing) { End(); base.Dispose(disposing); }
}

internal sealed class HeldWrite(BytePipe target, int after) : Stream
{
    private int writes;
    internal TaskCompletionSource Blocked { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource release = new(TaskCreationOptions.RunContinuationsAsynchronously);
    internal void Release() => release.TrySetResult();
    public override async ValueTask WriteAsync(ReadOnlyMemory<byte> bytes, CancellationToken cancel = default)
    {
        if (Interlocked.Increment(ref writes) > after) { Blocked.TrySetResult(); await release.Task; }
        await target.WriteAsync(bytes, cancel);
    }
    public override Task FlushAsync(CancellationToken cancel) => Task.CompletedTask;
    public override void Flush() { }
    public override bool CanRead => false;
    public override bool CanWrite => true;
    public override bool CanSeek => false;
    public override long Length => throw new NotSupportedException();
    public override long Position { get => throw new NotSupportedException(); set => throw new NotSupportedException(); }
    public override int Read(byte[] bytes, int offset, int count) => throw new NotSupportedException();
    public override void Write(byte[] bytes, int offset, int count) => WriteAsync(bytes.AsMemory(offset, count)).GetAwaiter().GetResult();
    public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
    public override void SetLength(long value) => throw new NotSupportedException();
}

internal sealed class ManualTime : TimeProvider
{
    private readonly Channel<Timer> created = Channel.CreateUnbounded<Timer>();
    public override ITimer CreateTimer(TimerCallback callback, object? state, TimeSpan dueTime, TimeSpan period)
    {
        var timer = new Timer(callback, state, dueTime);
        if (!created.Writer.TryWrite(timer)) throw new InvalidOperationException("timer queue closed");
        return timer;
    }
    internal async Task Expire(TimeSpan duration)
    {
        while (true)
        {
            var timer = await created.Reader.ReadAsync().AsTask().WaitAsync(TimeSpan.FromSeconds(5));
            if (timer.Disposed || timer.Due != duration) continue;
            timer.Fire();
            return;
        }
    }
    private sealed class Timer(TimerCallback callback, object? state, TimeSpan due) : ITimer
    {
        internal TimeSpan Due { get; private set; } = due;
        internal bool Disposed { get; private set; }
        internal void Fire() { if (!Disposed) callback(state); }
        public bool Change(TimeSpan dueTime, TimeSpan period) { Due = dueTime; return !Disposed; }
        public void Dispose() => Disposed = true;
        public ValueTask DisposeAsync() { Dispose(); return ValueTask.CompletedTask; }
    }
}

internal sealed class MemoryWorker : IWorker
{
    internal BytePipe Requests { get; } = new();
    internal BytePipe Responses { get; } = new();
    internal BytePipe Logs { get; } = new();
    internal HeldWrite? Hold { get; set; }
    public Stream Input => (Stream?)Hold ?? Requests;
    public Stream Output => Responses;
    public Stream Error => Logs;
    private readonly TaskCompletionSource<int> exit = new(TaskCreationOptions.RunContinuationsAsynchronously);
    public Task<int> Exited => exit.Task;
    internal bool Terminated { get; private set; }
    internal void Exit(int code) { Responses.End(); Logs.End(); exit.TrySetResult(code); }
    public void Terminate() { Terminated = true; Exit(125); Hold?.Release(); }
    public void Dispose() { Terminate(); Requests.Dispose(); Responses.Dispose(); Logs.Dispose(); }
}
