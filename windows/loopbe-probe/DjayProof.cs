namespace S4LoopBeProbe;

internal static class DjayProof
{
    internal static async Task RunAsync(byte expected, bool sendControl)
    {
        var route = new CrossfaderFeedback();
        var feedback = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var echo = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        void Received(uint packed)
        {
            if (route.Apply(packed) && route.Snapshot.Value == expected)
            {
                feedback.TrySetResult();
            }
            if (packed == (0x14B4u | ((uint)expected << 16)))
            {
                echo.TrySetResult();
            }
        }
        using (var port = NativePort.Open(null))
        {
            port.PacketReceived += Received;
            try
            {
                Program.Emit("crossfader-ready", new { expected, sendControl,
                    feedbackAddress = "channel 8 CC 20", controlAddress = "channel 5 CC 20" });
                if (sendControl)
                {
                    port.Send(0x14B4u | ((uint)expected << 16));
                    await echo.Task.WaitAsync(TimeSpan.FromSeconds(15));
                }
                await feedback.Task.WaitAsync(TimeSpan.FromSeconds(sendControl ? 20 : 120));
                Program.Emit("crossfader-feedback-confirmed", new { expected, sendControl,
                    accepted = route.Snapshot.Accepted });
            }
            finally
            {
                port.PacketReceived -= Received;
            }
        }
        Program.Emit("connection-closed", new { });
    }
}
