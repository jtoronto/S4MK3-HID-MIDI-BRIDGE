using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using S4Bridge.Core;
using S4Bridge.Models;
using S4Bridge.Platform;

namespace S4Bridge.Views;

internal sealed partial class SettingsView : Grid
{
    private readonly HostController host;
    private readonly SettingsWindow? owner;
    private Fields fields;
    private readonly TabControl tabs = new();
    private readonly SelectableTextBlock status = UiTheme.Selectable("");
    private readonly SelectableTextBlock error = UiTheme.Selectable("");
    private readonly TextBlock message = UiTheme.Text("", true);
    private readonly Button start;
    private readonly Button stop;
    private readonly Button details;
    private readonly Button save;
    private readonly Button reset;
    private readonly DispatcherTimer refresh;
    private bool ownerVisible;
    private bool ownerMinimized;
    private readonly List<Control> captureLocked = [];
    private TabItem[] builtTabs = [];
    internal IReadOnlyList<TabItem> BuiltTabs => builtTabs;
    private Control? profileControl;
    private Control? ledNotice;
    private ResponsePreview? response;
    private string errorDetail = "";
    public SettingsView(HostController controller, SettingsWindow? window = null)
    {
        host = controller;
        owner = window;
        host.ClearFieldErrors();
        fields = new(() => { host.Edited(); Update(); }, host.SetFieldError);
        refresh = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(250) };
        refresh.Tick += (_, _) => RefreshDiagnostics();
        start = Action("Start bridge", () => host.StartAsync());
        stop = Action("Stop", () => host.Supervisor.StopAsync());
        details = Action("Error details...",
            () => string.IsNullOrWhiteSpace(errorDetail) ? Task.CompletedTask : NoticeAsync("Bridge details", errorDetail));
        save = Action("Save preferences", SaveAsync); save.Classes.Add("accent");
        reset = Action("Reset preferences...", ResetAsync);
        var header = UiTheme.Stack(status, UiTheme.Actions(start, stop), error, details);
        header.Margin = new Thickness(UiTheme.Section);
        error.Bind(SelectableTextBlock.ForegroundProperty, error.GetResourceObservable(UiTheme.Error));
        var footer = UiTheme.Stack(message, UiTheme.Actions(reset, save));
        footer.Margin = new Thickness(UiTheme.Section);
        RowDefinitions = new("Auto,*,Auto");
        Children.Add(header); Grid.SetRow(tabs, 1); Children.Add(tabs);
        Grid.SetRow(footer, 2); Children.Add(footer);
        BuildTabs();
        host.Consent = ScreenConsentAsync;
        tabs.SelectionChanged += (_, _) => SetDiagnosticsVisibility();
        Update();
    }
    public Button Action(string label, Func<Task> operation)
    {
        var button = new Button { Content = label, MinWidth = UiTheme.ActionWidth };
        button.Click += async (_, _) => await Guard(operation);
        return button;
    }
    public async Task Guard(Func<Task> operation)
    {
        try { await operation(); }
        catch (Exception ex) { host.Report(ex); }
        Update();
    }
    internal void RefreshState() => Update();
    internal void RefreshVisibility(bool visible, bool minimized)
    {
        ownerVisible = visible;
        ownerMinimized = minimized;
        SetDiagnosticsVisibility();
    }
    internal void StopRefresh() => refresh.Stop();
    internal Task InstallMappingAsync() => ExportMappingAsync(true);
    internal Task ToggleScreensAsync() => host.Screens.SetEnabledAsync(
        !host.Store.Editor.Screen.Enabled, host.Supervisor, ScreenConsentAsync);
    private void BuildTabs()
    {
        var selected = tabs.SelectedIndex;
        captureLocked.Clear(); profileControl = null;
        host.ClearFieldErrors();
        fields = new(() => { host.Edited(); Update(); }, host.SetFieldError);
        tabs.ItemsSource = builtTabs = new TabItem[]
        {
            new(){Header="Bridge",Content=BridgeTab()},
            new(){Header="Screens",Content=ScreensTab()},
            new(){Header="Jog",Content=JogTab()},
            new(){Header="LEDs",Content=LedTab()},
            new(){Header="Palette",Content=PaletteTab()},
            new(){Header="Diagnostics",Content=DiagnosticsTab()}
        };
        tabs.SelectedIndex = Math.Max(0, selected);
        SetDiagnosticsVisibility();
    }
    private void Update()
    {
        status.Text = $"Bridge: {host.Supervisor.State} | Screens: {host.Screens.State}";
        errorDetail = host.FormatErrorDetail(fields.Error);
        var hasError = errorDetail.Length > 0;
        error.Text = errorDetail.Length > 200 ? errorDetail[..200] + "..." : errorDetail;
        error.IsVisible = hasError;
        details.IsVisible = hasError;
        details.IsEnabled = hasError;
        message.Text = host.Message;
        start.IsEnabled = host.CanStart && fields.Valid;
        stop.IsEnabled = host.CanStop;
        save.Content = host.Supervisor.HasOwnedResources ? "Save and restart" : "Save preferences";
        save.IsEnabled = !host.Operating && !host.Screens.Busy && !host.Store.RequiresReset
            && host.Supervisor.State is not ("stopping" or "checking") && fields.Valid;
        reset.IsEnabled = !host.Operating && !host.Screens.Busy;
        foreach (var field in captureLocked) field.IsEnabled = !host.Diagnostics.Capturing;
        if (profileControl is not null)
            profileControl.IsEnabled = !host.Screens.Busy && !host.Diagnostics.Capturing;
        if (ledNotice is not null) ledNotice.IsVisible = host.Store.Editor.Run.Profile != "full";
        UpdateScreens();
        UpdateDiagnosticsControls();
        response?.InvalidateVisual();
    }
    private async Task SaveAsync()
    {
        if (!fields.Valid) throw new InvalidDataException(fields.Error);
        await host.ApplyAsync();
    }
    private async Task ResetAsync()
    {
        if (await ConfirmAsync("Reset preferences?",
               "Restore editor defaults? Saved files remain unchanged until Save. Pending NI recovery is preserved.",
               "Reset editor"))
        { host.Reset(); BuildTabs(); }
    }
    private Task<ScreenConsent> ScreenConsentAsync(ScreenAccess access) =>
        owner is null ? Task.FromResult(ScreenConsent.Cancel) : owner.ScreenConsentAsync(access);
    private Task<bool> ConfirmAsync(string title, string text, string accept) =>
        owner is null ? throw new InvalidOperationException("Settings dialogs require the settings window.")
            : owner.ConfirmAsync(title, text, accept);
    private Task NoticeAsync(string title, string text) =>
        owner is null ? throw new InvalidOperationException("Settings dialogs require the settings window.")
            : owner.NoticeAsync(title, text);
    private static Task OpenPath(string path)
    {
        using var process = System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo(path) { UseShellExecute = true });
        if (process is null) throw new IOException($"Windows could not open {path}.");
        return Task.CompletedTask;
    }
    private Task RevealPreferences()
    {
        Directory.CreateDirectory(host.Store.Root); return OpenPath(host.Store.Root);
    }
}
