using Windows.Devices.Enumeration;
using Windows.Devices.Midi;

namespace S4LoopBeProbe;

internal static class LegacyWinRtCheck
{
    internal static async Task RunAsync()
    {
        var devices = await DeviceInformation.FindAllAsync(MidiOutPort.GetDeviceSelector())
            .AsTask().WaitAsync(TimeSpan.FromSeconds(15));
        var selected = devices.Single(device => device.Name == Program.PortName);
        Program.Emit("legacy-winrt-device", new { selected.Name, selected.Id });
        using (var receiver = NativePort.Open(0x007F14B7))
        using (var sender = await MidiOutPort.FromIdAsync(selected.Id)
            .AsTask().WaitAsync(TimeSpan.FromSeconds(15))
            ?? throw new InvalidOperationException("Legacy WinRT output did not open."))
        {
            sender.SendMessage(new MidiControlChangeMessage(7, 20, 127));
            Program.Emit("legacy-winrt-sent", new { packed = "0x007F14B7" });
            var received = await receiver.Received.WaitAsync(TimeSpan.FromSeconds(15));
            Program.Emit("legacy-winrt-received", new { packed = $"0x{received:X8}" });
        }
        Program.Emit("connection-closed", new { });
    }
}
