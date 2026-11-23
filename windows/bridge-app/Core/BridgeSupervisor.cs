using System.Diagnostics;

namespace S4Bridge.Core;

// allow: SIZE_OK -- Keep the owned-process lifecycle under one lock and launch identity; startup and output have separate partials.
public sealed partial class BridgeSupervisor
{
    private readonly object gate = new();
    private readonly SemaphoreSlim inputGate = new(1, 1);
    private Process? process;
    private LaunchState? launch;
    private IBloopEndpointLease? endpoint;
    private DiagnosticMotorReadiness? motorReadiness;
    private Task? starting;
    private Task<bool>? releasing;
    private Task workerExited = Task.CompletedTask;
    private Task completion = Task.CompletedTask;
    private Task? stopping;
    private CancellationTokenSource? preparationStop;
    private TaskCompletionSource? prepared;
    private TaskCompletionSource? startupEndpointBarrier;
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
            prepared = null;
            startupEndpointBarrier = null;
            preparationStop = null;
            StartProcess(spec);
        }
        Changed?.Invoke();
    }
    // Called under gate; the lease remains owned if starting the process or recording it fails.
    private void StartProcess(LaunchSpec spec, bool recordWorker = true)
    {
        var id = Guid.NewGuid();
        motorReadiness = DiagnosticMotorReadiness.Prepare(spec, id);
        var child = new Process { StartInfo = spec.StartInfo() };
        try { child.Start(); }
        catch { child.Dispose(); throw; }
        process = child;
        launch = new(id, child.Id, spec.Run.Profile, spec.MidiBackend);
        ActiveSpec = spec;
        request = 0;
        State = spec.Diagnostic ? "checking" : "starting";
        Error = "";
        try
        {
            motorReadiness?.Bind(child.Id);
            if (recordWorker) endpoint?.RecordWorker(child.Id, child.StartTime.ToUniversalTime());
        }
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

    private async Task ObserveExit(Process child, Guid id, Task readers)
    {
        await readers.ConfigureAwait(false);
        var exit = child.ExitCode;
        bool failed;
        Task? barrier;
        lock (gate)
        {
            if (!Current(child, id)) return;
            failed = exit != 0 || State == "failed";
            State = endpoint is null ? (failed ? "failed" : "stopped") : "stopping";
            motorReadiness?.Stop();
            if (exit != 0) Error = $"Bridge exited with status {exit}. See Diagnostics.";
            process = null;
            child.Dispose();
            barrier = startupEndpointBarrier?.Task;
        }
        if (barrier is not null) await barrier.ConfigureAwait(false);
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
            {
                preparationStop?.Cancel();
                return stopping ??= StopAfterStart(starting, preparationStop is not null);
            }
            return StopResources();
        }
    }
    private async Task StopAfterStart(Task pending, bool diagnosticPreparation)
    {
        try { await pending.ConfigureAwait(false); }
        catch (Exception error) { Output?.Invoke(Guid.Empty, $"START_ENDED: {error.Message}"); }
        Task stop;
        lock (gate) { stopping = null; stop = StopResources(); }
        await stop.ConfigureAwait(false);
        if (diagnosticPreparation) lock (gate)
        {
            if (process is null && endpoint is null) { State = "stopped"; Error = ""; }
        }
        Changed?.Invoke();
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
