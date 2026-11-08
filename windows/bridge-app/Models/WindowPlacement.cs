using Avalonia;

namespace S4Bridge.Models;

public sealed record WindowPlacement(Size ClientSize, PixelPoint Position)
{
    public static WindowPlacement Fit(PixelRect workingArea, double scaling,
        Size clientSize, Size frameSize, PixelPoint position)
    {
        var chromeWidth = Math.Max(0, frameSize.Width - clientSize.Width);
        var chromeHeight = Math.Max(0, frameSize.Height - clientSize.Height);
        var width = Math.Min(clientSize.Width, workingArea.Width / scaling - chromeWidth);
        var height = Math.Min(clientSize.Height, workingArea.Height / scaling - chromeHeight);
        var frameWidthPixels = (int)Math.Ceiling((width + chromeWidth) * scaling);
        var frameHeightPixels = (int)Math.Ceiling((height + chromeHeight) * scaling);
        var x = Math.Clamp(position.X, workingArea.X, workingArea.Right - frameWidthPixels);
        var y = Math.Clamp(position.Y, workingArea.Y, workingArea.Bottom - frameHeightPixels);
        return new(new Size(width, height), new PixelPoint(x, y));
    }
}
