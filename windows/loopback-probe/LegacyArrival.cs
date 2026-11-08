using Windows.Devices.Midi2.Enumeration;
using Windows.Devices.Midi2.Enumeration.Legacy;

namespace S4LoopbackProbe;

internal sealed class LegacyArrival : IDisposable
{
    private readonly MidiLegacyPortDeviceWatcher watcher = MidiLegacyPortDeviceWatcher.Create();
    private readonly HashSet<(string, Midi1PortFlow)> seen = [];
    private readonly TaskCompletionSource arrived =
        new(TaskCreationOptions.RunContinuationsAsynchronously);

    internal LegacyArrival()
    {
        watcher.Added += OnAdded;
        watcher.Start();
    }

    internal Task WaitAsync() => arrived.Task.WaitAsync(TimeSpan.FromSeconds(20));

    private void OnAdded(
        MidiLegacyPortDeviceWatcher sender, MidiLegacyPortDeviceInformationAddedEventArgs args)
    {
        var port = args.AddedDevice;
        var name = port.Name.StartsWith(PairConfiguration.NameA, StringComparison.Ordinal)
            ? PairConfiguration.NameA
            : port.Name.StartsWith(PairConfiguration.NameB, StringComparison.Ordinal)
                ? PairConfiguration.NameB : null;
        if (name is null)
        {
            return;
        }
        Program.Emit("legacy-port", new { port.Name, flow = port.Flow.ToString(), port.Number });
        lock (seen)
        {
            seen.Add((name, port.Flow));
            if (seen.Count == 4)
            {
                arrived.TrySetResult();
            }
        }
    }

    public void Dispose()
    {
        watcher.Stop();
        watcher.Added -= OnAdded;
    }
}
