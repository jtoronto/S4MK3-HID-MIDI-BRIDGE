using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Runtime.Serialization;
using System.Runtime.Serialization.Json;
using System.Text;
using System.Threading;

public sealed partial class WriterSession
{
    private void Launch()
    {
        Process child = new Process();
        bool childStarted = false;
        try
        {
            child.StartInfo = new ProcessStartInfo
            {
                FileName = injectorPath,
                Arguments = "--interactive --runtime=qjs -p " + targetPid + " -s " + Quote(agentPath),
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };
            if (!child.Start()) throw new InvalidOperationException("Injector did not start.");
            childStarted = true;
            lock (gate)
            {
                if (abandoned)
                {
                    Kill(child);
                    child.Dispose();
                    return;
                }
                process = child;
                stdoutThread = new Thread(delegate() { Drain(child.StandardOutput.BaseStream, StdoutPath, true); });
                stderrThread = new Thread(delegate() { Drain(child.StandardError.BaseStream, StderrPath, false); });
                stdoutThread.IsBackground = true;
                stderrThread.IsBackground = true;
                stdoutThread.Start();
                stderrThread.Start();
                Thread inputThread = new Thread(WriteInput);
                inputThread.IsBackground = true;
                inputThread.Start();
                Monitor.PulseAll(gate);
            }
        }
        catch (Exception ex)
        {
            if (childStarted) Kill(child);
            child.Dispose();
            Fail(ex);
        }
    }

    private InputCommand Enqueue(Action action)
    {
        InputCommand command = new InputCommand { Action = action };
        inputCommands.Enqueue(command);
        Monitor.PulseAll(gate);
        return command;
    }

    private void WriteInput()
    {
        while (true)
        {
            InputCommand command;
            lock (gate)
            {
                while (inputCommands.Count == 0 && !abandoned) Monitor.Wait(gate);
                if (abandoned) return;
                command = inputCommands.Dequeue();
            }
            try
            {
                command.Action();
                lock (gate)
                {
                    command.Done = true;
                    Monitor.PulseAll(gate);
                }
            }
            catch (Exception ex) { Fail(ex); return; }
        }
    }

    private void Fail(Exception ex)
    {
        lock (gate)
        {
            if (failure == null) failure = ex;
            Monitor.PulseAll(gate);
        }
    }

    private void Abort()
    {
        Stopwatch clock = Stopwatch.StartNew();
        Thread launcher;
        lock (gate) { launcher = launcherThread; }
        try { if (launcher != null) Join(launcher, clock, CleanupTimeout); }
        catch
        {
            lock (gate) { abandoned = true; Monitor.PulseAll(gate); }
            throw;
        }
        Process child;
        lock (gate) { child = process; }
        if (child != null && !completed && !child.HasExited)
        {
            lock (gate)
            {
                if (!stopped && !stopping)
                {
                    stopping = true;
                    Enqueue(delegate
                    {
                        child.StandardInput.WriteLine("STOP");
                        child.StandardInput.Flush();
                    });
                }
            }
            WaitForCleanup(delegate { return stopped; }, child, clock);
            InputCommand close = null;
            lock (gate)
            {
                if (stopped && !child.HasExited)
                    close = Enqueue(delegate { child.StandardInput.Close(); });
            }
            if (close != null) WaitForCleanup(delegate { return close.Done; }, child, clock);
            if (!child.HasExited)
                child.WaitForExit(Remaining(clock, CleanupTimeout));
            if (!child.HasExited) Kill(child);
        }
        lock (gate)
        {
            abandoned = true;
            Monitor.PulseAll(gate);
        }
        clock.Restart();
        Thread stdout;
        Thread stderr;
        lock (gate)
        {
            stdout = stdoutThread;
            stderr = stderrThread;
        }
        if (stdout != null && (stdout.ThreadState & System.Threading.ThreadState.Unstarted) == 0)
            Join(stdout, clock, CleanupTimeout);
        if (stderr != null && (stderr.ThreadState & System.Threading.ThreadState.Unstarted) == 0)
            Join(stderr, clock, CleanupTimeout);
        lock (gate)
        {
            if (drainFailure != null)
                throw new InvalidOperationException("Injector evidence could not be fully drained.", drainFailure);
        }
    }

    private void WaitForCleanup(Func<bool> condition, Process child, Stopwatch clock)
    {
        lock (gate)
        {
            while (!condition() && !child.HasExited)
            {
                int remaining = Remaining(clock, CleanupTimeout);
                if (remaining == 0) return;
                Monitor.Wait(gate, remaining);
            }
        }
    }

    private static void Kill(Process child)
    {
        try { if (!child.HasExited) child.Kill(); }
        catch (InvalidOperationException) { }
    }

}
