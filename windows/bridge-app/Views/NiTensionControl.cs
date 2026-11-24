using System.Globalization;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using S4Bridge.Core;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private Control NiTension()
    {
        var control = new NiTensionControl(host, Guard);
        niTensionControls.Add(control);
        return control;
    }
}

internal sealed class NiTensionControl : StackPanel
{
    private readonly HostController host;
    private readonly Slider slider;
    private readonly TextBlock readout;
    private readonly TextBlock status;
    private bool refreshing;

    public NiTensionControl(HostController host, Func<Func<Task>, Task> guard)
    {
        this.host = host;
        Spacing = 4;
        readout = UiTheme.Text("");
        status = UiTheme.Text("", true);
        var label = new Grid { ColumnDefinitions = new("*,Auto") };
        label.Children.Add(UiTheme.Text("NI jog wheel tension"));
        Grid.SetColumn(readout, 1);
        label.Children.Add(readout);
        slider = new Slider
        {
            Minimum = 0,
            Maximum = 255,
            TickFrequency = 1,
            IsSnapToTickEnabled = true
        };
        AutomationProperties.SetName(slider, "NI jog wheel tension");
        slider.PropertyChanged += async (_, e) =>
        {
            if (e.Property != RangeBase.ValueProperty || refreshing) return;
            var value = (int)slider.Value;
            readout.Text = value.ToString(CultureInfo.InvariantCulture);
            await guard(() => host.SetNiTensionAsync(value));
        };
        Children.Add(label);
        Children.Add(slider);
        Children.Add(status);
        Children.Add(UiTheme.Text("NI tension for both wheels (0-255, default 107). Changes are saved after adjustment; when NI Bridge is ready they are also sent live. This is separate from Direct HID tension.", true));
        Refresh();
    }

    public void Refresh()
    {
        refreshing = true;
        try
        {
            slider.IsEnabled = host.NiTensionEnabled;
            slider.Value = host.NiTensionValue;
            readout.Text = host.NiTensionValue.ToString(CultureInfo.InvariantCulture);
            status.Text = host.NiTensionStatus;
        }
        finally { refreshing = false; }
    }
}
