using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Runtime.Serialization;
using System.Runtime.Serialization.Json;
using System.Text;
using System.Threading;

public sealed partial class WriterSession : IDisposable
{
    [DataContract]
    private sealed class Message
    {
        [DataMember(Name = "kind")] public string Kind { get; set; }
        [DataMember(Name = "pid")] public int? Pid { get; set; }
        [DataMember(Name = "reason")] public string Reason { get; set; }
        [DataMember(Name = "hook_count")] public int? HookCount { get; set; }
        [DataMember(Name = "events")] public AgentEvent[] Events { get; set; }
        [DataMember(Name = "last_seq")] public long? LastSeq { get; set; }
        [DataMember(Name = "dropped")] public long? Dropped { get; set; }
        [DataMember(Name = "errors")] public long? Errors { get; set; }
        [DataMember(Name = "outstanding_calls")] public long? OutstandingCalls { get; set; }
    }

    [DataContract]
    private sealed class AgentEvent
    {
        [DataMember(Name = "kind")] public string Kind { get; set; }
    }

    private sealed class InputCommand
    {
        public Action Action;
        public bool Done;
    }

    private readonly string injectorPath;
    private readonly string agentPath;
    private readonly int targetPid;
    private readonly object gate = new object();
    private readonly Queue<InputCommand> inputCommands = new Queue<InputCommand>();
    private static readonly TimeSpan CleanupTimeout = TimeSpan.FromSeconds(5);
    private Process process;
    private Thread launcherThread;
    private Thread stdoutThread;
    private Thread stderrThread;
    private Exception failure;
    private Exception drainFailure;
    private bool started;
    private bool abandoned;
    private bool ready;
    private bool stopping;
    private bool stopped;
    private bool completed;

    public string StdoutPath { get; private set; }
    public string StderrPath { get; private set; }

    public WriterSession(string injectorPath, int targetPid, string agentPath, string evidenceDirectory)
    {
        if (String.IsNullOrEmpty(injectorPath)) throw new ArgumentException("Injector path is required.", "injectorPath");
        if (String.IsNullOrEmpty(agentPath)) throw new ArgumentException("Agent path is required.", "agentPath");
        if (targetPid <= 0) throw new ArgumentOutOfRangeException("targetPid");
        if (String.IsNullOrEmpty(evidenceDirectory)) throw new ArgumentException("Evidence directory is required.", "evidenceDirectory");
        this.injectorPath = injectorPath;
        this.targetPid = targetPid;
        this.agentPath = agentPath;
        Directory.CreateDirectory(evidenceDirectory);
        string prefix = Path.Combine(evidenceDirectory, "writer-" + Guid.NewGuid().ToString("N"));
        StdoutPath = prefix + ".stdout";
        StderrPath = prefix + ".stderr";
    }

    public void Start(TimeSpan timeout)
    {
        Stopwatch clock = StartClock(timeout);
        lock (gate)
        {
            if (started || abandoned) throw new InvalidOperationException("Session has already been started or disposed.");
            started = true;
        }
        // Create the evidence before launching, so even a launch failure has paths to inspect.
        using (new FileStream(StdoutPath, FileMode.CreateNew, FileAccess.Write, FileShare.Read)) { }
        using (new FileStream(StderrPath, FileMode.CreateNew, FileAccess.Write, FileShare.Read)) { }
        launcherThread = new Thread(Launch);
        launcherThread.IsBackground = true;
        launcherThread.Start();
        try
        {
            WaitFor(delegate { return ready; }, clock, timeout, "ready");
        }
        catch (Exception ex)
        {
            try { Abort(); }
            catch (Exception cleanup) { throw new AggregateException(ex, cleanup); }
            throw;
        }
    }

    public void ThrowIfFailed()
    {
        Exception error;
        bool needsCleanup;
        lock (gate)
        {
            error = failure;
            needsCleanup = !abandoned;
            if (error == null && (!started || abandoned))
                throw new InvalidOperationException("Writer session is not running.");
            if (error == null && !completed && process != null && process.HasExited)
                error = new InvalidOperationException("Injector exited before the session completed.");
        }
        if (error == null) return;
        if (needsCleanup)
        {
            try { Abort(); }
            catch (Exception cleanup) { throw new AggregateException(error, cleanup); }
        }
        throw new InvalidOperationException("Writer session failed; inspect its evidence files.", error);
    }

    public void Stop(TimeSpan timeout)
    {
        Stopwatch clock = StartClock(timeout);
        try
        {
            InputCommand stop;
            lock (gate)
            {
                if (failure != null) throw new InvalidOperationException("Writer session failed.", failure);
                if (!started || abandoned) throw new InvalidOperationException("Writer session is not running.");
                if (!ready || stopping || completed) throw new InvalidOperationException("Session cannot be stopped in its current state.");
                stopping = true;
                stop = Enqueue(delegate
                {
                    process.StandardInput.WriteLine("STOP");
                    process.StandardInput.Flush();
                });
            }
            WaitFor(delegate { return stop.Done; }, clock, timeout, "STOP write");
            WaitFor(delegate { return stopped; }, clock, timeout, "stopped acknowledgement");
            InputCommand close;
            lock (gate) { close = Enqueue(delegate { process.StandardInput.Close(); }); }
            WaitFor(delegate { return close.Done; }, clock, timeout, "stdin close");
            if (!process.WaitForExit(Remaining(clock, timeout)))
                throw new TimeoutException("Timed out waiting for injector exit.");
            Join(stdoutThread, clock, timeout);
            Join(stderrThread, clock, timeout);
            lock (gate)
            {
                if (failure != null) throw new InvalidOperationException("Writer session failed; inspect its evidence files.", failure);
                if (process.ExitCode != 0) throw new InvalidOperationException("Injector exited with code " + process.ExitCode + ".");
                completed = true;
            }
        }
        catch (Exception ex)
        {
            try { Abort(); }
            catch (Exception cleanup) { throw new AggregateException(ex, cleanup); }
            throw;
        }
    }

    public void Mark(string text)
    {
        if (text == null) throw new ArgumentNullException("text");
        if (text.IndexOfAny(new char[] { '\r', '\n' }) >= 0)
            throw new ArgumentException("Marker must be a single line.", "text");
        ThrowIfFailed();
        lock (gate)
        {
            if (failure != null) throw new InvalidOperationException("Writer session failed.", failure);
            if (!started || abandoned || (process != null && process.HasExited))
                throw new InvalidOperationException("Writer session is not running.");
            if (!ready || stopping || completed) throw new InvalidOperationException("Session cannot accept markers in its current state.");
            Enqueue(delegate
            {
                process.StandardInput.WriteLine("MARK " + text);
                process.StandardInput.Flush();
            });
        }
    }

    public void Dispose()
    {
        Abort();
    }

    private void WaitFor(Func<bool> condition, Stopwatch clock, TimeSpan timeout, string name)
    {
        lock (gate)
        {
            while (true)
            {
                if (failure != null) throw new InvalidOperationException("Writer session failed; inspect its evidence files.", failure);
                if (condition()) return;
                int remaining = Remaining(clock, timeout);
                if (remaining == 0) throw new TimeoutException("Timed out waiting for " + name + ".");
                Monitor.Wait(gate, remaining);
            }
        }
    }

    private static Stopwatch StartClock(TimeSpan timeout)
    {
        if (timeout <= TimeSpan.Zero || timeout.TotalMilliseconds > Int32.MaxValue)
            throw new ArgumentOutOfRangeException("timeout");
        return Stopwatch.StartNew();
    }

    private static int Remaining(Stopwatch clock, TimeSpan timeout)
    {
        double remaining = timeout.TotalMilliseconds - clock.Elapsed.TotalMilliseconds;
        return remaining <= 0 ? 0 : (int)Math.Ceiling(remaining);
    }

    private static void Join(Thread thread, Stopwatch clock, TimeSpan timeout)
    {
        if (!thread.Join(Remaining(clock, timeout))) throw new TimeoutException("Timed out draining injector output.");
    }

    private static string Quote(string value)
    {
        StringBuilder result = new StringBuilder("\"");
        int slashes = 0;
        foreach (char c in value)
        {
            if (c == '\\') { slashes++; continue; }
            if (c == '"') result.Append('\\', slashes * 2 + 1);
            else result.Append('\\', slashes);
            result.Append(c);
            slashes = 0;
        }
        result.Append('\\', slashes * 2);
        return result.Append('"').ToString();
    }
}
