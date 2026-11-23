using System.Text.Json;
using S4Bridge.Core;

namespace S4Bridge.Tests;

internal static class ScreenMotorDiagnosticsTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "screen motor launch overrides only runtime copy", () =>
        {
            using var fixture = new Fixture();
            var store = new PreferenceStore(fixture.Resources, fixture.Root);
            store.Editor.Run.RawReports = true;
            store.Editor.Run.LogLevel = "debug";
            store.Editor.Screen.Enabled = false;
            store.Save();
            var savedBytes = File.ReadAllBytes(fixture.PathFor("run-settings.json"));
            var config = Config(fixture, "normal", false);
            var normal = LaunchSpec.Runtime("engine", store, false, null, null);
            var diagnostic = LaunchSpec.Runtime("engine", store, false, null, config);
            var preview = LaunchSpec.EditorPreview("engine", store, config);
            Check.True(normal.Arguments.SequenceEqual(new[] {
                "--midi-backend", BloopEndpoint.Backend, "--managed-stdin", "--midi", "full",
                "--screen-control-stdin", "--led-config", Path.Combine(store.Root, "led-config.json"),
                "--jog-config", Path.Combine(store.Root, "jog-config.json"),
                "--tension-config", Path.Combine(store.Root, "tension-config.json"),
                "--until-stopped", "--raw"
            }));
            Check.Equal("debug", normal.StartInfo().Environment["RUST_LOG"]);
            Check.True(!diagnostic.Arguments.Contains("--raw"));
            Check.True(diagnostic.Arguments.Contains("--screens"));
            Check.Equal("info", diagnostic.StartInfo().Environment["RUST_LOG"]);
            var index = Array.IndexOf(diagnostic.Arguments, "--screen-motor-diagnostic-config");
            Check.True(index >= 0);
            Check.Equal(config, diagnostic.Arguments[index + 1]);
            Check.Equal(diagnostic.Preview, preview.Preview);
            Check.True(store.Saved.Run.RawReports && store.Editor.Run.RawReports);
            Check.Equal("debug", store.Saved.Run.LogLevel);
            Check.Equal("debug", store.Editor.Run.LogLevel);
            Check.True(savedBytes.SequenceEqual(File.ReadAllBytes(fixture.PathFor("run-settings.json"))));
        });

        Check.Add(tests, "screen motor accepts cases and rejects incompatible launches", () =>
        {
            using var fixture = new Fixture();
            var store = new PreferenceStore(fixture.Resources, fixture.Root);
            foreach (var name in new[] { "normal", "off", "acquired", "static", "updates" })
            {
                var config = Config(fixture, name, true);
                var launch = LaunchSpec.Runtime("engine", store, false, null, config);
                Check.Equal(name != "off", launch.Arguments.Contains("--screens"));
                Check.Reject(() => LaunchSpec.Runtime("engine", store, false, fixture.PathFor("midi.jsonl"), config));
            }
            var path = Config(fixture, "normal", false);
            foreach (var profile in new[] { "minimal", "probe" })
            {
                store.Editor.Run.Profile = profile;
                store.Save();
                Check.Reject(() => LaunchSpec.Runtime("engine", store, false, null, path));
            }
            Check.Reject(() => LaunchSpec.Runtime("engine", store, false, null, "relative.json"));
            Check.Reject(() => LaunchSpec.Runtime("engine", store, false, null, fixture.PathFor("missing.json")));
        });

        Check.Add(tests, "screen motor rejects malformed or unsafe config", () =>
        {
            using var fixture = new Fixture();
            var store = new PreferenceStore(fixture.Resources, fixture.Root);
            var path = fixture.PathFor("config.json");
            var output = fixture.PathFor("capture.jsonl");
            foreach (var json in new[] {
                "{}", "[]", "{", "{\"version\":1,\"version\":1,\"case\":\"normal\",\"capture_path\":\"x\",\"capture\":false}",
                JsonSerializer.Serialize(new { version = 2, @case = "normal", capture_path = output, capture = false }),
                JsonSerializer.Serialize(new { version = 1, @case = "unknown", capture_path = output, capture = true }),
                JsonSerializer.Serialize(new { version = 1, @case = "off", capture_path = output, capture = false }),
                JsonSerializer.Serialize(new { version = 1, @case = "normal", capture_path = "relative.jsonl", capture = false }),
                JsonSerializer.Serialize(new { version = 1, @case = "normal", capture_path = output, capture = false, extra = 1 })
            })
            {
                File.WriteAllText(path, json);
                Check.Reject(() => LaunchSpec.Runtime("engine", store, false, null, path));
            }
            File.WriteAllText(output, "existing");
            File.WriteAllText(path, JsonSerializer.Serialize(new { version = 1, @case = "normal", capture_path = output, capture = true }));
            Check.Reject(() => LaunchSpec.Runtime("engine", store, false, null, path));
        });
    }

    private static string Config(Fixture fixture, string name, bool capture)
    {
        var path = fixture.PathFor($"{name}-{capture}.json");
        File.WriteAllText(path, JsonSerializer.Serialize(new {
            version = 1, @case = name, capture_path = fixture.PathFor($"{name}-{capture}.jsonl"), capture
        }));
        return path;
    }
}
