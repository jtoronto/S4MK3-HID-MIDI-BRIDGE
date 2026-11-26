using System.Diagnostics;
using System.Text;

namespace S4ScreenSupervisor;

internal static class Program
{
    internal const string WorkerName = "s4-encdr-screen-worker.exe";

    internal static bool ValidSerial(string serial) =>
        serial.Length is > 0 and <= 64 && serial.All(c => c is >= 'A' and <= 'Z'
            or >= 'a' and <= 'z' or >= '0' and <= '9');

    public static async Task<int> Main(string[] args)
    {
        var output = Console.OpenStandardOutput();
        try
        {
            if (!OperatingSystem.IsWindows())
                throw new PlatformNotSupportedException("Windows is required");
            var input = Console.OpenStandardInput();
            if (args is ["--self-test-worker", var fixtureMode, "--worker-pid-file", var fixturePath]
                && SelfTest.ValidMode(fixtureMode))
                return await SelfTest.Worker(fixtureMode, fixturePath);
            ProcessStartInfo start;
            if (args.Length is 4 or 5 && args[0] == "--self-test" && args[2] == "--worker-pid-file"
                && (args.Length == 4 || args[4] == "--assignment-gate") && SelfTest.ValidMode(args[1]))
            {
                if (args.Length == 5)
                {
                    var gate = new byte[1];
                    if (await input.ReadAsync(gate) != 1 || gate[0] != 0xA5) return 1;
                }
                start = SelfTest.Start(args[1], Path.GetFullPath(args[3]));
            }
            else if (args is ["--serial", var serial] && ValidSerial(serial))
            {
                start = new ProcessStartInfo(Path.Combine(AppContext.BaseDirectory, WorkerName));
                start.ArgumentList.Add("--serial");
                start.ArgumentList.Add(serial);
            }
            else throw new ArgumentException("usage: --serial ASCII_ALPHANUMERIC_MAX64; or --self-test MODE --worker-pid-file NEW_PATH");
            using var parent = ParentLifetime.Open();
            using var worker = NativeWorker.Start(start, Job.Create());
            return await new Session(worker, input, output, parent.Exited).Run();
        }
        catch (Exception e)
        {
            await WriteError(output, e.Message);
            return 1;
        }
    }

    internal static string ErrorLine(string detail) =>
        "ERROR " + new string(detail.Select(c => c is >= ' ' and <= '~' ? c : '?').ToArray()) + "\n";

    internal static async Task WriteError(Stream output, string detail, TimeProvider? time = null)
    {
        try
        {
            // Even the error recipient may have stopped reading. Never flush Console.Out.
            await Task.Run(async () =>
            {
                await output.WriteAsync(Encoding.ASCII.GetBytes(ErrorLine(detail)));
                await output.FlushAsync();
            }).WaitAsync(TimeSpan.FromMilliseconds(250), time ?? TimeProvider.System);
        }
        catch (Exception e) when (e is IOException or ObjectDisposedException or TimeoutException) { }
    }
}
