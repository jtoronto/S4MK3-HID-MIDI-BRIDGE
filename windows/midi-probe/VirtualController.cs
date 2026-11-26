using Windows.Devices.Midi2;
using Windows.Devices.Midi2.Enumeration;
using Windows.Devices.Midi2.Transports.Virtual;

namespace S4MidiProbe;

internal sealed class VirtualController : IDisposable
{
    internal const string Name = "S4 MK3 Windows Probe";
    private readonly MidiSession session;
    private readonly MidiVirtualDevice device;
    private readonly MidiEndpointConnection connection;
    private readonly Action<uint> received;
    private bool disposed;

    private VirtualController(
        MidiSession session,
        MidiVirtualDevice device,
        MidiEndpointConnection connection,
        Action<uint> received)
    {
        this.session = session;
        this.device = device;
        this.connection = connection;
        this.received = received;
    }

    internal static VirtualController Create(Action<uint> received)
    {
        if (!MidiVirtualDeviceManager.IsTransportAvailable)
        {
            throw new InvalidOperationException("The Windows virtual MIDI transport is unavailable.");
        }

        var declaration = new MidiDeclaredEndpointInfo
        {
            Name = Name,
            ProductInstanceId = "S4PROBE20261103",
            SpecificationVersionMajor = 1,
            SpecificationVersionMinor = 1,
            SupportsMidi10Protocol = true,
            SupportsMidi20Protocol = false,
            HasStaticFunctionBlocks = true
        };
        var config = new MidiVirtualDeviceCreationConfig(
            Name, "Software-only S4 bridge feasibility test", "S4 MK3 Bridge", declaration)
        {
            CreateOnlyUmpEndpoints = false
        };
        config.FunctionBlocks.Add(new MidiFunctionBlock
        {
            Number = 0,
            Name = Name,
            IsActive = true,
            FirstGroup = new MidiGroup(0),
            GroupCount = 1,
            Direction = MidiFunctionBlockDirection.Bidirectional,
            RepresentsMidi10Connection =
                MidiFunctionBlockRepresentsMidi10Connection.YesBandwidthUnrestricted
        });

        var session = MidiSession.Create(Name)
            ?? throw new InvalidOperationException("Could not create the MIDI session.");
        VirtualController? controller = null;
        try
        {
            var device = MidiVirtualDeviceManager.CreateVirtualDevice(config)
                ?? throw new InvalidOperationException("Could not create the virtual device.");
            var connection = session.CreateEndpointConnection(device.DeviceEndpointDeviceId)
                ?? throw new InvalidOperationException("Could not create the device-side connection.");
            controller = new VirtualController(session, device, connection, received);
            if (connection.AddMessageProcessingPlugin(device) !=
                MidiMessageProcessingPluginAddResult.Succeeded)
            {
                throw new InvalidOperationException("Could not attach the virtual-device responder.");
            }
            device.SuppressHandledMessages = true;
            connection.MessageReceived += controller.OnMessage;
            device.ClientEndpointInUseChanged += OnClientUseChanged;
            if (!connection.Open())
            {
                throw new InvalidOperationException("Could not open the device-side connection.");
            }
            Program.Emit("device-open", new
            {
                name = Name,
                device.DeviceEndpointDeviceId,
                device.AssociationId,
                config.CreateOnlyUmpEndpoints
            });
            return controller;
        }
        catch
        {
            if (controller is null)
            {
                session.Dispose();
            }
            else
            {
                controller.Dispose();
            }
            throw;
        }
    }

    internal void Send(uint word)
    {
        var result = connection.SendSingleMessageWords(MidiClock.Now, word);
        if (!MidiEndpointConnection.SendMessageSucceeded(result))
        {
            throw new InvalidOperationException($"MIDI send failed: {result}");
        }
        Program.Emit("device-send", new { ump = $"0x{word:X8}" });
    }

    private void OnMessage(IMidiMessageReceivedEventSource sender, MidiMessageReceivedEventArgs args) =>
        received(args.PeekFirstWord());

    private static void OnClientUseChanged(
        MidiVirtualDevice sender, MidiVirtualDeviceClientEndpointInUseChangedEventArgs args) =>
        Program.Emit("client-use", new { inUse = sender.IsClientEndpointInUse });

    public void Dispose()
    {
        if (disposed)
        {
            return;
        }
        disposed = true;
        try
        {
            Program.Emit("device-disconnecting", new { name = Name });
            device.ClientEndpointInUseChanged -= OnClientUseChanged;
            connection.MessageReceived -= OnMessage;
            connection.RemoveMessageProcessingPlugin(device.PluginId);
            Program.Emit("responder-detached", new { name = Name });
        }
        finally
        {
            Program.Emit("session-closing", new { name = Name });
            session.Dispose();
            GC.KeepAlive(device);
        }
        Program.Emit("device-closed", new { name = Name });
    }
}
