using System.Runtime.InteropServices;

namespace S4MidiProbe;

internal sealed class WinMmPeer : IDisposable
{
    private readonly MidiInCallback callback;
    private readonly TaskCompletionSource<uint> received =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private nint input;
    private nint output;

    private WinMmPeer()
    {
        callback = OnMessage;
    }

    internal static WinMmPeer Open()
    {
        // Resolve actual WinMM indices in the consuming process, not SDK metadata indices.
        var (source, destination) = WinMmPorts.FindProbe();
        var peer = new WinMmPeer();
        try
        {
            Program.Emit("winmm-step", new { action = "open-output" });
            Check(midiOutOpen(out peer.output, destination.Number, 0, 0, 0), "open WinMM output");
            Program.Emit("winmm-step", new { action = "open-input" });
            Check(midiInOpen(out peer.input, source.Number, peer.callback, 0, 0x00030000),
                "open WinMM input");
            Program.Emit("winmm-step", new { action = "start-input" });
            Check(midiInStart(peer.input), "start WinMM input");
            Program.Emit("winmm-open", new { inputName = source.Name, outputName = destination.Name });
            return peer;
        }
        catch
        {
            peer.Dispose();
            throw;
        }
    }

    internal Task<uint> Received => received.Task;

    internal static async Task RunAsync()
    {
        using var peer = Open();
        Program.Emit("peer-ready", new { pid = Environment.ProcessId });
        var forward = await peer.Received.WaitAsync(TimeSpan.FromSeconds(10));
        peer.Send(0x00486FBF);
        Program.Emit("peer-passed", new { deviceToWinmm = $"0x{forward:X8}" });
    }

    internal void Send(uint message) =>
        Check(midiOutShortMsg(output, message), "send WinMM feedback");

    private void OnMessage(nint handle, uint message, nuint instance, nuint parameter, nuint timestamp)
    {
        if (message == 0x3C3)
        {
            var packed = checked((uint)parameter);
            Program.Emit("winmm-received", new { packed = $"0x{packed:X8}" });
            if (packed == 0x00357791)
            {
                received.TrySetResult(packed);
            }
        }
    }

    internal static void Check(uint result, string action)
    {
        if (result != 0)
        {
            Program.Emit("winmm-error", new { action, result });
            throw new InvalidOperationException($"{action}: WinMM error {result}");
        }
    }

    public void Dispose()
    {
        try
        {
            if (input != 0)
            {
                Check(midiInStop(input), "stop WinMM input");
                Check(midiInReset(input), "reset WinMM input");
                Check(midiInClose(input), "close WinMM input");
                input = 0;
            }
        }
        finally
        {
            if (output != 0)
            {
                Check(midiOutClose(output), "close WinMM output");
                output = 0;
            }
            GC.KeepAlive(callback);
        }
    }

    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    private delegate void MidiInCallback(
        nint handle, uint message, nuint instance, nuint parameter, nuint timestamp);

    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInOpen(
        out nint handle, uint id, MidiInCallback callback, nuint instance, uint flags);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutOpen(
        out nint handle, uint id, nuint callback, nuint instance, uint flags);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInStart(nint handle);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInStop(nint handle);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInReset(nint handle);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInClose(nint handle);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutShortMsg(nint handle, uint message);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutClose(nint handle);
}
