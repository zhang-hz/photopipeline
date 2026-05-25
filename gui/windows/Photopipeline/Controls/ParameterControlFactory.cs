using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Threading;
using Wpf.Ui.Controls;
using TextBox = System.Windows.Controls.TextBox;
using Button = System.Windows.Controls.Button;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Photopipeline.Controls;

public static class ParameterControlFactory
{
    public static FrameworkElement CreateControl(
        string parameterKey,
        Dictionary<string, object> schema,
        Dictionary<string, object> values,
        Action<string, object> onValueChanged)
    {
        schema ??= new Dictionary<string, object>();
        values ??= new Dictionary<string, object>();
        var type = GetString(schema, "type") ?? "string";
        var description = GetString(schema, "description") ?? parameterKey;

        var container = new StackPanel { Margin = new Thickness(0, 4, 0, 0) };

        var label = new TextBlock
        {
            Text = parameterKey,
            ToolTip = description,
            FontSize = 11,
            Foreground = System.Windows.Media.Brushes.Gray,
            Margin = new Thickness(0, 0, 0, 2)
        };
        container.Children.Add(label);

        var defaultValue = schema.TryGetValue("default", out var dv) ? dv : null;
        var currentValue = values.TryGetValue(parameterKey, out var cv) ? cv : defaultValue;

        FrameworkElement control = type switch
        {
            "bool" => CreateToggleSwitch(parameterKey, currentValue, values, onValueChanged),
            "int" => CreateNumericInput(parameterKey, schema, currentValue, values, onValueChanged, isInteger: true),
            "float" => CreateNumericInput(parameterKey, schema, currentValue, values, onValueChanged, isInteger: false),
            "enum" => CreateComboBox(schema, currentValue, values, parameterKey, onValueChanged),
            "path" => CreatePathPicker(parameterKey, currentValue, values, onValueChanged),
            _ => CreateStringInput(parameterKey, currentValue, values, onValueChanged)
        };

        container.Children.Add(control);
        return container;
    }

    private static ToggleSwitch CreateToggleSwitch(
        string key, object? currentValue, Dictionary<string, object> values,
        Action<string, object> onValueChanged)
    {
        var toggle = new ToggleSwitch { Margin = new Thickness(0) };
        if (currentValue is bool b) toggle.IsChecked = b;

        toggle.Checked += (_, _) => UpdateValue(key, true, values, onValueChanged);
        toggle.Unchecked += (_, _) => UpdateValue(key, false, values, onValueChanged);

        return toggle;
    }

    private static FrameworkElement CreateNumericInput(
        string key, Dictionary<string, object> schema, object? currentValue,
        Dictionary<string, object> values, Action<string, object> onValueChanged, bool isInteger)
    {
        var panel = new StackPanel { Orientation = Orientation.Horizontal };

        var textBox = new TextBox { Width = 100, Margin = new Thickness(0), VerticalContentAlignment = VerticalAlignment.Center };

        TryGetDouble(currentValue, out var cur);
        textBox.Text = isInteger ? ((int)cur).ToString() : cur.ToString("F2");

        if (TryGetDouble(schema, "step", out var step) && step > 0)
        {
            var downBtn = new Button { Content = "-", Width = 22, Height = 22, Padding = new Thickness(0), FontSize = 14 };
            var upBtn = new Button { Content = "+", Width = 22, Height = 22, Padding = new Thickness(0), FontSize = 14 };

            downBtn.Click += (_, _) =>
            {
                if (double.TryParse(textBox.Text, out var v))
                {
                    v = Math.Max(v - step, TryGetDouble(schema, "min", out var mn) ? mn : double.MinValue);
                    textBox.Text = isInteger ? ((int)v).ToString() : v.ToString("F2");
                    UpdateValueFromText(key, textBox.Text, values, onValueChanged, isInteger);
                }
            };
            upBtn.Click += (_, _) =>
            {
                if (double.TryParse(textBox.Text, out var v))
                {
                    v = Math.Min(v + step, TryGetDouble(schema, "max", out var mx) ? mx : double.MaxValue);
                    textBox.Text = isInteger ? ((int)v).ToString() : v.ToString("F2");
                    UpdateValueFromText(key, textBox.Text, values, onValueChanged, isInteger);
                }
            };

            panel.Children.Add(downBtn);
            panel.Children.Add(textBox);
            panel.Children.Add(upBtn);
        }
        else
        {
            panel.Children.Add(textBox);

            // Unit label
            if (schema.TryGetValue("unit", out var unit) && unit is string unitStr && unitStr.Length > 0)
            {
                var unitLabel = new TextBlock
                {
                    Text = unitStr,
                    FontSize = 11,
                    VerticalAlignment = VerticalAlignment.Center,
                    Margin = new Thickness(4, 0, 0, 0),
                    Foreground = System.Windows.Media.Brushes.Gray
                };
                panel.Children.Add(unitLabel);
            }
        }

        textBox.LostFocus += (_, _) =>
            UpdateValueFromText(key, textBox.Text, values, onValueChanged, isInteger);

        return panel;
    }

    private static ComboBox CreateComboBox(
        Dictionary<string, object> schema, object? currentValue,
        Dictionary<string, object> values, string key, Action<string, object> onValueChanged)
    {
        var combo = new ComboBox
        {
            Margin = new Thickness(0),
            Width = 180,
            HorizontalAlignment = HorizontalAlignment.Left
        };

        if (schema.TryGetValue("values", out var vals) && vals is System.Collections.IEnumerable items)
        {
            foreach (var item in items)
                combo.Items.Add(item?.ToString() ?? string.Empty);
        }

        if (currentValue is string sv && combo.Items.Contains(sv))
            combo.SelectedItem = sv;
        else if (combo.Items.Count > 0)
            combo.SelectedIndex = 0;

        combo.SelectionChanged += (_, _) =>
        {
            if (combo.SelectedItem is string selected)
                UpdateValue(key, selected, values, onValueChanged);
        };

        return combo;
    }

    private static FrameworkElement CreateStringInput(
        string key, object? currentValue, Dictionary<string, object> values,
        Action<string, object> onValueChanged)
    {
        var textBox = new TextBox
        {
            Margin = new Thickness(0),
            Width = 220,
            HorizontalAlignment = HorizontalAlignment.Left
        };
        if (currentValue != null) textBox.Text = currentValue.ToString();

        // Use LostFocus instead of TextChanged to avoid firing on every keystroke
        textBox.LostFocus += (_, _) =>
            UpdateValue(key, textBox.Text ?? string.Empty, values, onValueChanged);

        return textBox;
    }

    private static FrameworkElement CreatePathPicker(
        string key, object? currentValue, Dictionary<string, object> values,
        Action<string, object> onValueChanged)
    {
        var panel = new StackPanel { Orientation = Orientation.Horizontal };

        var textBox = new TextBox { Width = 200, Margin = new Thickness(0, 0, 4, 0) };
        if (currentValue is string path) textBox.Text = path;

        // Path picker: commit on LostFocus to avoid firing on every keystroke
        textBox.LostFocus += (_, _) =>
            UpdateValue(key, textBox.Text ?? string.Empty, values, onValueChanged);

        var browseBtn = new Button { Content = "...", Width = 32, Padding = new Thickness(0) };
        browseBtn.Click += (_, _) =>
        {
            var dialog = new Microsoft.Win32.OpenFileDialog
            {
                Title = $"Select {key}",
                Filter = "All files|*.*"
            };
            if (dialog.ShowDialog() == true)
            {
                textBox.Text = dialog.FileName;
                UpdateValue(key, dialog.FileName, values, onValueChanged);
            }
        };

        panel.Children.Add(textBox);
        panel.Children.Add(browseBtn);
        return panel;
    }

    private static void UpdateValueFromText(
        string key, string text, Dictionary<string, object> values,
        Action<string, object> onValueChanged, bool isInteger)
    {
        if (double.TryParse(text, out var d))
        {
            var val = isInteger ? (object)(int)d : d;
            UpdateValue(key, val, values, onValueChanged);
        }
    }

    private static void UpdateValue(
        string key, object value, Dictionary<string, object> values,
        Action<string, object> onValueChanged)
    {
        values[key] = value;
        onValueChanged(key, value);
    }

    // ── Helpers ──

    private static string? GetString(Dictionary<string, object> dict, string key)
    {
        if (dict.TryGetValue(key, out var val))
            return val?.ToString();
        return null;
    }

    private static bool TryGetDouble(object? value, out double result)
    {
        if (value == null) { result = 0; return false; }
        try { result = Convert.ToDouble(value); return true; }
        catch { result = 0; return false; }
    }

    private static bool TryGetDouble(Dictionary<string, object> dict, string key, out double result)
    {
        if (dict.TryGetValue(key, out var val))
            return TryGetDouble(val, out result);
        result = 0;
        return false;
    }
}
