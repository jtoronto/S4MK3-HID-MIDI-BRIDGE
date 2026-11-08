using Windows.Devices.Midi2.Transports.Loopback;

namespace S4LoopbackProbe;

internal static class PairConfiguration
{
    internal const string NameA = "S4 Loopback Bridge";
    // The existing independent WinMM peer selects this unique original-probe prefix.
    internal const string NameB = "S4 MK3 Windows Probe LB";
    private const string IdA = "S4LOOP20261103A";
    private const string IdB = "S4LOOP20261103B";

    internal static void Inspect() =>
        Program.Emit("loopbacks", MidiLoopbackManager.GetActiveLoopbackEntries()
            .Select(Describe).ToArray());

    internal static MidiLoopbackEntry Require() =>
        Find() ?? throw new InvalidOperationException("The isolated loopback pair is absent.");

    private static MidiLoopbackEntry? Find() =>
        MidiLoopbackManager.GetActiveLoopbackEntries().SingleOrDefault(entry =>
            entry.EndpointA.Name == NameA && entry.EndpointB.Name == NameB &&
            entry.EndpointA.EndpointDeviceId.Contains(IdA, StringComparison.OrdinalIgnoreCase) &&
            entry.EndpointB.EndpointDeviceId.Contains(IdB, StringComparison.OrdinalIgnoreCase));

    internal static void Create()
    {
        if (MidiLoopbackManager.GetActiveLoopbackEntries().Any(entry =>
                entry.EndpointA.Name == NameA || entry.EndpointB.Name == NameB) ||
            MidiLoopbackManager.DoesLoopbackAExist(IdA) ||
            MidiLoopbackManager.DoesLoopbackBExist(IdB))
        {
            throw new InvalidOperationException("The test identity already exists; not overwriting.");
        }
        var a = new MidiLoopbackEndpointDefinition(NameA, "S4 bridge-side diagnostic", IdA)
        {
            CreateOnlyUmpEndpoint = false
        };
        var b = new MidiLoopbackEndpointDefinition(NameB, "S4 application-side diagnostic", IdB)
        {
            CreateOnlyUmpEndpoint = false
        };
        var response = MidiLoopbackManager.CreateTransientLoopback(
            new MidiLoopbackCreationConfig(a, b));
        Program.Emit("created", new
        {
            response.Success,
            code = response.ErrorCode.ToString(),
            response.ErrorMessage
        });
        if (!response.Success)
        {
            throw new InvalidOperationException("Loopback creation failed.");
        }
        Program.Emit("pair", Describe(response.CreatedLoopbackEntry));
    }

    internal static void Remove()
    {
        var entry = Find();
        if (entry is null)
        {
            Program.Emit("removed", new { alreadyAbsent = true });
            return;
        }
        Program.Emit("removing", Describe(entry));
        var response = MidiLoopbackManager.RemoveTransientLoopback(
            new MidiLoopbackRemovalConfig(entry.AssociationId));
        Program.Emit("removed", new
        {
            response.Success,
            code = response.ErrorCode.ToString(),
            response.ErrorMessage
        });
        if (!response.Success)
        {
            throw new InvalidOperationException("Loopback removal failed.");
        }
    }

    private static object Describe(MidiLoopbackEntry entry) => new
    {
        entry.AssociationId,
        a = new { entry.EndpointA.Name, entry.EndpointA.EndpointDeviceId },
        b = new { entry.EndpointB.Name, entry.EndpointB.EndpointDeviceId }
    };
}
