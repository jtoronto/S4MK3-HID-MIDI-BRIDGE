using Windows.Devices.Midi2.Reporting;

namespace S4LoopBeProbe;

internal static class DjayObservation
{
    internal static async Task RunAsync(bool reportSessions = true, int seconds = 20)
    {
        using (var port = NativePort.Open(null))
        {
            if (reportSessions) Report();
            Program.Emit("listening", new { seconds, sendsMidi = false });
            await Task.Delay(TimeSpan.FromSeconds(seconds));
            if (reportSessions) Report();
            Program.Emit("capture-ended", new { messages = port.Messages });
        }
        Program.Emit("connection-closed", new { });
    }

    private static void Report() => Program.Emit("active-sessions", new
    {
        sessions = MidiReporting.GetActiveSessions().Select(session => new
        {
            session.ProcessId, session.ProcessName, session.SessionName,
            connections = session.Connections.Select(connection => new
            {
                connection.EndpointOrPortDeviceId, connection.InstanceCount
            })
        })
    });
}
