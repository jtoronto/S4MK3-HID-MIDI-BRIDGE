using S4Bridge.Core;
using Windows.Devices.Midi2;
using Windows.Devices.Midi2.Transports.BasicLoopback;

namespace S4Bridge.Platform;

public static class BloopDependencyProbe
{
    // The SDK can start Windows MIDI Services. A negative transport check does not
    // distinguish an absent transport from an SDK or service reporting failure.
    public static BloopDependencyProbeResult Check()
    {
        try
        {
            if (!MidiApi.EnsureServiceAvailable())
                return new(BloopDependencyStatus.ServiceUnavailable,
                    "Windows MIDI Services is unavailable.");
            if (!MidiBasicLoopbackManager.IsTransportAvailable)
                return new(BloopDependencyStatus.TransportUnavailable,
                    "Basic Loopback transport is unavailable or its availability could not be confirmed.");
            return new(BloopDependencyStatus.Available);
        }
        catch (Exception ex)
        {
            return new(BloopDependencyStatus.CheckFailed, ex.ToString());
        }
    }
}
