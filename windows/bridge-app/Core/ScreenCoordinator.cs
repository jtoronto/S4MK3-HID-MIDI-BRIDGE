using S4Bridge.Models;
using S4Bridge.Platform;

namespace S4Bridge.Core;

public enum ScreenConsent { Enable, WithoutScreens, Cancel }
public sealed class ScreenCoordinator(PreferenceStore store, INIServiceControl adapter)
{
    private readonly object gate = new();
    private Guid? launchId;
    private bool niBridge;
    private ulong generation;
    private TaskCompletionSource? released;
    public string State { get; private set; } = "ready";
    public string Detail { get; private set; } = "";
    public bool Busy { get; private set; }
    public event Action? Changed;
    public ScreenAccess Access => adapter.Inspect();
    public async Task<bool?> PrepareAsync(Func<ScreenAccess, Task<ScreenConsent>> consent)
    {
        if (store.Saved.Run.Profile != "full" || !store.Editor.Screen.Enabled)
        { State = "off"; return false; }
        var access = Access;
        Detail = access.Detail;
        if (!access.Available)
        {
            State = "blocked";
            Changed?.Invoke();
            var decision = await consent(access);
            // Neither consent nor remembered automatic stopping establishes access.
            return decision == ScreenConsent.Cancel ? null : false;
        }
        State = "pending";
        return true;
    }
    public void Started(Guid id, bool screens, bool usesNiBridge = false)
    {
        lock (gate)
        {
            launchId = id; generation = 0; released = null;
            niBridge = usesNiBridge;
            if (screens) State = "pending";
            else if (State != "blocked") State = "off";
        }
        Changed?.Invoke();
    }
    public void Accept(Guid id, string line)
    {
        lock (gate)
        {
            if (id != launchId || !line.StartsWith("S4_SCREEN_STATUS ", StringComparison.Ordinal)) return;
            var fields = line.Split(' ');
            if (fields.Length != 3) return;
            ulong received;
            try { received = ManagedStatus.ParseId(fields[1]); }
            catch (InvalidDataException) { return; }
            if (received != generation) return;
            switch (fields[2])
            {
                case "active":
                    if (niBridge || (store.Editor.Screen.Enabled && Access.Available && released is null))
                        State = "active";
                    break;
                case "inactive":
                    if (State != "blocked") State = niBridge || store.Editor.Screen.Enabled ? "ready" : "off";
                    released?.TrySetResult(); break;
                case "failed":
                    State = "failed"; Detail = niBridge
                        ? "NI Bridge lost screen or platter control. See Diagnostics."
                        : "Screens failed; MIDI and jog remain independent.";
                    released?.TrySetException(new IOException("Screen release failed; ownership remains uncertain.")); break;
                case "pending":
                    if (released is null && State != "blocked") State = "pending"; break;
                default: return;
            }
        }
        Changed?.Invoke();
    }
    public async Task SetEnabledAsync(bool enabled, BridgeSupervisor supervisor,
        Func<ScreenAccess, Task<ScreenConsent>> consent)
    {
        if (Busy) throw new InvalidOperationException("A screen operation is already in progress.");
        Busy = true; Changed?.Invoke();
        try
        {
            var previous = store.Editor.Screen with { };
            store.SaveScreen(previous with { Enabled = enabled });
            if (!enabled)
            {
                if (supervisor.HasChild && supervisor.ActiveSpec?.Run.Profile == "full")
                    await ReleaseAsync(supervisor);
                State = "off";
                return;
            }
            var prepared = await PrepareAsync(consent);
            if (prepared is null) { store.SaveScreen(previous); return; }
            if (prepared == true && supervisor.HasChild)
                await supervisor.SendScreenAsync(checked(++generation), "ON");
        }
        finally { Busy = false; Changed?.Invoke(); }
    }
    public async Task RetryAsync(BridgeSupervisor supervisor)
    {
        if (!Access.Available) throw new IOException(Access.Detail);
        if (Busy || !supervisor.HasChild || supervisor.ActiveSpec?.Run.Profile != "full")
            throw new InvalidOperationException("Start a full-profile bridge before retrying screens.");
        await supervisor.SendScreenAsync(checked(++generation), "RETRY");
    }
    private async Task ReleaseAsync(BridgeSupervisor supervisor)
    {
        Task wait;
        lock (gate)
        {
            if (released is not null) throw new InvalidOperationException("Screen release remains unresolved.");
            released = new(TaskCreationOptions.RunContinuationsAsynchronously);
            wait = released.Task;
            State = "releasing";
        }
        try
        {
            await supervisor.SendScreenAsync(checked(++generation), "OFF");
            await wait.WaitAsync(TimeSpan.FromSeconds(5));
            lock (gate) released = null;
        }
        catch (Exception ex) when (ex is TimeoutException or IOException or InvalidOperationException)
        {
            State = "failed"; Detail = $"Screen release unconfirmed: {ex.Message}. No restoration attempted.";
            throw new IOException(Detail, ex);
        }
    }
    public void Exited(Guid id)
    {
        lock (gate)
        {
            if (launchId != id) return;
            launchId = null;
            released?.TrySetResult(); released = null;
            if (State != "blocked" && !(niBridge && State == "failed"))
                State = niBridge || store.Editor.Screen.Enabled ? "ready" : "off";
        }
        Changed?.Invoke();
    }
    public void RequireOwnedRecovery()
    {
        throw new IOException(store.Editor.Screen.RestorePending
            ? "Pending NI recovery preserved. Exact service/binary, prior state, operation and verified transition evidence must be reviewed before restoration."
            : "NI recovery unavailable: no verified app-owned Windows service transition exists. No service state was changed.");
    }
}
