using System.Runtime.InteropServices;

namespace S4LoopbackProbe;

internal sealed class NativePeer : IDisposable
{
    private readonly MidiCallback callback;
    private readonly uint expected;
    private readonly TaskCompletionSource<uint> received =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private nint input;
    private nint output;

    private NativePeer(uint expected)
    {
        this.expected = expected;
        callback = OnMessage;
    }

    internal static NativePeer Open(string name, uint expected)
    {
        var peer = new NativePeer(expected);
        try
        {
            for (uint i = 0; i < midiInGetNumDevs(); i++)
            {
                Check(midiInGetDevCapsW(i, out var caps, (uint)Marshal.SizeOf<InputCaps>()));
                if (caps.Name == name)
                {
                    Check(midiInOpen(out peer.input, i, peer.callback, 0, 0x00030000));
                }
            }
            for (uint i = 0; i < midiOutGetNumDevs(); i++)
            {
                Check(midiOutGetDevCapsW(i, out var caps, (uint)Marshal.SizeOf<OutputCaps>()));
                if (caps.Name == name)
                {
                    Check(midiOutOpen(out peer.output, i, 0, 0, 0));
                }
            }
            if (peer.input == 0 || peer.output == 0)
            {
                throw new InvalidOperationException($"The exact WinMM ports are absent: {name}");
            }
            Check(midiInStart(peer.input));
            return peer;
        }
        catch
        {
            peer.Dispose();
            throw;
        }
    }

    internal Task<uint> Received => received.Task;
    internal void Send(uint packed) => Check(midiOutShortMsg(output, packed));

    internal static async Task RunAsync(bool note = false)
    {
        using var peer = Open(PairConfiguration.NameB, 0x00357791);
        Program.Emit("peer-ready", new { pid = Environment.ProcessId });
        await peer.received.Task.WaitAsync(TimeSpan.FromSeconds(10));
        var reverse = note ? 0x00357791u : 0x00486FBFu;
        peer.Send(reverse);
        Program.Emit("peer-return-sent", new { packed = $"0x{reverse:X8}", handlesHeldOpen = true });
        var ack = await Task.Run(Console.ReadLine).WaitAsync(TimeSpan.FromSeconds(20));
        if (ack != "received")
        {
            throw new InvalidOperationException("Parent did not acknowledge receipt.");
        }
        Program.Emit("peer-passed", new { acknowledged = true });
    }

    private void OnMessage(nint handle, uint message, nuint instance, nuint value, nuint timestamp)
    {
        if (message == 0x3C3)
        {
            var packed = checked((uint)value);
            Program.Emit("winmm-received", new { packed = $"0x{packed:X8}" });
            if (packed == expected)
            {
                received.TrySetResult(packed);
            }
        }
    }

    private static void Check(uint result)
    {
        if (result != 0)
        {
            throw new InvalidOperationException($"WinMM error {result}");
        }
    }

    public void Dispose()
    {
        try
        {
            if (input != 0)
            {
                Check(midiInStop(input));
                Check(midiInReset(input));
                Check(midiInClose(input));
            }
        }
        finally
        {
            if (output != 0)
            {
                Check(midiOutClose(output));
            }
            GC.KeepAlive(callback);
        }
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct InputCaps
    {
        public ushort Manufacturer, Product;
        public uint Version;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string Name;
        public uint Support;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct OutputCaps
    {
        public ushort Manufacturer, Product;
        public uint Version;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string Name;
        public ushort Technology, Voices, Notes, ChannelMask;
        public uint Support;
    }

    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    private delegate void MidiCallback(nint h, uint m, nuint instance, nuint value, nuint time);
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiInGetNumDevs();
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiOutGetNumDevs();
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInGetDevCapsW(nuint id, out InputCaps caps, uint size);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutGetDevCapsW(nuint id, out OutputCaps caps, uint size);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInOpen(out nint h, uint id, MidiCallback cb, nuint user, uint flags);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutOpen(out nint h, uint id, nuint cb, nuint user, uint flags);
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiInStart(nint h);
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiInStop(nint h);
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiInReset(nint h);
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiInClose(nint h);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutShortMsg(nint h, uint value);
    [DllImport("winmm.dll", ExactSpelling = true)] private static extern uint midiOutClose(nint h);
}
