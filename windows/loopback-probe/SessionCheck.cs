using Windows.Devices.Midi2;
using Windows.Devices.Midi2.Reporting;

namespace S4LoopbackProbe;

internal static class SessionCheck
{
    internal static async Task RunAsync(bool listen)
    {
        var pair = PairConfiguration.Require();
        using var arrival = new LegacyArrival();
        await arrival.WaitAsync();
        var returned = new TaskCompletionSource<uint>(
            TaskCreationOptions.RunContinuationsAsynchronously);
        var messages = 0;
        void Received(IMidiMessageReceivedEventSource sender, MidiMessageReceivedEventArgs args)
        {
            var count = args.FillWords(out var word, out var word1, out var word2, out var word3);
            Program.Emit("raw-received", new
            {
                count,
                words = new[] { word, word1, word2, word3 }.Select(value => $"0x{value:X8}")
            });
            if ((word >> 28) == 2)
            {
                Interlocked.Increment(ref messages);
                Program.Emit("received", new { ump = $"0x{word:X8}" });
                if (word == 0x20BF6F48)
                {
                    returned.TrySetResult(word);
                }
            }
        }
        var session = MidiSession.Create("S4 loopback connection test")
            ?? throw new InvalidOperationException("Session creation failed.");
        var connection = session.CreateEndpointConnection(pair.EndpointA.EndpointDeviceId)
            ?? throw new InvalidOperationException("Endpoint connection creation failed.");
        connection.MessageReceived += Received;
        try
        {
            if (!connection.Open())
            {
                throw new InvalidOperationException("Loopback connection did not open.");
            }
            Program.Emit("connection-open", new { pair.AssociationId, listen });
            if (listen)
            {
                ReportClients(pair.EndpointA.EndpointDeviceId);
                ReportClients(pair.EndpointB.EndpointDeviceId);
                Program.Emit("listening", new { seconds = 20, sendsMidi = false });
                await Task.Delay(TimeSpan.FromSeconds(20));
                ReportClients(pair.EndpointA.EndpointDeviceId);
                ReportClients(pair.EndpointB.EndpointDeviceId);
                Program.Emit("capture-ended", new { messages });
            }
            else
            {
                await IndependentPeer.VerifyAsync(() =>
                {
                    var send = connection.SendSingleMessageWords(MidiClock.Now, 0x20917735);
                    if (!MidiEndpointConnection.SendMessageSucceeded(send))
                    {
                        throw new InvalidOperationException($"Loopback send failed: {send}");
                    }
                }, returned.Task);
            }
        }
        finally
        {
            connection.MessageReceived -= Received;
            Program.Emit("connection-closing", new { pair.AssociationId });
            session.Dispose();
            Program.Emit("connection-closed", new { pair.AssociationId });
        }
        var retained = PairConfiguration.Require();
        if (retained.AssociationId != pair.AssociationId)
        {
            throw new InvalidOperationException("The pair changed during connection closure.");
        }
        Program.Emit("passed", new { pairRetained = true, pair.AssociationId });
    }

    internal static async Task RunNativeAsync(bool note = false)
    {
        var pair = PairConfiguration.Require();
        using var arrival = new LegacyArrival();
        await arrival.WaitAsync();
        Program.Emit("native-connection-opening", new { pair.AssociationId });
        using (var owner = NativePeer.Open(PairConfiguration.NameA,
            note ? 0x00357791u : 0x00486FBFu))
        {
            await IndependentPeer.VerifyAsync(() => owner.Send(0x00357791), owner.Received, note);
            Program.Emit("native-connection-closing", new { pair.AssociationId });
        }
        Program.Emit("native-connection-closed", new { pair.AssociationId });
        if (PairConfiguration.Require().AssociationId != pair.AssociationId)
        {
            throw new InvalidOperationException("Pair changed after WinMM closure.");
        }
        Program.Emit("passed", new { pairRetained = true, mode = "WinMM", pair.AssociationId });
    }

    private static void ReportClients(string endpoint) =>
        Program.Emit("connected-clients", new
        {
            endpoint,
            sessions = MidiReporting.FindAllSessionsWithMatchingOpenUmpEndpoint(endpoint, true)
                .Select(session => new { session.ProcessId, session.ProcessName,
                    session.SessionName, connections = session.Connections.Select(connection =>
                        new { connection.EndpointOrPortDeviceId, connection.InstanceCount }) })
        });
}
