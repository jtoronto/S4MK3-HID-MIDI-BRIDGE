using System.Diagnostics;
using System.Text.Json;

namespace S4LoopBeProbe;

internal static class Exchange
{
    internal static async Task VerifyAsync(NativePort port)
    {
        var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var exited = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        using var peer = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = Environment.ProcessPath
                    ?? throw new InvalidOperationException("Cannot locate the harness."),
                UseShellExecute = false,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            },
            EnableRaisingEvents = true
        };
        peer.StartInfo.ArgumentList.Add("--peer");
        peer.Exited += (_, _) =>
        {
            ready.TrySetException(new InvalidOperationException($"Peer exited: {peer.ExitCode}"));
            exited.TrySetResult();
        };
        peer.OutputDataReceived += (_, args) =>
        {
            if (args.Data is { } line)
            {
                Console.WriteLine(line);
                using var record = JsonDocument.Parse(line);
                if (record.RootElement.GetProperty("stage").GetString() == "peer-ready")
                {
                    ready.TrySetResult();
                }
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
            throw new InvalidOperationException("Cannot start independent peer.");
        }
        peer.BeginOutputReadLine();
        peer.BeginErrorReadLine();
        try
        {
            await ready.Task.WaitAsync(TimeSpan.FromSeconds(15));
            port.Send(Program.Forward);
            var reverse = await port.Received.WaitAsync(TimeSpan.FromSeconds(15));
            await peer.StandardInput.WriteLineAsync("received");
            await peer.StandardInput.FlushAsync();
            await exited.Task.WaitAsync(TimeSpan.FromSeconds(15));
            peer.WaitForExit();
            if (peer.ExitCode != 0)
            {
                throw new InvalidOperationException($"Independent peer failed: {peer.ExitCode}");
            }
            Program.Emit("round-trip", new { forward = $"0x{Program.Forward:X8}",
                reverse = $"0x{reverse:X8}", peerPid = peer.Id });
        }
        finally
        {
            peer.StandardInput.Close();
            await exited.Task.WaitAsync(TimeSpan.FromSeconds(20));
            peer.WaitForExit();
        }
    }

    internal static async Task PeerAsync()
    {
        using (var port = NativePort.Open(Program.Forward))
        {
            Program.Emit("peer-ready", new { });
            await port.Received.WaitAsync(TimeSpan.FromSeconds(15));
            port.Send(Program.Reverse);
            var ack = await Task.Run(Console.ReadLine).WaitAsync(TimeSpan.FromSeconds(20));
            if (ack != "received")
            {
                throw new InvalidOperationException("Parent did not acknowledge the return.");
            }
            Program.Emit("peer-passed", new { acknowledged = true });
        }
        Program.Emit("peer-closed", new { });
    }
}
