using System;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;

public sealed class WriterTraceCommandResult
{
    public int ExitCode { get; set; }
    public string Stdout { get; set; }
    public string Stderr { get; set; }
}

public sealed class WriterTraceCommands
{
    public ProcessStartInfo Stop { get; set; }
    public ProcessStartInfo Cancel { get; set; }
}

// The timer operates without a PowerShell runspace or console input.
public sealed class WriterTraceDeadline : IDisposable
{
    private readonly WriterTraceCommands commands;
    private readonly Timer timer;
    private readonly ManualResetEventSlim finished = new ManualResetEventSlim();
    private int stopping;
    public bool DeadlineReached { get; private set; }
    public WriterTraceCommandResult Result { get; private set; }
    public WriterTraceCommandResult CancelResult { get; private set; }
    public Exception Failure { get; private set; }

    public WriterTraceDeadline(WriterTraceCommands commands, TimeSpan duration)
    {
        if (commands == null || commands.Stop == null || commands.Cancel == null)
            throw new ArgumentException("Owned stop and cancel commands are required.");
        if (duration <= TimeSpan.Zero || duration.TotalSeconds > 180)
            throw new ArgumentOutOfRangeException("duration");
        this.commands = commands;
        timer = new Timer(delegate { Finish(true); }, null, Timeout.InfiniteTimeSpan, Timeout.InfiniteTimeSpan);
        timer.Change(duration, Timeout.InfiniteTimeSpan);
    }

    public void Stop()
    {
        Finish(false);
        if (!finished.Wait(TimeSpan.FromSeconds(40)))
            throw new TimeoutException("Owned WPR cleanup did not finish.");
        if (Failure != null) throw new InvalidOperationException("Owned WPR stop failed.", Failure);
        if (Result == null || Result.ExitCode != 0)
            throw new InvalidOperationException("WPR stop failed; inspect stop/cancel results.");
    }

    private void Finish(bool deadline)
    {
        if (Interlocked.CompareExchange(ref stopping, 1, 0) != 0) return;
        DeadlineReached = deadline;
        timer.Change(Timeout.Infinite, Timeout.Infinite);
        try
        {
            try { Result = Run(commands.Stop, 15000); }
            catch (Exception error) { Failure = error; }
            if (Failure != null || Result.ExitCode != 0)
            {
                try { CancelResult = Run(commands.Cancel, 15000); }
                catch (Exception error)
                {
                    Failure = Failure == null ? error : new AggregateException(Failure, error);
                }
            }
        }
        finally { finished.Set(); }
    }

    public static WriterTraceCommandResult Run(ProcessStartInfo info, int timeoutMilliseconds)
    {
        info.UseShellExecute = false;
        info.CreateNoWindow = true;
        info.RedirectStandardOutput = true;
        info.RedirectStandardError = true;
        using (Process process = Process.Start(info))
        {
            if (process == null) throw new InvalidOperationException("Trace command did not start.");
            Task<string> stdout = process.StandardOutput.ReadToEndAsync();
            Task<string> stderr = process.StandardError.ReadToEndAsync();
            if (!process.WaitForExit(timeoutMilliseconds))
            {
                process.Kill();
                process.WaitForExit(1000);
                throw new TimeoutException("Owned trace command timed out: " + info.Arguments);
            }
            if (!Task.WaitAll(new Task[] { stdout, stderr }, 1000))
                throw new TimeoutException("Trace command output did not close.");
            return new WriterTraceCommandResult
            {
                ExitCode = process.ExitCode,
                Stdout = stdout.GetAwaiter().GetResult(),
                Stderr = stderr.GetAwaiter().GetResult()
            };
        }
    }

    public void Dispose()
    {
        try { Stop(); }
        finally { timer.Dispose(); }
    }
}
