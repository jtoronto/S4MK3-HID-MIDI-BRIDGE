using Windows.Devices.Midi2.Reporting;

namespace S4MidiProbe;

internal static class Probe
{
    internal static async Task RunAsync(TimeSpan? listenDuration)
    {
        using var observation = new EndpointObservation();
        await observation.StartAsync();
        var returned = new TaskCompletionSource<uint>(
            TaskCreationOptions.RunContinuationsAsynchronously);
        var messageCount = 0;
        void OnMessage(uint word)
        {
            if ((word >> 28) == 2)
            {
                Interlocked.Increment(ref messageCount);
                Program.Emit("device-received", new { ump = $"0x{word:X8}" });
                if (word == 0x20BF6F48)
                {
                    returned.TrySetResult(word);
                }
            }
        }

        VirtualController? controller = null;
        try
        {
            controller = VirtualController.Create(OnMessage);
            await observation.WaitForArrivalAsync();
            if (listenDuration is { } duration)
            {
                Program.Emit("listening", new
                {
                    name = VirtualController.Name,
                    seconds = duration.TotalSeconds,
                    sendsMidi = false
                });
                ReportClients(observation.EndpointId);
                // This is the requested capture duration, not a synchronization delay.
                await Task.Delay(duration);
                ReportClients(observation.EndpointId);
                Program.Emit("capture-ended", new { messages = messageCount });
            }
            else
            {
                await PeerProcess.VerifyAsync(controller, returned.Task);
            }
        }
        catch (Exception error)
        {
            Program.Emit("operation-failure", new { error.Message, detail = error.ToString() });
            throw;
        }
        finally
        {
            controller?.Dispose();
            await observation.WaitForRemovalAsync();
        }
        Program.Emit("passed", new { mode = listenDuration is null ? "self-test" : "listen" });
    }

    private static void ReportClients(string endpointId)
    {
        var sessions = MidiReporting.FindAllSessionsWithMatchingOpenUmpEndpoint(endpointId, true);
        Program.Emit("connected-clients", sessions.Select(session => new
        {
            session.ProcessId,
            session.ProcessName,
            session.SessionName,
            connections = session.Connections.Select(connection => new
            {
                connection.EndpointOrPortDeviceId,
                connection.InstanceCount
            })
        }).ToArray());
    }
}
