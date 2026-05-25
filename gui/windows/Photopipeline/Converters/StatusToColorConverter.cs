using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;

namespace Photopipeline.Converters;

[ValueConversion(typeof(string), typeof(SolidColorBrush))]
public sealed class StatusToColorConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        string status = value?.ToString()?.ToLowerInvariant() ?? "idle";
        Color color = status switch
        {
            "running" or "processing" => Color.FromRgb(0, 120, 212),
            "paused" or "pending" => Color.FromRgb(255, 185, 0),
            "completed" or "done" or "success" => Color.FromRgb(107, 191, 90),
            "failed" or "error" => Color.FromRgb(231, 72, 86),
            "connected" or "healthy" => Color.FromRgb(107, 191, 90),
            "disconnected" or "unavailable" => Color.FromRgb(231, 72, 86),
            _ => Color.FromRgb(102, 102, 102),
        };
        var brush = new SolidColorBrush(color);
        brush.Freeze();
        return brush;
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
