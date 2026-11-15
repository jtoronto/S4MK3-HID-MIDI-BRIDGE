using System.Diagnostics;
using System.Text;

namespace S4Bridge.Core;

public sealed class BridgeSupervisor
{
    private readonly object gate = new();
    private readonly SemaphoreSlim inputGate = new(1, 1);
    private Process? process;
    private LaunchState? launch;
    private IBloopEndpointLease? endpoint;
    private Task? starting;
    private Task<bool>? releasing;
    private Task workerExited = Task.CompletedTask;
    private Task completion = Task.CompletedTask;
    private Task? stopping;
    private ulong request;
    public event Action? Changed;
    public event Action<Guid>? Launched;
    public event Action<Guid, string>? Output;
    public event Action<Guid, string>? Protocol;
    public event Action<Guid, string>? StderrLine;
    public event Action<Guid, int>? Exited;
    public LaunchState? Launch { get { lock (gate) return launch; } }
    public bool HasChild { get { lock (gate) return process is not null; } }
    public bool HasOwnedResources
    {
        get { lock (gate) return process is not null || endpoint is not null || starting is { IsCompleted: false }; }
    }
    public string State { get; private set; } = "stopped";
    public string Error { get; private set; } = "";
    public LaunchSpec? ActiveSpec { get; private set; }
    public Task Completion { get { lock (gate) return completion; } }

    public void Start(LaunchSpec spec)
    {
        lock (gate)
        {
            if (HasOwnedResources) throw new InvalidOperationException("Bridge resources are already owned. Stop before starting.");
            stopping = null;
            StartProcess(spec);
        }
        Changed?.Invoke();
    }
    public Task StartAsync(LaunchSpec spec, IBloopEndpointOwner owner)
    {
        lock (gate)
        {
            if (HasOwnedResources) throw new InvalidOperationException("Bridge resources are already owned. Stop before starting.");
            State = "starting";
            Error = "";
            stopping = null;
            starting = Task.Run(() => StartOwned(spec, owner));
            return starting;
        }
    }
    private async Task StartOwned(LaunchSpec spec, IBloopEndpointOwner owner)
    {
        Changed?.Invoke();
        try
        {
            var lease = await owner.PrepareAsync().ConfigureAwait(false);
            lock (gate) endpoint = lease;
            Output?.Invoke(Guid.Empty, $"BLOOP_PREPARED: {lease.AssociationId}");
            await lease.CreateAsync().ConfigureAwait(false);
            Output?.Invoke(Guid.Empty, $"BLOOP_CREATED: {lease.EndpointDeviceId}");
            lock (gate) StartProcess(spec);
            Changed?.Invoke();
        }
        catch (Exception error)
        {
            Output?.Invoke(Guid.Empty, $"BLOOP_START_FAILED: {error.Message}");
            lock (gate) { State = "failed"; Error = error.Message; }
            Exception? cleanupError = null;
            try
            {
                Task? stop;
                lock (gate) stop = process is null ? null : StopOwned(process, launch!.Id);
                if (stop is not null) await stop.ConfigureAwait(false);
                else await ReleaseEndpoint().ConfigureAwait(false);
            }
            catch (Exception cleanup) { cleanupError = cleanup; }
            lock (gate) State = "failed";
            Changed?.Invoke();
            if (cleanupError is not null)
                throw new IOException(Error, new AggregateException(error, cleanupError));
            throw;
        }
    }
    // Called under gate; the lease remains owned if starting the process or recording it fails.
    private void StartProcess(LaunchSpec spec)
    {
        var child = new Process { StartInfo = spec.StartInfo() };
        var id = Guid.NewGuid();
        try { child.Start(); }
        catch { child.Dispose(); throw; }
        process = child;
        launch = new(id, child.Id, spec.Run.Profile, spec.MidiBackend);
        ActiveSpec = spec;
        request = 0;
        State = spec.Diagnostic ? "checking" : "starting";
        Error = "";
        try { endpoint?.RecordWorker(child.Id, child.StartTime.ToUniversalTime()); }
        finally
        {
            Launched?.Invoke(id);
            var stdout = Drain(child.StandardOutput, child, id, true);
            var stderr = Drain(child.StandardError, child, id, false);
            workerExited = Task.WhenAll(child.WaitForExitAsync(), stdout, stderr);
            completion = ObserveExit(child, id, workerExited);
        }
    }
    private bool Current(Process child, Guid id) =>
        ReferenceEquals(process, child) && launch?.Id == id && launch.ProcessId == child.Id;

    private async Task Drain(StreamReader reader, Process child, Guid id, bool stdout)
    {
        var buffer = new char[4096];
        var line = new StringBuilder();
        var discarded = false;
        try
        {
            while (await reader.ReadAsync(buffer).ConfigureAwait(false) is var count && count > 0)
            {
                for (var i = 0; i < count; i++)
                {
                    var c = buffer[i];
                    if (c == '\n')
                    {
                        Ingest(child, id, discarded ? "[oversized output line rejected]" :
                            line.ToString().TrimEnd('\r'), stdout);
                        line.Clear(); discarded = false;
                    }
                    else if (!discarded)
                    {
                        if (line.Length == 65536) { line.Clear(); discarded = true; }
                        else line.Append(c);
                    }
                }
            }
            if (line.Length > 0) Ingest(child, id, line.ToString(), stdout);
        }
        catch (Exception ex) when (ex is IOException or ObjectDisposedException)
        {
            lock (gate)
            {
                if (!Current(child, id)) return;
                Error = $"Process output read failed: {ex.Message}";
                State = "failed";
            }
            Changed?.Invoke();
        }
    }
    private void Ingest(Process child, Guid id, string text, bool stdout)
    {
        bool changed = false;
        lock (gate)
        {
            if (!Current(child, id)) return;
            if (stdout && text.StartsWith(ManagedStatus.Prefix, StringComparison.Ordinal))
            {
                try
                {
                    if (!launch!.Accept(ManagedStatus.Parse(text)))
                        text = "Rejected stale/mismatched lifecycle status: " + text;
                    else
                    {
                        State = launch.State;
                        if (launch.Status?.Error is { } failure) Error = failure;
                        changed = true;
                    }
                }
                catch (Exception ex) when (ex is System.Text.Json.JsonException or InvalidDataException
                         or KeyNotFoundException or InvalidOperationException or FormatException)
                { text = $"Rejected invalid lifecycle status ({ex.Message}): {text}"; }
            }
            if (stdout) Protocol?.Invoke(id, text);
            else StderrLine?.Invoke(id, text);
            Output?.Invoke(id, text);
        }
        if (changed) Changed?.Invoke();
    }
    private async Task ObserveExit(Process child, Guid id, Task readers)
    {
        await readers.ConfigureAwait(false);
        var exit = child.ExitCode;
        bool failed;
        lock (gate)
        {
            if (!Current(child, id)) return;
            failed = exit != 0 || State == "failed";
            State = endpoint is null ? (failed ? "failed" : "stopped") : "stopping";
            if (exit != 0) Error = $"Bridge exited with status {exit}. See Diagnostics.";
            process = null;
            child.Dispose();
        }
        var removed = await ReleaseEndpoint().ConfigureAwait(false);
        lock (gate) State = failed || !removed ? "failed" : "stopped";
        Output?.Invoke(id, $"EXIT: {exit}");
        Exited?.Invoke(id, exit);
        Changed?.Invoke();
    }
    public Task StopAsync()
    {
        lock (gate)
        {
            if (starting is { IsCompleted: false })
                return stopping ??= StopAfterStart(starting);
            return StopResources();
        }
    }
    private async Task StopAfterStart(Task pending)
    {
        try { await pending.ConfigureAwait(false); }
        catch (Exception error) { Output?.Invoke(Guid.Empty, $"START_ENDED: {error.Message}"); }
        Task stop;
        lock (gate) { stopping = null; stop = StopResources(); }
        await stop.ConfigureAwait(false);
    }
    private Task StopResources()
    {
        if (process is not null) return stopping ??= StopOwned(process, launch!.Id);
        if (endpoint is null) return Task.CompletedTask;
        return stopping is { IsCompleted: false } ? stopping : stopping = CleanupOnly();
    }
    private async Task CleanupOnly()
    {
        State = "stopping";
        Changed?.Invoke();
        if (!await ReleaseEndpoint().ConfigureAwait(false)) throw new IOException(Error);
        lock (gate) { State = "stopped"; Error = ""; }
        Changed?.Invoke();
    }
    private Task<bool> ReleaseEndpoint()
    {
        lock (gate)
        {
            if (endpoint is null) return Task.FromResult(true);
            return releasing is { IsCompleted: false } ? releasing : releasing = ReleaseOwnedEndpoint(endpoint);
        }
    }
    private async Task<bool> ReleaseOwnedEndpoint(IBloopEndpointLease lease)
    {
        try
        {
            await lease.DisposeAsync().ConfigureAwait(false);
            lock (gate) endpoint = null;
            Output?.Invoke(Guid.Empty, $"BLOOP_REMOVED: {lease.AssociationId}");
            return true;
        }
        catch (Exception error)
        {
            lock (gate)
            {
                State = "failed";
                var detail = $"BLOOP endpoint cleanup failed: {error.Message}. Use Stop to retry.";
                Error = string.IsNullOrWhiteSpace(Error) ? detail : Error + "\n" + detail;
            }
            Changed?.Invoke();
            return false;
        }
    }
    private async Task StopOwned(Process child, Guid id)
    {
        Task exited;
        Task worker;
        ulong stopRequest;
        lock (gate)
        {
            var next = checked(++request);
            stopRequest = next;
            launch!.RequestStop(next);
            State = "stopping";
            exited = completion;
            worker = workerExited;
        }
        Changed?.Invoke();
        try
        {
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
            await WriteInputAsync(child, id, $"S4_BRIDGE_CONTROL {stopRequest} STOP", true, deadline.Token).ConfigureAwait(false);
            await worker.WaitAsync(deadline.Token).ConfigureAwait(false);
        }
        catch (Exception ex) when (ex is OperationCanceledException or IOException or ObjectDisposedException)
        {
            lock (gate)
            {
                if (Current(child, id))
                {
                    launch!.Unconfirmed();
                    State = "failed";
                    Error = $"Stop unconfirmed: {ex.Message}. Process and endpoint retained; neutral state unknown. No automatic kill or service restoration.";
                    Changed?.Invoke();
                    throw new IOException(Error, ex);
                }
            }
        }
        // The native SDK call is not cancellable. Retain ownership until it finishes.
        await exited.ConfigureAwait(false);
        if (HasOwnedResources && endpoint is not null) throw new IOException(Error);
    }
    public async Task SendScreenAsync(ulong id, string command)
    {
        Process child;
        Guid launchId;
        lock (gate)
        {
            if (process is null || ActiveSpec?.Run.Profile != "full" || State is "stopping" or "failed")
                throw new InvalidOperationException("Start a healthy full-profile bridge first.");
            if (command is not ("ON" or "OFF" or "RETRY")) throw new InvalidDataException("Invalid screen command.");
            child = process;
            launchId = launch!.Id;
        }
        await WriteInputAsync(child, launchId, $"S4_SCREEN_CONTROL {id} {command}", false, CancellationToken.None);
    }
    public async Task SendMidiCaptureMarkerAsync(ulong id)
    {
        Process child;
        Guid launchId;
        lock (gate)
        {
            if (process is null || ActiveSpec?.Run.Profile != "full" || State != "ready")
                throw new InvalidOperationException("Start a healthy full-profile bridge first.");
            child = process;
            launchId = launch!.Id;
        }
        await WriteInputAsync(child, launchId, $"S4_MIDI_CAPTURE_MARK {id} issue", false, CancellationToken.None);
    }
    private async Task WriteInputAsync(Process child, Guid id, string line, bool stoppingCommand, CancellationToken cancellation)
    {
        await inputGate.WaitAsync(cancellation).ConfigureAwait(false);
        try
        {
            lock (gate)
            {
                if (!Current(child, id) || (!stoppingCommand && State is "stopping" or "failed"))
                    throw new IOException("Bridge input is no longer available for this command.");
            }
            await child.StandardInput.WriteLineAsync(line.AsMemory(), cancellation).ConfigureAwait(false);
            await child.StandardInput.FlushAsync(cancellation).ConfigureAwait(false);
        }
        finally { inputGate.Release(); }
    }
    public void CloseInput()
    {
        inputGate.Wait();
        try { lock (gate) process?.StandardInput.Close(); }
        finally { inputGate.Release(); }
    }
}
