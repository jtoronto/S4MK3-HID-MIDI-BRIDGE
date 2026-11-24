namespace S4Bridge.Core;

internal sealed class NiTensionEdits
{
    private readonly object gate = new();
    private readonly Func<int, CancellationToken, Task> apply;
    private readonly Func<CancellationToken, Task> delay;
    private readonly SemaphoreSlim applyGate = new(1, 1);
    private readonly SemaphoreSlim flushGate = new(1, 1);
    private readonly Action<int> persist;
    private CancellationTokenSource? current;
    private bool flushing;
    private int value;

    public NiTensionEdits(int initial, Func<int, CancellationToken, Task> apply,
        Func<CancellationToken, Task>? delay = null, Action<int>? persist = null)
    {
        value = initial;
        this.apply = apply;
        this.persist = persist ?? (_ => throw new InvalidOperationException("Persistence unavailable."));
        this.delay = delay ?? (token => Task.Delay(TimeSpan.FromMilliseconds(125), token));
    }
    public int Value { get { lock (gate) return value; } }
    public bool Pending { get { lock (gate) return current is not null || flushing; } }
    public event Action? Changed;

    public async Task FlushAsync()
    {
        await flushGate.WaitAsync().ConfigureAwait(false);
        try
        {
            int latest;
            lock (gate)
            {
                if (current is null) return;
                flushing = true;
                latest = value;
                current.Cancel();
            }
            await applyGate.WaitAsync().ConfigureAwait(false);
            try { persist(latest); }
            finally { applyGate.Release(); }
        }
        finally
        {
            lock (gate) flushing = false;
            flushGate.Release();
            Changed?.Invoke();
        }
    }

    public async Task SetAsync(int next)
    {
        CancellationTokenSource source;
        lock (gate)
        {
            current?.Cancel();
            current = source = new CancellationTokenSource();
            value = next;
        }
        Changed?.Invoke();
        try
        {
            await delay(source.Token).ConfigureAwait(false);
            await applyGate.WaitAsync(source.Token).ConfigureAwait(false);
            try
            {
                source.Token.ThrowIfCancellationRequested();
                await apply(next, source.Token).ConfigureAwait(false);
            }
            finally { applyGate.Release(); }
        }
        catch (OperationCanceledException) when (source.IsCancellationRequested) { }
        finally
        {
            lock (gate) { if (current == source) current = null; }
            source.Dispose();
            Changed?.Invoke();
        }
    }
}
