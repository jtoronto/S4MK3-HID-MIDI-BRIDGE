using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Threading;
using S4Bridge.Core;
using S4Bridge.Models;
using S4Bridge.Platform;

namespace S4Bridge.Views;

internal sealed class SettingsWindow : Window
{
    private readonly HostController host;
    private bool quitting;
    public SettingsView View { get; }
    public SettingsWindow(HostController controller, Func<Task> requestQuit)
    {
        host = controller;
        View = new SettingsView(controller, this, requestQuit);
        Title = "Bridge settings"; Width = UiTheme.Width; Height = UiTheme.Height;
        MinWidth = UiTheme.MinWidth; MinHeight = UiTheme.MinHeight;
        FontSize = UiTheme.Body;
        Content = View;
        host.Changed += () => Dispatcher.UIThread.Post(View.RefreshState);
        PropertyChanged += (_, e) =>
        {
            if (e.Property == IsVisibleProperty || e.Property == WindowStateProperty)
                View.RefreshVisibility(IsVisible, WindowState == WindowState.Minimized);
        };
        Closing += (_, e) => { if (!quitting) { e.Cancel = true; Hide(); } };
        Opened += (_, _) => { FitInitialWorkingArea(); View.RefreshState(); };
    }
    private void FitInitialWorkingArea()
    {
        var screen = Screens.ScreenFromWindow(this) ?? Screens.Primary;
        if (screen is null || FrameSize is not { } frame) return;
        var placement = WindowPlacement.Fit(screen.WorkingArea, screen.Scaling,
            ClientSize, frame, Position);
        MinWidth = Math.Min(UiTheme.MinWidth, placement.ClientSize.Width);
        MinHeight = Math.Min(UiTheme.MinHeight, placement.ClientSize.Height);
        Width = placement.ClientSize.Width;
        Height = placement.ClientSize.Height;
        Position = placement.Position;
    }
    public Task Guard(Func<Task> operation) => View.Guard(operation);
    public Task<bool> ConfirmAsync(string title, string text, string accept)
    {
        Window? dialog = null;
        var yes = View.Action(accept, () => { dialog!.Close(true); return Task.CompletedTask; });
        var no = View.Action("Cancel", () => { dialog!.Close(false); return Task.CompletedTask; });
        no.IsCancel = true;
        dialog = Dialog(title, UiTheme.Stack(UiTheme.Text(text), UiTheme.Actions(no, yes)));
        return dialog.ShowDialog<bool>(this);
    }
    public Task NoticeAsync(string title, string text)
    {
        Window? dialog = null;
        var close = View.Action("Close", () => { dialog!.Close(); return Task.CompletedTask; });
        close.IsCancel = true;
        dialog = Dialog(title, UiTheme.Stack(new ScrollViewer
        { Content = UiTheme.Selectable(text), MaxHeight = UiTheme.LogHeight }, close));
        return dialog.ShowDialog(this);
    }
    public async Task<ScreenConsent> ScreenConsentAsync(ScreenAccess access)
    {
        if (!access.Available)
        {
            return await ConfirmAsync("Screens unavailable", access.Detail +
                "\n\nContinue with controls, LEDs and jog without screens? No service or driver action will be performed.",
                "Continue without screens") ? ScreenConsent.WithoutScreens : ScreenConsent.Cancel;
        }
        return ScreenConsent.Enable;
    }
    private static Window Dialog(string title, Control body)
    {
        var window = new Window
        {
            Title = title,
            Width = UiTheme.MinWidth,
            SizeToContent = SizeToContent.Height,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Content = body
        };
        body.Margin = new Thickness(UiTheme.Section);
        return window;
    }
    public async Task<bool> QuitAsync()
    {
        await host.QuitAsync();
        quitting = true; View.StopRefresh(); Close(); return true;
    }
    public void ShowSettings()
    {
        Show(); WindowState = WindowState.Normal; Activate();
    }
    public Task InstallMappingAsync() => View.InstallMappingAsync();
    public Task ToggleScreensAsync() => View.ToggleScreensAsync();
}
