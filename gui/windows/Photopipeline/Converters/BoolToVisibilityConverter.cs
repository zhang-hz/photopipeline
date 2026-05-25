using System.Globalization;
using System.Windows;
using System.Windows.Data;

namespace Photopipeline.Converters;

[ValueConversion(typeof(bool), typeof(Visibility))]
public sealed class BoolToVisibilityConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        bool boolValue = value is bool bv && bv;
        if (parameter is string s && s.Equals("Invert", StringComparison.OrdinalIgnoreCase))
            boolValue = !boolValue;
        return boolValue ? Visibility.Visible : Visibility.Collapsed;
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
    {
        if (value is Visibility visibility)
        {
            bool result = visibility == Visibility.Visible;
            if (parameter is string s && s.Equals("Invert", StringComparison.OrdinalIgnoreCase))
                result = !result;
            return result;
        }
        return false;
    }
}
