using System.Buffers.Binary;
using System.Text;

namespace S4ScreenSupervisor.Tests;

internal static partial class Program
{
    private static readonly TimeSpan CircuitBreaker = TimeSpan.FromSeconds(10);
    private static int passed;
    private static string supervisorPath = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../supervisor/s4-screen-supervisor.exe"));
    internal static void Check(bool condition, string detail)
    {
        if (!condition) throw new InvalidOperationException(detail);
    }
    private static async Task Test(string name, Func<Task> action)
    {
        await action().WaitAsync(CircuitBreaker);
        passed++;
        Console.WriteLine("PASS " + name);
    }
    internal static async Task Line(Stream output, string text) =>
        await output.WriteAsync(Encoding.ASCII.GetBytes(text + "\n"));
    private static async Task<string> ReadLine(Stream input)
        => await ReadLineCore(input).WaitAsync(TimeSpan.FromSeconds(5));

    private static async Task<string> ReadLineCore(Stream input)
    {
        var text = new StringBuilder();
        var b = new byte[1];
        while (await input.ReadAsync(b) != 0)
        {
            if (b[0] == '\n') return text.ToString();
            text.Append((char)b[0]);
        }
        throw new EndOfStreamException("expected protocol response");
    }
    internal static byte[] Frame()
    {
        var bytes = new byte[307205];
        bytes[0] = 1;
        BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(1), 307200);
        // Distinct displays and every pixel-byte value; independent of renderer.
        for (int i = 5; i < bytes.Length; i++) bytes[i] = (byte)((i * 37 + i / 153600) % 256);
        return bytes;
    }
    private static Task Never() => new TaskCompletionSource().Task;
    private static async Task Arm(MemoryWorker worker)
    {
        var arm = new byte[1];
        await worker.Requests.ReadExactlyAsync(arm);
        Check(arm[0] == 0xA5, "ARM byte changed");
    }
    private sealed class Harness : IDisposable
    {
        internal MemoryWorker Worker { get; } = new();
        internal BytePipe Input { get; } = new();
        internal BytePipe Output { get; } = new();
        internal ManualTime Clock { get; } = new();
        internal TaskCompletionSource ParentLost { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        internal Task<int> Start(Stream? output = null) => new Session(Worker, Input, output ?? Output,
            ParentLost.Task, Clock).Run();
        internal async Task Ready()
        {
            await Arm(Worker);
            await Line(Worker.Responses, "READY");
            Check(await ReadLine(Output) == "READY", "READY was not forwarded exactly");
        }
        internal async Task Stop()
        {
            await Input.WriteAsync(new byte[] { 2, 0, 0, 0, 0 });
            var stop = new byte[5];
            await Worker.Requests.ReadExactlyAsync(stop);
            Check(stop.SequenceEqual(new byte[] { 2, 0, 0, 0, 0 }), "STOP bytes changed");
            await Line(Worker.Responses, "STOPPED");
            Worker.Exit(0);
        }
        public void Dispose() { Input.Dispose(); Worker.Dispose(); Output.Dispose(); }
    }

    public static async Task<int> Main(string[] args)
    {
        try
        {
            if (args.Length > 0 && args[0] == "--fixture") return await Fixture(args[1..]);
            if (args is ["--windows", "--supervisor", var path])
            { supervisorPath = Path.GetFullPath(path); await NativeTests(); }
            else if (args is ["--windows"]) { await NativeTests(); }
            else if (args.Length == 0) { await PortableTests(); }
            else throw new ArgumentException("usage: [--windows [--supervisor EXE]]");
            Console.WriteLine($"{passed} tests passed");
            return 0;
        }
        catch (Exception e) { Console.Error.WriteLine(e); return 1; }
    }

    private static async Task PortableTests()
    {
        await Test("fragmented frame is relayed byte for byte and stop reaps before STOPPED", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await h.Ready();
            var frame = Frame();
            for (int i = 0; i < 2; i++)
            {
                foreach (var chunk in frame.Chunk(997)) await h.Input.WriteAsync(chunk);
                var received = new byte[frame.Length];
                await h.Worker.Requests.ReadExactlyAsync(received);
                Check(received.SequenceEqual(frame), "header or display pixels changed");
                await Line(h.Worker.Responses, "FRAME_OK");
                Check(await ReadLine(h.Output) == "FRAME_OK", "FRAME_OK missing");
                frame[5] ^= 0xFF;
            }
            await h.Stop();
            Check(await run == 0 && h.Worker.Terminated, "successful stop did not close owned job");
            Check(await ReadLine(h.Output) == "STOPPED", "STOPPED missing after reap");
        });
        await Test("record parser rejects malformed headers and truncated records", async () =>
        {
            foreach (var bytes in new byte[][] {
                [3,0,0,0,0], [1,0,0,0,0], [2,1,0,0,0], [1,0,0,0,128], [1,0],
                Frame()[..^1] })
            {
                try
                {
                    await Wire.ReadRecord(new MemoryStream(bytes), TimeProvider.System,
                        TimeSpan.FromSeconds(1), default);
                    throw new InvalidOperationException("malformed record admitted");
                }
                catch (Exception e) when (e is IOException or InvalidDataException) { }
            }
        });
        await Test("reply receipt rejects stale duplicate early and malformed ACKs", async () =>
        {
            foreach (var text in new[] { "FRAME_OK\n", "STOPPED\n", "READY\nREADY\n", "READY\nFRAME_OK\n",
                "READY", "READY\rjunk\n", "RÉADY\n", new string('a', 16385) + "\n" })
            {
                var replies = new Replies();
                _ = replies.Expect("READY");
                bool parserFailed = false;
                try { await Wire.ReadReplies(new MemoryStream(Encoding.UTF8.GetBytes(text)), replies.Receive, default); }
                catch (Exception e) when (e is IOException or InvalidDataException) { parserFailed = true; }
                Check(parserFailed || replies.Failure.Task.IsCompleted, "unexpected response admitted: " + text[..Math.Min(text.Length, 30)]);
            }
            var completed = new Replies();
            _ = completed.Expect("READY");
            completed.Receive("READY");
            _ = completed.Expect("FRAME_OK");
            completed.Receive("FRAME_OK");
            completed.Receive("FRAME_OK");
            Check(completed.Failure.Task.IsCompleted, "duplicate ACK was saved for next frame");
        });
        await Test("second frame while ACK is pending fails instead of queueing", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await h.Ready();
            await h.Input.WriteAsync(Frame());
            await h.Worker.Requests.ReadExactlyAsync(new byte[307205]);
            await h.Input.WriteAsync(Frame());
            Check(await run == 1 && h.Worker.Terminated, "busy frame did not fail");
            Check((await ReadLine(h.Output)).StartsWith("ERROR ", StringComparison.Ordinal), "missing failure response");
        });
        await Test("worker ERROR detail is relayed exactly without stdout diagnostics", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await Arm(h.Worker);
            var detail = "worker fault " + new string('x', 2000);
            await Line(h.Worker.Responses, "ERROR " + detail);
            Check(await run == 1, "worker error reported success");
            Check(await ReadLine(h.Output) == "ERROR " + detail, "worker detail was changed");
        });
        foreach (var cause in new[] { "parent-eof", "parent-loss", "worker-eof" })
            await Test(cause + " terminates owned worker while idle", async () =>
            {
                using var h = new Harness();
                var run = h.Start();
                await h.Ready();
                switch (cause)
                {
                    case "parent-eof": h.Input.End(); break;
                    case "parent-loss": h.ParentLost.SetResult(); break;
                    case "worker-eof": h.Worker.Responses.End(); break;
                }
                Check(await run == 1 && h.Worker.Terminated, "lost peer left worker alive");
            });
        await Test("connect timeout is independent of blocked worker call", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await Arm(h.Worker);
            await h.Clock.Expire(TimeSpan.FromSeconds(8));
            Check(await run == 1 && h.Worker.Terminated, "connect deadline not enforced");
        });
        await Test("blocked worker pipe write times out and is reaped", async () =>
        {
            using var h = new Harness();
            h.Worker.Hold = new(h.Worker.Requests, after: 1);
            var run = h.Start();
            await h.Ready();
            await h.Input.WriteAsync(Frame());
            await h.Worker.Hold.Blocked.Task;
            await h.Clock.Expire(TimeSpan.FromSeconds(3));
            Check(await run == 1 && h.Worker.Terminated, "blocked write retained worker");
        });
        await Test("STOP bypasses blocked frame write and never reports STOPPED", async () =>
        {
            using var h = new Harness();
            h.Worker.Hold = new(h.Worker.Requests, after: 1);
            var run = h.Start();
            await h.Ready();
            await h.Input.WriteAsync(Frame());
            await h.Worker.Hold.Blocked.Task;
            await h.Input.WriteAsync(new byte[] { 2,0,0,0,0 });
            Check(await run == 1 && h.Worker.Terminated, "STOP waited on pipe");
            Check((await ReadLine(h.Output)).StartsWith("ERROR ", StringComparison.Ordinal), "forced stop claimed restoration");
        });
        await Test("unresponsive STOP has an independent deadline", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await h.Ready();
            await h.Input.WriteAsync(new byte[] { 2,0,0,0,0 });
            await h.Worker.Requests.ReadExactlyAsync(new byte[5]);
            await h.Clock.Expire(TimeSpan.FromMilliseconds(500));
            Check(await run == 1 && h.Worker.Terminated, "STOP deadline not enforced");
        });
        await Test("partial parent record has a deadline after its first byte", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await h.Ready();
            await h.Input.WriteAsync(new byte[] { 1 });
            await h.Clock.Expire(TimeSpan.FromSeconds(3));
            Check(await run == 1 && h.Worker.Terminated, "partial record hung");
        });
        await Test("blocked bridge stdout cannot keep worker alive", async () =>
        {
            using var h = new Harness();
            var held = new HeldWrite(h.Output, after: 0);
            var run = h.Start(held);
            await Arm(h.Worker);
            await Line(h.Worker.Responses, "READY");
            await held.Blocked.Task;
            await h.Clock.Expire(TimeSpan.FromSeconds(3));
            await h.Clock.Expire(TimeSpan.FromMilliseconds(250));
            Check(await run == 1 && h.Worker.Terminated, "stdout stalled cleanup");
            held.Release();
        });
        await Test("stderr floods drain independently of the protocol", async () =>
        {
            using var h = new Harness();
            var run = h.Start();
            await h.Ready();
            for (int i = 0; i < 256; i++) await h.Worker.Logs.WriteAsync(new byte[4096]);
            await h.Stop();
            Check(await run == 0 && await ReadLine(h.Output) == "STOPPED", "stderr corrupted protocol");
        });
        await Test("real local child handles ARM frames ACK and STOP over redirected pipes", async () =>
        {
            using var child = Spawn("worker", "normal");
            var errors = child.StandardError.ReadToEndAsync();
            try
            {
                await child.StandardInput.BaseStream.WriteAsync(new byte[] { 0xA5 });
                Check(await ReadLine(child.StandardOutput.BaseStream) == "READY", "fixture did not accept ARM");
                await child.StandardInput.BaseStream.WriteAsync(Frame());
                Check(await ReadLine(child.StandardOutput.BaseStream) == "FRAME_OK", "fixture did not parse full frame");
                await child.StandardInput.BaseStream.WriteAsync(new byte[] { 2,0,0,0,0 });
                Check(await ReadLine(child.StandardOutput.BaseStream) == "STOPPED", "fixture did not stop");
                await child.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                Check(child.ExitCode == 0 && await errors == "ARMED\nFRAME\nSTOP\n", "fixture failed or polluted stderr");
            }
            finally
            {
                if (!child.HasExited) child.Kill();
                await child.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
            }
        });
        Console.WriteLine("Portable stream/lifecycle tests; Windows Job semantics not exercised.");
    }
}
