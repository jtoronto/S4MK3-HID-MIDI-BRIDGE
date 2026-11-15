using System.Text.Json.Nodes;
using S4Bridge.Core;

namespace S4Bridge.Tests;

internal static class ProtocolTests
{
    public static string Status(string profile = "full", string state = "ready", string? id = null,
        string backend = "bloop") =>
        "S4_BRIDGE_STATUS " + new JsonObject
        {
            ["version"] = 1,
            ["state"] = state,
            ["profile"] = profile,
            ["backend"] = backend,
            ["request_id"] = id,
            ["readiness"] = new JsonObject
            {
                ["hid"] = true,
                ["controller_initialized"] = true,
                ["midi_input"] = profile == "full" ? (JsonNode?)JsonValue.Create(true) : null,
                ["midi_output"] = profile != "probe" ? (JsonNode?)JsonValue.Create(true) : null,
                ["feedback"] = profile == "full" ? "unknown" : "not_applicable"
            },
            ["error"] = state == "failed" ? new JsonObject { ["stage"] = "cleanup", ["message"] = "fixture failure" } : null
        }.ToJsonString(new() { WriteIndented = false });
    public static void Register(List<(string, Func<Task>)> tests)
    {
        foreach (var profile in new[] { "full", "minimal", "probe" })
            Check.Add(tests, "aggregate validates readiness for " + profile, () =>
            {
                var status = ManagedStatus.Parse(Status(profile));
                Check.Equal(profile, status.Profile); Check.Equal("ready", status.State);
            });
        foreach (var mutation in new[] { "version", "hid", "profile", "request", "feedback", "missing", "error" })
            Check.Add(tests, "aggregate rejects invalid " + mutation, () =>
            {
                var obj = JsonNode.Parse(Status()[ManagedStatus.Prefix.Length..])!.AsObject();
                switch (mutation)
                {
                    case "version": obj["version"] = 2; break;
                    case "hid": obj["readiness"]!["hid"] = false; break;
                    case "profile": obj["profile"] = "probe"; break;
                    case "request": obj["request_id"] = "1"; break;
                    case "feedback": obj["readiness"]!["feedback"] = "fake"; break;
                    case "missing": obj.Remove("error"); break;
                    case "error": obj["error"] = new JsonObject(); break;
                }
                Check.Reject(() => ManagedStatus.Parse(ManagedStatus.Prefix + obj.ToJsonString()));
            });
        Check.Add(tests, "request IDs reject signs Unicode and overflow while preserving u64 max", () =>
        {
            foreach (var id in new[] { "", "-1", "+1", " 1", "1 ", "１２", "18446744073709551616" })
                Check.Reject(() => ManagedStatus.ParseId(id));
            Check.Equal(ulong.MaxValue, ManagedStatus.ParseId("18446744073709551615"));
        });
        Check.Add(tests, "stop revokes readiness and rejects wrong request and later ready", () =>
        {
            var launch = new LaunchState(Guid.NewGuid(), 42, "full");
            Check.True(launch.Accept(ManagedStatus.Parse(Status())));
            launch.RequestStop(7);
            Check.True(!launch.IsReady);
            Check.True(!launch.Accept(ManagedStatus.Parse(Status(state: "stopping", id: "8"))));
            Check.True(!launch.Accept(ManagedStatus.Parse(Status())));
            Check.True(launch.Accept(ManagedStatus.Parse(Status(state: "stopping", id: "7"))));
        });
        Check.Add(tests, "failed or unconfirmed launch cannot regain readiness", () =>
        {
            var failed = new LaunchState(Guid.NewGuid(), 42, "full");
            Check.True(failed.Accept(ManagedStatus.Parse(Status(state: "failed"))));
            Check.True(!failed.Accept(ManagedStatus.Parse(Status())));
            var timedOut = new LaunchState(Guid.NewGuid(), 43, "full"); timedOut.Unconfirmed();
            Check.True(!timedOut.Accept(ManagedStatus.Parse(Status())));
            Check.True(!timedOut.IsReady);
        });
        Check.Add(tests, "profile and backend mismatch never authorize launch", () =>
        {
            foreach (var backend in new[] { "bloop", "loopbe" })
            {
                var launch = new LaunchState(Guid.NewGuid(), 42, "probe", backend);
                Check.True(!launch.Accept(ManagedStatus.Parse(Status())));
                var native = ManagedStatus.Parse(Status("probe")) with { Backend = "core-midi" };
                Check.True(!launch.Accept(native)); Check.True(!launch.IsReady);
                var other = backend == "bloop" ? "loopbe" : "bloop";
                Check.True(!launch.Accept(ManagedStatus.Parse(Status("probe", backend: other))));
                Check.True(launch.Accept(ManagedStatus.Parse(Status("probe", backend: backend))));
            }
        });
        Check.Add(tests, "launch profiles exclude full settings in minimal and probe", () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root);
            foreach (var profile in new[] { "full", "minimal", "probe" })
            {
                store.Editor.Run.Profile = profile; store.Save();
                var spec = LaunchSpec.Runtime("engine.exe", store, true);
                Check.True(spec.Arguments.Contains("--managed-stdin"));
                Check.Equal(profile == "full", spec.Arguments.Contains("--screens"));
                Check.Equal(profile == "full", spec.Arguments.Contains("--tension-config"));
                Check.Equal(profile != "probe", spec.Arguments.Contains("--midi"));
            }
        });
    }
}
