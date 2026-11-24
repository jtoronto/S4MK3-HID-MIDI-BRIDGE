using Avalonia;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using S4Bridge.Core;
using S4Bridge.Models;
using S4Bridge.Views;

namespace S4Bridge.Tests;

internal static class NiTensionViewTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "NI tension slider is accessible and persists stopped edits", () =>
        {
            using var session = HeadlessUnitTestSession.StartNew(typeof(HeadlessTestApp));
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(10));
            session.Dispatch(async () =>
            {
                using var fixture = new Fixture();
                var host = new HostController(fixture.Resources, fixture.PathFor("state"));
                using var diagnostics = host.Diagnostics;
                var view = new SettingsView(host);
                var window = new Window { Width = 860, Height = 720, Content = view };
                try
                {
                    window.Show();
                    Pump();
                    Check.True(!view.BuiltTabs[0].GetLogicalDescendants().OfType<Slider>()
                        .Any(s => AutomationProperties.GetName(s) == "NI jog wheel tension"));
                    var mode = view.BuiltTabs[0].GetLogicalDescendants().OfType<ComboBox>()
                        .Single(c => AutomationProperties.GetName(c) == "Bridge mode");
                    mode.SelectedItem = "NI Bridge";
                    Pump();
                    Check.True(!view.BuiltTabs[0].GetLogicalDescendants().OfType<NiTensionControl>().Any());
                    var jog = view.BuiltTabs.Single(t => Equals(t.Header, "Jog"));
                    jog.IsSelected = true;
                    Pump();
                    var slider = jog.GetLogicalDescendants().OfType<Slider>()
                        .Single(s => AutomationProperties.GetName(s) == "NI jog wheel tension");
                    Check.Equal(0d, slider.Minimum);
                    Check.Equal(255d, slider.Maximum);
                    Check.Equal(1d, slider.TickFrequency);
                    Check.True(slider.IsSnapToTickEnabled && slider.IsEnabled);
                    slider.BringIntoView();
                    Pump();
                    var scroll = slider.GetVisualAncestors().OfType<ScrollViewer>().First();
                    var position = slider.TranslatePoint(default, scroll);
                    Check.True(position is { } point && point.Y >= 0 &&
                        point.Y + slider.Bounds.Height <= scroll.Bounds.Height,
                        "NI tension slider is outside the scroll viewport");
                    Check.True(slider.Focus(), "NI tension slider is not keyboard focusable");
                    var saved = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                    host.Changed += () =>
                    {
                        if (host.Store.HasSavedNiTension && host.Store.NiTension.Value == 255)
                            saved.TrySetResult();
                    };
                    slider.Value = 254;
                    window.KeyPressQwerty(PhysicalKey.ArrowRight, RawInputModifiers.None);
                    Pump();
                    Check.Equal(255d, slider.Value);
                    await saved.Task.WaitAsync(timeout.Token);
                    Check.Equal(255, new PreferenceStore(fixture.Resources, fixture.PathFor("state")).NiTension.Value);
                    Check.True(!host.Supervisor.HasChild, "Stopped edit launched a child");
                }
                finally { view.StopRefresh(); window.Close(); }
                return true;
            }, timeout.Token).WaitAsync(timeout.Token).GetAwaiter().GetResult();
        });
        Check.Add(tests, "NI tension slider sends a running edit to the owned fixture", () =>
        {
            using var session = HeadlessUnitTestSession.StartNew(typeof(HeadlessTestApp));
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(15));
            session.Dispatch(async () =>
            {
                using var fixture = new Fixture();
                var host = new HostController(fixture.Resources, fixture.PathFor("state"));
                using var diagnostics = host.Diagnostics;
                host.SelectMode(BridgeMode.NiBridge);
                host.Store.Save();
                var view = new SettingsView(host);
                var window = new Window { Width = 860, Height = 720, Content = view };
                var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var active = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var received = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                host.Supervisor.Changed += () =>
                {
                    if (host.Supervisor.State == "ready") ready.TrySetResult();
                };
                host.Supervisor.Protocol += (_, line) =>
                {
                    if (line == "S4_SCREEN_STATUS 0 active") active.TrySetResult();
                };
                host.Supervisor.Output += (_, line) =>
                {
                    if (line == "INPUT ni_jog_tension 128") received.TrySetResult();
                };
                var (exe, args) = Fixture.Child("capture-pipes", "full");
                var spec = LaunchSpec.Runtime(exe, host.Store, false, null, null);
                Check.True(spec.UsesNiBridge, "Fixture launch must use NI Bridge mode");
                try
                {
                    window.Show();
                    host.Supervisor.Start(spec with { Arguments = [.. args, .. spec.Arguments] });
                    await ready.Task.WaitAsync(timeout.Token);
                    await host.Supervisor.SendScreenAsync(0, "ON");
                    await active.Task.WaitAsync(timeout.Token);
                    view.RefreshState();
                    Pump();
                    var jog = view.BuiltTabs.Single(t => Equals(t.Header, "Jog"));
                    jog.IsSelected = true;
                    Pump();
                    var control = jog.GetLogicalDescendants().OfType<NiTensionControl>().Single();
                    var slider = control.GetLogicalDescendants().OfType<Slider>().Single();
                    Check.True(slider.IsEnabled);
                    slider.Value = 128;
                    await received.Task.WaitAsync(timeout.Token);
                    view.RefreshState();
                    Pump();
                    Check.True(control.GetLogicalDescendants().OfType<TextBlock>()
                        .Any(text => text.Text == host.NiTensionStatus && text.IsVisible));
                }
                finally
                {
                    if (host.Supervisor.HasChild)
                    {
                        host.Supervisor.CloseInput();
                        await host.Supervisor.Completion.WaitAsync(timeout.Token);
                    }
                    view.StopRefresh();
                    window.Close();
                }
                return true;
            }, timeout.Token).WaitAsync(timeout.Token).GetAwaiter().GetResult();
        });
    }

    private static void Pump()
    {
        AvaloniaHeadlessPlatform.ForceRenderTimerTick();
        Dispatcher.UIThread.RunJobs();
    }
}
