using Avalonia;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.LogicalTree;
using S4Bridge.Core;
using S4Bridge.Diagnostics;
using S4Bridge.Models;
using S4Bridge.Views;

namespace S4Bridge.Tests;

internal static class ControlTests
{
    private static HostController RealHost(Fixture f)
    {
        var resources = f.PathFor("res");
        Directory.CreateDirectory(resources);
        foreach (var name in new[] { "default-led-config.json", "default-jog-config.json", "default-tension-config.json" })
            File.Copy(Path.Combine(f.Resources, name), Path.Combine(resources, name));
        File.WriteAllText(Path.Combine(resources, "s4mk3-hid-midi-bridge.exe"), "stub");
        return new HostController(resources, f.PathFor("state"));
    }
    private static TextBox FindTextBox(ILogical root, string name) =>
        root.GetLogicalDescendants().OfType<TextBox>()
            .Single(b => AutomationProperties.GetName(b) == name);
    private static ComboBox FindCombo(ILogical root, string name) =>
        root.GetLogicalDescendants().OfType<ComboBox>()
            .Single(b => AutomationProperties.GetName(b) == name);
    private static Slider FindSlider(ILogical root, string name) =>
        root.GetLogicalDescendants().OfType<Slider>()
            .Single(b => AutomationProperties.GetName(b) == name);
    private static CheckBox FindCheck(ILogical root, string label) =>
        root.GetLogicalDescendants().OfType<CheckBox>()
            .Single(b => b.Content is string text && text == label);
    private static Button FindButton(ILogical root, string label) =>
        root.GetLogicalDescendants().OfType<Button>()
            .Single(b => b.Content is string text && text == label);
    private static string[] StateFiles(Fixture f)
    {
        var state = f.PathFor("state");
        return Directory.Exists(state) ? Directory.GetFiles(state, "*", SearchOption.AllDirectories) : [];
    }
    private static void TypeText(TextBox box, string text)
    {
        box.Text = text;
        box.RaiseEvent(new TextChangedEventArgs(TextBox.TextChangedEvent));
    }
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "experimental NI decks locks mode and omits Direct HID diagnostics", () =>
        {
            using var f = new Fixture();
            var host = new HostController(f.Resources, f.PathFor("experimental"), experimentalNiDecks: true);
            using var diagnostics = host.Diagnostics;
            var view = new SettingsView(host);
            try
            {
                Check.Equal(BridgeMode.NiBridge, host.SelectedMode);
                Check.True(!host.CanChangeMode);
                Check.True(!view.BuiltTabs[1].GetLogicalDescendants().OfType<Button>()
                    .Any(b => Equals(b.Content, "Retry verified access")));
                Check.True(!view.BuiltTabs[5].GetLogicalDescendants().OfType<CheckBox>()
                    .Any(b => Equals(b.Content, "Capture MIDI on the next full-profile launch")));
                Check.True(!view.BuiltTabs[5].GetLogicalDescendants().OfType<Button>()
                    .Any(b => Equals(b.Content, "Start motor capture")));
                Check.Reject(() => view.ToggleScreensAsync().GetAwaiter().GetResult());
            }
            finally { view.StopRefresh(); }
        });
        Check.Add(tests, "MIDI opt-in and marker control follow actual capture lifecycle", () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            using var diagnostics = host.Diagnostics;
            var view = new SettingsView(host);
            try
            {
                view.GetLogicalDescendants().OfType<TabControl>().Single().SelectedIndex = 5;
                var optIn = FindCheck(view.BuiltTabs[5], "Capture MIDI on the next full-profile launch");
                var mark = FindButton(view.BuiltTabs[5], "Mark issue");
                Check.True(optIn.IsChecked != true);
                Check.True(!mark.IsVisible && !mark.IsEnabled);
                optIn.IsChecked = true;
                Check.True(host.MidiCapture.EnabledForNextLaunch);
                var path = host.MidiCapture.PrepareLaunch(host.Store.Root, "full")!;
                var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var acknowledged = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                host.Supervisor.Changed += () => { if (host.Supervisor.State == "ready") ready.TrySetResult(); };
                host.MidiCapture.Changed += () =>
                {
                    if (host.MidiCapture.LastMarkerAccepted == true) acknowledged.TrySetResult();
                };
                var (exe, args) = Fixture.Child("capture-lifecycle", "full", path);
                host.Supervisor.Start(new(exe, args, new() { Profile = "full" }));
                try
                {
                    ready.Task.WaitAsync(TimeSpan.FromSeconds(10)).GetAwaiter().GetResult();
                    view.RefreshState();
                    Check.True(optIn.IsChecked != true);
                    Check.True(mark.IsVisible && mark.IsEnabled);
                    mark.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
                    acknowledged.Task.WaitAsync(TimeSpan.FromSeconds(10)).GetAwaiter().GetResult();
                    Check.Equal<ulong?>(1, host.MidiCapture.LastMarkerId);
                    host.Supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10)).GetAwaiter().GetResult();
                    Check.Equal("finished", host.MidiCapture.State);
                    Check.Equal(1UL, host.MidiCapture.RecordCount);
                    view.RefreshState();
                    Check.True(!mark.IsVisible && !mark.IsEnabled);
                }
                finally
                {
                    if (host.Supervisor.HasChild)
                    {
                        host.Supervisor.CloseInput();
                        host.Supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10)).GetAwaiter().GetResult();
                    }
                }
            }
            finally { view.StopRefresh(); }
        });
        Check.Add(tests, "command preview follows editor when stopped and owned child when running", () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            using var diagnostics = host.Diagnostics;
            host.Store.Editor.Run.Profile = "full";
            host.Store.Save();
            var view = new SettingsView(host);
            try
            {
                view.GetLogicalDescendants().OfType<TabControl>().Single().SelectedIndex = 5;
                var command = (SelectableTextBlock)typeof(SettingsView)
                    .GetField("command", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance)!
                    .GetValue(view)!;
                string Preview()
                {
                    view.RefreshVisibility(true, false);
                    return command.Text!;
                }
                host.Store.Editor.Screen.Enabled = true;
                host.Store.Editor.Run.LimitDuration = true;
                host.Store.Editor.Run.DurationSeconds = 37;
                host.Store.Editor.Run.RawReports = true;
                foreach (var profile in new[] { "minimal", "probe", "full" })
                {
                    host.Store.Editor.Run.Profile = profile;
                    var text = Preview();
                    Check.True(text.Contains("\"--seconds\" \"37\"", StringComparison.Ordinal));
                    Check.True(text.Contains("\"--raw\"", StringComparison.Ordinal));
                    Check.Equal(profile != "probe", text.Contains("\"--midi\"", StringComparison.Ordinal));
                    if (profile != "probe")
                        Check.True(text.Contains($"\"--midi\" \"{profile}\"", StringComparison.Ordinal));
                    Check.True(!text.Contains("\"--screens\"", StringComparison.Ordinal));
                    Check.True(!text.Contains("\"--screen-control-stdin\"", StringComparison.Ordinal));
                    foreach (var config in new[] { "led", "jog", "tension" })
                        Check.Equal(profile == "full", text.Contains($"\"--{config}-config\"", StringComparison.Ordinal));
                }
                var saved = LaunchSpec.Runtime(host.Executable, host.Store, false);
                Check.Equal("full", saved.Run.Profile);
                Check.True(!saved.Arguments.Contains("--screens"));
                Check.True(!saved.Arguments.Contains("--screen-control-stdin"));
                Check.True(!saved.Arguments.Contains("--raw"));
                Check.True(saved.Arguments.Contains("--until-stopped"));
                var diagnostic = LaunchSpec.Runtime(host.Executable, host.Store, true, null, null);
                Check.True(diagnostic.Arguments.Contains("--screens"));
                Check.True(diagnostic.Arguments.Contains("--screen-control-stdin"));

                host.SelectMode(BridgeMode.NiBridge);
                Check.Equal(BridgeMode.NiBridge, host.SelectedMode);
                var niPreview = Preview();
                Check.True(niPreview.Contains("\"--ni-decks\"", StringComparison.Ordinal));
                Check.True(niPreview.Contains("\"--midi\" \"full\"", StringComparison.Ordinal));
                Check.True(niPreview.Contains("\"--until-stopped\"", StringComparison.Ordinal));
                Check.True(!niPreview.Contains("\"--screens\"", StringComparison.Ordinal));
                Check.True(!niPreview.Contains("\"--screen-control-stdin\"", StringComparison.Ordinal));
                host.Store.Save();
                var niRuntime = LaunchSpec.Runtime(host.Executable, host.Store, false, null, null);
                Check.True(niRuntime.Arguments.Contains("--ni-decks"));
                Check.Equal("full", niRuntime.Run.Profile);
                Check.True(!niRuntime.Run.LimitDuration);
                Check.True(niRuntime.Arguments.Contains("--midi-backend"));
                Check.True(niRuntime.Arguments.Contains("bloop"));
                host.Store.Editor.Screen.Enabled = false;
                host.Store.Editor.Run.LimitDuration = false;
                host.Store.Editor.Run.RawReports = false;
                var unlimited = Preview();
                Check.True(unlimited.Contains("\"--until-stopped\"", StringComparison.Ordinal));
                Check.True(!unlimited.Contains("\"--seconds\"", StringComparison.Ordinal));
                Check.True(!unlimited.Contains("\"--raw\"", StringComparison.Ordinal));
                Check.True(!unlimited.Contains("\"--screens\"", StringComparison.Ordinal));
                Check.True(!unlimited.Contains("\"--screen-control-stdin\"", StringComparison.Ordinal));

                var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                host.Supervisor.Changed += () => { if (host.Supervisor.State == "ready") ready.TrySetResult(); };
                var (exe, args) = Fixture.Child("managed", "probe");
                var active = new LaunchSpec(exe, args, new() { Profile = "probe" });
                host.Supervisor.Start(active);
                try
                {
                    ready.Task.WaitAsync(TimeSpan.FromSeconds(10)).GetAwaiter().GetResult();
                    Check.Equal(active.Preview, Preview());
                }
                finally
                {
                    host.Supervisor.CloseInput();
                    host.Supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10)).GetAwaiter().GetResult();
                }
                Check.True(Preview().Contains("\"--ni-decks\"", StringComparison.Ordinal));
            }
            finally { view.StopRefresh(); }
        });
        Check.Add(tests, "owner visibility suspends diagnostics presentation while capture continues", () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            using var diagnostics = host.Diagnostics;
            host.Store.Editor.Run.LogLevel = "debug";
            var view = new SettingsView(host);
            try
            {
                var tabs = view.GetLogicalDescendants().OfType<TabControl>().Single();
                tabs.SelectedIndex = 5;
                var log = view.BuiltTabs[5].GetLogicalDescendants().OfType<IncrementalLogView>().Single();
                var timer = (Avalonia.Threading.DispatcherTimer)typeof(SettingsView)
                    .GetField("refresh", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance)!
                    .GetValue(view)!;
                host.Diagnostics.Log.Append("before hide");
                view.RefreshVisibility(true, false);
                Check.True(timer.IsEnabled);
                Check.True(log.Text!.Contains("before hide", StringComparison.Ordinal));
                view.RefreshVisibility(false, false);
                Check.True(!timer.IsEnabled);
                var launch = Guid.NewGuid();
                host.Diagnostics.BeginLaunch(launch);
                host.Diagnostics.StartCapture(f.PathFor("state"), host.Store.Editor.Run);
                host.Diagnostics.AcceptStderr(launch, "MOTOR_JOG_BEND hidden");
                host.Diagnostics.Log.Append("while hidden");
                view.RefreshVisibility(false, false);
                Check.True(!log.Text!.Contains("while hidden", StringComparison.Ordinal));
                using (var capture = new StreamReader(new FileStream(host.Diagnostics.CapturePath!, FileMode.Open, FileAccess.Read, FileShare.ReadWrite)))
                    Check.True(capture.ReadToEnd().Contains("MOTOR_JOG_BEND hidden", StringComparison.Ordinal));
                view.RefreshVisibility(true, false);
                Check.True(timer.IsEnabled);
                Check.True(log.Text!.Contains("while hidden", StringComparison.Ordinal));
                view.RefreshVisibility(true, true);
                Check.True(!timer.IsEnabled);
                host.Diagnostics.Log.Append("while minimized");
                view.RefreshVisibility(true, true);
                Check.True(!log.Text!.Contains("while minimized", StringComparison.Ordinal));
                view.RefreshVisibility(true, false);
                Check.True(log.Text!.Contains("while minimized", StringComparison.Ordinal));
                host.Diagnostics.StopCapture();
            }
            finally { view.StopRefresh(); }
        });
        Check.Add(tests, "actual read-only Avalonia log receives incremental append and trim", () =>
        {
            var view = new IncrementalLogView();
            view.Apply(new(1, 0, "first line\n"));
            Check.Equal("first line\n", view.Text);
            view.Apply(new(2, 6, "tail\n"));
            Check.Equal("line\ntail\n", view.Text);
            Check.True(view.IsReadOnly);
        });
        Check.Add(tests, "incremental log participates in the standard TextBox theme", () =>
        {
            var view = new IncrementalLogView();
            Check.Equal(typeof(TextBox), view.StyleKey);
        });
        Check.Add(tests, "incremental log preserves a user selection during append", () =>
        {
            var view = new IncrementalLogView { Text = "user selection" };
            view.SelectionStart = 1; view.SelectionEnd = 4;
            view.Apply(new(1, 0, " more"));
            Check.Equal("user selection more", view.Text);
            Check.Equal(1, view.SelectionStart); Check.Equal(4, view.SelectionEnd);
        });
        Check.Add(tests, "invalid numeric drafts disable tray and window start without launching", () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            var view = new SettingsView(host);
            var bridge = view.BuiltTabs[0];
            var duration = FindTextBox(bridge, "Run duration (seconds)");
            var startButton = FindButton(view, "Start bridge");
            var saveButton = FindButton(view, "Save preferences");
            var previous = host.Store.Editor.Run.DurationSeconds;
            Check.True(host.FieldsValid);
            Check.True(host.CanStart);
            Check.True(startButton.IsEnabled);
            var before = StateFiles(f);
            TypeText(duration, "0");
            Check.Equal(previous, host.Store.Editor.Run.DurationSeconds);
            Check.True(!host.FieldsValid);
            Check.True(!host.CanStart);
            Check.True(!startButton.IsEnabled);
            Check.True(!saveButton.IsEnabled);
            view.IsVisible = false;
            view.RefreshState();
            Check.True(!host.CanStart);
            var startAttempt = host.StartAsync();
            Check.True(startAttempt.IsFaulted);
            Check.True(startAttempt.Exception?.InnerException is InvalidDataException startError
                && startError.Message.Contains("Run duration", StringComparison.Ordinal));
            var saveAttempt = host.ApplyAsync();
            Check.True(saveAttempt.IsFaulted);
            Check.True(saveAttempt.Exception?.InnerException is InvalidDataException);
            Check.True(!host.Supervisor.HasChild);
            Check.Equal(before.Length, StateFiles(f).Length);
            TypeText(duration, previous == 120 ? "121" : "120");
            Check.True(host.FieldsValid);
            Check.True(host.CanStart);
            view.IsVisible = true;
            view.RefreshState();
            Check.True(startButton.IsEnabled);
            Check.True(saveButton.IsEnabled);
        });
        Check.Add(tests, "motor assist strength disables with either backspin switch while retaining its value", () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            host.Store.Editor.Jog.BackspinEnabled = true;
            host.Store.Editor.Jog.BackspinAssistEnabled = true;
            host.Store.Editor.Jog.BackspinAssistStrength = 37;
            var view = new SettingsView(host);
            var jog = view.BuiltTabs[2];
            var detection = FindCheck(jog, "Continue detected backspin after release");
            var assist = FindCheck(jog, "Use motor assistance for backspins");
            var strength = FindSlider(jog, "Motor assist strength (%)");
            Check.True(detection.IsChecked == true);
            Check.True(assist.IsChecked == true);
            Check.True(strength.IsEnabled);
            assist.IsChecked = false;
            Check.True(!strength.IsEnabled);
            Check.Equal(37, host.Store.Editor.Jog.BackspinAssistStrength);
            assist.IsChecked = true;
            Check.True(strength.IsEnabled);
            detection.IsChecked = false;
            Check.True(!strength.IsEnabled);
            Check.Equal(37, host.Store.Editor.Jog.BackspinAssistStrength);
            detection.IsChecked = true;
            Check.True(strength.IsEnabled);
            Check.Equal(37, host.Store.Editor.Jog.BackspinAssistStrength);
        });
        Check.Add(tests, "bridge mode cannot change during screen operations or motor capture", () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            host.Store.Editor.Run.LogLevel = "debug";
            var consent = new TaskCompletionSource<ScreenConsent>(TaskCreationOptions.RunContinuationsAsynchronously);
            var enabling = host.Screens.SetEnabledAsync(true, host.Supervisor, _ => consent.Task);
            Check.True(host.Screens.Busy);
            Check.True(!host.CanChangeMode);
            Check.Reject(() => host.SelectMode(BridgeMode.NiBridge));
            Check.Equal("full", host.Store.Editor.Run.Profile);
            consent.SetResult(ScreenConsent.WithoutScreens);
            Check.True(enabling.Wait(TimeSpan.FromSeconds(10)));
            Check.True(!host.Screens.Busy);
            Check.True(host.CanChangeMode);
            host.Diagnostics.StartCapture(f.PathFor("state"), host.Store.Editor.Run);
            Check.True(!host.CanChangeMode);
            Check.Reject(() => host.SelectMode(BridgeMode.NiBridge));
            Check.Equal("full", host.Store.Editor.Run.Profile);
            host.Diagnostics.StopCapture();
            Check.True(host.CanChangeMode);
        });
        Check.Add(tests, "startup error detail is empty when engine is packaged and preferences are valid", () =>
        {
            using var f = new Fixture();
            var resources = f.PathFor("res");
            Directory.CreateDirectory(resources);
            foreach (var name in new[] { "default-led-config.json", "default-jog-config.json", "default-tension-config.json" })
                File.Copy(Path.Combine(f.Resources, name), Path.Combine(resources, name));
            File.WriteAllText(Path.Combine(resources, "s4mk3-hid-midi-bridge.exe"), "stub");
            var host = new S4Bridge.Core.HostController(resources, f.PathFor("state"));
            Check.Equal("", host.FormatErrorDetail());
            Check.Equal("field error", host.FormatErrorDetail("field error"));
            host.Report(new InvalidDataException("boom"));
            Check.Equal("boom\nfield error", host.FormatErrorDetail("field error"));
        });
        tests.Add(("successful screen access never enables error details across its lifecycle", async () =>
        {
            using var f = new Fixture();
            var host = RealHost(f);
            File.WriteAllText(Path.Combine(f.PathFor("res"), "screen-access.json"),
                """{"version":1,"route":"engine_managed_ni_session","accepted":true,"evidence":"test fixture"}""");
            var view = new SettingsView(host);
            var details = FindButton(view, "Error details...");
            Check.Equal((bool?)true, await host.Screens.PrepareAsync(_ => Task.FromResult(ScreenConsent.Enable)));
            Check.True(!string.IsNullOrWhiteSpace(host.Screens.Detail));
            var id = Guid.NewGuid();
            host.Store.Editor.Screen.Enabled = false;
            host.Screens.Started(id, true, true);
            foreach (var status in new[] { "pending", "active", "inactive" })
            {
                host.Screens.Accept(id, "S4_SCREEN_STATUS 0 " + status);
                view.RefreshState();
                Check.Equal(status == "pending" ? "pending" : status == "active" ? "active" : "ready",
                    host.Screens.State);
                Check.Equal("", host.FormatErrorDetail());
                Check.True(!details.IsVisible && !details.IsEnabled);
            }
            host.Screens.Exited(id);
            Check.Equal("ready", host.Screens.State);
            Check.Equal("", host.FormatErrorDetail());
            await host.Screens.SetEnabledAsync(false, host.Supervisor, _ => Task.FromResult(ScreenConsent.Cancel));
            Check.Equal("off", host.Screens.State);
            Check.Equal("", host.FormatErrorDetail());
            var releaseError = new IOException("Unconfirmed release fixture");
            host.Report(releaseError);
            view.RefreshState();
            Check.Equal(releaseError.Message, host.FormatErrorDetail());
            Check.True(details.IsVisible && details.IsEnabled);
        }));
        foreach (var state in new[] { "blocked", "failed" })
            tests.Add(($"screen {state} remains in error details even when disabled", async () =>
            {
                using var f = new Fixture();
                var host = RealHost(f);
                if (state == "blocked")
                    await host.Screens.PrepareAsync(_ => Task.FromResult(ScreenConsent.Cancel));
                else
                {
                    var id = Guid.NewGuid();
                    host.Screens.Started(id, true);
                    host.Screens.Accept(id, "S4_SCREEN_STATUS 0 failed");
                }
                Check.Equal(state, host.Screens.State);
                Check.True(!string.IsNullOrWhiteSpace(host.Screens.Detail));
                host.Store.Editor.Screen.Enabled = false;
                Check.Equal(host.Screens.Detail, host.FormatErrorDetail());
                var view = new SettingsView(host);
                var details = FindButton(view, "Error details...");
                Check.True(details.IsVisible && details.IsEnabled);
            }));
        Check.Add(tests, "initial high DPI window moves above taskbar without shrinking a fitting client", () =>
        {
            var result = WindowPlacement.Fit(new PixelRect(0, 0, 1920, 1020), 1.25,
                new Size(860, 720), new Size(872, 750), new PixelPoint(137, 129));
            Check.Equal(new Size(860, 720), result.ClientSize);
            Check.Equal(new PixelPoint(137, 82), result.Position);
        });
        Check.Add(tests, "initial window accounts for native chrome when scaled working area is smaller", () =>
        {
            var result = WindowPlacement.Fit(new PixelRect(0, 0, 1280, 680), 2,
                new Size(860, 720), new Size(872, 750), new PixelPoint(100, 100));
            Check.Equal(new Size(628, 310), result.ClientSize);
            Check.Equal(new PixelPoint(0, 0), result.Position);
        });
        Check.Add(tests, "initial window respects negative monitor origin and top working area inset", () =>
        {
            var result = WindowPlacement.Fit(new PixelRect(-1920, 40, 1920, 1000), 1.25,
                new Size(860, 720), new Size(872, 750), new PixelPoint(-2000, 0));
            Check.Equal(new Size(860, 720), result.ClientSize);
            Check.Equal(new PixelPoint(-1920, 40), result.Position);
        });
    }
}
