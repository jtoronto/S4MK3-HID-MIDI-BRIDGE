using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;

namespace S4Bridge.Models;

public static class PreferenceCodec
{
    public static readonly JsonSerializerOptions Snake = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = true,
        UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow
    };
    public static readonly JsonSerializerOptions Camel = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true
    };
    public static readonly string[] Colors = [
        "red","carrot","orange","honey","yellow","lime","green","aqua","cyan",
        "sky","blue","purple","fuchsia","magenta","azalea","salmon","white"];

    public static T Decode<T>(string input, string defaults) where T : class
    {
        var root = Object(input);
        var basis = Object(defaults);
        var run = typeof(T) == typeof(RunPreferences);
        var jog = typeof(T) == typeof(JogPreferences);
        if (jog)
        {
            foreach (var obj in new[] { basis, root })
            {
                if (obj.Remove("platter_position_sensitivity", out var retired) && retired is not null)
                    _ = Finite(retired);
            }
            basis.TryAdd("tt_brake_gain", null);
            basis.TryAdd("tt_nudge_gain", null);
            if (root.ContainsKey("tt_pitch_bend_gain"))
                foreach (var key in new[] { "tt_brake_gain", "tt_nudge_gain" })
                    if (!root.ContainsKey(key)) basis[key] = null;
        }
        foreach (var entry in root)
        {
            if (!basis.ContainsKey(entry.Key) && !run)
                throw new InvalidDataException($"Unknown setting: {entry.Key}");
            basis[entry.Key] = entry.Value?.DeepClone();
        }
        // Run files historically require every field, unlike the partial Rust configs.
        if (run) basis = root;
        var options = run || typeof(T) == typeof(ScreenPreferences) ? Camel : Snake;
        var json = basis.ToJsonString();
        ValidateShape<T>(basis);
        var value = JsonSerializer.Deserialize<T>(json, options)
            ?? throw new InvalidDataException("Preferences must be an object.");
        Validate(value);
        return value;
    }

    public static string Encode<T>(T value)
    {
        Validate(value);
        return JsonSerializer.Serialize(value,
            value is RunPreferences or ScreenPreferences ? Camel : Snake);
    }

    private static JsonObject Object(string json)
    {
        using var document = JsonDocument.Parse(json, new JsonDocumentOptions { MaxDepth = 16 });
        if (document.RootElement.ValueKind != JsonValueKind.Object)
            throw new InvalidDataException("Preferences must be a JSON object.");
        var keys = new HashSet<string>(StringComparer.Ordinal);
        foreach (var key in document.RootElement.EnumerateObject())
            if (!keys.Add(key.Name)) throw new InvalidDataException($"Duplicate setting: {key.Name}");
        return JsonNode.Parse(json)!.AsObject();
    }

    private static void ValidateShape<T>(JsonObject root)
    {
        // JsonSerializer enforces exact string/bool/integer types. Check raw tokens
        // before binding too: integral float/exponent tokens are not Rust integers.
        var type = typeof(T);
        var options = type == typeof(RunPreferences) || type == typeof(ScreenPreferences) ? Camel : Snake;
        foreach (var property in type.GetProperties().Where(p => p.CanWrite))
        {
            var key = options.PropertyNamingPolicy!.ConvertName(property.Name);
            if (type == typeof(RunPreferences) && !root.ContainsKey(key))
                throw new InvalidDataException($"Run preferences require {key}.");
            if (root.TryGetPropertyValue(key, out var node) && property.PropertyType == typeof(int))
            {
                var raw = node?.ToJsonString() ?? "";
                if (raw.Length == 0 || !raw.TrimStart('-').All(char.IsAsciiDigit)
                    || !int.TryParse(raw, out _))
                    throw new InvalidDataException($"{key} requires an integer token.");
            }
        }
    }

    private static double Finite(JsonNode node)
    {
        var number = node.GetValue<double>();
        if (!double.IsFinite(number)) throw new InvalidDataException("Number must be finite.");
        return number;
    }

    public static void Validate<T>(T value)
    {
        switch (value)
        {
            case RunPreferences run:
                Choice(run.Profile, ["full", "minimal", "probe"], "profile");
                Choice(run.LogLevel, ["warn", "info", "debug"], "logLevel");
                Range(run.DurationSeconds, 1, 604800, "durationSeconds"); break;
            case TensionPreferences tension:
                Range(tension.Tension, -100, 100, "tension"); break;
            case ScreenPreferences: break;
            case LedPreferences led: ValidateLed(led); break;
            case JogPreferences jog: ValidateJog(jog); break;
            default: throw new InvalidDataException("Unsupported preference type.");
        }
    }

    public static void Range(double number, double min, double max, string name)
    {
        if (!double.IsFinite(number) || number < min || number > max)
            throw new InvalidDataException($"{name} must be between {min} and {max}.");
    }
    public static void Choice(string value, string[] choices, string name)
    {
        if (!choices.Contains(value, StringComparer.Ordinal))
            throw new InvalidDataException($"Unsupported {name}: {value}");
    }
    private static void ValidateLed(LedPreferences led)
    {
        foreach (var (colors, count) in new[] { (led.DeckColors,4), (led.StemColors,4),
                     (led.DjayPadColors,8), (led.FixedHotcueColors,8) })
        {
            if (colors is null || colors.Length != count)
                throw new InvalidDataException($"Palette requires exactly {count} colors.");
            foreach (var color in colors) Choice(color, Colors, "palette color");
        }
        foreach (var color in new[] { led.DjayWhite, led.MuteColor, led.LoopColor })
            Choice(color, Colors, "color");
        Choice(led.HotcueColors, ["djay", "fixed_slots"], "hotcue colors");
        Choice(led.InactiveDisplay, ["dim", "dark"], "inactive display");
        Choice(led.QuantizeMixed, ["dim", "dark"], "mixed quantize");
        Range(led.ActiveBrightness, 1, 127, "active brightness");
        Range(led.InactiveBrightness, 0, 127, "inactive brightness");
        Range(led.PaletteActiveIntensity, 0, 3, "active palette intensity");
        Range(led.PaletteInactiveIntensity, 0, 3, "inactive palette intensity");
        Range(led.ChasePeriodMs, 250, 10000, "chase period");
        Range(led.MeterGamma, 0.1, 5, "meter gamma");
        if (!new[] { 0, 125, 127 }.Contains(led.MeterBrightness))
            throw new InvalidDataException("Meter brightness must be 0, 125 or 127.");
        Range(led.MeterStaleMs, 100, 5000, "meter freshness");
        if (led.TempoCenterTolerance is { } tolerance)
        {
            Range(tolerance, double.Epsilon, 0.1, "tempo center tolerance");
            if ((float)tolerance <= 0) throw new InvalidDataException("Tolerance underflows Rust f32.");
        }
    }
    private static void ValidateJog(JogPreferences jog)
    {
        Range(jog.ScratchSpeed, double.Epsilon, double.MaxValue, "scratch speed");
        Range(jog.PitchBendSpeed, double.Epsilon, double.MaxValue, "pitch bend speed");
        Range(jog.BackspinAcceleration, double.Epsilon, double.MaxValue, "backspin acceleration");
        Range(jog.BackspinSpeedIncrease, 0, double.MaxValue, "backspin speed increase");
        if (!double.IsFinite(jog.ScratchSpeed + jog.BackspinSpeedIncrease))
            throw new InvalidDataException("Backspin speed overflows scratch speed.");
        Choice(jog.ScratchDirection, ["normal", "reversed"], "scratch direction");
        Range(jog.ScratchReaction, 0, 150, "scratch reaction");
        Range(jog.PitchBendReaction, 0, 150, "pitch bend reaction");
        Range(jog.BackspinAssistStrength, 0, 100, "backspin assist strength");
        Range(jog.TtPitchBendFloorPercent, 1, 95, "TT floor");
        Range(jog.TtPitchBendGain, 0.1, 10, "TT shared gain");
        if (jog.TtBrakeGain is { } brake) Range(brake, 0.1, 20, "brake gain");
        if (jog.TtNudgeGain is { } nudge) Range(nudge, 0.1, 10, "nudge gain");
        Range(jog.TtBrakeCurveExponent, 0.25, 4, "brake shape");
        Range(jog.TtNudgeCurveExponent, 0.25, 4, "nudge shape");
        Range(jog.PlatterTargetRpm, 1, 45, "target RPM");
        Range(jog.PlatterMaxForce, 1, 25000, "max force");
        foreach (var force in new[] { jog.PlatterBaseForce, jog.PlatterForcePerRpm, jog.PlatterKp, jog.PlatterKi })
            Range(force, 0, 25000, "motor force");
        Range(jog.PlatterForceRisePerSecond, 0, 1000000, "force rise");
        Range(jog.PlatterStartupForceRisePerSecond, 0, 1000000, "startup rise");
        Range(jog.PlatterStartupMs, 0, 2000, "startup duration");
        Range(jog.PlatterOverspeedLimitRpm, 0, 250, "overspeed");
    }
}
