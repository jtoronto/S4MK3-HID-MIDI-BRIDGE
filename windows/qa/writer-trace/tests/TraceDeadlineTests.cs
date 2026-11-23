using System;
using System.Diagnostics;
using System.IO;
using System.Threading;

public static class TraceDeadlineTests
{
    private static ProcessStartInfo Command(string command)
    {
        return new ProcessStartInfo("/bin/sh", "-c \"" + command + "\"");
    }

    public static void Run(string directory)
    {
        string receipt = Path.Combine(directory, "deadline-stopped");
        using (ManualResetEventSlim created = new ManualResetEventSlim())
        using (FileSystemWatcher watcher = new FileSystemWatcher(directory))
        {
            watcher.Filter = "deadline-stopped";
            watcher.Created += delegate { created.Set(); };
            watcher.EnableRaisingEvents = true;
            WriterTraceCommands commands = new WriterTraceCommands
            {
                Stop = Command("echo stopped > '" + receipt + "'"),
                Cancel = Command("exit 19")
            };
            using (WriterTraceDeadline deadline = new WriterTraceDeadline(
                commands, TimeSpan.FromMilliseconds(200)))
            {
                if (!created.Wait(TimeSpan.FromSeconds(5)))
                    throw new Exception("Independent timer did not start cleanup.");
                deadline.Stop();
                if (!deadline.DeadlineReached || deadline.Result.ExitCode != 0 ||
                    deadline.CancelResult != null)
                    throw new Exception("Unexpected independent deadline result.");
            }
        }

        string failureReceipt = Path.Combine(directory, "deadline-failed");
        WriterTraceCommands failureCommands = new WriterTraceCommands
        {
            Stop = Command("echo stop >> '" + failureReceipt + "'; exit 7"),
            Cancel = Command("echo cancel >> '" + failureReceipt + "'; exit 0")
        };
        WriterTraceDeadline failed = new WriterTraceDeadline(failureCommands, TimeSpan.FromSeconds(10));
        try
        {
            bool rejected = false;
            try { failed.Stop(); }
            catch (InvalidOperationException) { rejected = true; }
            if (!rejected || failed.CancelResult == null || failed.CancelResult.ExitCode != 0 ||
                String.Join(",", File.ReadAllLines(failureReceipt)) != "stop,cancel")
                throw new Exception("Failed stop did not preserve one scoped cancellation.");
        }
        finally
        {
            try { failed.Dispose(); }
            catch (InvalidOperationException)
            {
                if (failed.Result == null || failed.Result.ExitCode != 7) throw;
            }
        }
        Console.WriteLine("TraceDeadline tests passed.");
    }
}
