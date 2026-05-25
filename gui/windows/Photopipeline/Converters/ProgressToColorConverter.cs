using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;

namespace Photopipeline.Converters;

[ValueConversion(typeof(double), typeof(SolidColorBrush))]
public sealed class ProgressToColorConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        double progress = value is double d ? d : 0.0;
        Brush result = progress switch
        {
            >= 1.0 => new SolidColorBrush(Color.FromRgb(107, 191, 90)),
            >= 0.5 => new SolidColorBrush(Color.FromRgb(255, 185, 0)),
            > 0.0 => new SolidColorBrush(Color.FromRgb(0, 120, 212)),
            _ => new SolidColorBrush(Color.FromRgb(102, 102, 102)),
        };
        result.Freeze();
        return result;
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
