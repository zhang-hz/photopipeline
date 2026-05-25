using CommunityToolkit.Mvvm.ComponentModel;

namespace Photopipeline.Models;

public sealed partial class ImageEntry : ObservableObject
{
    [ObservableProperty]
    private string _filePath = string.Empty;

    [ObservableProperty]
    private string _fileName = string.Empty;

    [ObservableProperty]
    private string _format = string.Empty;

    [ObservableProperty]
    private uint _width;

    [ObservableProperty]
    private uint _height;

    [ObservableProperty]
    private ulong _fileSizeBytes;

    [ObservableProperty]
    private string _pixelFormat = string.Empty;

    [ObservableProperty]
    private string _colorSpace = string.Empty;

    [ObservableProperty]
    private byte[]? _thumbnailData;

    [ObservableProperty]
    private ImageMetadata? _metadata;

    [ObservableProperty]
    private ImageStatus _status = ImageStatus.None;

    [ObservableProperty]
    private string _statusMessage = string.Empty;

    public static ImageEntry FromImageInfo(ImageInfo info)
    {
        return new ImageEntry
        {
            FilePath = info.Path,
            FileName = info.FileName,
            Format = info.Format,
            Width = info.Width,
            Height = info.Height,
            FileSizeBytes = info.FileSizeBytes,
            PixelFormat = info.PixelFormat,
            ColorSpace = info.ColorSpace,
            Metadata = info.Metadata,
        };
    }
}

public enum ImageStatus
{
    None,
    Original,
    Overridden,
    Error
}
