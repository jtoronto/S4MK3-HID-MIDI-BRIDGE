using System.Reflection;
using System.Threading.Channels;
using S4Bridge.Core;
using S4Bridge.Models;

namespace S4Bridge.Tests;

internal static class NiTensionProcessTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        tests.Add(("NI managed host initializes, coalesces and restores tension across launches", Run));
        tests.Add(("stop hook failure still stops the managed child", StopAfterHookFailure));
    }

    private static async Task StopAfterHookFailure()
    {
        var supervisor = new BridgeSupervisor();
        var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var acknowledged = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
        supervisor.Output += (_, line) => { if (line == "STOP_ACK") acknowledged.TrySetResult(); };
        supervisor.BeforeStopAsync = () => Task.FromException(new IOException("Simulated disk full"));
        var (exe, args) = Fixture.Child("managed", "probe");
        supervisor.Start(new(exe, args, new() { Profile = "probe" }));
        var timeout = TimeSpan.FromSeconds(10);
        try
        {
            await ready.Task.WaitAsync(timeout);
            try { await supervisor.StopAsync().WaitAsync(timeout); }
            catch (IOException error) { Check.True(error.Message.Contains("disk full", StringComparison.Ordinal)); }
            await acknowledged.Task.WaitAsync(timeout);
            await supervisor.Completion.WaitAsync(timeout);
            Check.True(!supervisor.HasChild);
        }
        finally
        {
            if (supervisor.HasChild) supervisor.CloseInput();
            await supervisor.Completion.WaitAsync(timeout);
        }
    }

    private static async Task Run()
    {
        using var f = new Fixture();
        var host = new HostController(f.Resources, f.Root);
        using var diagnostics = host.Diagnostics;
        host.SelectMode(BridgeMode.NiBridge);
        host.Store.Save();
        var releaseEdits = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var edits = typeof(HostController).GetField("niTensionEdits", BindingFlags.Instance | BindingFlags.NonPublic)!
            .GetValue(host)!;
        typeof(NiTensionEdits).GetField("delay", BindingFlags.Instance | BindingFlags.NonPublic)!
            .SetValue(edits, (Func<CancellationToken, Task>)(token => releaseEdits.Task.WaitAsync(token)));

        var lines = Channel.CreateUnbounded<(Guid Id, string Text)>();
        host.Supervisor.Output += (id, line) => lines.Writer.TryWrite((id, line));
        var owner = new EndpointOwner();
        var (exe, args) = Fixture.Child("ni-tension", "full");
        var spec = LaunchSpec.Runtime(exe, host.Store, false, null, null);
        Check.True(spec.UsesNiBridge);
        spec = spec with { Arguments = [.. args, .. spec.Arguments] };
        var timeout = TimeSpan.FromSeconds(10);
        var observed = new List<string>();
        async Task WaitFor(string expected)
        {
            using var deadline = new CancellationTokenSource(timeout);
            while (true)
            {
                var (_, text) = await lines.Reader.ReadAsync(deadline.Token);
                if (text.StartsWith("FIXTURE_TENSION ", StringComparison.Ordinal))
                    observed.Add(text);
                if (text == expected) return;
            }
        }
        void Stale(Guid id, string text)
        {
            var protocol = (Action<Guid, string>)typeof(BridgeSupervisor)
                .GetField("Protocol", BindingFlags.Instance | BindingFlags.NonPublic)!
                .GetValue(host.Supervisor)!;
            protocol(id, text);
        }
        try
        {
            await host.Supervisor.StartAsync(spec, owner).WaitAsync(timeout);
            var firstId = host.Supervisor.Launch!.Id;
            await WaitFor("FIXTURE_EARLY_SCREEN");
            Check.Equal("starting", host.Supervisor.State);
            Check.Equal(0, observed.Count);
            await host.Supervisor.SendScreenAsync(1, "ON");
            await WaitFor("FIXTURE_TENSION 107");
            await host.Supervisor.SendScreenAsync(2, "ON");
            await WaitFor("FIXTURE_BARRIER");
            Check.Equal(1, observed.Count);

            var first = host.SetNiTensionAsync(10);
            var second = host.SetNiTensionAsync(20);
            var third = host.SetNiTensionAsync(231);
            releaseEdits.SetResult();
            await Task.WhenAll(first, second, third).WaitAsync(timeout);
            await WaitFor("FIXTURE_TENSION 231");
            await host.Supervisor.SendScreenAsync(2, "ON");
            await WaitFor("FIXTURE_BARRIER");
            Check.Equal(2, observed.Count);
            Check.Equal(231, new PreferenceStore(f.Resources, f.Root).NiTension.Value);
            await host.Supervisor.StopAsync().WaitAsync(timeout);

            observed.Clear();
            await host.Supervisor.StartAsync(spec, owner).WaitAsync(timeout);
            var secondId = host.Supervisor.Launch!.Id;
            Check.True(firstId != secondId);
            await WaitFor("FIXTURE_EARLY_SCREEN");
            await host.Supervisor.SendScreenAsync(1, "ON");
            await WaitFor("FIXTURE_TENSION 231");
            Stale(firstId, "S4_SCREEN_STATUS 0 inactive");
            Stale(firstId, "S4_NI_JOG_TENSION submitted 231");
            Check.True(host.CanSetNiTension);
            Check.Equal("Submitting 231", host.NiTensionStatus);
            await host.Supervisor.SendScreenAsync(2, "ON");
            await WaitFor("FIXTURE_BARRIER");
            Check.Equal("Tension request sent to NIHardwareService: 231", host.NiTensionStatus);
            Check.Equal(1, observed.Count);
            await host.Supervisor.StopAsync().WaitAsync(timeout);
        }
        finally
        {
            releaseEdits.TrySetResult();
            if (host.Supervisor.HasChild) host.Supervisor.CloseInput();
            await host.Supervisor.Completion.WaitAsync(timeout);
            if (host.Supervisor.HasOwnedResources)
                await host.Supervisor.StopAsync().WaitAsync(timeout);
        }
    }

    private sealed class EndpointOwner : IBloopEndpointOwner
    {
        public Task<IBloopEndpointLease> PrepareAsync() => Task.FromResult<IBloopEndpointLease>(new Lease());
    }

    private sealed class Lease : IBloopEndpointLease
    {
        public Guid AssociationId { get; } = Guid.NewGuid();
        public string? EndpointDeviceId => "test-endpoint";
        public Task CreateAsync() => Task.CompletedTask;
        public void RecordWorker(int processId, DateTime startTimeUtc) { }
        public ValueTask DisposeAsync() => ValueTask.CompletedTask;
    }
}
