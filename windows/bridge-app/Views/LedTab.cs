using Avalonia.Controls;
using S4Bridge.Models;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private Control LedTab()
    {
        var led = host.Store.Editor.Led;
        Control N(string label, string property, double min, double max, bool integer = true)
        {
            var info = typeof(LedPreferences).GetProperty(property)!;
            return fields.Number(label, () => Convert.ToDouble(info.GetValue(led)), value =>
                info.SetValue(led, info.PropertyType == typeof(int) ? (object)checked((int)value!) : value!), min, max, integer);
        }
        ledNotice = UiTheme.Text("LED preferences apply only to the Full profile. Minimal and Probe keep their original light behavior.", true);
        return UiTheme.BodyView(ledNotice,
            UiTheme.SectionView("Brightness and inactive lights",
                N("Active lamp brightness", nameof(led.ActiveBrightness), 1, 127),
                N("Inactive lamp brightness", nameof(led.InactiveBrightness), 0, 127),
                fields.Choice("Inactive display", led.InactiveDisplay, ["dim", "dark"], v => led.InactiveDisplay = v),
                N("Active palette intensity", nameof(led.PaletteActiveIntensity), 0, 3),
                N("Inactive palette intensity", nameof(led.PaletteInactiveIntensity), 0, 3),
                fields.Choice("Mixed quantize state", led.QuantizeMixed, ["dim", "dark"], v => led.QuantizeMixed = v)),
            host.SelectedMode == BridgeMode.NiBridge
                ? UiTheme.SectionView("Jog rings",
                    UiTheme.Text("NI manages ring motion. Deck colors and active palette intensity still apply.", true))
                : UiTheme.SectionView("Jog rings",
                N("Playback revolution (ms)", nameof(led.ChasePeriodMs), 250, 10000),
                fields.Toggle("Flash whole ring during loops", led.LoopEnabled, v => led.LoopEnabled = v),
                fields.Choice("Loop color", led.LoopColor, PreferenceCodec.Colors, v => led.LoopColor = v),
                UiTheme.Text("Normal-mode rings illustrate playback, scratch and pause; TT follows physical movement. No absolute track position or end warning. Firmware controls loop flash cadence.", true)),
            UiTheme.SectionView("Channel meters",
                fields.Choice("Meter brightness", led.MeterBrightness.ToString(), ["0", "125", "127"], v => led.MeterBrightness = int.Parse(v)),
                N("Level curve", nameof(led.MeterGamma), 0.1, 5, false),
                N("Level freshness (ms)", nameof(led.MeterStaleMs), 100, 5000),
                UiTheme.Text("0 off, 125 dim, 127 bright. Master meters remain hardware-driven.", true)),
            UiTheme.SectionView("Tempo-center indicator",
                fields.Number("Center tolerance (blank disables)", () => led.TempoCenterTolerance, v => led.TempoCenterTolerance = v,
                    double.Epsilon, 0.1, false, true),
                UiTheme.Text("Leave disabled unless the physical detent has been calibrated. This is not software pickup.", true)),
            UiTheme.Actions(Action("Import LED JSON...", ImportLedAsync), Action("Export LED JSON...", ExportLedAsync)));
    }
    private Control PaletteTab()
    {
        var led = host.Store.Editor.Led;
        Control Colors(string title, string[] values, string[] labels)
        {
            var controls = new List<Control>();
            for (var i = 0; i < values.Length; i++)
            {
                var slot = i;
                controls.Add(fields.Choice(labels[i], values[i], PreferenceCodec.Colors, v => values[slot] = v));
            }
            return UiTheme.SectionView(title, [.. controls]);
        }
        return UiTheme.BodyView(
            Colors("Deck colors", led.DeckColors, ["Deck A", "Deck B", "Deck C", "Deck D"]),
            UiTheme.SectionView("Hotcue presentation",
                fields.Choice("Hotcue colors", led.HotcueColors, ["djay", "fixed_slots"], v => led.HotcueColors = v),
                fields.Choice("Djay white token", led.DjayWhite, PreferenceCodec.Colors, v => led.DjayWhite = v),
                UiTheme.Text("Use Djay colors to follow each hotcue's color, or choose fixed pad colors below. Unrecognized colors stay dark.", true)),
            Colors("Djay color-token correspondence", led.DjayPadColors, Enumerable.Range(1, 8).Select(i => $"Token {i}").ToArray()),
            Colors("Optional fixed hotcue slots", led.FixedHotcueColors, Enumerable.Range(1, 8).Select(i => $"Pad {i}").ToArray()),
            Colors("Stem colors", led.StemColors, ["Drums", "Bass", "Harmonic", "Vocals"]),
            UiTheme.SectionView("Stem mute display",
                fields.Toggle("Bright means muted", led.StemMuteBright, v => led.StemMuteBright = v),
                fields.Choice("Engaged mute color", led.MuteColor, PreferenceCodec.Colors, v => led.MuteColor = v),
                UiTheme.Text("Mute lights show the mute switch. Another stem's Solo setting can still affect what you hear. Palette changes save with LED preferences.", true)));
    }
}
