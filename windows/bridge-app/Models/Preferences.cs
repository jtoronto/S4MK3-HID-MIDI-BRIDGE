using System.Text.Json.Serialization;

namespace S4Bridge.Models;

public enum BridgeMode { DirectHid, NiBridge }

public sealed record RunPreferences
{
    public BridgeMode Mode { get; set; }
    public string Profile { get; set; } = "full";
    public bool LimitDuration { get; set; }
    public int DurationSeconds { get; set; } = 3600;
    public bool RawReports { get; set; }
    public string LogLevel { get; set; } = "info";
}

public sealed record ScreenPreferences
{
    public bool Enabled { get; set; } = true;
    public bool AutomaticallyStopNI { get; set; }
    public bool RestoreNIOnStop { get; set; } = true;
    public bool RestorePending { get; set; }
}

public sealed record TensionPreferences
{
    public int Tension { get; set; }
}

public sealed record NiTensionPreferences
{
    public int Value { get; set; } = 107;
}

// Values are initialized exclusively by decoding the packaged examples.
public sealed record JogPreferences
{
    public double ScratchSpeed { get; set; }
    public int ScratchReaction { get; set; }
    public string ScratchDirection { get; set; } = "";
    public double PitchBendSpeed { get; set; }
    public int PitchBendReaction { get; set; }
    public bool BackspinEnabled { get; set; }
    public bool BackspinAssistEnabled { get; set; }
    public int BackspinAssistStrength { get; set; }
    public double BackspinAcceleration { get; set; }
    public double BackspinSpeedIncrease { get; set; }
    public bool VinylEnabledAtStartup { get; set; }
    public bool TtVinylEnabledAtStartup { get; set; }
    public int TtPitchBendFloorPercent { get; set; }
    public double TtPitchBendGain { get; set; }
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public double? TtBrakeGain { get; set; }
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public double? TtNudgeGain { get; set; }
    public double TtBrakeCurveExponent { get; set; }
    public double TtNudgeCurveExponent { get; set; }
    public int PlatterTargetRpm { get; set; }
    public int PlatterMaxForce { get; set; }
    public int PlatterBaseForce { get; set; }
    public int PlatterForcePerRpm { get; set; }
    public int PlatterKp { get; set; }
    public int PlatterKi { get; set; }
    public int PlatterForceRisePerSecond { get; set; }
    public int PlatterStartupForceRisePerSecond { get; set; }
    public int PlatterStartupMs { get; set; }
    public int PlatterOverspeedLimitRpm { get; set; }
    [JsonIgnore] public double EffectiveBrakeGain => TtBrakeGain ?? 2 * TtPitchBendGain;
    [JsonIgnore] public double EffectiveNudgeGain => TtNudgeGain ?? TtPitchBendGain;
}

public sealed record LedPreferences
{
    public string[] DeckColors { get; set; } = [];
    public string[] StemColors { get; set; } = [];
    public string[] DjayPadColors { get; set; } = [];
    public string DjayWhite { get; set; } = "";
    public string HotcueColors { get; set; } = "";
    public string[] FixedHotcueColors { get; set; } = [];
    public int ActiveBrightness { get; set; }
    public int InactiveBrightness { get; set; }
    public string InactiveDisplay { get; set; } = "";
    public int PaletteActiveIntensity { get; set; }
    public int PaletteInactiveIntensity { get; set; }
    public bool StemMuteBright { get; set; }
    public string MuteColor { get; set; } = "";
    public string QuantizeMixed { get; set; } = "";
    public double? TempoCenterTolerance { get; set; }
    public bool LoopEnabled { get; set; }
    public string LoopColor { get; set; } = "";
    public int ChasePeriodMs { get; set; }
    public int MeterBrightness { get; set; }
    public double MeterGamma { get; set; }
    public int MeterStaleMs { get; set; }
}

public sealed record Preferences(RunPreferences Run, LedPreferences Led,
    JogPreferences Jog, TensionPreferences Tension, ScreenPreferences Screen)
{
    public Preferences Copy() => new(Run with { }, Led with
    {
        DeckColors = [.. Led.DeckColors],
        StemColors = [.. Led.StemColors],
        DjayPadColors = [.. Led.DjayPadColors],
        FixedHotcueColors = [.. Led.FixedHotcueColors]
    }, Jog with { }, Tension with { }, Screen with { });
}
