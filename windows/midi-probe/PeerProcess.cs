using System.Diagnostics;
using System.Text.Json;

namespace S4MidiProbe;

internal static class PeerProcess
{
    internal static async Task VerifyAsync(VirtualController controller, Task<uint> returned)
    {
        var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var exited = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        using var child = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = Environment.ProcessPath
                    ?? throw new InvalidOperationException("Could not locate the probe executable."),
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            },
            EnableRaisingEvents = true
        };
        child.StartInfo.ArgumentList.Add("--winmm-peer");
        // Register process and output signals before starting the independent consumer.
        child.Exited += (_, _) =>
        {
            ready.TrySetException(new InvalidOperationException(
                $"WinMM peer exited before becoming ready: {child.ExitCode}"));
            exited.TrySetResult();
        };
        child.OutputDataReceived += (_, args) =>
        {
            if (args.Data is not { } line)
            {
                return;
            }
            Console.WriteLine(line);
            using var message = JsonDocument.Parse(line);
            if (message.RootElement.GetProperty("stage").GetString() == "peer-ready")
            {
                ready.TrySetResult();
            }
        };
        child.ErrorDataReceived += (_, args) =>
        {
            if (args.Data is { } line)
            {
                Program.Emit("peer-stderr", new { line });
            }
        };
        if (!child.Start())
        {
            throw new InvalidOperationException("Could not launch the independent WinMM peer.");
        }
        child.BeginOutputReadLine();
        child.BeginErrorReadLine();
        try
        {
            await ready.Task.WaitAsync(TimeSpan.FromSeconds(20));
            controller.Send(0x20917735);
            var reverse = await returned.WaitAsync(TimeSpan.FromSeconds(15));
            await exited.Task.WaitAsync(TimeSpan.FromSeconds(20));
            child.WaitForExit(); // Drain the redirected output after the exit notification.
            if (child.ExitCode != 0)
            {
                throw new InvalidOperationException($"WinMM peer failed: {child.ExitCode}");
            }
            Program.Emit("round-trip", new
            {
                deviceToWinmm = "0x00357791",
                winmmToDevice = $"0x{reverse:X8}",
                passed = true
            });
        }
        finally
        {
            // The peer's own bounded receive wait closes its handles. Never force-kill it.
            await exited.Task.WaitAsync(TimeSpan.FromSeconds(20));
            child.WaitForExit();
        }
    }
}
