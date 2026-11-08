using System.Text.Json;
using Windows.Devices.Midi2;
using Windows.Devices.Midi2.Transports.Loopback;

namespace S4LoopbackProbe;

internal static class Program
{
    private static async Task<int> Main(string[] args)
    {
        try
        {
            if (args.Length != 1)
            {
                throw new ArgumentException(
                    "--inspect | --create | --self-test | --listen | --remove");
            }
            if (args[0] is "--winmm-peer" or "--winmm-note-peer")
            {
                await NativePeer.RunAsync(args[0] == "--winmm-note-peer");
                return 0;
            }
            Emit("api", new
            {
                service = MidiApi.EnsureServiceAvailable(),
                available = MidiLoopbackManager.IsTransportAvailable,
                systemProvided = MidiApi.IsSystemProvided()
            });
            switch (args[0])
            {
                case "--inspect":
                    PairConfiguration.Inspect();
                    break;
                case "--create":
                    PairConfiguration.Create();
                    break;
                case "--remove":
                    PairConfiguration.Remove();
                    break;
                case "--self-test":
                    await SessionCheck.RunAsync(false);
                    break;
                case "--winmm-test":
                    await SessionCheck.RunNativeAsync();
                    break;
                case "--winmm-note-test":
                    await SessionCheck.RunNativeAsync(true);
                    break;
                case "--listen":
                    await SessionCheck.RunAsync(true);
                    break;
                default:
                    throw new ArgumentException("Unknown loopback operation.");
            }
            return 0;
        }
        catch (Exception error)
        {
            Emit("failure", new
            {
                type = error.GetType().FullName,
                error.Message,
                hresult = $"0x{error.HResult:X8}",
                detail = error.ToString()
            });
            return 1;
        }
    }

    internal static void Emit(string stage, object data) =>
        Console.WriteLine(JsonSerializer.Serialize(new { stage, data }));
}
