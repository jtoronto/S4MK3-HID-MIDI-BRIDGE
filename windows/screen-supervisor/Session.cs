using System.Text;
using System.Threading.Channels;

namespace S4ScreenSupervisor;

internal interface IWorker : IDisposable
{
    Stream Input { get; }
    Stream Output { get; }
    Stream Error { get; }
    Task<int> Exited { get; }
    void Terminate();
}

internal sealed record Budgets
{
    internal TimeSpan Connect { get; init; } = TimeSpan.FromSeconds(8);
    internal TimeSpan Send { get; init; } = TimeSpan.FromSeconds(3);
    internal TimeSpan Stop { get; init; } = TimeSpan.FromMilliseconds(500);
    internal TimeSpan Reap { get; init; } = TimeSpan.FromSeconds(2);
}

internal sealed class Session(IWorker worker, Stream input, Stream output, Task parentLost,
    TimeProvider? time = null, Budgets? limits = null)
{
    private readonly TimeProvider clock = time ?? TimeProvider.System;
    private readonly Budgets budget = limits ?? new();
    private readonly Replies replies = new();
    private readonly TaskCompletionSource stop = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly Channel<(byte[] Bytes, Task Ack)> frames = Channel.CreateBounded<(byte[], Task)>(1);
    private readonly CancellationTokenSource lifetime = new();
    private readonly List<Task> pumps = [];
    private sealed class StopRequested : Exception;

    private void Fail(string reason) => replies.Failure.TrySetResult(reason);

    private Task Pump(Func<Task> action) => Task.Run(async () =>
    {
        try { await action(); }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (Exception e) when (e is IOException or InvalidDataException or ObjectDisposedException or InvalidOperationException or TimeoutException)
        { if (!lifetime.IsCancellationRequested) Fail(e.Message); }
    });

    private Task Write(Stream stream, ReadOnlyMemory<byte> bytes) => Task.Run(async () =>
    {
        await stream.WriteAsync(bytes);
        await stream.FlushAsync();
    });

    private async Task Guard(Task task, TimeSpan deadline, bool stopping = false)
    {
        var candidates = stopping
            ? new[] { task, replies.Failure.Task, parentLost }
            : new[] { task, replies.Failure.Task, parentLost, stop.Task, worker.Exited };
        await Task.WhenAny(candidates).WaitAsync(deadline, clock);
        if (replies.Failure.Task.IsCompleted) throw new IOException(await replies.Failure.Task);
        if (parentLost.IsCompleted) throw new IOException("parent exited");
        if (!stopping && stop.Task.IsCompleted) throw new StopRequested();
        if (task.IsCompleted) { await task; return; }
        throw new IOException("worker exited before acknowledgement");
    }

    private async Task Emit(string line)
    {
        // Bound bridge stdout as well as worker stdin. An unread response cannot
        // keep the owned worker alive indefinitely.
        await Guard(Write(output, Encoding.ASCII.GetBytes(line + "\n")), budget.Send, stopping: true);
    }

    private async Task ReadParent()
    {
        while (true)
        {
            var record = await Wire.ReadRecord(input, clock, budget.Send, lifetime.Token);
            if (record is null) { Fail("parent stdin EOF"); return; }
            if (record[0] == 2) { stop.TrySetResult(); return; }
            var ack = replies.Expect("FRAME_OK");
            if (!frames.Writer.TryWrite((record, ack)))
                throw new InvalidDataException("more than one frame in flight");
        }
    }

    internal async Task<int> Run()
    {
        string? failure = null;
        var ready = replies.Expect("READY");
        pumps.Add(Pump(ReadParent));
        pumps.Add(Pump(async () =>
        {
            await Wire.ReadReplies(worker.Output, replies.Receive, lifetime.Token);
            if (!replies.Stopped) Fail("worker stdout EOF");
        }));
        pumps.Add(Pump(async () =>
        {
            var bytes = new byte[4096];
            // Never mirror logs to a potentially blocked console or accumulate them.
            while (await worker.Error.ReadAsync(bytes, lifetime.Token) != 0) { }
        }));
        try
        {
            async Task Connect()
            {
                await Write(worker.Input, new byte[] { 0xA5 });
                await ready;
            }
            await Guard(Connect(), budget.Connect);
            await Emit("READY");
            while (true)
            {
                var next = frames.Reader.ReadAsync(lifetime.Token).AsTask();
                await Guard(next, Timeout.InfiniteTimeSpan);
                var frame = await next;
                async Task Send()
                {
                    await Write(worker.Input, frame.Bytes);
                    await frame.Ack;
                }
                await Guard(Send(), budget.Send);
                await Emit("FRAME_OK");
            }
        }
        catch (StopRequested)
        {
            // Never queue STOP behind a blocked send, and never reuse its ACK.
            if (!replies.Idle) failure = "STOP interrupted an in-flight operation";
            else
            {
                try
                {
                    var stopped = replies.Expect("STOPPED");
                    async Task Stop()
                    {
                        await Write(worker.Input, Wire.Stop);
                        await stopped;
                        if (await worker.Exited != 0) throw new IOException("worker failed after STOPPED");
                    }
                    await Guard(Stop(), budget.Stop, stopping: true);
                }
                catch (Exception e) when (e is IOException or TimeoutException) { failure = e.Message; }
            }
        }
        catch (Exception e) when (e is IOException or InvalidDataException or TimeoutException or InvalidOperationException)
        { failure = e is TimeoutException ? "screen operation deadline" : e.Message; }
        finally
        {
            // Terminate even after a cooperative worker exit: descendants must not
            // retain pipe ends or an owned job. Retain the process handle to reap.
            try { worker.Terminate(); }
            catch (Exception e) { failure ??= "job termination failed: " + e.Message; }
            try { await worker.Exited.WaitAsync(budget.Reap, clock); }
            catch (Exception e) { failure ??= "worker reap failed: " + e.Message; }
            lifetime.Cancel();
            try { await Task.WhenAll(pumps).WaitAsync(budget.Reap, clock); }
            catch (Exception e) { failure ??= "pipe drain incomplete: " + e.Message; }
            lifetime.Dispose();
        }
        if (failure is not null)
        {
            await Program.WriteError(output, failure, clock);
            return 1;
        }
        try { await Emit("STOPPED"); return 0; }
        catch (Exception e) when (e is IOException or TimeoutException)
        { await Program.WriteError(output, e.Message, clock); return 1; }
    }
}
