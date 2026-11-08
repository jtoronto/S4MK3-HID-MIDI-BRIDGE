using Avalonia.Controls;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private CheckBox? screensEnabled;
    private bool updatingScreens;
    private TextBlock? screenState;
    private Control ScreensTab()
    {
        screensEnabled = new CheckBox { Content = "Enable S4 screens", IsChecked = host.Store.Editor.Screen.Enabled };
        screensEnabled.IsCheckedChanged += async (_, _) =>
        {
            if (updatingScreens) return;
            await Guard(() => host.Screens.SetEnabledAsync(screensEnabled.IsChecked == true, host.Supervisor, ScreenConsentAsync));
        };
        screenState = UiTheme.Text("");
        var automatic = new CheckBox
        {
            Content = "Automatically stop NI driver (unavailable)",
            IsEnabled = false,
            IsChecked = host.Store.Editor.Screen.AutomaticallyStopNI
        };
        var restore = new CheckBox
        {
            Content = "Restore an app-owned NI transition on stop",
            IsChecked = host.Store.Editor.Screen.RestoreNIOnStop
        };
        restore.IsCheckedChanged += async (_, _) => await Guard(() =>
        {
            host.Store.SaveScreen(host.Store.Editor.Screen with { RestoreNIOnStop = restore.IsChecked == true });
            return Task.CompletedTask;
        });
        return UiTheme.BodyView(
            UiTheme.SectionView("S4 screens", screensEnabled, screenState,
                UiTheme.Text("HOTCUE/SAMPLES: jump, active loop, key and range. STEMS/MUTE: mute/solo legend. Full profile only.", true),
                Action("Retry verified access", () => host.Screens.RetryAsync(host.Supervisor))),
            UiTheme.SectionView("Native Instruments access", UiTheme.Selectable(host.Screens.Access.Detail), automatic, restore,
                UiTheme.Text("Screen preferences save immediately. No service name, driver replacement, startup policy or force-stop strategy is inferred from inventory. Remembered consent never establishes access.", true)),
            UiTheme.SectionView("Recovery",
                UiTheme.Text("Pending recovery is retained across reset, crash and quit. Restoration requires a verified app-owned running-to-stopped transition with exact service/binary and prior-state evidence."),
                Action("Review owned recovery", () => { host.Screens.RequireOwnedRecovery(); return Task.CompletedTask; }),
                UiTheme.Text("Stop NI driver and Restart NI driver are unavailable until root establishes an authorized access and transition policy. Stop the bridge before returning to another controller application.", true)));
    }
    private void UpdateScreens()
    {
        if (screensEnabled is null) return;
        updatingScreens = true;
        screensEnabled.IsChecked = host.Store.Editor.Screen.Enabled;
        screensEnabled.IsEnabled = !host.Screens.Busy && !host.Operating && host.Supervisor.State != "stopping";
        updatingScreens = false;
        if (screenState is not null)
            screenState.Text = $"Screens: {host.Screens.State}" +
                (host.Store.Editor.Screen.RestorePending ? " | NI recovery pending" : "");
    }
}
