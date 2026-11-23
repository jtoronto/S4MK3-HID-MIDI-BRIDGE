using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using S4Bridge.Core;
using S4Bridge.Platform;
using S4Bridge.Views;

namespace S4Bridge;

internal sealed class BridgeApplication : Application
{
    private FileStream? instance;
    private TrayIcon? tray;
    private TrayImages? trayImages;
    private SettingsWindow? settings;
    private HostController? host;
    private IClassicDesktopStyleApplicationLifetime? desktop;
    private bool exiting;
    private readonly List<(NativeMenuItem Item, Func<bool> Enabled)> menuGates = [];
    private readonly NativeMenuItem bridgeStatus = new() { IsEnabled = false };
    private readonly NativeMenuItem screenStatus = new() { IsEnabled = false };
    private readonly NativeMenuItem start = new("Start bridge");
    private readonly NativeMenuItem stop = new("Stop bridge");
    private readonly NativeMenuItem screenToggle = new("Enable S4 screens") { ToggleType = NativeMenuItemToggleType.CheckBox };
    public override void Initialize() => Styles.Add(new FluentTheme());
    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime lifetime) return;
        desktop = lifetime; desktop.ShutdownMode = ShutdownMode.OnExplicitShutdown;
        try
        {
            var root = PreferenceStore.LocalRoot();
            Directory.CreateDirectory(root);
            try { instance = new(Path.Combine(root, ".instance.lock"), FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None); }
            catch (IOException ex) { throw new IOException("Another S4 Bridge host may already be running. Close it before starting this copy.", ex); }
            var resources = Path.Combine(AppContext.BaseDirectory, "Resources");
            var experimental = LaunchSpec.ExperimentalEnabled(
                Environment.GetEnvironmentVariable(LaunchSpec.ExperimentalEnvironmentVariable));
            var preferencesRoot = PreferenceStore.HostPreferencesRoot(root, experimental);
            host = new(resources, preferencesRoot, new BloopEndpointOwner(root), experimental);
            settings = new(host);
            desktop.MainWindow = settings;
            trayImages = new(Path.Combine(resources, "AppIcon.png"));
            tray = new TrayIcon { Icon = trayImages.Idle, ToolTipText = "S4 MK3 Bridge", IsVisible = true };
            var menu = new NativeMenu();
            foreach (var item in new NativeMenuItemBase[]
            {
                bridgeStatus,screenStatus,new NativeMenuItemSeparator(),start,stop,screenToggle,
                Menu("Retry screens",()=>host.Screens.RetryAsync(host.Supervisor),
                    ()=>!host.ExperimentalNiDecks && host.Supervisor.State=="ready" && host.Supervisor.ActiveSpec?.Run.Profile=="full" && !host.Screens.Busy),
                Menu("Review NI recovery",()=>{host.Screens.RequireOwnedRecovery();return Task.CompletedTask;}),
                new NativeMenuItemSeparator(),
                Menu("Settings...",()=>{settings.ShowSettings();return Task.CompletedTask;}),
                Menu("Install Djay mapping...",()=>{settings.ShowSettings();return settings.InstallMappingAsync();},
                    ()=>!host.Exporter.IsExporting && File.Exists(host.Executable)),
                Menu("Check controller",()=>{host.Check();return Task.CompletedTask;},
                    ()=>!host.Supervisor.HasOwnedResources && !host.Operating && File.Exists(host.Executable)),
                Menu("Quit S4 MK3 Bridge",QuitAsync)
            }) menu.Items.Add(item);
            tray.Menu = menu;
            menu.NeedsUpdate += (_, _) => UpdateTray();
            start.Click += async (_, _) => await settings.Guard(() => host.StartAsync());
            stop.Click += async (_, _) => await settings.Guard(() => host.Supervisor.StopAsync());
            screenToggle.Click += async (_, _) => await settings.Guard(() => settings.ToggleScreensAsync());
            tray.Clicked += (_, _) => settings.ShowSettings();
            TrayIcon.SetIcons(this, new TrayIcons { tray });
            host.Changed += () => Dispatcher.UIThread.Post(UpdateTray);
            desktop.ShutdownRequested += async (_, e) =>
            {
                if (exiting) return;
                e.Cancel = true;
                await settings.Guard(QuitAsync);
            };
            desktop.Exit += (_, _) => { tray.Dispose(); instance.Dispose(); };
            UpdateTray(); settings.ShowSettings();
        }
        catch (Exception ex)
        {
            instance?.Dispose();
            var close = new Button { Content = "Close" };
            var error = new Window
            {
                Title = "S4 Bridge could not start",
                Width = UiTheme.MinWidth,
                SizeToContent = SizeToContent.Height,
                Content = UiTheme.Stack(UiTheme.Selectable(ex.Message), close)
            };
            desktop.MainWindow = error;
            close.Click += (_, _) => desktop.Shutdown(1);
            error.Closed += (_, _) => desktop.Shutdown(1);
            error.Show();
        }
        base.OnFrameworkInitializationCompleted();
    }
    private NativeMenuItem Menu(string title, Func<Task> operation, Func<bool>? enabled = null)
    {
        var item = new NativeMenuItem(title);
        item.Click += async (_, _) => { if (settings is not null) await settings.Guard(operation); };
        if (enabled is not null) menuGates.Add((item, enabled));
        return item;
    }
    private void UpdateTray()
    {
        if (host is null || tray is null) return;
        bridgeStatus.Header = "Bridge: " + host.Supervisor.State;
        screenStatus.Header = "Screens: " + host.Screens.State;
        start.IsEnabled = host.CanStart; stop.IsEnabled = host.CanStop;
        screenToggle.IsChecked = host.Store.Editor.Screen.Enabled;
        screenToggle.IsEnabled = !host.ExperimentalNiDecks && !host.Screens.Busy && host.Supervisor.State != "stopping";
        foreach (var (item, enabled) in menuGates) item.IsEnabled = enabled();
        tray.ToolTipText = $"S4 MK3 Bridge | {host.Supervisor.State} | Screens {host.Screens.State}";
        if (trayImages is not null)
            tray.Icon = host.Screens.NeedsAttention ? trayImages.Attention :
                host.Supervisor.State == "ready" ? trayImages.Healthy : trayImages.Idle;
    }
    private async Task QuitAsync()
    {
        if (exiting || settings is null || desktop is null) return;
        if (await settings.QuitAsync()) { exiting = true; desktop.Shutdown(); }
    }
}
