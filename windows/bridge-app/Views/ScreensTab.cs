using Avalonia.Controls;
using S4Bridge.Models;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private CheckBox? screensEnabled;
    private bool updatingScreens;
    private TextBlock? screenState;
    private Control ScreensTab()
    {
        screensEnabled = null;
        screenState = UiTheme.Text("");
        if (host.CanUseLegacyScreenControls) return LegacyScreensTab();
        var niBridge = host.SelectedMode == BridgeMode.NiBridge;
        return UiTheme.BodyView(
            UiTheme.SectionView("S4 screens", screenState,
                UiTheme.Text(niBridge
                    ? "Screens start with NI Bridge and need NIHardwareService. No separate screen switch is required."
                    : "Screens are not supported in Direct HID. Stop the bridge and select NI Bridge on the Bridge tab to use them.")),
            UiTheme.SectionView("On the screens",
                UiTheme.Text("HOTCUE/SAMPLES: jump, active loop, key and range. STEMS/MUTE: mute and solo controls.", true)),
            UiTheme.Text("Stop the bridge before returning to another controller application.", true));
    }
    private Control LegacyScreensTab()
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
            Content = "Automatically stop NI driver (not available)",
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
            UiTheme.SectionView("Screen diagnostics", screensEnabled, screenState,
                UiTheme.Text("HOTCUE/SAMPLES: jump, active loop, key and range. STEMS/MUTE: mute/solo legend. Full profile only.", true),
                Action("Retry verified access", () => host.Screens.RetryAsync(host.Supervisor))),
            UiTheme.SectionView("Native Instruments access", UiTheme.Selectable(host.Screens.Access.Detail), automatic, restore,
                UiTheme.Text("Screen diagnostic preferences save immediately. The app does not stop services, replace drivers or change startup settings.", true)),
            UiTheme.SectionView("Recovery",
                UiTheme.Text("Pending recovery is retained across reset, crash and quit. Review the recorded service change before attempting restoration."),
                Action("Review owned recovery", () => { host.Screens.RequireOwnedRecovery(); return Task.CompletedTask; }),
                UiTheme.Text("Automatic service stop and restart are not available. Stop the bridge before returning to another controller application.", true)));
    }
    private void UpdateScreens()
    {
        if (screensEnabled is not null)
        {
            updatingScreens = true;
            screensEnabled.IsChecked = host.Store.Editor.Screen.Enabled;
            screensEnabled.IsEnabled = host.CanUseLegacyScreenControls &&
                !host.Screens.Busy && !host.Operating && host.Supervisor.State != "stopping";
            updatingScreens = false;
        }
        if (screenState is not null)
            screenState.Text = $"Screens: {host.ScreenStatus}" +
                (host.Store.Editor.Screen.RestorePending ? " | NI recovery pending" : "");
    }
}
