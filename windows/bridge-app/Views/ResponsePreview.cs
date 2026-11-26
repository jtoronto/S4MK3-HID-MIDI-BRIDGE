using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using S4Bridge.Models;

namespace S4Bridge.Views;

internal sealed class ResponsePreview(Func<JogPreferences> source) : Control
{
    public override void Render(DrawingContext context)
    {
        var jog = source();
        var foreground = UiTheme.Brush(this, UiTheme.Secondary);
        var accent = UiTheme.Brush(this, UiTheme.Accent);
        var pen = new Pen(accent, UiTheme.Tiny / 2);
        var baseline = Bounds.Height / 2;
        var max = Math.Max(Math.Abs(ResponseCurve.Rate(jog, 0)), Math.Abs(ResponseCurve.Rate(jog, jog.PlatterTargetRpm * 2)));
        if (!double.IsFinite(max) || max <= 0) return;
        context.DrawLine(new Pen(foreground), new(0, baseline), new(Bounds.Width, baseline));
        Point? previous = null;
        const int samples = 128;
        for (var i = 0; i <= samples; i++)
        {
            var measured = jog.PlatterTargetRpm * 2.0 * i / samples;
            var rate = ResponseCurve.Rate(jog, measured);
            var point = new Point(Bounds.Width * i / samples, baseline - rate / max * baseline);
            if (previous is { } start) context.DrawLine(pen, start, point);
            previous = point;
        }
    }
    protected override Size MeasureOverride(Size availableSize) =>
        new(double.IsFinite(availableSize.Width) ? availableSize.Width : UiTheme.MinWidth, UiTheme.PlotHeight);
}
