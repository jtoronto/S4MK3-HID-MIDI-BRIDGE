using System.Diagnostics;
using System.Globalization;
using System.Text;

namespace S4ScreenSupervisor;

// Explicit hardware-free production-launch fixture. It runs the same Session,
// native worker ownership and parent watch, but cannot select an NI worker.
internal static class SelfTest
{
    internal static bool ValidMode(string mode) =>
        mode is "normal" or "connect-stall" or "send-stall" or "stop-stall" or "blocked-write" or "flood";

    internal static ProcessStartInfo Start(string mode, string path, bool assignmentGate = false)
    {
        var executable = Environment.ProcessPath ?? throw new IOException("missing executable path");
        var start = new ProcessStartInfo(executable);
        if (Path.GetFileNameWithoutExtension(executable) == "dotnet")
            start.ArgumentList.Add(Path.Combine(AppContext.BaseDirectory, "s4-screen-supervisor.dll"));
        start.ArgumentList.Add(assignmentGate ? "--self-test" : "--self-test-worker");
        start.ArgumentList.Add(mode);
        start.ArgumentList.Add("--worker-pid-file");
        start.ArgumentList.Add(path);
        if (assignmentGate) start.ArgumentList.Add("--assignment-gate");
        return start;
    }

    internal static async Task<int> Worker(string mode, string? pidFile)
    {
        if (!ValidMode(mode)) throw new ArgumentException("invalid self-test worker mode");
        var input = Console.OpenStandardInput();
        var output = Console.OpenStandardOutput();
        var error = Console.OpenStandardError();
        var arm = new byte[1];
        if (await input.ReadAsync(arm) != 1 || arm[0] != 0xA5) return 0;
        if (pidFile is not null)
        {
            // Publish a complete marker atomically. Existing marker paths are never
            // overwritten. Observers subscribe before starting their outer owner.
            var temporary = pidFile + "." + Guid.NewGuid().ToString("N") + ".tmp";
            try
            {
                await using (var file = new FileStream(temporary, FileMode.CreateNew, FileAccess.Write))
                    await file.WriteAsync(Encoding.ASCII.GetBytes(Environment.ProcessId.ToString(CultureInfo.InvariantCulture) + "\n"));
                File.Move(temporary, pidFile);
            }
            finally
            {
                if (File.Exists(temporary)) File.Delete(temporary);
            }
        }
        await Emit(error, "ARMED");
        if (mode == "connect-stall") { await Never(); return 0; }
        await Emit(output, "READY");
        if (mode == "blocked-write") { await Never(); return 0; }
        if (mode == "flood")
            for (int i = 0; i < 512; i++) await error.WriteAsync(new byte[4096]);
        while (true)
        {
            var record = await Wire.ReadRecord(input, TimeProvider.System, TimeSpan.FromSeconds(5), default);
            if (record is null) return 0;
            if (record[0] == 2)
            {
                await Emit(error, "STOP");
                if (mode == "stop-stall") { await Never(); return 0; }
                await Emit(output, "STOPPED");
                return 0;
            }
            await Emit(error, "FRAME");
            if (mode == "send-stall") { await Never(); return 0; }
            await Emit(output, "FRAME_OK");
        }
    }

    private static Task Never() => new TaskCompletionSource().Task;
    private static async Task Emit(Stream stream, string line)
    {
        await stream.WriteAsync(Encoding.ASCII.GetBytes(line + "\n"));
        await stream.FlushAsync();
    }
}
