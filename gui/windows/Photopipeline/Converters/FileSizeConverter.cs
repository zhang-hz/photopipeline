using System.Globalization;
using System.Windows.Data;

namespace Photopipeline.Converters;

[ValueConversion(typeof(long), typeof(string))]
public sealed class FileSizeConverter : IValueConverter
{
    private static readonly string[] Suffixes = { "B", "KB", "MB", "GB", "TB" };

    public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        long bytes = value switch
        {
            ulong u => (long)u,
            long l => l,
            int i => i,
            _ => 0L
        };

        if (bytes == 0) return "0 B";
        if (bytes < 0) return "--";

        int order = 0;
        double size = bytes;
        while (size >= 1024 && order < Suffixes.Length - 1)
        {
            order++;
            size /= 1024;
        }

        return $"{size:0.##} {Suffixes[order]}";
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
