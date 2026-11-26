namespace S4Bridge.Models;

public static class ResponseCurve
{
    public static double Rate(JogPreferences jog, double measuredRpm)
    {
        var error = measuredRpm - jog.PlatterTargetRpm;
        if (Math.Abs(error) <= 2.25 + 1e-9) return 0;
        var gain = error < 0 ? jog.EffectiveBrakeGain : jog.EffectiveNudgeGain;
        var exponent = error < 0 ? jog.TtBrakeCurveExponent : jog.TtNudgeCurveExponent;
        var onset = Math.Clamp((Math.Abs(error) - 2.25), 0, 1);
        var smooth = onset * onset * (3 - 2 * onset);
        var fraction = Math.Min(Math.Abs(error) / jog.PlatterTargetRpm, 1);
        return error * smooth * gain * Math.Pow(fraction, exponent - 1) * 48;
    }
}
