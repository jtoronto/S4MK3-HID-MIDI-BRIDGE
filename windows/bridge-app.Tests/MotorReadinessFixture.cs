namespace S4Bridge.Tests;

internal static class MotorReadinessFixture
{
    internal static async Task<int> Run()
    {
        Console.WriteLine(ProtocolTests.Status("full", "starting"));
        Console.WriteLine(ProtocolTests.Status("full"));
        Console.WriteLine("S4_SCREEN_DIAG_READY off");
        Console.Error.WriteLine("INFO MOTOR_PLAYBACK_GATE side=\"right\" deck=2 tt_enabled=true loaded=None playing=Some(true) internal=None thru=None playback_gate_open=false");
        Console.Error.WriteLine("FIXTURE_PARTIAL_GATE");
        while (await Console.In.ReadLineAsync() is { } line)
        {
            var fields = line.Split(' ');
            if (fields.Length == 3 && fields[0] == "S4_SCREEN_CONTROL" && fields[2] == "ON")
            {
                Console.Error.WriteLine("INFO MOTOR_PLAYBACK_GATE side=\"right\" deck=2 tt_enabled=true loaded=Some(true) playing=Some(true) internal=Some(true) thru=Some(false) playback_gate_open=true");
                Console.Error.WriteLine("FIXTURE_AUTHORIZED_GATE");
            }
            if (fields.Length == 3 && fields[0] == "S4_BRIDGE_CONTROL" && fields[2] == "STOP")
            {
                Console.WriteLine(ProtocolTests.Status("full", "stopping", fields[1]));
                return 0;
            }
        }
        return 0;
    }
}
