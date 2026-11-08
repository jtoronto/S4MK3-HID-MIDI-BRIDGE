using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;

namespace S4Bridge.Views;

internal sealed class TrayImages(string source)
{
    public WindowIcon Idle { get; } = Create(source, UiTheme.Idle);
    public WindowIcon Healthy { get; } = Create(source, UiTheme.Healthy);
    public WindowIcon Attention { get; } = Create(source, UiTheme.Attention);
    private static WindowIcon Create(string path, Color state)
    {
        using var image = new Bitmap(path);
        using var target = new RenderTargetBitmap(new(UiTheme.TraySize, UiTheme.TraySize));
        using (var context = target.CreateDrawingContext())
        {
            context.DrawImage(image, new Rect(0, 0, UiTheme.TraySize, UiTheme.TraySize));
            context.DrawEllipse(new SolidColorBrush(state), null,
                new Rect(UiTheme.TraySize - UiTheme.TrayBadge, UiTheme.TraySize - UiTheme.TrayBadge,
                    UiTheme.TrayBadge, UiTheme.TrayBadge));
        }
        using var stream = new MemoryStream();
        target.Save(stream); stream.Position = 0;
        return new WindowIcon(stream);
    }
}
