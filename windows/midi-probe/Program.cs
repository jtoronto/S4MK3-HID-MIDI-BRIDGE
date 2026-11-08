using System.Text.Json;
using Windows.Devices.Midi2;

namespace S4MidiProbe;

internal static class Program
{
    private static async Task<int> Main(string[] args)
    {
        try
        {
            if (args.Length == 0 ||
                (args[0] is not ("--detect" or "--self-test" or "--listen" or "--winmm-peer")) ||
                (args[0] == "--listen" ? args.Length != 2 : args.Length != 1))
            {
                throw new ArgumentException(
                    "Usage: S4MidiProbe.exe --detect | --self-test | --listen SECONDS");
            }

            if (!OperatingSystem.IsWindowsVersionAtLeast(10, 0, 26200))
            {
                throw new PlatformNotSupportedException("Windows 11 25H2 or later is required.");
            }

            if (args[0] == "--winmm-peer")
            {
                await WinMmPeer.RunAsync();
                return 0;
            }

            var serviceAvailable = MidiApi.EnsureServiceAvailable();
            Emit("api", new
            {
                systemProvided = MidiApi.IsSystemProvided(),
                serviceAvailable,
                apiVersion = typeof(MidiApi).Assembly.GetName().Version?.ToString(),
                os = Environment.OSVersion.VersionString
            });
            if (!serviceAvailable)
            {
                return 2;
            }

            switch (args[0])
            {
                case "--detect":
                    return 0;
                case "--self-test":
                    await Probe.RunAsync(null);
                    return 0;
                case "--listen":
                    if (!int.TryParse(args[1], out var seconds) || seconds is < 1 or > 600)
                    {
                        throw new ArgumentException("Listen duration must be 1-600 seconds.");
                    }
                    await Probe.RunAsync(TimeSpan.FromSeconds(seconds));
                    return 0;
                default:
                    throw new InvalidOperationException("Unrecognized probe mode.");
            }
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
