namespace S4Bridge.Core;

public static class BloopEndpoint
{
    public const string Backend = "bloop";
    public const string PortName = "S4 MK3 Bridge BLOOP";
    public const string UniqueId = "s4mk3-hid-midi-bridge-bloop-v1";
    public const string Preview14InstallerUrl = "https://github.com/microsoft/MIDI/releases/download/inbox-preview-14/Windows.MIDI.Services.Transports.1.0.52-preview.14-x64.exe";
    public const string Preview14ReleaseUrl = "https://github.com/microsoft/MIDI/releases/tag/inbox-preview-14";
}

public enum BloopDependencyStatus
{
    Available,
    ServiceUnavailable,
    TransportUnavailable,
    CheckFailed
}

public sealed record BloopDependencyProbeResult(BloopDependencyStatus Status, string? Detail = null);

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
