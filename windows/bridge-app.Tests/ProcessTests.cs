using System.Reflection;
using System.Xml.Linq;
using S4Bridge.Core;
using S4Bridge.Models;

namespace S4Bridge.Tests;

internal static class ProcessTests
{
    public static void RegisterScreenMotor(List<(string, Func<Task>)> tests)
    {
        tests.Add(("diagnostic begins only after prepared worker and endpoint publication", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("receipt"));
            owner.Lease.CreateBarrier = new(TaskCreationOptions.RunContinuationsAsynchronously);
            var supervisor = new BridgeSupervisor();
            var preparing = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var prepared = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var began = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.Output += (_, line) =>
            {
                if (line == "FIXTURE_PREPARING") preparing.TrySetResult();
                if (line == "S4_SCREEN_DIAG_PREPARED 1") prepared.TrySetResult();
                if (line == "FIXTURE_BEGAN_WITH_ENDPOINT") began.TrySetResult();
            };
            var (exe, args) = Fixture.Child("screen-diag-gated", "full", owner.Lease.ReceiptPath);
            var config = f.PathFor("diagnostic.json");
            File.WriteAllText(config, System.Text.Json.JsonSerializer.Serialize(new {
                version = 1, @case = "normal", capture_path = f.PathFor("capture.jsonl"), capture = false
            }));
            var store = new PreferenceStore(f.Resources, f.Root);
            var generated = LaunchSpec.Runtime(exe, store, false, null, config);
            var starting = supervisor.StartAsync(generated with { Arguments = [.. args, .. generated.Arguments] }, owner);
            try
            {
                await preparing.Task.WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(!owner.Lease.CreateEntered.Task.IsCompleted);
                await supervisor.SendScreenAsync(1, "ON");
                await owner.Lease.CreateEntered.Task.WaitAsync(TimeSpan.FromSeconds(10));
                await prepared.Task.WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(supervisor.HasChild);
                Check.True(!began.Task.IsCompleted);
                owner.Lease.CreateBarrier.TrySetResult();
                await starting.WaitAsync(TimeSpan.FromSeconds(10));
                await began.Task.WaitAsync(TimeSpan.FromSeconds(10));
                await supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(!supervisor.HasOwnedResources);
            }
            finally
            {
                owner.Lease.CreateBarrier.TrySetResult();
                if (supervisor.HasChild) supervisor.CloseInput();
                try { await starting.WaitAsync(TimeSpan.FromSeconds(10)); }
                catch (Exception ex) when (ex is IOException or InvalidOperationException or OperationCanceledException) { }
                await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
                if (supervisor.HasOwnedResources) await supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
            }
        }));
        foreach (var mode in new[] { "screen-diag-fail", "screen-diag-wait" })
        {
            tests.Add(($"diagnostic {mode} leaves no endpoint or worker", async () =>
            {
                using var f = new Fixture();
                var owner = new TestEndpointOwner(f.PathFor("receipt"));
                var supervisor = new BridgeSupervisor();
                var waiting = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                supervisor.Output += (_, line) => { if (line == "FIXTURE_WAITING") waiting.TrySetResult(); };
                var (exe, args) = Fixture.Child(mode, "full", owner.Lease.ReceiptPath,
                    "--prepare-screen-motor-diagnostics");
                var starting = supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner);
                try
                {
                    if (mode == "screen-diag-wait")
                    {
                        await waiting.Task.WaitAsync(TimeSpan.FromSeconds(10));
                        supervisor.CloseInput();
                    }
                    await Check.RejectAsync(() => starting.WaitAsync(TimeSpan.FromSeconds(10)));
                    await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
                    Check.True(!owner.Lease.CreateEntered.Task.IsCompleted);
                    Check.True(!supervisor.HasOwnedResources);
                    Check.True(!File.Exists(owner.Lease.ReceiptPath));
                }
                finally
                {
                    if (supervisor.HasChild) supervisor.CloseInput();
                    await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
                }
            }));
        }
        tests.Add(("diagnostic endpoint failure terminates prepared worker", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("receipt"));
            owner.Lease.FailCreate = true;
            var supervisor = new BridgeSupervisor();
            var (exe, args) = Fixture.Child("screen-diag", "full", owner.Lease.ReceiptPath,
                "--prepare-screen-motor-diagnostics");
            try
            {
                await Check.RejectAsync(() => supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner)
                    .WaitAsync(TimeSpan.FromSeconds(10)));
                await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(owner.Lease.CreateEntered.Task.IsCompletedSuccessfully);
                Check.True(!supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
            }
            finally
            {
                if (supervisor.HasChild) supervisor.CloseInput();
                await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
            }
        }));
        tests.Add(("diagnostic Stop before preparation leaves no endpoint or worker", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("receipt"));
            var supervisor = new BridgeSupervisor();
            var waiting = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.Output += (_, line) => { if (line == "FIXTURE_WAITING") waiting.TrySetResult(); };
            var (exe, args) = Fixture.Child("screen-diag-wait", "full", owner.Lease.ReceiptPath,
                "--prepare-screen-motor-diagnostics");
            var starting = supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner);
            try
            {
                await waiting.Task.WaitAsync(TimeSpan.FromSeconds(10));
                await supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
                try { await starting.WaitAsync(TimeSpan.FromSeconds(10)); }
                catch (Exception ex) when (ex is IOException or InvalidOperationException or OperationCanceledException) { }
                Check.True(!owner.Lease.CreateEntered.Task.IsCompleted);
                Check.True(!supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
            }
            finally
            {
                if (supervisor.HasChild) supervisor.CloseInput();
                await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
            }
        }));
    }

    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "NI decks opt-in preserves default launch and isolates preferences", () =>
        {
            using var f = new Fixture();
            Check.True(LaunchSpec.ExperimentalEnabled("1"));
            foreach (var value in new[] { null, "", "true", " 1", "0", "01" })
                Check.True(!LaunchSpec.ExperimentalEnabled(value));
            Check.Equal(f.Root, PreferenceStore.HostPreferencesRoot(f.Root, false));
            var root = PreferenceStore.HostPreferencesRoot(f.Root, true);
            Check.True(root != f.Root);
            var store = new PreferenceStore(f.Resources, root);
            store.Editor.Run.Profile = "minimal";
            store.Editor.Run.LimitDuration = true;
            store.Save();
            var normal = LaunchSpec.Runtime("engine", store, false, null, null);
            Check.Equal("minimal", normal.Run.Profile);
            Check.True(!normal.Arguments.Contains("--ni-decks"));
            Check.True(normal.Arguments.Contains("--seconds"));
            var spec = LaunchSpec.Experimental("engine", store, false, null, null);
            foreach (var arg in new[] { "--ni-decks", "--midi", "full", "--midi-backend", "bloop",
                "--until-stopped", "--managed-stdin", "--led-config", "--jog-config" })
                Check.True(spec.Arguments.Contains(arg), arg);
            foreach (var arg in new[] { "--seconds", "--screen-control-stdin", "--tension-config",
                "--screen-motor-diagnostic-config", "--screens", "--midi-capture" })
                Check.True(!spec.Arguments.Contains(arg), arg);
            Check.Equal("full", spec.Run.Profile);
            Check.True(!spec.Run.LimitDuration);
            Check.Reject(() => LaunchSpec.Experimental("engine", store, false, f.PathFor("capture"), null));
            Check.Reject(() => LaunchSpec.Experimental("engine", store, false, null, f.PathFor("config")));
        });
        tests.Add(("persisted NI mode launches full indefinite BLOOP and accepts NI screen status", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("receipt"));
            var seed = new PreferenceStore(f.Resources, f.Root);
            seed.Editor.Run.Mode = BridgeMode.NiBridge;
            seed.Editor.Run.Profile = "minimal";
            seed.Editor.Run.LimitDuration = true;
            seed.Editor.Screen.Enabled = false;
            seed.Save();

            var host = new HostController(f.Resources, f.Root, owner);
            using var diagnostics = host.Diagnostics;
            Check.Equal(BridgeMode.NiBridge, host.SelectedMode);
            var preview = LaunchSpec.EditorPreview(host.Executable, host.Store, null);
            Check.True(preview.Preview.Contains("\"--ni-decks\"", StringComparison.Ordinal));
            Check.True(preview.Preview.Contains("\"--until-stopped\"", StringComparison.Ordinal));
            Check.True(!preview.Arguments.Contains("--screens"));
            Check.True(!preview.Arguments.Contains("--screen-control-stdin"));

            var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var launchId = Guid.Empty;
            host.Supervisor.Changed += () =>
            {
                if (host.Supervisor.State == "ready") ready.TrySetResult();
            };
            host.Supervisor.Launched += id => launchId = id;
            var (exe, args) = Fixture.Child("managed", "full");
            var spec = LaunchSpec.Runtime(exe, host.Store, false, null, null);
            foreach (var argument in new[] { "--ni-decks", "--midi", "full", "--midi-backend", "bloop",
                "--until-stopped", "--managed-stdin" })
                Check.True(spec.Arguments.Contains(argument), argument);
            foreach (var argument in new[] { "--seconds", "--screens", "--screen-control-stdin" })
                Check.True(!spec.Arguments.Contains(argument), argument);

            var starting = host.Supervisor.StartAsync(
                spec with { Arguments = [.. args, .. spec.Arguments] }, owner);
            try
            {
                await starting.WaitAsync(TimeSpan.FromSeconds(10));
                await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(launchId != Guid.Empty);
                Check.True(host.Supervisor.ActiveSpec?.UsesNiBridge == true);
                Check.Equal("pending", host.ScreenStatus);
                Check.True(File.Exists(owner.Lease.ReceiptPath));
                host.Screens.Accept(launchId, "S4_SCREEN_STATUS 0 active");
                Check.Equal("active", host.ScreenStatus);
                await host.Supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(!host.Supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
            }
            finally
            {
                if (host.Supervisor.HasChild) host.Supervisor.CloseInput();
                try { await starting.WaitAsync(TimeSpan.FromSeconds(10)); }
                catch (Exception ex) when (ex is IOException or InvalidOperationException or OperationCanceledException) { }
                await host.Supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
                if (host.Supervisor.HasOwnedResources)
                    await host.Supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
            }
        }));
        foreach (var profile in new[] { "full", "minimal", "probe" })
        {
            tests.Add(($"real supervised pipes STOP and drain tail for {profile}", async () =>
            {
                using var f = new Fixture();
                var supervisor = new BridgeSupervisor();
                var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
                var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var output = new List<string>();
                supervisor.Output += (_, line) => { lock (output) output.Add(line); };
                supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
                var (exe, args) = Fixture.Child("managed", profile);
                var spec = new LaunchSpec(exe, args, new() { Profile = profile });
                owner.Lease.AssertRelease = () =>
                {
                    Check.True(!supervisor.HasChild);
                    lock (output) Check.True(output.Contains("TAIL_DRAINED"));
                };
                if (profile == "probe") supervisor.Start(spec);
                else await supervisor.StartAsync(spec, owner);
                try
                {
                    await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
                    Check.True(supervisor.HasChild);
                    Check.Equal(profile != "probe", File.Exists(owner.Lease.ReceiptPath));
                    if (profile != "probe") Check.Equal<int?>(supervisor.Launch!.ProcessId, owner.Lease.WorkerId);
                    await supervisor.StopAsync();
                    Check.True(!supervisor.HasChild); Check.Equal("stopped", supervisor.State);
                    Check.True(!supervisor.HasOwnedResources);
                    Check.True(!File.Exists(owner.Lease.ReceiptPath));
                    lock (output)
                    {
                        Check.True(output.Contains("STOP_ACK")); Check.True(output.Contains("TAIL_DRAINED"));
                        Check.Equal("EXIT: 0", output.Last());
                    }
                }
                finally { if (supervisor.HasChild) { supervisor.CloseInput(); await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10)); } }
            }
            ));
        }
        tests.Add(("managed stdin EOF converges on actual child exit", async () =>
        {
            var supervisor = new BridgeSupervisor(); var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
            var (exe, args) = Fixture.Child("managed", "probe");
            supervisor.Start(new(exe, args, new() { Profile = "probe" }));
            await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
            supervisor.CloseInput(); await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
            Check.Equal("stopped", supervisor.State); Check.True(!supervisor.HasChild);
        }
        ));
        tests.Add(("frozen stop deadline retains live child and unconfirmed neutral until actual exit", async () =>
        {
            using var f = new Fixture();
            var supervisor = new BridgeSupervisor(); var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            owner.Lease.AssertRelease = () => Check.True(!supervisor.HasChild);
            supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
            var (exe, args) = Fixture.Child("hang", "full");
            await supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner);
            try
            {
                await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
                var first = supervisor.StopAsync(); var second = supervisor.StopAsync();
                Check.True(ReferenceEquals(first, second));
                await Check.RejectAsync(() => first);
                Check.True(supervisor.HasChild); Check.Equal("failed", supervisor.State);
                Check.True(supervisor.Launch!.StopUnconfirmed); Check.True(!supervisor.Launch.IsReady);
                Check.True(File.Exists(owner.Lease.ReceiptPath));
            }
            finally
            {
                supervisor.CloseInput();
                await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
            }
            Check.Equal("failed", supervisor.State); Check.True(!supervisor.HasChild);
            Check.True(!supervisor.HasOwnedResources);
            Check.True(!File.Exists(owner.Lease.ReceiptPath));
        }
        ));
        tests.Add(("stderr status-shaped text cannot authorize readiness", async () =>
        {
            var supervisor = new BridgeSupervisor(); var observed = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.Output += (_, line) => { if (line == "STDERR_STATUS_SENT") observed.TrySetResult(); };
            var (exe, args) = Fixture.Child("stderr", "probe");
            supervisor.Start(new(exe, args, new() { Profile = "probe" }));
            await observed.Task.WaitAsync(TimeSpan.FromSeconds(10));
            Check.Equal("starting", supervisor.State);
            supervisor.CloseInput(); await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
        }
        ));
        tests.Add(("stderr motor telemetry ingests to current capture without authorizing readiness", async () =>
        {
            var supervisor = new BridgeSupervisor();
            var diagnostics = new S4Bridge.Diagnostics.DiagnosticHub();
            supervisor.Launched += diagnostics.BeginLaunch;
            supervisor.Protocol += diagnostics.Accept;
            supervisor.StderrLine += diagnostics.AcceptStderr;
            supervisor.Output += (_, line) => diagnostics.Log.Append(line);
            var stderrSent = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.StderrLine += (_, line) => { if (line == ProtocolTests.Status("full")) stderrSent.TrySetResult(); };
            supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
            using var f = new Fixture();
            var (exe, args) = Fixture.Child("motor-stderr", "full");
            supervisor.Start(new(exe, args, new() { Profile = "full" }));
            try
            {
                await stderrSent.Task.WaitAsync(TimeSpan.FromSeconds(10));
                Check.Equal("starting", supervisor.State);
                await supervisor.SendScreenAsync(1, "ON");
                await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
                diagnostics.StartCapture(f.Root, new() { Profile = "full", LogLevel = "debug" });
                await supervisor.StopAsync();
                Check.Equal("stopped", supervisor.State);
                diagnostics.StopCapture();
                var capture = File.ReadAllText(diagnostics.CapturePath!);
                Check.True(capture.Contains("MOTOR_JOG_BEND", StringComparison.Ordinal));
                Check.True(capture.Contains("MOTOR_REGULATOR", StringComparison.Ordinal));
                Check.True(capture.Contains("S4_MOTOR_CONFIG", StringComparison.Ordinal));
                Check.True(!capture.Contains("UNRELATED_STDERR_NOISE", StringComparison.Ordinal));
                Check.True(!capture.Contains("S4_BRIDGE_STATUS", StringComparison.Ordinal));
                Check.True(diagnostics.Log.Text.Contains("MOTOR_JOG_BEND", StringComparison.Ordinal));
            }
            finally { if (supervisor.HasChild) { supervisor.CloseInput(); await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10)); } }
        }
        ));
        tests.Add(("nonzero actual process exit outranks success-looking text", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            var supervisor = new BridgeSupervisor(); var (exe, args) = Fixture.Child("exit7", "full");
            owner.Lease.AssertRelease = () => Check.True(!supervisor.HasChild);
            await supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner);
            await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
            Check.Equal("failed", supervisor.State); Check.True(!supervisor.HasChild);
            Check.True(supervisor.Error.Contains("7", StringComparison.Ordinal));
            Check.True(!supervisor.HasOwnedResources);
            Check.True(!File.Exists(owner.Lease.ReceiptPath));
        }
        ));
        tests.Add(("failed endpoint publication releases the prepared ownership receipt", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            owner.Lease.FailCreate = true;
            var supervisor = new BridgeSupervisor();
            var (exe, args) = Fixture.Child("managed", "full");
            await Check.RejectAsync(() => supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner));
            Check.Equal("failed", supervisor.State);
            Check.True(!supervisor.HasOwnedResources);
            Check.True(!File.Exists(owner.Lease.ReceiptPath));
        }));
        tests.Add(("worker receipt failure stops the actual worker before releasing endpoint ownership", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            owner.Lease.FailRecordWorker = true;
            var supervisor = new BridgeSupervisor();
            owner.Lease.AssertRelease = () => Check.True(!supervisor.HasChild);
            var (exe, args) = Fixture.Child("managed", "full");
            try
            {
                await Check.RejectAsync(() => supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner));
                Check.Equal("failed", supervisor.State);
                Check.True(!supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
            }
            finally
            {
                if (supervisor.HasChild) { supervisor.CloseInput(); await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10)); }
            }
        }));
        tests.Add(("failed process start removes the already published endpoint", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            var supervisor = new BridgeSupervisor();
            try
            {
                await supervisor.StartAsync(new(f.PathFor("missing-engine"), [], new() { Profile = "full" }), owner);
                throw new InvalidOperationException("Missing engine unexpectedly started.");
            }
            catch (System.ComponentModel.Win32Exception)
            {
                Check.Equal("failed", supervisor.State);
                Check.True(!supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
            }
        }));
        tests.Add(("endpoint removal failure retains ownership and Stop retries without another worker", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            owner.Lease.FailCleanupOnce = true;
            var supervisor = new BridgeSupervisor();
            owner.Lease.AssertRelease = () => Check.True(!supervisor.HasChild);
            var (exe, args) = Fixture.Child("managed", "full");
            await supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner);
            try
            {
                await Check.RejectAsync(() => supervisor.StopAsync());
                Check.True(!supervisor.HasChild && supervisor.HasOwnedResources);
                Check.True(File.Exists(owner.Lease.ReceiptPath));
                Check.Equal("failed", supervisor.State);
                Check.Reject(() => supervisor.Start(new(exe, args, new() { Profile = "full" })));
                await supervisor.StopAsync();
                Check.True(!supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
                Check.Equal("stopped", supervisor.State);
            }
            finally { if (supervisor.HasOwnedResources) await supervisor.StopAsync(); }
        }));
        tests.Add(("Stop during endpoint publication waits for creation and actual worker exit", async () =>
        {
            using var f = new Fixture();
            var owner = new TestEndpointOwner(f.PathFor("bloop-receipt.json"));
            owner.Lease.CreateBarrier = new(TaskCreationOptions.RunContinuationsAsynchronously);
            var supervisor = new BridgeSupervisor();
            owner.Lease.AssertRelease = () => Check.True(!supervisor.HasChild);
            var (exe, args) = Fixture.Child("managed", "full");
            var starting = supervisor.StartAsync(new(exe, args, new() { Profile = "full" }), owner);
            try
            {
                await owner.Lease.CreateEntered.Task.WaitAsync(TimeSpan.FromSeconds(10));
                var stopping = supervisor.StopAsync();
                Check.True(!stopping.IsCompleted);
                Check.True(!supervisor.HasChild && File.Exists(owner.Lease.ReceiptPath));
                Check.True(ReferenceEquals(stopping, supervisor.StopAsync()));
                owner.Lease.CreateBarrier.SetResult();
                await starting.WaitAsync(TimeSpan.FromSeconds(10));
                await stopping.WaitAsync(TimeSpan.FromSeconds(10));
                Check.True(!supervisor.HasOwnedResources);
                Check.True(!File.Exists(owner.Lease.ReceiptPath));
                Check.Equal("stopped", supervisor.State);
            }
            finally
            {
                owner.Lease.CreateBarrier.TrySetResult();
                await starting.WaitAsync(TimeSpan.FromSeconds(10));
                if (supervisor.HasOwnedResources) await supervisor.StopAsync();
            }
        }));
        tests.Add(("mapping exporter reads unsaved editor with isolated real utility process", async () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root); store.Save();
            store.Editor.Jog.ScratchSpeed = 4.1;
            var exporter = new MappingExporter(AppHost(), f.PathFor("Temp"));
            var destination = f.PathFor("Windows mapping.djayMidiMapping");
            await exporter.ExportAsync(destination, store.Editor.Jog);
            var doc = XDocument.Load(destination);
            Check.Equal("4.1", doc.Descendants("real").Single().Value);
            Check.Equal(2.7, store.Saved.Jog.ScratchSpeed);
            Check.True(!exporter.IsExporting); Check.Equal(0, Directory.GetDirectories(f.PathFor("Temp")).Length);
        }
        ));
        tests.Add(("mapping install stages outside Djay and preserves an existing mapping", async () =>
        {
            using var f = new Fixture();
            var djay = f.PathFor(Path.Combine("Music", "djay", "MIDI Mappings"));
            Directory.CreateDirectory(djay);
            var existing = Path.Combine(djay, "S4 MK3 Bridge Windows.djayMidiMapping");
            File.WriteAllText(existing, "USER EDIT");
            var exporter = new MappingExporter(AppHost(), f.PathFor("Temp"));
            var jog = new PreferenceStore(f.Resources, f.Root).Editor.Jog;
            var staged = await exporter.StageInstallAsync(f.PathFor("LocalState"), jog);
            Check.True(!Path.GetFullPath(staged).StartsWith(Path.GetFullPath(djay) + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase));
            Check.Equal("S4 MK3 Bridge Windows.djayMidiMapping", Path.GetFileName(staged));
            Check.True(File.Exists(staged));
            Check.Equal("USER EDIT", File.ReadAllText(existing));
            Check.Equal(1, Directory.GetFiles(djay).Length);
        }
        ));
        tests.Add(("mapping export preserves existing mappings without launching replacement", async () =>
        {
            using var f = new Fixture(); var destination = f.PathFor("user.djayMidiMapping");
            File.WriteAllText(destination, "ORIGINAL");
            var jog = new PreferenceStore(f.Resources, f.Root).Editor.Jog;
            var exporter = new MappingExporter(AppHost(), f.PathFor("Temp"));
            await Check.RejectAsync(() => exporter.ExportAsync(destination, jog));
            Check.Equal("ORIGINAL", File.ReadAllText(destination));
            Check.True(!Directory.Exists(f.PathFor("Temp"))); Check.True(!exporter.IsExporting);
        }
        ));
        foreach (var failure in new[] { "nonzero", "missing", "foreign endpoint" })
            tests.Add(($"mapping {failure} output cannot install a destination", async () =>
            {
                using var f = new Fixture(); var jog = new PreferenceStore(f.Resources, f.Root).Editor.Jog;
                if (failure == "nonzero") jog.ScratchReaction = 149;
                else if (failure == "missing") jog.ScratchSpeed = 13;
                else jog.ScratchReaction = 148;
                var exporter = new MappingExporter(AppHost(), f.PathFor("Temp"));
                var destination = f.PathFor("new.djayMidiMapping");
                await Check.RejectAsync(() => exporter.ExportAsync(destination, jog));
                Check.True(!File.Exists(destination)); Check.True(!exporter.IsExporting);
                Check.Equal(0, Directory.GetDirectories(f.PathFor("Temp")).Length);
            }
            ));
        tests.Add(("screen OFF waits for current request acknowledgement through live pipes", async () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root); store.Save();
            var screens = new ScreenCoordinator(store, new TestAccess(true)); var supervisor = new BridgeSupervisor();
            var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            supervisor.Launched += id => screens.Started(id, true);
            supervisor.Protocol += screens.Accept;
            supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
            var (exe, args) = Fixture.Child("managed", "full");
            supervisor.Start(new(exe, args, new() { Profile = "full" }));
            try
            {
                await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
                await screens.SetEnabledAsync(false, supervisor, _ => Task.FromResult(ScreenConsent.Cancel));
                Check.Equal("off", screens.State); Check.True(supervisor.HasChild);
                Check.True(!new PreferenceStore(f.Resources, f.Root).Editor.Screen.Enabled);
            }
            finally { await supervisor.StopAsync(); }
        }
        ));
    }
    private static string AppHost() => Path.Combine(AppContext.BaseDirectory,
        OperatingSystem.IsWindows() ? "S4Bridge.Tests.exe" : "S4Bridge.Tests");

    // Local ownership fault boundary. Real SDK behavior is exercised by the native test executable.
    private sealed class TestEndpointOwner(string receiptPath) : IBloopEndpointOwner
    {
        public TestEndpointLease Lease { get; } = new(receiptPath);
        public Task<IBloopEndpointLease> PrepareAsync()
        {
            File.WriteAllText(Lease.ReceiptPath, "prepared");
            return Task.FromResult<IBloopEndpointLease>(Lease);
        }
    }
    private sealed class TestEndpointLease(string receiptPath) : IBloopEndpointLease
    {
        public Guid AssociationId { get; } = Guid.NewGuid();
        public string? EndpointDeviceId { get; private set; }
        public string ReceiptPath { get; } = receiptPath;
        public int? WorkerId { get; private set; }
        public bool FailCreate { get; set; }
        public bool FailRecordWorker { get; set; }
        public bool FailCleanupOnce { get; set; }
        public Action AssertRelease { get; set; } = () => { };
        public TaskCompletionSource? CreateBarrier { get; set; }
        public TaskCompletionSource CreateEntered { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public async Task CreateAsync()
        {
            CreateEntered.TrySetResult();
            if (CreateBarrier is not null) await CreateBarrier.Task;
            File.WriteAllText(ReceiptPath, "published");
            if (FailCreate) throw new IOException("Endpoint publication failed after its side effect.");
            EndpointDeviceId = "test-bloop-endpoint";
        }
        public void RecordWorker(int processId, DateTime startTimeUtc)
        {
            using var child = System.Diagnostics.Process.GetProcessById(processId);
            Check.Equal(child.StartTime.ToUniversalTime(), startTimeUtc);
            WorkerId = processId;
            if (FailRecordWorker) throw new IOException("Worker receipt write failed.");
        }
        public ValueTask DisposeAsync()
        {
            AssertRelease();
            if (FailCleanupOnce)
            {
                FailCleanupOnce = false;
                throw new IOException("Endpoint removal failed.");
            }
            File.Delete(ReceiptPath);
            return ValueTask.CompletedTask;
        }
    }
}
