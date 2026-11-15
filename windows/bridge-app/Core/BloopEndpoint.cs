namespace S4Bridge.Core;

public static class BloopEndpoint
{
    public const string Backend = "bloop";
    public const string PortName = "S4 MK3 Bridge BLOOP";
    public const string UniqueId = "s4mk3-hid-midi-bridge-bloop-v1";
}

public interface IBloopEndpointOwner
{
    Task<IBloopEndpointLease> PrepareAsync();
}

public interface IBloopEndpointLease : IAsyncDisposable
{
    Guid AssociationId { get; }
    string? EndpointDeviceId { get; }
    Task CreateAsync();
    void RecordWorker(int processId, DateTime startTimeUtc);
}
