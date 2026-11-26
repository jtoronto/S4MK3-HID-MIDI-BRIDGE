using System.Diagnostics;
using System.Text.Json;

namespace S4LoopbackProbe;

internal static class IndependentPeer
{
    internal static async Task VerifyAsync(Action send, Task<uint> returned, bool note = false)
    {
        var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var exited = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        using var peer = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = Environment.ProcessPath
                    ?? throw new InvalidOperationException("Cannot locate the loopback harness."),
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                RedirectStandardInput = true,
                CreateNoWindow = true
            },
            EnableRaisingEvents = true
        };
        peer.StartInfo.ArgumentList.Add(note ? "--winmm-note-peer" : "--winmm-peer");
        peer.Exited += (_, _) =>
        {
            ready.TrySetException(new InvalidOperationException(
                $"WinMM peer exited before ready: {peer.ExitCode}"));
            exited.TrySetResult();
        };
        peer.OutputDataReceived += (_, args) =>
        {
            if (args.Data is not { } line)
            {
                return;
            }
            Console.WriteLine(line);
            using var record = JsonDocument.Parse(line);
            if (record.RootElement.GetProperty("stage").GetString() == "peer-ready")
            {
                ready.TrySetResult();
            }
        };
        peer.ErrorDataReceived += (_, args) =>
        {
            if (args.Data is { } line)
            {
                Program.Emit("peer-stderr", new { line });
            }
        };
        if (!peer.Start())
        {
            throw new InvalidOperationException("WinMM peer did not start.");
        }
        peer.BeginOutputReadLine();
        peer.BeginErrorReadLine();
        try
        {
            await ready.Task.WaitAsync(TimeSpan.FromSeconds(20));
            send();
            var reverse = await returned.WaitAsync(TimeSpan.FromSeconds(15));
            await peer.StandardInput.WriteLineAsync("received");
            await peer.StandardInput.FlushAsync();
            await exited.Task.WaitAsync(TimeSpan.FromSeconds(20));
            peer.WaitForExit();
            if (peer.ExitCode != 0)
            {
                throw new InvalidOperationException($"WinMM peer failed: {peer.ExitCode}");
            }
            Program.Emit("round-trip", new
            {
                forward = "0x00357791",
                reverse = $"0x{reverse:X8}",
                passed = true
            });
        }
        finally
        {
            // Unblock the peer on a failed receive as well as on a successful acknowledgement.
            peer.StandardInput.Close();
            await exited.Task.WaitAsync(TimeSpan.FromSeconds(20));
            peer.WaitForExit();
        }
    }
}
