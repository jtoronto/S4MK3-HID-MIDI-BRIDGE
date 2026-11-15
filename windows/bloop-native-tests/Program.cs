using System.Diagnostics;
using System.Text.Json;
using S4Bridge.Core;
using S4Bridge.Platform;
using Windows.Devices.Midi2.ServiceConfig;
using Windows.Devices.Midi2.Transports.BasicLoopback;

namespace S4LoopBeProbe;

internal static class Program
{
    internal const string PortName = BloopEndpoint.PortName;
    internal const uint Forward = 0x00113C90;
    internal const uint Reverse = 0x00223D90;
    private static readonly object LogLock = new();
    private static string? logPath;

    private static async Task<int> Main(string[] args)
    {
        try
        {
            if (args.Length == 1 && args[0] == "--peer")
            {
                await Exchange.PeerAsync();
                return 0;
            }
            if (args.Length == 1 && args[0] == "--hold-worker")
            {
                Emit("worker-ready", new { });
                await Console.In.ReadLineAsync();
                return 0;
            }
            if (args.Length == 2 && args[0] == "--stale-child")
            {
                var childLease = await new BloopEndpointOwner(args[1]).PrepareAsync();
                await childLease.CreateAsync();
                Emit("stale-child-created", new { childLease.AssociationId, childLease.EndpointDeviceId });
                return 0; // Simulate host exit without disposing the lease.
            }
            if (args.Length != 2 || args[0] != "--bloop-native")
                throw new ArgumentException("Usage: BloopNativeTests.exe --bloop-native <fresh evidence-directory>");
            var evidence = Path.GetFullPath(args[1]);
            if (Directory.Exists(evidence) || File.Exists(evidence))
                throw new IOException("The evidence directory must not already exist.");
            Directory.CreateDirectory(evidence);
            logPath = Path.Combine(evidence, "native-events.jsonl");
            await MainScenario(Path.Combine(evidence, "main"));
            await StaleScenario(Path.Combine(evidence, "stale"));
            await ForeignScenario(Path.Combine(evidence, "foreign"));
            await MalformedScenario(Path.Combine(evidence, "malformed"));
            Emit("all-passed", new { });
            return 0;
        }
        catch (Exception error)
        {
            Emit("failure", new { type = error.GetType().FullName, error.Message, detail = error.ToString() });
            return 1;
        }
    }

    private static async Task MainScenario(string root)
    {
        var owner = new BloopEndpointOwner(root);
        var lease = await owner.PrepareAsync();
        var id = lease.AssociationId;
        try
        {
            Emit("prepared", new
            {
                lease.AssociationId,
                configurationFilePath = MidiServiceTransportPluginConfigManager.ConfigFilePath
            });
            using (var receipt = JsonDocument.Parse(File.ReadAllText(Path.Combine(root, "bloop-owner.json"))))
            {
                Require(receipt.RootElement.GetProperty("AssociationId").GetGuid() == id, "Preassigned association was not receipted.");
                Require(receipt.RootElement.GetProperty("EndpointDeviceId").ValueKind == JsonValueKind.Null,
                    "Prepared receipt unexpectedly contains an endpoint.");
            }
            Require(!Active().Any(x => x.AssociationId == id),
                "Prepare published the endpoint.");
            await lease.CreateAsync();
            Require(!string.IsNullOrWhiteSpace(lease.EndpointDeviceId), "Create did not return an endpoint ID.");
            using (var receipt = JsonDocument.Parse(File.ReadAllText(Path.Combine(root, "bloop-owner.json"))))
                Require(receipt.RootElement.GetProperty("EndpointDeviceId").GetString() == lease.EndpointDeviceId,
                    "Created endpoint ID was not receipted.");
            var ports = NativePort.Enumerate().Where(p => p.Name == PortName).ToArray();
            Require(ports.Count(p => p.Input) == 1 && ports.Count(p => !p.Input) == 1,
                "Expected exactly one legacy input and one legacy output.");
            Emit("legacy-discovery", new { ports });
            using var port = NativePort.Open(Reverse);
            await VerifyExchange(port);
            await VerifyWorkerLease(lease, root);
        }
        finally
        {
            await lease.DisposeAsync();
        }
        Require(!File.Exists(Path.Combine(root, "bloop-owner.json")), "Receipt survived successful cleanup.");
        Require(!Active().Any(x => x.AssociationId == id),
            "Owned association survived successful cleanup.");
        Emit("exact-removal", new { associationId = id });
    }

    private static async Task VerifyWorkerLease(IBloopEndpointLease lease, string root)
    {
        using var worker = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = Environment.ProcessPath ?? throw new InvalidOperationException("Cannot locate native test executable."),
                UseShellExecute = false,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                CreateNoWindow = true
            }
        };
        worker.StartInfo.ArgumentList.Add("--hold-worker");
        if (!worker.Start()) throw new InvalidOperationException("Cannot start test worker.");
        try
        {
            var ready = await worker.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(15));
            Require(ready != null && JsonDocument.Parse(ready).RootElement.GetProperty("stage").GetString() == "worker-ready",
                "Test worker did not become ready.");
            lease.RecordWorker(worker.Id, worker.StartTime.ToUniversalTime());
            using (var receipt = JsonDocument.Parse(File.ReadAllText(Path.Combine(root, "bloop-owner.json"))))
                Require(receipt.RootElement.GetProperty("WorkerPid").GetInt32() == worker.Id,
                    "Worker identity was not receipted.");
            var blocked = false;
            try { await lease.DisposeAsync(); }
            catch (InvalidOperationException) { blocked = true; }
            Require(blocked, "Cleanup did not block while the recorded worker was live.");
            Emit("live-worker-protected", new { worker.Id });
        }
        finally
        {
            worker.StandardInput.Close();
            try { await worker.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(15)); }
            catch (TimeoutException)
            {
                worker.Kill(entireProcessTree: true);
                await worker.WaitForExitAsync();
                throw;
            }
        }
    }

    private static async Task VerifyExchange(NativePort port)
    {
        using var peer = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = Environment.ProcessPath ?? throw new InvalidOperationException("Cannot locate native test executable."),
                UseShellExecute = false,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            }
        };
        peer.StartInfo.ArgumentList.Add("--peer");
        if (!peer.Start()) throw new InvalidOperationException("Cannot start MIDI exchange peer.");
        var errors = peer.StandardError.ReadToEndAsync();
        try
        {
            while (true)
            {
                var line = await peer.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(15));
                if (line == null) throw new InvalidOperationException("MIDI exchange peer exited before becoming ready.");
                using var record = JsonDocument.Parse(line);
                Emit("peer-event", new { line });
                if (record.RootElement.GetProperty("stage").GetString() == "peer-ready") break;
            }
            port.Send(Forward);
            Require(await port.Received.WaitAsync(TimeSpan.FromSeconds(15)) == Reverse, "MIDI return message did not match.");
            await peer.StandardInput.WriteLineAsync("received");
            await peer.StandardInput.FlushAsync();
            await peer.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(20));
            Emit("peer-exit", new { peer.ExitCode, output = await peer.StandardOutput.ReadToEndAsync(), errors = await errors });
            Require(peer.ExitCode == 0, "MIDI exchange peer failed.");
            Emit("round-trip", new { forward = Forward, reverse = Reverse });
        }
        finally
        {
            if (!peer.HasExited)
            {
                peer.Kill(entireProcessTree: true);
                await peer.WaitForExitAsync();
            }
        }
    }

    private static async Task StaleScenario(string root)
    {
        using var child = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = Environment.ProcessPath ?? throw new InvalidOperationException("Cannot locate native test executable."),
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            }
        };
        child.StartInfo.ArgumentList.Add("--stale-child");
        child.StartInfo.ArgumentList.Add(root);
        if (!child.Start()) throw new InvalidOperationException("Cannot start stale-owner child.");
        var stdout = child.StandardOutput.ReadToEndAsync();
        var stderr = child.StandardError.ReadToEndAsync();
        try
        {
            await child.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(30));
            var output = await stdout;
            var error = await stderr;
            Emit("stale-child-exit", new { child.ExitCode, output, error });
            Require(child.ExitCode == 0, "Stale-owner child failed.");
            using var receipt = JsonDocument.Parse(File.ReadAllText(Path.Combine(root, "bloop-owner.json")));
            var oldId = receipt.RootElement.GetProperty("AssociationId").GetGuid();
            Require(Active().Any(x => x.AssociationId == oldId),
                "Stale child did not leave its association active.");
            var recovered = await new BloopEndpointOwner(root).PrepareAsync();
            try
            {
                Require(!Active().Any(x => x.AssociationId == oldId),
                    "Stale association was not removed.");
                Emit("stale-recovered", new { oldId, replacement = recovered.AssociationId });
            }
            finally { await recovered.DisposeAsync(); }
        }
        finally
        {
            if (!child.HasExited)
            {
                child.Kill(entireProcessTree: true);
                await child.WaitForExitAsync();
            }
            var receiptPath = Path.Combine(root, "bloop-owner.json");
            if (File.Exists(receiptPath))
            {
                var cleanup = await new BloopEndpointOwner(root).PrepareAsync();
                await cleanup.DisposeAsync();
            }
        }
    }

    private static async Task ForeignScenario(string root)
    {
        var definition = new MidiBasicLoopbackEndpointDefinition
        {
            Name = PortName,
            Description = "Foreign native-test endpoint",
            UniqueId = "s4-test-" + Guid.NewGuid().ToString("N")[..16]
        };
        var config = new MidiBasicLoopbackCreationConfig(definition) { IsMuted = false };
        Directory.CreateDirectory(root);
        var fixtureReceipt = Path.Combine(root, "foreign-fixture.json");
        File.WriteAllText(fixtureReceipt, JsonSerializer.Serialize(new
        {
            config.AssociationId, definition.Name, definition.Description, definition.UniqueId
        }));
        try
        {
            var response = MidiBasicLoopbackManager.CreateTransientLoopback(config);
            Require(response != null && response.Success && response.CreatedLoopbackEntry != null,
                "Could not create the foreign collision fixture.");
            var blocked = false;
            try
            {
                var unexpected = await new BloopEndpointOwner(root).PrepareAsync();
                await unexpected.DisposeAsync();
            }
            catch (InvalidOperationException) { blocked = true; }
            Require(blocked, "A foreign same-name endpoint did not block preparation.");
            Require(Active().Any(x => x.AssociationId == config.AssociationId),
                "The foreign endpoint was removed.");
            Emit("foreign-preserved", new { config.AssociationId });
        }
        finally
        {
            var matching = Active()
                .Where(x => x.AssociationId == config.AssociationId).ToArray();
            Require(matching.Length <= 1, "Foreign fixture association is ambiguous.");
            if (matching.Length == 1)
            {
                Require(matching[0].Name == definition.Name && matching[0].Description == definition.Description,
                    "Foreign fixture metadata changed; it was not removed.");
                var removal = MidiBasicLoopbackManager.RemoveTransientLoopback(new MidiBasicLoopbackRemovalConfig(config.AssociationId));
                Require(removal != null && removal.Success, "Foreign test fixture cleanup failed.");
                Require(!Active().Any(x => x.AssociationId == config.AssociationId),
                    "Foreign fixture association survived removal.");
            }
            File.Delete(fixtureReceipt);
        }
    }

    private static async Task MalformedScenario(string root)
    {
        Directory.CreateDirectory(root);
        var path = Path.Combine(root, "bloop-owner.json");
        const string invalid = "{invalid";
        File.WriteAllText(path, invalid);
        var blocked = false;
        try
        {
            var unexpected = await new BloopEndpointOwner(root).PrepareAsync();
            await unexpected.DisposeAsync();
        }
        catch (JsonException) { blocked = true; }
        Require(blocked && File.ReadAllText(path) == invalid, "Malformed receipt was not preserved and rejected.");
        File.Delete(path);
        Emit("malformed-rejected", new { });
    }

    private static void Require(bool condition, string message)
    {
        if (!condition) throw new InvalidOperationException(message);
    }

    private static List<(Guid AssociationId, string? Name, string? Description)> Active()
    {
        var response = MidiServiceTransportPluginConfigManager.SendCommand(
            new MidiServiceTransportCommand(MidiBasicLoopbackManager.TransportId, MidiServiceTransportCommonCommands.ListEntries));
        var payload = response?.ResponseJson;
        if (response == null || response.Status != MidiServiceConfigResponseStatus.Success || payload == null)
            throw new InvalidOperationException("The transport did not return a successful endpoint list.");
        using var json = JsonDocument.Parse(payload.Stringify());
        var entries = json.RootElement.GetProperty("entries");
        Require(entries.ValueKind == JsonValueKind.Array, "The endpoint list is not an array.");
        return entries.EnumerateArray().Select(x => (
            Guid.Parse(x.GetProperty("associationIdentifier").GetString()!),
            x.GetProperty("name").GetString(),
            x.GetProperty("description").GetString())).ToList();
    }

    internal static void Emit(string stage, object data)
    {
        var line = JsonSerializer.Serialize(new { stage, data });
        lock (LogLock)
        {
            Console.WriteLine(line);
            if (logPath != null) File.AppendAllText(logPath, line + Environment.NewLine);
        }
    }
}
