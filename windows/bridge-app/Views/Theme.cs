using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;

namespace S4Bridge.Views;

internal static class UiTheme
{
    public const double Tiny = 4, Row = 8, Related = 12, Section = 20, Group = 32;
    public const double Body = 14, Caption = 12, Heading = 18, Page = 24, Display = 30;
    public const double FieldWidth = 120, ActionWidth = 180, LogHeight = 240, PlotHeight = 120;
    public const double MinWidth = 760, MinHeight = 540, Width = 860, Height = 720;
    public const string Foreground = "SystemControlForegroundBaseHighBrush";
    public const string Secondary = "SystemControlForegroundBaseMediumBrush";
    public const string Error = "SystemControlErrorTextForegroundBrush";
    public const string Accent = "SystemControlHighlightAccentBrush";
    public const string Surface = "SystemControlBackgroundAltHighBrush";
    public const int TraySize = 32;
    public const double TrayBadge = 12;
    public static readonly Color Healthy = Color.Parse("#0F7B0F");
    public static readonly Color Attention = Color.Parse("#C42B1C");
    public static readonly Color Idle = Color.Parse("#767676");
    public static IBrush Brush(Control control, string key) =>
        control.TryFindResource(key, out var value) && value is IBrush brush ? brush :
        throw new InvalidOperationException($"Required Fluent resource missing: {key}");

    public static TextBlock Text(string text, bool hint = false) => new()
    {
        Text = text,
        TextWrapping = TextWrapping.Wrap,
        FontSize = hint ? Caption : Body
    };
    public static SelectableTextBlock Selectable(string text) => new()
    {
        Text = text,
        TextWrapping = TextWrapping.Wrap,
        FontSize = Body
    };
    public static StackPanel Stack(params Control[] controls)
    {
        var panel = new StackPanel { Spacing = Related };
        foreach (var control in controls) panel.Children.Add(control);
        return panel;
    }
    public static WrapPanel Actions(params Control[] controls)
    {
        var panel = new WrapPanel { Orientation = Orientation.Horizontal };
        foreach (var control in controls)
        {
            control.Margin = new Thickness(0, 0, Row, Row);
            panel.Children.Add(control);
        }
        return panel;
    }
    public static Control SectionView(string title, params Control[] controls)
    {
        var panel = Stack(controls);
        panel.Spacing = Related;
        var heading = Text(title); heading.FontSize = Heading; heading.FontWeight = FontWeight.SemiBold;
        return Stack(heading, panel);
    }
    public static ScrollViewer BodyView(params Control[] controls)
    {
        var stack = Stack(controls); stack.Spacing = Group; stack.Margin = new Thickness(Section);
        return new() { Content = stack, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };
    }
}
