using System.Text.Json;

namespace S4LoopBeProbe;

internal static class Program
{
    internal const string PortName = "LoopBe Internal MIDI";
    internal const uint Forward = 0x00357791;
    internal const uint Reverse = 0x00486FBF;

    private static async Task<int> Main(string[] args)
    {
        try
        {
            if (args.Length != 1)
            {
                throw new ArgumentException("--inspect | --self-test | --listen | --listen-raw");
            }
            switch (args[0])
            {
                case "--inspect":
                    Emit("ports", new { ports = NativePort.Enumerate() });
                    break;
                case "--self-test":
                    using (var port = NativePort.Open(Reverse))
                    {
                        await Exchange.VerifyAsync(port);
                        Emit("connection-closing", new { });
                    }
                    Emit("connection-closed", new { });
                    Emit("passed", new { twoWay = true });
                    break;
                case "--peer":
                    await Exchange.PeerAsync();
                    break;
                case "--listen":
                    await DjayObservation.RunAsync();
                    break;
                case "--listen-raw":
                    await DjayObservation.RunAsync(reportSessions: false, seconds: 120);
                    break;
                case "--crossfade-right":
                    await DjayProof.RunAsync(127, true);
                    break;
                case "--crossfade-left":
                    await DjayProof.RunAsync(0, true);
                    break;
                case "--watch-crossfade-right":
                    await DjayProof.RunAsync(127, false);
                    break;
                case "--watch-crossfade-left":
                    await DjayProof.RunAsync(0, false);
                    break;
                case "--route-check":
                    CrossfaderFeedback.Check();
                    break;
                case "--legacy-winrt-check":
                    await LegacyWinRtCheck.RunAsync();
                    break;
                default:
                    throw new ArgumentException("Unknown LoopBe1 operation.");
            }
            return 0;
        }
        catch (Exception error)
        {
            Emit("failure", new { type = error.GetType().FullName, error.Message,
                hresult = $"0x{error.HResult:X8}", detail = error.ToString() });
            return 1;
        }
    }

    internal static void Emit(string stage, object data) =>
        Console.WriteLine(JsonSerializer.Serialize(new { stage, pid = Environment.ProcessId, data }));
}
