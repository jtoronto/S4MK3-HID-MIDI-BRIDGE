using System.Globalization;
using System.Text.Json;

namespace S4Bridge.Diagnostics;

public sealed record Wheel(int Deck, double? Rpm, double Bend, string Mode)
{
    public string Present(bool stale)
    {
        var label = $"Deck {"ABCD"[Deck - 1]}";
        if (stale || Mode == "stale" || Rpm is null) return $"{label} | Stale | RPM -- | Bend --";
        return $"{label} | {Mode.Replace('_', ' ')} | {Signed(Rpm.Value)} RPM | {Signed(Bend)} counts/s";
    }
    private static string Signed(double number) =>
        (Math.Abs(number) < 0.05 ? 0 : number).ToString("+0.0;-0.0;0.0", CultureInfo.InvariantCulture);
}
public sealed record PlatterSnapshot(ulong HostUs, Wheel Left, Wheel Right)
{
    public const string Prefix = "S4_PLATTER_STATUS ";
    private static readonly string[] Modes = [
        "off","waiting","bend","scratch","paused_scratch","manual_scratch","blocked","stale","seek","grid"];
    public static PlatterSnapshot Parse(string line)
    {
        using var doc = JsonDocument.Parse(line[Prefix.Length..]);
        var root = doc.RootElement;
        var wheels = root.GetProperty("wheels");
        if (wheels.ValueKind != JsonValueKind.Array || wheels.GetArrayLength() != 2)
            throw new InvalidDataException("Exactly two platter wheels required.");
        return new(root.GetProperty("host_us").GetUInt64(),
            ParseWheel(wheels[0], [1, 3]), ParseWheel(wheels[1], [2, 4]));
    }
    private static Wheel ParseWheel(JsonElement node, int[] decks)
    {
        var deck = node.GetProperty("deck").GetInt32();
        if (!decks.Contains(deck)) throw new InvalidDataException("Invalid wheel deck.");
        var mode = node.GetProperty("mode").GetString();
        if (mode is null || !Modes.Contains(mode)) throw new InvalidDataException("Invalid wheel mode.");
        var raw = node.GetProperty("rpm");
        double? rpm = raw.ValueKind == JsonValueKind.Null ? null : raw.GetDouble();
        var bend = node.GetProperty("bend_counts_per_second").GetDouble();
        if (!double.IsFinite(bend) || (rpm is { } speed && !double.IsFinite(speed)))
            throw new InvalidDataException("Platter values must be finite.");
        return new(deck, rpm, bend, mode);
    }
}
