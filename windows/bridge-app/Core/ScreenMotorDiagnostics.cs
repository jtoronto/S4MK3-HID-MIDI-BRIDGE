using System.Text.Json;

namespace S4Bridge.Core;

internal static class ScreenMotorDiagnostics
{
    internal const string EnvironmentVariable = "S4_SCREEN_MOTOR_DIAGNOSTIC_CONFIG";

    internal static string Validate(string path, bool preview = false)
    {
        if (!Path.IsPathFullyQualified(path) || !File.Exists(path))
            throw new InvalidDataException("Screen/motor diagnostic config must be an existing absolute file.");
        using var document = JsonDocument.Parse(File.ReadAllText(path));
        var root = document.RootElement;
        if (root.ValueKind != JsonValueKind.Object)
            throw new InvalidDataException("Screen/motor diagnostic config must be an object.");
        var properties = root.EnumerateObject().ToArray();
        if (properties.Length != 4 ||
            properties.Select(p => p.Name).Distinct(StringComparer.Ordinal).Count() != 4 ||
            properties.Any(p => p.Name is not ("version" or "case" or "capture_path" or "capture")))
            throw new InvalidDataException("Screen/motor diagnostic config has missing, duplicate, or unknown fields.");
        if (!root.TryGetProperty("version", out var version) || version.ValueKind != JsonValueKind.Number ||
            !version.TryGetInt32(out var number) || number != 1 ||
            !root.TryGetProperty("case", out var caseValue) || caseValue.ValueKind != JsonValueKind.String ||
            caseValue.GetString() is not ("normal" or "off" or "acquired" or "static" or "updates") ||
            !root.TryGetProperty("capture_path", out var capturePath) || capturePath.ValueKind != JsonValueKind.String ||
            !root.TryGetProperty("capture", out var capture) || capture.ValueKind is not (JsonValueKind.True or JsonValueKind.False))
            throw new InvalidDataException("Invalid screen/motor diagnostic config values.");
        var output = capturePath.GetString()!;
        if (!Path.IsPathFullyQualified(output) || (!preview && File.Exists(output)) || Directory.Exists(output) ||
            !string.Equals(Path.GetExtension(output), ".jsonl", StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("Diagnostic capture path must be a new absolute .jsonl file.");
        if (!capture.GetBoolean() && caseValue.GetString() != "normal")
            throw new InvalidDataException("Only the normal case may disable capture.");
        return caseValue.GetString()!;
    }
}
