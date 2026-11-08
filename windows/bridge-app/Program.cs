using Avalonia;

namespace S4Bridge;

internal static class Program
{
    [STAThread]
    public static void Main(string[] args) =>
        AppBuilder.Configure<BridgeApplication>().UsePlatformDetect().LogToTrace()
            .StartWithClassicDesktopLifetime(args);
}
