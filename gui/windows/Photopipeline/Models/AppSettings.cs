using CommunityToolkit.Mvvm.ComponentModel;

namespace Photopipeline.Models;

public sealed partial class AppSettings : ObservableObject
{
    [ObservableProperty]
    private string _serverPath = "photopipeline-server.exe";

    [ObservableProperty]
    private int _serverPort = 50051;

    [ObservableProperty]
    private bool _autoStartServer = true;

    [ObservableProperty]
    private string _theme = "Dark";

    [ObservableProperty]
    private int _thumbnailSize = 256;

    [ObservableProperty]
    private int _maxRecentFiles = 10;

    [ObservableProperty]
    private string _defaultOutputFormat = "TIFF";

    [ObservableProperty]
    private string _defaultOutputDirectory = string.Empty;

    [ObservableProperty]
    private int _jpegQuality = 95;

    [ObservableProperty]
    private bool _embedMetadata = true;

    [ObservableProperty]
    private double _windowWidth = 1440;

    [ObservableProperty]
    private double _windowHeight = 900;

    [ObservableProperty]
    private double _windowLeft = double.NaN;

    [ObservableProperty]
    private double _windowTop = double.NaN;

    [ObservableProperty]
    private bool _isMaximized;
}
