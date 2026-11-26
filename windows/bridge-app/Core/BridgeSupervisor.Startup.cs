using System.Diagnostics;

namespace S4Bridge.Core;

public sealed partial class BridgeSupervisor
{
    public Task StartAsync(LaunchSpec spec, IBloopEndpointOwner owner)
    {
        lock (gate)
        {
            if (HasOwnedResources) throw new InvalidOperationException("Bridge resources are already owned. Stop before starting.");
            State = "starting";
            Error = "";
            stopping = null;
            prepared = null;
            startupEndpointBarrier = null;
            preparationStop = spec.Arguments.Contains("--prepare-screen-motor-diagnostics")
                ? new CancellationTokenSource() : null;
            starting = Task.Run(() => StartOwned(spec, owner));
            return starting;
        }
    }
    private async Task StartOwned(LaunchSpec spec, IBloopEndpointOwner owner)
    {
        Changed?.Invoke();
        try
        {
            try
            {
                var lease = await owner.PrepareAsync().ConfigureAwait(false);
                lock (gate) endpoint = lease;
                Output?.Invoke(Guid.Empty, $"BLOOP_PREPARED: {lease.AssociationId}");
                if (preparationStop is not null)
                {
                    Process child;
                    Guid id;
                    Task ready;
                    lock (gate)
                    {
                        preparationStop.Token.ThrowIfCancellationRequested();
                        prepared = new(TaskCreationOptions.RunContinuationsAsynchronously);
                        startupEndpointBarrier = new(TaskCreationOptions.RunContinuationsAsynchronously);
                        StartProcess(spec, recordWorker: false);
                        child = process!;
                        id = launch!.Id;
                        ready = prepared.Task;
                    }
                    Changed?.Invoke();
                    using var deadline = CancellationTokenSource.CreateLinkedTokenSource(preparationStop.Token);
                    deadline.CancelAfter(TimeSpan.FromSeconds(10));
                    var exited = child.WaitForExitAsync();
                    if (await Task.WhenAny(ready, exited).WaitAsync(deadline.Token).ConfigureAwait(false) == exited)
                        throw new IOException("Bridge exited during diagnostic preparation.");
                    await ready.WaitAsync(deadline.Token).ConfigureAwait(false);
                    Task creation;
                    lock (gate)
                    {
                        preparationStop.Token.ThrowIfCancellationRequested();
                        if (!Current(child, id) || child.HasExited)
                            throw new IOException("Bridge exited during diagnostic preparation.");
                        creation = lease.CreateAsync();
                    }
                    await creation.ConfigureAwait(false);
                    Output?.Invoke(Guid.Empty, $"BLOOP_CREATED: {lease.EndpointDeviceId}");
                    lock (gate)
                    {
                        preparationStop.Token.ThrowIfCancellationRequested();
                        if (!Current(child, id) || child.HasExited)
                            throw new IOException("Bridge exited during endpoint creation.");
                        lease.RecordWorker(child.Id, child.StartTime.ToUniversalTime());
                    }
                    await WriteInputAsync(child, id, "S4_SCREEN_DIAG_BEGIN 1", false,
                        preparationStop.Token).ConfigureAwait(false);
                }
                else
                {
                    await lease.CreateAsync().ConfigureAwait(false);
                    Output?.Invoke(Guid.Empty, $"BLOOP_CREATED: {lease.EndpointDeviceId}");
                    lock (gate) StartProcess(spec);
                    Changed?.Invoke();
                }
            }
            finally
            {
                lock (gate) startupEndpointBarrier?.TrySetResult();
            }
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
}
