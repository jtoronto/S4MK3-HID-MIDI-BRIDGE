using System.Diagnostics;
using System.Text;

namespace S4Bridge.Core;

public sealed class BridgeSupervisor
{
    private readonly object gate = new();
    private readonly SemaphoreSlim inputGate = new(1, 1);
    private Process? process;
    private LaunchState? launch;
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
    public string State { get; private set; } = "stopped";
    public string Error { get; private set; } = "";
    public LaunchSpec? ActiveSpec { get; private set; }
    public Task Completion { get { lock (gate) return completion; } }

    public void Start(LaunchSpec spec)
    {
        lock (gate)
        {
            if (process is not null) throw new InvalidOperationException("A bridge process is already owned.");
            var child = new Process { StartInfo = spec.StartInfo() };
            var id = Guid.NewGuid();
            try { child.Start(); }
            catch { child.Dispose(); throw; }
            process = child;
            launch = new(id, child.Id, spec.Run.Profile);
            ActiveSpec = spec;
            request = 0;
            stopping = null;
            State = spec.Diagnostic ? "checking" : "starting";
            Error = "";
            Launched?.Invoke(id);
            var stdout = Drain(child.StandardOutput, child, id, true);
            var stderr = Drain(child.StandardError, child, id, false);
            completion = ObserveExit(child, id, Task.WhenAll(stdout, stderr));
        }
        Changed?.Invoke();
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
        await child.WaitForExitAsync().ConfigureAwait(false);
        await readers.ConfigureAwait(false);
        var exit = child.ExitCode;
        lock (gate)
        {
            if (!Current(child, id)) return;
            var failed = exit != 0 || State == "failed";
            State = failed ? "failed" : "stopped";
            if (exit != 0) Error = $"Bridge exited with status {exit}. See Diagnostics.";
            process = null;
            child.Dispose();
        }
        Output?.Invoke(id, $"EXIT: {exit}");
        Exited?.Invoke(id, exit);
        Changed?.Invoke();
    }
    public Task StopAsync()
    {
        lock (gate)
        {
            if (process is null) return Task.CompletedTask;
            return stopping ??= StopOwned(process, launch!.Id);
        }
    }
    private async Task StopOwned(Process child, Guid id)
    {
        Task exited;
        ulong stopRequest;
        lock (gate)
        {
            var next = checked(++request);
            stopRequest = next;
            launch!.RequestStop(next);
            State = "stopping";
            exited = completion;
        }
        Changed?.Invoke();
        try
        {
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
            await WriteInputAsync(child, id, $"S4_BRIDGE_CONTROL {stopRequest} STOP", true, deadline.Token).ConfigureAwait(false);
            await exited.WaitAsync(deadline.Token).ConfigureAwait(false);
        }
        catch (Exception ex) when (ex is OperationCanceledException or IOException or ObjectDisposedException)
        {
            lock (gate)
            {
                if (!Current(child, id)) return;
                launch!.Unconfirmed();
                State = "failed";
                Error = $"Stop unconfirmed: {ex.Message}. Process retained; neutral state unknown. No automatic kill or service restoration.";
            }
            Changed?.Invoke();
            throw new IOException(Error, ex);
        }
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
