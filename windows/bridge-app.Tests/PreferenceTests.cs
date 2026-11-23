using System.Text.Json.Nodes;
using S4Bridge.Core;
using S4Bridge.Models;

namespace S4Bridge.Tests;

internal static class PreferenceTests
{
    private static string Default(string area) => File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Resources", $"default-{area}-config.json"));
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "packaged defaults decode and retain selected force and tension", () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root);
            Check.Equal(-77, store.Editor.Tension.Tension); Check.Equal(7000, store.Editor.Jog.PlatterBaseForce);
            Check.Equal(10.0, store.Editor.Jog.EffectiveBrakeGain); Check.Equal(3.0, store.Editor.Jog.EffectiveNudgeGain);
            store.Editor.Jog.PlatterBaseForce = 200; store.Reset();
            Check.Equal(7000, store.Editor.Jog.PlatterBaseForce);
            Check.Equal(0, PreferenceCodec.Decode<TensionPreferences>("{}", "{\"tension\":0}").Tension);
        });
        foreach (var raw in new[] { "true", "null", "\"1\"", "1.0", "1e0", "1.5", "-101", "101", "[]", "{}" })
            Check.Add(tests, "tension rejects " + raw, () => Check.Reject(() =>
                PreferenceCodec.Decode<TensionPreferences>("{\"tension\":" + raw + "}", Default("tension"))));
        foreach (var (json, label) in new[]{
            ("{\"unknown\":1}","unknown"),("{\"deck_colors\":[]}","short palette"),
            ("{\"active_brightness\":0}","active low"),("{\"meter_brightness\":126}","invalid meter mode"),
            ("{\"meter_gamma\":5.1}","gamma high"),("{\"tempo_center_tolerance\":0}","zero tolerance"),
            ("{\"deck_colors\":[\"red\",\"blue\",\"white\",\"fake\"]}","unsupported color"),
            ("{\"inactive_brightness\":16.0}","float integer"),("{\"loop_enabled\":1}","boolean number"),
            ("{\"meter_stale_ms\":99}","stale low")})
            Check.Add(tests, "LED rejects " + label, () => Check.Reject(() =>
                PreferenceCodec.Decode<LedPreferences>(json, Default("led"))));
        foreach (var json in new[]{
            "{\"scratch_speed\":0}","{\"scratch_reaction\":151}","{\"platter_target_rpm\":33.0}",
            "{\"platter_startup_ms\":2001}","{\"tt_brake_gain\":21}","{\"tt_nudge_gain\":0.09}",
            "{\"tt_nudge_curve_exponent\":4.1}","{\"backspin_assist_strength\":101}",
            "{\"backspin_speed_increase\":-1}","{\"platter_position_sensitivity\":\"ignored?\"}",
            "{\"unknown\":true}","{\"platter_max_force\":25001}"})
            Check.Add(tests, "jog rejects " + json, () => Check.Reject(() =>
                PreferenceCodec.Decode<JogPreferences>(json, Default("jog"))));
        Check.Add(tests, "shared gain migration clears missing overrides and drops retired value", () =>
        {
            var jog = PreferenceCodec.Decode<JogPreferences>(
                "{\"tt_pitch_bend_gain\":4,\"platter_position_sensitivity\":14}", Default("jog"));
            Check.Equal(8.0, jog.EffectiveBrakeGain); Check.Equal(4.0, jog.EffectiveNudgeGain);
            var json = JsonNode.Parse(PreferenceCodec.Encode(jog))!.AsObject();
            Check.True(!json.ContainsKey("platter_position_sensitivity"));
            Check.True(!json.ContainsKey("tt_nudge_gain"));
        });
        Check.Add(tests, "run requires fields but ignores extras while screens reject extras", () =>
        {
            var run = PreferenceCodec.Encode(new RunPreferences());
            Check.Reject(() => PreferenceCodec.Decode<RunPreferences>("{}", run));
            var extra = JsonNode.Parse(run)!.AsObject(); extra["future"] = true;
            Check.Equal("full", PreferenceCodec.Decode<RunPreferences>(extra.ToJsonString(), run).Profile);
            Check.Reject(() => PreferenceCodec.Decode<ScreenPreferences>("{\"future\":true}",
                PreferenceCodec.Encode(new ScreenPreferences())));
        });
        Check.Add(tests, "legacy persisted run settings without a mode default to Direct HID", () =>
        {
            using var f = new Fixture();
            var legacy = JsonNode.Parse(PreferenceCodec.Encode(new RunPreferences()))!.AsObject();
            legacy.Remove("mode");
            File.WriteAllText(f.PathFor("run-settings.json"), legacy.ToJsonString());
            var store = new PreferenceStore(f.Resources, f.Root);
            Check.Equal(BridgeMode.DirectHid, store.Editor.Run.Mode);
            Check.Equal(BridgeMode.DirectHid, store.Saved.Run.Mode);
        });
        Check.Add(tests, "save persists snapshots and invalid editor preserves every existing file", () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root);
            store.Editor.Jog.ScratchSpeed = 4.7; store.Editor.Led.LoopColor = "blue"; store.Save();
            var before = Directory.GetFiles(f.Root).ToDictionary(p => p, File.ReadAllText);
            store.Editor.Tension.Tension = 101;
            Check.Reject(store.Save);
            foreach (var (path, content) in before) Check.Equal(content, File.ReadAllText(path));
            Check.Equal(-77, store.Saved.Tension.Tension);
            var reload = new PreferenceStore(f.Resources, f.Root);
            Check.Equal(4.7, reload.Editor.Jog.ScratchSpeed); Check.Equal("blue", reload.Editor.Led.LoopColor);
        });
        Check.Add(tests, "corrupt preferences fail closed and reset retains pending recovery", () =>
        {
            using var f = new Fixture();
            var corrupt = "{\"restorePending\":true,\"unknown\":0}";
            File.WriteAllText(f.PathFor("screen-settings.json"), corrupt);
            var store = new PreferenceStore(f.Resources, f.Root);
            Check.True(store.RequiresReset); Check.Reject(store.Save);
            Check.Equal(corrupt, File.ReadAllText(f.PathFor("screen-settings.json")));
            store.Reset(); Check.True(store.Editor.Screen.RestorePending);
            store.Save();
            Check.True(new PreferenceStore(f.Resources, f.Root).Editor.Screen.RestorePending);
        });
        Check.Add(tests, "LED import replaces editor only and cannot clear corrupt tension", () =>
        {
            using var f = new Fixture(); File.WriteAllText(f.PathFor("tension-config.json"), "{\"tension\":true}");
            var store = new PreferenceStore(f.Resources, f.Root);
            store.ImportLed("{\"loop_color\":\"cyan\"}");
            Check.Equal("cyan", store.Editor.Led.LoopColor); Check.True(store.RequiresReset);
            Check.Equal("green", store.Saved.Led.LoopColor);
        });
        Check.Add(tests, "motorized reset does not rewrite mapping backspin tension or LED choices", () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root);
            store.Editor.Jog.ScratchSpeed = 8; store.Editor.Jog.BackspinAcceleration = 123;
            store.Editor.Jog.PlatterTargetRpm = 45; store.Editor.Tension.Tension = 50; store.Editor.Led.LoopColor = "red";
            store.ResetMotorized();
            Check.Equal(33, store.Editor.Jog.PlatterTargetRpm); Check.Equal(8.0, store.Editor.Jog.ScratchSpeed);
            Check.Equal(123.0, store.Editor.Jog.BackspinAcceleration); Check.Equal(50, store.Editor.Tension.Tension);
            Check.Equal("red", store.Editor.Led.LoopColor);
        });
        Check.Add(tests, "failed sibling replacement leaves no temp file", () =>
        {
            using var f = new Fixture(); var destination = f.PathFor("directory"); Directory.CreateDirectory(destination);
            var rejected = false;
            try { PreferenceStore.AtomicWrite(destination, "data"); }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException) { rejected = true; }
            Check.True(rejected, "Directory destination replacement must fail.");
            Check.Equal(0, Directory.GetFiles(f.Root, "*.tmp").Length);
            Check.True(Directory.Exists(destination));
        });
        Check.Add(tests, "partial file save reports failure but saved snapshots reflect successful files", () =>
        {
            using var f = new Fixture(); var store = new PreferenceStore(f.Resources, f.Root);
            store.Editor.Run.Profile = "probe"; store.Editor.Led.LoopColor = "blue";
            Directory.CreateDirectory(f.PathFor("jog-config.json"));
            Check.Reject(store.Save);
            Check.Equal("probe", store.Saved.Run.Profile); Check.Equal("blue", store.Saved.Led.LoopColor);
            var reload = new PreferenceStore(f.Resources, f.Root);
            Check.True(reload.RequiresReset); Check.Equal("probe", reload.Editor.Run.Profile);
            Check.Equal("blue", reload.Editor.Led.LoopColor);
        });
    }
}
