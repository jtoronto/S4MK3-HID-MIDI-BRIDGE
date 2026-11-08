using Windows.Devices.Midi2.Enumeration;
using Windows.Devices.Midi2.Enumeration.Legacy;

namespace S4MidiProbe;

internal sealed class EndpointObservation : IDisposable
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(20);
    private readonly MidiEndpointDeviceWatcher endpoints =
        MidiEndpointDeviceWatcher.Create(MidiEndpointDeviceInformationFilters.AllStandardEndpoints);
    private readonly MidiLegacyPortDeviceWatcher ports = MidiLegacyPortDeviceWatcher.Create();
    private readonly TaskCompletionSource endpointEnumeration = NewSignal();
    private readonly TaskCompletionSource portEnumeration = NewSignal();
    private readonly TaskCompletionSource<string> endpoint =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource<uint> source =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource<uint> destination =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource endpointRemoved = NewSignal();
    private readonly TaskCompletionSource sourceRemoved = NewSignal();
    private readonly TaskCompletionSource destinationRemoved = NewSignal();

    internal string EndpointId => endpoint.Task.GetAwaiter().GetResult();
    internal uint SourceNumber => source.Task.GetAwaiter().GetResult();
    internal uint DestinationNumber => destination.Task.GetAwaiter().GetResult();

    internal EndpointObservation()
    {
        endpoints.Added += OnEndpointAdded;
        endpoints.Updated += OnEndpointUpdated;
        endpoints.Removed += OnEndpointRemoved;
        endpoints.EnumerationCompleted += OnEndpointEnumeration;
        ports.Added += OnPortAdded;
        ports.Removed += OnPortRemoved;
        ports.EnumerationCompleted += OnPortEnumeration;
    }

    internal async Task StartAsync()
    {
        endpoints.Start();
        ports.Start();
        await Task.WhenAll(endpointEnumeration.Task, portEnumeration.Task).WaitAsync(Timeout);
        if (endpoint.Task.IsCompleted || source.Task.IsCompleted || destination.Task.IsCompleted)
        {
            throw new InvalidOperationException(
                "An S4 probe endpoint already exists. Stop its owning probe before running another.");
        }
    }

    internal async Task WaitForArrivalAsync()
    {
        await Task.WhenAll(endpoint.Task, source.Task, destination.Task).WaitAsync(Timeout);
        Program.Emit("ports-ready", new
        {
            name = VirtualController.Name,
            endpointId = EndpointId,
            winmmInput = SourceNumber,
            winmmOutput = DestinationNumber
        });
    }

    internal async Task WaitForRemovalAsync()
    {
        var removals = new List<Task>();
        if (endpoint.Task.IsCompletedSuccessfully)
        {
            removals.Add(endpointRemoved.Task);
        }
        if (source.Task.IsCompletedSuccessfully)
        {
            removals.Add(sourceRemoved.Task);
        }
        if (destination.Task.IsCompletedSuccessfully)
        {
            removals.Add(destinationRemoved.Task);
        }
        await Task.WhenAll(removals).WaitAsync(Timeout);
        Program.Emit("cleanup", new
        {
            endpointRemoved = endpointRemoved.Task.IsCompletedSuccessfully,
            sourceRemoved = sourceRemoved.Task.IsCompletedSuccessfully,
            destinationRemoved = destinationRemoved.Task.IsCompletedSuccessfully
        });
    }

    private void ObserveEndpoint(MidiEndpointDeviceInformation information)
    {
        if (Matches(information.Name))
        {
            endpoint.TrySetResult(information.EndpointDeviceId);
        }
    }

    private void ObservePort(MidiLegacyPortDeviceInformation information)
    {
        if (!Matches(information.Name))
        {
            return;
        }
        Program.Emit("legacy-port", new
        {
            information.Name,
            flow = information.Flow.ToString(),
            information.Number,
            information.AssociatedEndpointDeviceId
        });
        switch (information.Flow)
        {
            case Midi1PortFlow.MidiMessageSource:
                source.TrySetResult(information.Number);
                break;
            case Midi1PortFlow.MidiMessageDestination:
                destination.TrySetResult(information.Number);
                break;
            default:
                throw new InvalidOperationException($"Unknown MIDI port flow: {information.Flow}");
        }
    }

    private static bool Matches(string name) =>
        name.StartsWith(VirtualController.Name, StringComparison.Ordinal);

    private static TaskCompletionSource NewSignal() =>
        new(TaskCreationOptions.RunContinuationsAsynchronously);

    private void OnEndpointAdded(
        MidiEndpointDeviceWatcher sender, MidiEndpointDeviceInformationAddedEventArgs args) =>
        ObserveEndpoint(args.AddedDevice);

    private void OnEndpointUpdated(
        MidiEndpointDeviceWatcher sender, MidiEndpointDeviceInformationUpdatedEventArgs args) =>
        ObserveEndpoint(args.UpdatedDevice);

    private void OnEndpointRemoved(
        MidiEndpointDeviceWatcher sender, MidiEndpointDeviceInformationRemovedEventArgs args)
    {
        if (endpoint.Task.IsCompletedSuccessfully &&
            string.Equals(args.RemovedDevice.EndpointDeviceId, EndpointId,
                StringComparison.OrdinalIgnoreCase))
        {
            endpointRemoved.TrySetResult();
        }
    }

    private void OnPortAdded(
        MidiLegacyPortDeviceWatcher sender, MidiLegacyPortDeviceInformationAddedEventArgs args) =>
        ObservePort(args.AddedDevice);

    private void OnPortRemoved(
        MidiLegacyPortDeviceWatcher sender, MidiLegacyPortDeviceInformationRemovedEventArgs args)
    {
        if (Matches(args.RemovedDevice.Name))
        {
            switch (args.RemovedDevice.Flow)
            {
                case Midi1PortFlow.MidiMessageSource:
                    sourceRemoved.TrySetResult();
                    break;
                case Midi1PortFlow.MidiMessageDestination:
                    destinationRemoved.TrySetResult();
                    break;
                default:
                    throw new InvalidOperationException("Unknown removed MIDI port flow.");
            }
        }
    }

    private void OnEndpointEnumeration(MidiEndpointDeviceWatcher sender, object args) =>
        endpointEnumeration.TrySetResult();

    private void OnPortEnumeration(MidiLegacyPortDeviceWatcher sender, object args) =>
        portEnumeration.TrySetResult();

    public void Dispose()
    {
        ports.Stop();
        endpoints.Stop();
        endpoints.Added -= OnEndpointAdded;
        endpoints.Updated -= OnEndpointUpdated;
        endpoints.Removed -= OnEndpointRemoved;
        endpoints.EnumerationCompleted -= OnEndpointEnumeration;
        ports.Added -= OnPortAdded;
        ports.Removed -= OnPortRemoved;
        ports.EnumerationCompleted -= OnPortEnumeration;
    }
}
