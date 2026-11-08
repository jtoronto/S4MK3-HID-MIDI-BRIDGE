using S4Bridge.Diagnostics;
using S4Bridge.Models;
using S4Bridge.Core;
using S4Bridge.Platform;

namespace S4Bridge.Tests;

internal sealed class TestClock : TimeProvider
{
    public DateTimeOffset Now { get; set; } = DateTimeOffset.Parse("2026-11-04T00:00:00Z");
    public override DateTimeOffset GetUtcNow() => Now;
}
internal sealed class TestAccess(bool available) : INIServiceControl
{
    public ScreenAccess Inspect() => new(available, null, "Test access boundary");
}
internal static class DiagnosticsTests
{
    public const string Platter = "S4_PLATTER_STATUS {\"host_us\":1234,\"wheels\":[" +
        "{\"deck\":1,\"rpm\":33.04,\"bend_counts_per_second\":0,\"mode\":\"bend\"}," +
        "{\"deck\":2,\"rpm\":-1,\"bend_counts_per_second\":-42,\"mode\":\"scratch\"}]}";
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "bounded log deltas recreate retained Unicode text", () =>
        {
            var log = new BoundedLog(); var displayed = ""; long cursor = 0;
            foreach (var input in new[]{new string('a',40000),string.Concat(Enumerable.Repeat("\U0001f3a7",20000)),
                        new string('b',70000),"tail"})
            {
                log.Append(input);
                var delta = log.ReadSince(cursor, displayed.Length);
                displayed = displayed[delta.RemovePrefix..] + delta.Append; cursor = delta.Cursor;
                Check.Equal(log.Text, displayed); Check.True(displayed.Length <= 60000);
                Check.True(!char.IsLowSurrogate(displayed[0]));
            }
        });
        Check.Add(tests, "hidden diagnostics retain fresh telemetry without publishing and expire by clock", () =>
        {
            var clock = new TestClock(); using var hub = new DiagnosticHub(clock); var id = Guid.NewGuid();
            hub.BeginLaunch(id); hub.Accept(id, Platter);
            Check.True(!hub.Refresh(false, true));
            Check.Equal("Decks A / C | Stopped", hub.Presentation.Left);
            Check.True(hub.Refresh(true, true));
            Check.True(hub.Presentation.Left.Contains("33.0 RPM", StringComparison.Ordinal));
            Check.True(!hub.Refresh(true, true));
            clock.Now += TimeSpan.FromSeconds(0.501);
            Check.True(hub.Refresh(true, true)); Check.True(hub.Presentation.Left.Contains("Stale", StringComparison.Ordinal));
        });
        Check.Add(tests, "malformed or wrong-deck platter values are rejected", () =>
        {
            foreach (var input in new[]{Platter.Replace("\"deck\":1","\"deck\":2"),Platter.Replace("33.04","true"),
                       Platter.Replace("\"bend\"","\"fake\""),Platter.Replace("\"host_us\":1234","\"host_us\":-1")})
                Check.Reject(() => PlatterSnapshot.Parse(input));
        });
        Check.Add(tests, "capture uses launched receipt and survives hidden log trim", () =>
        {
            using var f = new Fixture(); using var hub = new DiagnosticHub(); var id = Guid.NewGuid();
            hub.BeginLaunch(id); hub.Accept(id, "S4_MOTOR_CONFIG {\"jog\":{\"platter_target_rpm\":45}}");
            hub.StartCapture(f.Root, new() { Profile = "full", LogLevel = "debug" });
            hub.Accept(Guid.NewGuid(), "MOTOR_REGULATOR wrong launch");
            hub.Accept(id, "MOTOR_REGULATOR first");
            hub.Log.Append(new string('x', 70000)); hub.Accept(id, "MOTOR_TT_STATE last");
            hub.StopCapture();
            var text = File.ReadAllText(hub.CapturePath!);
            Check.True(text.Contains("\"platter_target_rpm\":45", StringComparison.Ordinal));
            Check.True(text.Contains("MOTOR_REGULATOR first", StringComparison.Ordinal));
            Check.True(text.Contains("MOTOR_TT_STATE last", StringComparison.Ordinal));
            Check.True(!text.Contains("wrong launch", StringComparison.Ordinal));
            Check.True(text.Contains("# Stopped:", StringComparison.Ordinal));
        });
        Check.Add(tests, "capture refuses nondebug and minimal settings before creating files", () =>
        {
            using var f = new Fixture(); using var hub = new DiagnosticHub();
            Check.Reject(() => hub.StartCapture(f.Root, new()));
            Check.Reject(() => hub.StartCapture(f.Root, new() { Profile = "minimal", LogLevel = "debug" }));
            Check.True(!Directory.Exists(f.PathFor("Diagnostics")));
        });
        tests.Add(("unavailable access offers screenless or cancellation without service change", async () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root); store.Save();
            var screens = new ScreenCoordinator(store, new TestAccess(false));
            Check.Equal((bool?)null, await screens.PrepareAsync(_ => Task.FromResult(ScreenConsent.Cancel)));
            Check.Equal((bool?)false, await screens.PrepareAsync(_ => Task.FromResult(ScreenConsent.WithoutScreens)));
            Check.Equal("blocked", screens.State); Check.True(!store.Editor.Screen.RestorePending);
        }
        ));
        tests.Add(("accepted engine access follows guarded current launch and startup screen status", async () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root); store.Save();
            var screens = new ScreenCoordinator(store, new TestAccess(true)); var id = Guid.NewGuid();
            Check.Equal((bool?)true, await screens.PrepareAsync(_ => Task.FromResult(ScreenConsent.Enable)));
            screens.Started(id, true);
            screens.Accept(Guid.NewGuid(), "S4_SCREEN_STATUS 0 active"); Check.Equal("pending", screens.State);
            screens.Accept(id, "S4_SCREEN_STATUS 1 active"); Check.Equal("pending", screens.State);
            screens.Accept(id, "S4_SCREEN_STATUS 0 active"); Check.Equal("active", screens.State);
            screens.Exited(Guid.NewGuid()); Check.Equal("active", screens.State);
        }
        ));
        Check.Add(tests, "curve preserves quiet band and asymmetric selected gains", () =>
        {
            using var f = new Fixture(); var jog = new PreferenceStore(f.Resources, f.Root).Editor.Jog;
            Check.Equal(0.0, ResponseCurve.Rate(jog, 33 + 2.25));
            jog.TtBrakeGain = 2; jog.TtNudgeGain = 1; jog.TtBrakeCurveExponent = 1; jog.TtNudgeCurveExponent = 1;
            Check.Equal(-960.0, ResponseCurve.Rate(jog, 23)); Check.Equal(480.0, ResponseCurve.Rate(jog, 43));
        });
    }
}
