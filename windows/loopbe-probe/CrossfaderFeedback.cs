namespace S4LoopBeProbe;

internal sealed class CrossfaderFeedback
{
    // Output is channel 8 CC 20; input is channel 5 CC 20. Channels are human-based.
    private readonly object gate = new();
    private byte? value;
    private int accepted;

    internal (byte? Value, int Accepted) Snapshot
    {
        get { lock (gate) { return (value, accepted); } }
    }

    internal bool Apply(uint packed)
    {
        lock (gate)
        {
            var before = value;
            var data = (packed >> 16) & 0xFF;
            var isFeedback = (packed & 0xFFFF) == 0x14B7 && data <= 127;
            if (isFeedback)
            {
                value = checked((byte)data);
                accepted++;
            }
            Program.Emit("crossfader-route", new { packed = $"0x{packed:X8}", isFeedback,
                before, after = value, accepted });
            return isFeedback;
        }
    }

    internal static void Check()
    {
        var route = new CrossfaderFeedback();
        foreach (var packet in new uint[] { 0x007F14B4, 0x000014B4, 0x007F15B7, 0x007F1497 })
        {
            if (route.Apply(packet) || route.Snapshot != (null, 0))
            {
                throw new InvalidOperationException("Input echo or unrelated channel updated feedback.");
            }
        }
        if (!route.Apply(0x007F14B7) || route.Snapshot != ((byte?)127, 1)
            || !route.Apply(0x004014B7) || route.Snapshot != ((byte?)64, 2)
            || !route.Apply(0x000014B7) || route.Snapshot != ((byte?)0, 3))
        {
            throw new InvalidOperationException("Dedicated crossfader feedback was not decoded.");
        }
        if (route.Apply(0x007F14B4) || route.Snapshot != ((byte?)0, 3))
        {
            throw new InvalidOperationException("Echo overwrote confirmed software state.");
        }
        Program.Emit("route-check-passed", new { });
    }
}
