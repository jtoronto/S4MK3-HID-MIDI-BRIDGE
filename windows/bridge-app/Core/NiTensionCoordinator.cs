namespace S4Bridge.Core;

// The NI protocol reports a value, not a request ID or measured wheel force.
internal sealed class NiTensionCoordinator
{
    private readonly object gate = new();
    private readonly SemaphoreSlim sendGate = new(1, 1);
    private readonly Func<Guid, int, Task> send;
    private Guid? launch;
    private int? pending;
    private bool initialized;
    private bool ready;
    private long revision;
    private string status = "Not submitted";
    private string? failureStatus;
    public event Action? Changed;

    public NiTensionCoordinator(Func<Guid, int, Task> send) => this.send = send;
    public string Status { get { lock (gate) return status; } }
    public bool Ready { get { lock (gate) return ready; } }

    public void Started(Guid id)
    {
        lock (gate) { launch = id; pending = null; initialized = false; ready = false; revision++; failureStatus = null; status = "Not submitted"; }
        Changed?.Invoke();
    }
    public void Edited()
    {
        lock (gate) { revision++; initialized = true; pending = null; status = "Saving latest value"; }
        Changed?.Invoke();
    }
    public void Saved()
    {
        lock (gate) { pending = null; initialized = false; failureStatus = null; status = "Saved; not submitted"; }
        Changed?.Invoke();
    }
    public void Failed(Exception error)
    {
        lock (gate) { pending = null; status = failureStatus = $"NI tension failed: {error.Message}"; }
        Changed?.Invoke();
    }
    public void Exited(Guid id)
    {
        lock (gate)
        {
            if (launch != id) return;
            launch = null; pending = null; initialized = false; ready = false; status = failureStatus ?? "Not submitted";
        }
        Changed?.Invoke();
    }
    public async Task InitializeAsync(Guid id, int value)
    {
        long expected;
        lock (gate)
        {
            if (launch != id || initialized || !ready) return;
            initialized = true;
            expected = revision;
        }
        try { await SubmitAsync(id, value, expectedRevision: expected); }
        catch (Exception error)
        {
            lock (gate)
            {
                if (launch != id || revision != expected) return;
                status = $"Submission failed: {error.Message}";
            }
            Changed?.Invoke();
        }
    }
    public async Task SubmitAsync(Guid id, int value, CancellationToken cancellation = default,
        long? expectedRevision = null)
    {
        await sendGate.WaitAsync(cancellation).ConfigureAwait(false);
        try
        {
            cancellation.ThrowIfCancellationRequested();
            lock (gate)
            {
                if (expectedRevision is { } expected && revision != expected) return;
                if (launch != id || !ready) throw new IOException("NI bridge session is not ready.");
                initialized = true;
                pending = value;
                status = $"Submitting {value}";
            }
            Changed?.Invoke();
            try { await send(id, value).ConfigureAwait(false); }
            catch (Exception error)
            {
                lock (gate)
                {
                    if (launch == id && pending == value)
                    { pending = null; status = $"Submission failed: {error.Message}"; }
                }
                Changed?.Invoke();
                throw;
            }
        }
        finally { sendGate.Release(); }
    }
    public void Accept(Guid id, string line)
    {
        const string prefix = "S4_NI_JOG_TENSION submitted ";
        lock (gate)
        {
            if (launch != id) return;
            if (line == "S4_SCREEN_STATUS 0 active")
            {
                ready = true;
            }
            else if (line is "S4_SCREEN_STATUS 0 pending" or "S4_SCREEN_STATUS 0 inactive" or "S4_SCREEN_STATUS 0 failed")
            {
                ready = false;
                pending = null;
                initialized = false;
                revision++;
                status = "NI session unavailable";
            }
            else
            {
            if (launch != id || pending is not { } value ||
                line != prefix + value.ToString(System.Globalization.CultureInfo.InvariantCulture)) return;
            pending = null;
            failureStatus = null;
            status = $"Tension request sent to NIHardwareService: {value}";
            }
        }
        Changed?.Invoke();
    }
}
