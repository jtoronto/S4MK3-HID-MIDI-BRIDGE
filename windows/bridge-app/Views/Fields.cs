using System.Globalization;
using Avalonia;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;

namespace S4Bridge.Views;

internal sealed class Fields(Action edited, Action<string, string?>? reportFieldError = null)
{
    private readonly Dictionary<string, string> errors = [];
    public bool Valid => errors.Count == 0;
    public string Error => string.Join("\n", errors.Values);
    public Control Number(string title, Func<double?> get, Action<double?> set,
        double min, double max, bool integer = false, bool optional = false)
    {
        var label = UiTheme.Text(title);
        var input = new TextBox
        {
            Text = get()?.ToString("G", CultureInfo.InvariantCulture) ?? "",
            Width = UiTheme.FieldWidth,
            HorizontalAlignment = HorizontalAlignment.Right
        };
        AutomationProperties.SetName(input, title);
        var error = UiTheme.Text("", true);
        error.Bind(TextBlock.ForegroundProperty, error.GetResourceObservable(UiTheme.Error));
        var grid = new Grid { ColumnDefinitions = new("*,Auto"), ColumnSpacing = UiTheme.Related };
        grid.Children.Add(label); Grid.SetColumn(input, 1); grid.Children.Add(input);
        void Update()
        {
            var text = input.Text ?? "";
            var valid = double.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out var value)
                && double.IsFinite(value) && value >= min && value <= max
                && (!integer || (text.TrimStart('-').All(char.IsAsciiDigit)
                                 && text.Length > 0 && value == Math.Truncate(value)));
            if (optional && string.IsNullOrWhiteSpace(text))
            {
                set(null); errors.Remove(title); reportFieldError?.Invoke(title, null);
                error.Text = ""; edited(); return;
            }
            if (valid)
            {
                set(value); errors.Remove(title); reportFieldError?.Invoke(title, null);
                error.Text = "";
            }
            else
            {
                var message = $"{title}: enter {(integer ? "an integer" : "a number")} from {min} to {max}{(optional ? ", or blank to inherit" : "")}.";
                errors[title] = message; reportFieldError?.Invoke(title, message);
                error.Text = message;
            }
            edited();
        }
        input.TextChanged += (_, _) => Update();
        return UiTheme.Stack(grid, error);
    }
    public Control Toggle(string title, bool value, Action<bool> set)
    {
        var checkbox = new CheckBox { Content = title, IsChecked = value };
        checkbox.IsCheckedChanged += (_, _) => { set(checkbox.IsChecked == true); edited(); };
        return checkbox;
    }
    public Control Choice(string title, string value, string[] values, Action<string> set)
    {
        var combo = new ComboBox
        {
            ItemsSource = values,
            SelectedItem = value,
            HorizontalAlignment = HorizontalAlignment.Stretch
        };
        AutomationProperties.SetName(combo, title);
        combo.SelectionChanged += (_, _) => { if (combo.SelectedItem is string item) { set(item); edited(); } };
        var panel = UiTheme.Stack(UiTheme.Text(title), combo);
        panel.PropertyChanged += (_, e) => { if (e.Property == InputElement.IsEnabledProperty) combo.IsEnabled = panel.IsEnabled; };
        return panel;
    }
    public Control Slider(string title, double value, Action<double> set,
        double min, double max, double step)
    {
        var readout = UiTheme.Text(value.ToString("G", CultureInfo.InvariantCulture));
        var label = new Grid { ColumnDefinitions = new("*,Auto") };
        label.Children.Add(UiTheme.Text(title)); Grid.SetColumn(readout, 1); label.Children.Add(readout);
        var slider = new Slider
        {
            Minimum = min,
            Maximum = max,
            Value = value,
            TickFrequency = step,
            IsSnapToTickEnabled = true
        };
        AutomationProperties.SetName(slider, title);
        slider.PropertyChanged += (_, e) =>
        {
            if (e.Property != Avalonia.Controls.Primitives.RangeBase.ValueProperty) return;
            set(slider.Value); readout.Text = slider.Value.ToString("G", CultureInfo.InvariantCulture); edited();
        };
        var panel = UiTheme.Stack(label, slider);
        panel.PropertyChanged += (_, e) => { if (e.Property == InputElement.IsEnabledProperty) slider.IsEnabled = panel.IsEnabled; };
        return panel;
    }
}
