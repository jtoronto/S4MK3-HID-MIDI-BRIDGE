using System.Text.Json;
using S4Bridge.Core;
using S4Bridge.Models;

namespace S4Bridge.Tests;

internal static class MotorReadinessTests
{
    internal static void Register(List<(string, Func<Task>)> tests)
    {
        tests.Add(("diagnostic readiness requires current launch motor authorization", async () =>
        {
            using var fixture = new Fixture();
            var output = fixture.PathFor("capture.jsonl");
            var receipt = fixture.PathFor("capture.readiness.json");
            var config = fixture.PathFor("config.json");
            File.WriteAllText(config, JsonSerializer.Serialize(new
            {
                version = 1, @case = "off", capture = true, capture_path = output
            }));
            var supervisor = new BridgeSupervisor();
            var lifecycleReady = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var caseReady = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var partial = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var authorized = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.Changed += () =>
            {
                if (supervisor.State == "ready") lifecycleReady.TrySetResult();
            };
            supervisor.StderrLine += (_, line) =>
            {
                if (line == "FIXTURE_PARTIAL_GATE") partial.TrySetResult();
                if (line == "FIXTURE_AUTHORIZED_GATE") authorized.TrySetResult();
            };
            supervisor.Protocol += (_, line) =>
            {
                if (line == "S4_SCREEN_DIAG_READY off") caseReady.TrySetResult();
            };
            var (exe, args) = Fixture.Child("motor-readiness", "full");
            try
            {
                supervisor.Start(new(exe, [.. args, "--screen-motor-diagnostic-config", config],
                    new RunPreferences { Profile = "full" }));
                await Task.WhenAll(lifecycleReady.Task, caseReady.Task, partial.Task).WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(File.Exists(receipt), "Diagnostic motor authorization receipt was not produced.");
                using (var pending = JsonDocument.Parse(File.ReadAllText(receipt)))
                {
                    var root = pending.RootElement;
                    Check.True(root.GetProperty("receiver_ready").GetBoolean());
                    Check.True(!root.GetProperty("ready").GetBoolean());
                    Check.Equal(JsonValueKind.Null, root.GetProperty("right").GetProperty("loaded").ValueKind);
                    Check.Equal(JsonValueKind.Null, root.GetProperty("right").GetProperty("internal").ValueKind);
                    Check.Equal(supervisor.Launch!.ProcessId, root.GetProperty("pid").GetInt32());
                }
                await supervisor.SendScreenAsync(1, "ON");
                await authorized.Task.WaitAsync(TimeSpan.FromSeconds(10));
                using (var ready = JsonDocument.Parse(File.ReadAllText(receipt)))
                {
                    var root = ready.RootElement;
                    Check.True(root.GetProperty("ready").GetBoolean());
                    Check.Equal(supervisor.Launch!.Id.ToString(), root.GetProperty("launch_id").GetString());
                    Check.Equal(2, root.GetProperty("right").GetProperty("deck").GetInt32());
                }
                await supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
                using var stopped = JsonDocument.Parse(File.ReadAllText(receipt));
                Check.True(!stopped.RootElement.GetProperty("ready").GetBoolean());
            }
            finally
            {
                if (supervisor.HasChild) supervisor.CloseInput();
                await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
            }
        }));
    }
}
