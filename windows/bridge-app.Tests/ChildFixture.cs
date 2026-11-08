using System.Text.Json.Nodes;
using System.Text.Json;
using System.Xml.Linq;

namespace S4Bridge.Tests;

// Test-only downstream executable. Never copied to a host package or labeled Rust.
internal static class ChildFixture
{
    public static async Task<int> Run(string[] args)
    {
        if (args.Contains("--generate-mapping")) return Generate(args);
        var profile = args.ElementAtOrDefault(1) ?? "full";
        if (args[0] == "exit7") { Console.WriteLine("SUCCESS-LOOKING OUTPUT"); return 7; }
        if (args[0] == "motor-stderr")
        {
            Console.Error.WriteLine(ProtocolTests.Status(profile));
            Console.WriteLine("STDERR_STATUS_SENT");
            Console.WriteLine(ProtocolTests.Status(profile, "starting"));
            if (await Console.In.ReadLineAsync() is null) return 0;
            Console.WriteLine(ProtocolTests.Status(profile));
            while (await Console.In.ReadLineAsync() is { } line)
            {
                var fields = line.Split(' ');
                if (fields.Length == 3 && fields[0] == "S4_BRIDGE_CONTROL" && fields[2] == "STOP")
                {
                    Console.WriteLine("S4_MOTOR_CONFIG {\"jog\":{\"platter_target_rpm\":33}}");
                    Console.Error.WriteLine("2026-11-04T00:00:00.000000Z DEBUG MOTOR_JOG_BEND side=\"left\" deck=1 steps=-2");
                    Console.Error.WriteLine("2026-11-04T00:00:00.020000Z DEBUG MOTOR_REGULATOR data={\"host_us\":20000}");
                    Console.Error.WriteLine("2026-11-04T00:00:00.030000Z DEBUG UNRELATED_STDERR_NOISE ignored=true");
                    Console.WriteLine("STOP_BURST_SENT");
                    Console.WriteLine(ProtocolTests.Status(profile, "stopping", fields[1]));
                    Console.WriteLine("STOP_ACK");
                    Console.Error.WriteLine("TAIL_DRAINED");
                    return 0;
                }
            }
            Console.WriteLine(ProtocolTests.Status(profile, "stopping"));
            Console.WriteLine("EOF_ACK"); return 0;
        }
        if (args[0] == "stderr")
        {
            Console.Error.WriteLine(ProtocolTests.Status(profile));
            Console.WriteLine("STDERR_STATUS_SENT");
            await Console.In.ReadLineAsync(); return 0;
        }
        Console.WriteLine(ProtocolTests.Status(profile, "starting"));
        if (args[0] == "capture-lifecycle")
            Console.WriteLine("S4_MIDI_CAPTURE_STATUS " + JsonSerializer.Serialize(new
                { version = 1, state = "started", path = args[2], max_file_bytes = 1048576 }));
        Console.WriteLine(ProtocolTests.Status(profile));
        Console.WriteLine("S4_MOTOR_CONFIG {\"jog\":{\"platter_target_rpm\":33}}");
        while (await Console.In.ReadLineAsync() is { } line)
        {
            if (args[0] == "capture-pipes") Console.WriteLine("INPUT " + line);
            var fields = line.Split(' ');
            if (args[0] == "capture-lifecycle" && fields.Length == 3 &&
                fields[0] == "S4_MIDI_CAPTURE_MARK" && fields[2] == "issue")
                Console.WriteLine("S4_MIDI_CAPTURE_STATUS " + JsonSerializer.Serialize(new
                    { version = 1, state = "marker", request_id = fields[1], accepted = true,
                      sequence = 0, elapsed_us = 1, reason = (string?)null }));
            if (fields.Length == 3 && fields[0] == "S4_BRIDGE_CONTROL" && fields[2] == "STOP")
            {
                if (args[0] == "hang") { Console.WriteLine("STOP_IGNORED"); continue; }
                if (args[0] == "capture-lifecycle")
                    Console.WriteLine("S4_MIDI_CAPTURE_STATUS " + JsonSerializer.Serialize(new
                    {
                        version = 1, state = "finished", path = args[2], error = (string?)null,
                        counts = new { attempted = 1, queued = 1, written = 1, dropped_full = 0,
                            dropped_oversized = 0, dropped_disconnected = 0, dropped_limit = 0, dropped_limit_queued = 0,
                            unwritten = 0, truncated = false }
                    }));
                Console.WriteLine(ProtocolTests.Status(profile, "stopping", fields[1]));
                Console.WriteLine("STOP_ACK");
                Console.Error.WriteLine("TAIL_DRAINED");
                return 0;
            }
            if (fields.Length == 3 && fields[0] == "S4_SCREEN_CONTROL")
                Console.WriteLine($"S4_SCREEN_STATUS {fields[1]} {(fields[2] == "OFF" ? "inactive" : "active")}");
        }
        Console.WriteLine(ProtocolTests.Status(profile, "stopping"));
        Console.WriteLine("EOF_ACK"); return 0;
    }
    public static int Generate(string[] args)
    {
        var output = args[Array.IndexOf(args, "--generate-mapping") + 1];
        var config = args[Array.IndexOf(args, "--jog-config") + 1];
        var jog = JsonNode.Parse(File.ReadAllText(config))!.AsObject();
        if (jog["scratch_reaction"]!.GetValue<int>() == 149) { Console.Error.WriteLine("GENERATOR_FAILURE"); return 7; }
        if (jog["scratch_speed"]!.GetValue<double>() == 13) return 0;
        var doc = new XDocument(new XElement("plist", new XElement("dict",
            new XElement("key", "endpoint"), new XElement("string", "LoopBe Internal MIDI"),
            new XElement("key", "scratch"), new XElement("real", jog["scratch_speed"]!.GetValue<double>()))));
        using var stream = new FileStream(output, FileMode.CreateNew, FileAccess.Write);
        doc.Save(stream); return 0;
    }
}
