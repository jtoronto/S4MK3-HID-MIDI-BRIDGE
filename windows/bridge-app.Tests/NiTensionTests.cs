using S4Bridge.Core;
using S4Bridge.Models;

namespace S4Bridge.Tests;

internal static class NiTensionTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        NiTensionProcessTests.Register(tests);
        Check.Add(tests, "NI tension persists separately and validates its range", () =>
        {
            using var f = new Fixture();
            var store = new PreferenceStore(f.Resources, f.Root);
            Check.Equal(107, store.NiTension.Value);
            Check.True(!store.HasSavedNiTension);
            foreach (var value in new[] { 0, 107, 255 })
            {
                store.SaveNiTension(value);
                var loaded = new PreferenceStore(f.Resources, f.Root);
                Check.True(loaded.HasSavedNiTension);
                Check.Equal(value, loaded.NiTension.Value);
            }
            Check.Reject(() => store.SaveNiTension(-1));
            Check.Reject(() => store.SaveNiTension(256));
            Check.Equal(255, new PreferenceStore(f.Resources, f.Root).NiTension.Value);
        });
        tests.Add(("NI tension stopped edits save without a child", async () =>
        {
            using var f = new Fixture();
            var host = new HostController(f.Resources, f.Root);
            using var diagnostics = host.Diagnostics;
            Check.True(!host.CanSetNiTension);
            host.SelectMode(BridgeMode.NiBridge);
            Check.True(host.CanSetNiTension);
            await host.SetNiTensionAsync(128).WaitAsync(TimeSpan.FromSeconds(5));
            Check.Equal(128, host.NiTension);
            Check.Equal(128, new PreferenceStore(f.Resources, f.Root).NiTension.Value);
            Check.True(!host.Supervisor.HasChild);
        }));
        tests.Add(("NI tension drag applies only the last value", async () =>
        {
            var release = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var applied = new List<int>();
            var edits = new NiTensionEdits(107, (value, _) =>
            {
                applied.Add(value);
                return Task.CompletedTask;
            }, token => release.Task.WaitAsync(token));
            var first = edits.SetAsync(10);
            var second = edits.SetAsync(20);
            var third = edits.SetAsync(30);
            Check.Equal(30, edits.Value);
            release.SetResult();
            await Task.WhenAll(first, second, third).WaitAsync(TimeSpan.FromSeconds(5));
            Check.Equal(1, applied.Count);
            Check.Equal(30, applied[0]);
        }));
        tests.Add(("NI tension accepts only matching launch and receipt", async () =>
        {
            var sent = new List<int>();
            var coordinator = new NiTensionCoordinator((_, value) =>
            {
                sent.Add(value);
                return Task.CompletedTask;
            });
            var launch = Guid.NewGuid();
            coordinator.Started(launch);
            coordinator.Accept(Guid.NewGuid(), "S4_SCREEN_STATUS 0 active");
            Check.True(!coordinator.Ready);
            coordinator.Accept(launch, "S4_SCREEN_STATUS 0 active");
            await coordinator.SubmitAsync(launch, 25);
            await coordinator.SubmitAsync(launch, 244);
            coordinator.Accept(launch, "S4_NI_JOG_TENSION submitted 25");
            coordinator.Accept(Guid.NewGuid(), "S4_NI_JOG_TENSION submitted 244");
            Check.Equal("Submitting 244", coordinator.Status);
            coordinator.Accept(launch, "S4_NI_JOG_TENSION submitted 244");
            Check.Equal("Tension request sent to NIHardwareService: 244", coordinator.Status);
            coordinator.Exited(launch);
            coordinator.Accept(launch, "S4_SCREEN_STATUS 0 active");
            Check.True(!coordinator.Ready);
            Check.Equal(2, sent.Count);
        }));
        tests.Add(("NI tension serializes saves across overlapping edits", async () =>
        {
            var entered = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var release = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var applied = new List<int>();
            var edits = new NiTensionEdits(107, async (value, _) =>
            {
                if (value == 10) { entered.SetResult(); await release.Task; }
                applied.Add(value);
            }, _ => Task.CompletedTask);
            var first = edits.SetAsync(10);
            await entered.Task.WaitAsync(TimeSpan.FromSeconds(5));
            var second = edits.SetAsync(20);
            var third = edits.SetAsync(30);
            release.SetResult();
            await Task.WhenAll(first, second, third).WaitAsync(TimeSpan.FromSeconds(5));
            Check.Equal(2, applied.Count);
            Check.Equal(10, applied[0]);
            Check.Equal(30, applied[1]);
        }));
        tests.Add(("NI tension flush saves the latest pending edit without sending", async () =>
        {
            var release = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var sent = new List<int>();
            var saved = new List<int>();
            var edits = new NiTensionEdits(107, (value, _) =>
            {
                sent.Add(value);
                return Task.CompletedTask;
            }, token => release.Task.WaitAsync(token), saved.Add);
            var first = edits.SetAsync(10);
            var second = edits.SetAsync(20);
            await edits.FlushAsync().WaitAsync(TimeSpan.FromSeconds(5));
            await Task.WhenAll(first, second).WaitAsync(TimeSpan.FromSeconds(5));
            Check.Equal(0, sent.Count);
            Check.Equal(1, saved.Count);
            Check.Equal(20, saved[0]);
        }));
    }
}
