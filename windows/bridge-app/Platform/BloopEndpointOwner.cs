using System.Diagnostics;
using System.Text.Json;
using S4Bridge.Core;
using Windows.Devices.Midi2;
using Windows.Devices.Midi2.ServiceConfig;
using Windows.Devices.Midi2.Transports.BasicLoopback;

namespace S4Bridge.Platform;

public sealed class BloopEndpointOwner(string root) : IBloopEndpointOwner
{
    private const string Description = "Transient MIDI endpoint owned by S4 MK3 Bridge.";
    private readonly string receiptPath = Path.Combine(root, "bloop-owner.json");

    public Task<IBloopEndpointLease> PrepareAsync()
    {
        Directory.CreateDirectory(root);
        var guard = new FileStream(Path.Combine(root, "bloop-owner.lock"), FileMode.OpenOrCreate,
            FileAccess.ReadWrite, FileShare.None);
        try
        {
            if (!MidiApi.EnsureServiceAvailable())
                throw new InvalidOperationException("Windows MIDI Services is unavailable. Install or enable the supported Windows MIDI Services components.");
            if (!MidiBasicLoopbackManager.IsTransportAvailable)
                throw new InvalidOperationException("Basic Loopback transport is unavailable. Install the supported Windows MIDI Services Preview 13 transports.");

            if (File.Exists(receiptPath))
            {
                var old = ReadReceipt(receiptPath);
                EnsureAbsent(old.HostPid, old.HostStartUtc);
                if (old.WorkerPid is int pid && old.WorkerStartUtc is DateTime start)
                    EnsureAbsent(pid, start);
                Cleanup(receiptPath, old);
            }

            CheckSaved(null);
            var entries = Active();
            if (entries.Any(e => e.Name.Equals(BloopEndpoint.PortName, StringComparison.OrdinalIgnoreCase)))
                throw new InvalidOperationException("An unowned Basic Loopback endpoint already has the bridge name; it was not changed.");

            var definition = new MidiBasicLoopbackEndpointDefinition
            {
                Name = BloopEndpoint.PortName,
                Description = Description,
                UniqueId = BloopEndpoint.UniqueId
            };
            var config = new MidiBasicLoopbackCreationConfig(definition) { IsMuted = false };
            using var process = Process.GetCurrentProcess();
            var receipt = new Receipt(1, config.AssociationId, BloopEndpoint.PortName, Description,
                BloopEndpoint.UniqueId, null, process.Id, process.StartTime.ToUniversalTime(), null, null);
            if (receipt.AssociationId == Guid.Empty)
                throw new InvalidOperationException("The SDK did not assign a Basic Loopback association.");
            Save(receiptPath, receipt);
            return Task.FromResult<IBloopEndpointLease>(new Lease(receiptPath, receipt, config, guard));
        }
        catch
        {
            guard.Dispose();
            throw;
        }
    }

    private sealed record Receipt(int Version, Guid AssociationId, string Name, string Description,
        string UniqueId, string? EndpointDeviceId, int HostPid, DateTime HostStartUtc,
        int? WorkerPid, DateTime? WorkerStartUtc);

    private sealed record ActiveEntry(Guid AssociationId, string EndpointDeviceId, string Name, string Description);

    private static Receipt ReadReceipt(string path)
    {
        var receipt = JsonSerializer.Deserialize<Receipt>(File.ReadAllText(path))
            ?? throw new InvalidDataException("The BLOOP ownership receipt is empty.");
        if (receipt.Version != 1 || receipt.AssociationId == Guid.Empty ||
            receipt.Name != BloopEndpoint.PortName || receipt.Description != Description ||
            receipt.UniqueId != BloopEndpoint.UniqueId || receipt.HostPid <= 0 ||
            receipt.HostStartUtc.Kind != DateTimeKind.Utc || receipt.HostStartUtc == default ||
            (receipt.WorkerPid.HasValue != receipt.WorkerStartUtc.HasValue) ||
            (receipt.WorkerPid.HasValue && (receipt.WorkerPid <= 0 ||
                receipt.WorkerStartUtc!.Value.Kind != DateTimeKind.Utc || receipt.WorkerStartUtc == default)) ||
            receipt.EndpointDeviceId == "")
            throw new InvalidDataException("The BLOOP ownership receipt is invalid; it has been preserved.");
        return receipt;
    }

    private static void Save(string path, Receipt receipt) =>
        PreferenceStore.AtomicWrite(path, JsonSerializer.Serialize(receipt));

    private static void EnsureAbsent(int pid, DateTime start)
    {
        Process process;
        try
        {
            process = Process.GetProcessById(pid);
        }
        catch (ArgumentException)
        {
            return;
        }
        using (process)
            if (process.StartTime.ToUniversalTime() == start && !process.HasExited)
                throw new InvalidOperationException($"BLOOP owner process {pid} is still running.");
    }

    private static List<ActiveEntry> Active()
    {
        // The Preview 13 convenience enumerator turns failures into an empty list.
        // Keep the status-bearing command and its pinned transport JSON in this boundary.
        var command = new MidiServiceTransportCommand(MidiBasicLoopbackManager.TransportId,
            MidiServiceTransportCommonCommands.ListEntries);
        var response = MidiServiceTransportPluginConfigManager.SendCommand(command);
        if (response == null || response.Status != MidiServiceConfigResponseStatus.Success || response.ResponseJson == null)
            throw new InvalidOperationException("Cannot reliably enumerate Basic Loopback endpoints; ownership was preserved.");
        using var json = JsonDocument.Parse(response.ResponseJson.Stringify());
        if (!json.RootElement.TryGetProperty("entries", out var entries) || entries.ValueKind != JsonValueKind.Array)
            throw new InvalidDataException("Basic Loopback returned an invalid endpoint list.");
        var result = new List<ActiveEntry>();
        foreach (var item in entries.EnumerateArray())
        {
            if (!Guid.TryParse(item.GetProperty("associationIdentifier").GetString(), out var id) || id == Guid.Empty)
                throw new InvalidDataException("Basic Loopback returned an invalid association.");
            result.Add(new ActiveEntry(id, item.GetProperty("endpointId").GetString() ?? "",
                item.GetProperty("name").GetString() ?? "", item.GetProperty("description").GetString() ?? ""));
        }
        return result;
    }

    private static void CheckSaved(Guid? association)
    {
        var path = MidiServiceTransportPluginConfigManager.ConfigFilePath;
        // The SDK documents an empty path as no registered configuration file.
        if (string.IsNullOrEmpty(path)) return;
        using var json = JsonDocument.Parse(File.ReadAllText(path));
        if (json.RootElement.ValueKind != JsonValueKind.Object)
            throw new InvalidDataException("The Windows MIDI Services configuration is not an object.");
        if (!json.RootElement.TryGetProperty("endpointTransportPluginSettings", out var plugins))
            return;
        if (plugins.ValueKind != JsonValueKind.Object)
            throw new InvalidDataException("The Windows MIDI Services transport configuration is invalid.");
        foreach (var plugin in plugins.EnumerateObject())
        {
            if (!Guid.TryParse(plugin.Name, out var transport) || transport != MidiBasicLoopbackManager.TransportId)
                continue;
            if (plugin.Value.ValueKind != JsonValueKind.Object)
                throw new InvalidDataException("The saved Basic Loopback configuration is invalid.");
            if (!plugin.Value.TryGetProperty("create", out var create))
                continue;
            if (create.ValueKind != JsonValueKind.Object)
                throw new InvalidDataException("The saved Basic Loopback entries are invalid.");
            foreach (var entry in create.EnumerateObject())
            {
                if (!Guid.TryParse(entry.Name, out var id) || entry.Value.ValueKind != JsonValueKind.Object)
                    throw new InvalidDataException("The saved Basic Loopback association is invalid.");
                if (association == id)
                    throw new InvalidOperationException("The owned Basic Loopback association is saved; it will not be removed.");
                if (!entry.Value.TryGetProperty("endpoint", out var endpoint) || endpoint.ValueKind != JsonValueKind.Object)
                    throw new InvalidDataException("The saved Basic Loopback endpoint is invalid.");
                if ((endpoint.TryGetProperty("name", out var name) && name.GetString() == BloopEndpoint.PortName) ||
                    (endpoint.TryGetProperty("uniqueIdentifier", out var uid) && uid.GetString() == BloopEndpoint.UniqueId))
                    throw new InvalidOperationException("A saved Basic Loopback conflicts with the bridge; it was not changed.");
            }
        }
    }

    private static void Cleanup(string path, Receipt receipt)
    {
        CheckSaved(receipt.AssociationId);
        var matching = Active().Where(e => e.AssociationId == receipt.AssociationId).ToArray();
        if (matching.Length > 1)
            throw new InvalidDataException("Multiple Basic Loopback entries share the receipt association.");
        if (matching.Length == 1)
        {
            var entry = matching[0];
            if (entry.Name != receipt.Name || entry.Description != receipt.Description ||
                string.IsNullOrWhiteSpace(entry.EndpointDeviceId) ||
                (receipt.EndpointDeviceId != null && !entry.EndpointDeviceId.Equals(receipt.EndpointDeviceId, StringComparison.OrdinalIgnoreCase)))
                throw new InvalidOperationException("Basic Loopback metadata does not match the receipt; it was not removed.");
            var result = MidiBasicLoopbackManager.RemoveTransientLoopback(new MidiBasicLoopbackRemovalConfig(receipt.AssociationId));
            if (result == null || !result.Success)
                throw new InvalidOperationException($"Basic Loopback removal failed: {result?.ErrorCode} {result?.ErrorMessage}");
            if (Active().Any(e => e.AssociationId == receipt.AssociationId))
                throw new InvalidOperationException("The Basic Loopback association is still present.");
        }
        CheckSaved(receipt.AssociationId);
        var current = ReadReceipt(path);
        if (current != receipt)
            throw new InvalidOperationException("The BLOOP ownership receipt changed during cleanup.");
        File.Delete(path);
    }

    private sealed class Lease(string path, Receipt initial, MidiBasicLoopbackCreationConfig config,
        FileStream guard) : IBloopEndpointLease
    {
        private readonly SemaphoreSlim mutex = new(1, 1);
        private Receipt receipt = initial;
        private bool disposed;
        private bool createAttempted;

        public Guid AssociationId => receipt.AssociationId;
        public string? EndpointDeviceId => receipt.EndpointDeviceId;

        public async Task CreateAsync()
        {
            await mutex.WaitAsync().ConfigureAwait(false);
            try
            {
                ObjectDisposedException.ThrowIf(disposed, this);
                if (createAttempted)
                    throw new InvalidOperationException("Basic Loopback creation was already attempted.");
                createAttempted = true;
                var response = await Task.Run(() => MidiBasicLoopbackManager.CreateTransientLoopback(config)).ConfigureAwait(false);
                if (response == null || !response.Success || response.CreatedLoopbackEntry == null)
                    throw new InvalidOperationException($"Basic Loopback creation failed: {response?.ErrorCode} {response?.ErrorMessage}");
                var entry = response.CreatedLoopbackEntry;
                if (entry.AssociationId != receipt.AssociationId || string.IsNullOrWhiteSpace(entry.EndpointDeviceId) ||
                    entry.Name != receipt.Name || entry.Description != receipt.Description)
                    throw new InvalidOperationException("Basic Loopback returned mismatched endpoint metadata.");
                var updated = receipt with { EndpointDeviceId = entry.EndpointDeviceId };
                Save(path, updated);
                receipt = updated;
            }
            finally { mutex.Release(); }
        }

        public void RecordWorker(int processId, DateTime startTimeUtc)
        {
            mutex.Wait();
            try
            {
                ObjectDisposedException.ThrowIf(disposed, this);
                if (receipt.EndpointDeviceId == null || processId <= 0 || startTimeUtc.Kind != DateTimeKind.Utc || startTimeUtc == default)
                    throw new ArgumentException("Worker identity requires a created endpoint, a positive PID and UTC start time.");
                var updated = receipt with { WorkerPid = processId, WorkerStartUtc = startTimeUtc };
                Save(path, updated);
                receipt = updated;
            }
            finally { mutex.Release(); }
        }

        public async ValueTask DisposeAsync()
        {
            await mutex.WaitAsync().ConfigureAwait(false);
            try
            {
                if (disposed) return;
                if (receipt.WorkerPid is int pid && receipt.WorkerStartUtc is DateTime start)
                    EnsureAbsent(pid, start);
                await Task.Run(() => Cleanup(path, receipt)).ConfigureAwait(false);
                disposed = true;
                guard.Dispose();
            }
            finally { mutex.Release(); }
        }
    }
}
