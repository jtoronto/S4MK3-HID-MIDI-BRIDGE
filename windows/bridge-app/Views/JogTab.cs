using Avalonia.Controls;
using S4Bridge.Models;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private Control JogTab()
    {
        var jog = host.Store.Editor.Jog;
        response = new ResponsePreview(() => jog);
        Control N(string label, string property, double min, double max, bool integer = false, bool optional = false)
        {
            var info = typeof(JogPreferences).GetProperty(property)!;
            return fields.Number(label, () => info.GetValue(jog) is { } value ? Convert.ToDouble(value) : null,
                value => info.SetValue(jog, value is null ? null : info.PropertyType == typeof(int) ?
                    (object)checked((int)value.Value) : value.Value), min, max, integer, optional);
        }
        var advanced = new Expander
        {
            Header = "Advanced settings",
            IsExpanded = false,
            Content = UiTheme.Stack(
                UiTheme.SectionView("Playback and touch",
                    N("Target speed (RPM)", nameof(jog.PlatterTargetRpm), 1, 45, true),
                    fields.Toggle("Start Normal mode with Vinyl on", jog.VinylEnabledAtStartup, v => jog.VinylEnabledAtStartup = v),
                    fields.Toggle("Start TT mode with Vinyl on", jog.TtVinylEnabledAtStartup, v => jog.TtVinylEnabledAtStartup = v),
                    N("Switch to Scratch below (% target)", nameof(jog.TtPitchBendFloorPercent), 1, 95, true)),
                UiTheme.SectionView("Motor response",
                    N("Maximum force", nameof(jog.PlatterMaxForce), 1, 25000, true),
                    N("Speed-based force (units/RPM)", nameof(jog.PlatterForcePerRpm), 0, 25000, true),
                    N("Speed correction (Kp)", nameof(jog.PlatterKp), 0, 25000, true),
                    N("Sustained speed correction (Ki)", nameof(jog.PlatterKi), 0, 25000, true),
                    N("Running force rise (units/s)", nameof(jog.PlatterForceRisePerSecond), 0, 1000000, true),
                    N("Startup duration (ms)", nameof(jog.PlatterStartupMs), 0, 2000, true),
                    N("Startup force rise (units/s)", nameof(jog.PlatterStartupForceRisePerSecond), 0, 1000000, true),
                    N("Overspeed limit (RPM, 0 off)", nameof(jog.PlatterOverspeedLimitRpm), 0, 250, true),
                    N("Legacy shared TT gain", nameof(jog.TtPitchBendGain), 0.1, 10)))
        };
        var assistStrength = fields.Slider("Motor assist strength (%)", jog.BackspinAssistStrength, v => jog.BackspinAssistStrength = (int)v, 0, 100, 1);
        assistStrength.IsEnabled = jog.BackspinEnabled && jog.BackspinAssistEnabled;
        var assistToggle = fields.Toggle("Use motor assistance for backspins", jog.BackspinAssistEnabled, v =>
        {
            jog.BackspinAssistEnabled = v;
            assistStrength.IsEnabled = jog.BackspinEnabled && v;
        });
        var backspinControls = UiTheme.Stack(
            N("Detection threshold (counts/s squared)", nameof(jog.BackspinAcceleration), double.Epsilon, double.MaxValue),
            fields.Slider("Extra scratch speed (%)", jog.BackspinSpeedIncrease, v => jog.BackspinSpeedIncrease = v, 0, 10, 0.1),
            assistToggle,
            assistStrength);
        backspinControls.IsEnabled = jog.BackspinEnabled;
        var detection = fields.Toggle("Continue detected backspin after release", jog.BackspinEnabled, v =>
        { jog.BackspinEnabled = v; backspinControls.IsEnabled = v; assistStrength.IsEnabled = v && jog.BackspinAssistEnabled; });
        return UiTheme.BodyView(
            UiTheme.SectionView("Motorized platters",
                UiTheme.Text("Experimental. Save and restart applies motor settings; no mapping export is required. Turn TT off or Stop bridge before closing Djay.", true),
                Action("Restore motorized defaults", () =>
                { host.Store.ResetMotorized(); host.Edited(); BuildTabs(); return Task.CompletedTask; }),
                fields.Slider("Base force (units)", jog.PlatterBaseForce, v => jog.PlatterBaseForce = (int)v, 0, 25000, 50),
                UiTheme.Text("Weak to Strong. Baseline drive plus speed-based drive and corrections; force units are not calibrated torque.", true),
                UiTheme.SectionView("Braking response",
                    N("Braking strength (x, blank inherits)", nameof(jog.TtBrakeGain), 0.1, 20, false, true),
                    UiTheme.Text($"Blank braking gain inherits twice the shared gain (currently {jog.EffectiveBrakeGain:G}x).", true),
                    N("Braking shape", nameof(jog.TtBrakeCurveExponent), 0.25, 4)),
                UiTheme.SectionView("Nudge response",
                    N("Nudge strength (x, blank inherits)", nameof(jog.TtNudgeGain), 0.1, 10, false, true),
                    N("Nudge shape", nameof(jog.TtNudgeCurveExponent), 0.25, 4),
                    UiTheme.Text("Lower shape emphasizes small movements; higher shape softens them. Quiet within 2.25 RPM. Preview is MIDI count rate, not audible playback speed.", true),
                    response),
                advanced),
            UiTheme.SectionView("Jog wheel feel",
                fields.Slider("Wheel tension", host.Store.Editor.Tension.Tension,
                    v => host.Store.Editor.Tension.Tension = (int)v, -100, 100, 1),
                UiTheme.Text("Assist (-100)       Neutral (0)       Brake (100). Shared by both physical wheels, not saved per deck.", true)),
            UiTheme.SectionView("Backspin", detection, backspinControls,
                UiTheme.Text("Normal-mode assist and TT playback assistance are separate. Threshold and added speed affect runtime gestures, not ordinary mapping sensitivity.", true)),
            UiTheme.SectionView("Scratch and pitch bend mapping",
                N("Scratch speed (%)", nameof(jog.ScratchSpeed), double.Epsilon, double.MaxValue),
                N("Scratch reaction (%)", nameof(jog.ScratchReaction), 0, 150, true),
                fields.Choice("Touch-scratch direction", jog.ScratchDirection, ["normal", "reversed"], v => jog.ScratchDirection = v),
                N("Pitch bend speed (%)", nameof(jog.PitchBendSpeed), double.Epsilon, double.MaxValue),
                N("Pitch bend reaction (%)", nameof(jog.PitchBendReaction), 0, 150, true),
                Action("Install Djay mapping...", () => ExportMappingAsync(true)),
                UiTheme.Text("Unsaved editor values are exported. Install and select the new mapping in Djay; saving or restarting alone does not apply these mapping values.", true)));
    }
}
