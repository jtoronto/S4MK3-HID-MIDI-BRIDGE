using System.Runtime.InteropServices;

namespace S4MidiProbe;

internal static class WinMmPorts
{
    internal sealed record Port(uint Number, string Name);

    internal static (Port Source, Port Destination) FindProbe()
    {
        var sources = new List<Port>();
        var destinations = new List<Port>();
        var inputCount = midiInGetNumDevs();
        var outputCount = midiOutGetNumDevs();
        for (uint number = 0; number < inputCount; number++)
        {
            WinMmPeer.Check(midiInGetDevCapsW(number, out var caps,
                checked((uint)Marshal.SizeOf<MidiInCaps>())), "enumerate WinMM input");
            sources.Add(new Port(number, caps.Name));
        }
        for (uint number = 0; number < outputCount; number++)
        {
            WinMmPeer.Check(midiOutGetDevCapsW(number, out var caps,
                checked((uint)Marshal.SizeOf<MidiOutCaps>())), "enumerate WinMM output");
            destinations.Add(new Port(number, caps.Name));
        }
        Program.Emit("winmm-ports", new { sources, destinations });
        var source = sources.SingleOrDefault(port => Matches(port.Name))
            ?? throw new InvalidOperationException("The probe is absent from native WinMM inputs.");
        var destination = destinations.SingleOrDefault(port => Matches(port.Name))
            ?? throw new InvalidOperationException("The probe is absent from native WinMM outputs.");
        return (source, destination);
    }

    private static bool Matches(string name) =>
        name.StartsWith(VirtualController.Name, StringComparison.Ordinal);

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct MidiInCaps
    {
        public ushort Manufacturer;
        public ushort Product;
        public uint Version;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string Name;
        public uint Support;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct MidiOutCaps
    {
        public ushort Manufacturer;
        public ushort Product;
        public uint Version;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string Name;
        public ushort Technology;
        public ushort Voices;
        public ushort Notes;
        public ushort ChannelMask;
        public uint Support;
    }

    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInGetNumDevs();
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutGetNumDevs();
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiInGetDevCapsW(nuint id, out MidiInCaps caps, uint size);
    [DllImport("winmm.dll", ExactSpelling = true)]
    private static extern uint midiOutGetDevCapsW(nuint id, out MidiOutCaps caps, uint size);
}
