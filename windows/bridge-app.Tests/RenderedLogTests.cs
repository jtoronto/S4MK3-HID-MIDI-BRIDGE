using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Presenters;
using Avalonia.Headless;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using S4Bridge.Views;

namespace S4Bridge.Tests;

public sealed class HeadlessTestApp : Application
{
    public override void Initialize() => Styles.Add(new FluentTheme());
}

internal static class RenderedLogTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "log append keeps the containing diagnostics page in place", () =>
        {
            using var session = HeadlessUnitTestSession.StartNew(typeof(HeadlessTestApp));
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(10));
            session.Dispatch(() =>
            {
                var log = new IncrementalLogView { Height = 120 };
                var caretRequests = 0;
                log.AddHandler(Control.RequestBringIntoViewEvent, (_, e) =>
                {
                    if (e.TargetObject is TextPresenter) caretRequests++;
                }, RoutingStrategies.Bubble, true);
                var capture = new Button { Content = "Start motor capture", Height = 48 };
                var page = new ScrollViewer
                {
                    Content = new StackPanel
                    {
                        Children =
                        {
                            capture,
                            new Border { Height = 500 },
                            log,
                            new Border { Height = 100 }
                        }
                    }
                };
                var window = new Window { Width = 400, Height = 220, Content = page };
                try
                {
                    window.Show();
                    Pump();
                    log.Apply(new(1, 0, string.Concat(Enumerable.Repeat("log line\n", 40))));
                    Pump();
                    capture.Focus();
                    Check.True(capture.IsFocused, "Capture button did not receive focus");
                    page.Offset = default;
                    Pump();
                    var inner = log.GetVisualDescendants().OfType<ScrollViewer>().Single();
                    var before = page.Offset;
                    caretRequests = 0;
                    log.Apply(new(2, 0, "next line\n"));
                    Pump();
                    Check.True(caretRequests > 0, "No caret bring-into-view request observed");
                    Check.True(inner.Offset.Y >= inner.Extent.Height - inner.Viewport.Height - 1,
                        $"Log did not follow its tail: offset {inner.Offset}, extent {inner.Extent}, viewport {inner.Viewport}");
                    Check.True(page.Offset == before,
                        $"Page moved from {before} to {page.Offset}; caret requests: {caretRequests}");
                    Check.True(capture.IsFocused, "Log append changed the focused control");
                    log.SelectionStart = 9;
                    log.SelectionEnd = 12;
                    log.Apply(new(3, 9, "final line\n"));
                    Pump();
                    Check.Equal(0, log.SelectionStart);
                    Check.Equal(3, log.SelectionEnd);
                    Check.Equal("log", log.SelectedText);
                    Check.True(log.CanCopy);
                    Check.True(log.Text!.EndsWith("next line\nfinal line\n", StringComparison.Ordinal));
                    Check.Equal(before, page.Offset);
                    log.Copy();
                    Check.Equal("log", window.Clipboard!.TryGetTextAsync().GetAwaiter().GetResult());
                }
                finally { window.Close(); }
            }, timeout.Token).WaitAsync(timeout.Token).GetAwaiter().GetResult();
        });
    }

    private static void Pump()
    {
        AvaloniaHeadlessPlatform.ForceRenderTimerTick();
        Dispatcher.UIThread.RunJobs();
    }
}
