using System.Runtime.InteropServices;

namespace S4LoopBeProbe;

internal sealed class NativePort : IDisposable
{
    internal sealed record Port(bool Input, uint Index, string Name, uint Version);
    private readonly MidiCallback callback;
    private readonly uint? expected;
    private readonly TaskCompletionSource<uint> received =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private nint input;
    private nint output;
    private int messages;

    private NativePort(uint? expected)
    {
        this.expected = expected;
        callback = OnMessage;
    }

    internal Task<uint> Received => received.Task;
    internal int Messages => Volatile.Read(ref messages);
    internal event Action<uint>? PacketReceived;

    internal static IReadOnlyList<Port> Enumerate()
    {
        var ports = new List<Port>();
        for (uint i = 0; i < midiInGetNumDevs(); i++)
        {
            Check(midiInGetDevCapsW(i, out var caps, (uint)Marshal.SizeOf<InputCaps>()));
            ports.Add(new(true, i, caps.Name, caps.Version));
        }
        for (uint i = 0; i < midiOutGetNumDevs(); i++)
        {
            Check(midiOutGetDevCapsW(i, out var caps, (uint)Marshal.SizeOf<OutputCaps>()));
            ports.Add(new(false, i, caps.Name, caps.Version));
        }
        return ports;
    }

    internal static NativePort Open(uint? expected)
    {
        var ports = Enumerate().Where(port => port.Name == Program.PortName).ToArray();
        var source = ports.Single(port => port.Input);
        var destination = ports.Single(port => !port.Input);
        var owner = new NativePort(expected);
        try
        {
            Check(midiInOpen(out owner.input, source.Index, owner.callback, 0, 0x00030000));
            Check(midiOutOpen(out owner.output, destination.Index, 0, 0, 0));
            Check(midiInStart(owner.input));
            Program.Emit("connection-open", new { name = Program.PortName,
                input = source.Index, output = destination.Index });
            return owner;
        }
        catch
        {
            owner.Dispose();
            throw;
        }
    }

    internal void Send(uint packed)
    {
        Check(midiOutShortMsg(output, packed));
        Program.Emit("sent", new { packed = $"0x{packed:X8}" });
    }

    private void OnMessage(nint handle, uint message, nuint instance, nuint value, nuint timestamp)
    {
        if (message == 0x3C3)
        {
            var packed = checked((uint)value);
            Interlocked.Increment(ref messages);
            Program.Emit("received", new { packed = $"0x{packed:X8}" });
            PacketReceived?.Invoke(packed);
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
