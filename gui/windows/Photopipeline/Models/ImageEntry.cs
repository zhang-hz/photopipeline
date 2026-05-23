using CommunityToolkit.Mvvm.ComponentModel;
using System;

namespace Photopipeline.Models;

public sealed partial class ImageEntry : ObservableObject
{
    [ObservableProperty]
    private string _id = Guid.NewGuid().ToString("N");

    [ObservableProperty]
    private string _filePath = string.Empty;

    [ObservableProperty]
    private string _fileName = string.Empty;

    [ObservableProperty]
    private string? _thumbnailPath;

    [ObservableProperty]
    private ulong _fileSize;

    [ObservableProperty]
    private int _width;

    [ObservableProperty]
    private int _height;

    [ObservableProperty]
    private string _colorSpace = "sRGB";

    [ObservableProperty]
    private string _bitDepth = "8";

    [ObservableProperty]
    private bool _hasMetadataModified;

    [ObservableProperty]
    private ImageOverrideStatus _overrideStatus = ImageOverrideStatus.None;

    [ObservableProperty]
    private double _processingProgress;

    [ObservableProperty]
    private bool _isSelected;

    [ObservableProperty]
    private bool _isProcessing;

    [ObservableProperty]
    private bool _hasError;

    [ObservableProperty]
    private string _errorMessage = string.Empty;
}

public enum ImageOverrideStatus
{
    None,
    Original,
    Overridden,
    Error
}
