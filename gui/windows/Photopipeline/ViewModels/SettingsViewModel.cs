using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Services;

namespace Photopipeline.ViewModels;

public sealed partial class SettingsViewModel : ViewModelBase
{
    private readonly ISettingsService _settings;

    [ObservableProperty] private string _theme = "Dark";
    [ObservableProperty] private string _serverPath = "photopipeline-server.exe";
    [ObservableProperty] private int _serverPort = 50051;
    [ObservableProperty] private bool _autoStartServer = true;
    [ObservableProperty] private string _defaultOutputFormat = "TIFF";
    [ObservableProperty] private string _defaultOutputDirectory = string.Empty;
    [ObservableProperty] private int _jpegQuality = 95;
    [ObservableProperty] private bool _embedMetadata = true;
    [ObservableProperty] private int _thumbnailSize = 256;
    [ObservableProperty] private int _maxRecentFiles = 10;

    public IReadOnlyList<string> Themes { get; } = new[] { "Dark", "Light" };
    public IReadOnlyList<string> OutputFormats { get; } = new[] { "TIFF", "JPEG", "PNG", "WebP", "HEIF", "AVIF", "JPEG XL" };

    public SettingsViewModel(ILogger<SettingsViewModel> logger, ISettingsService settings) : base(logger)
    {
        _settings = settings;
        LoadFromCurrent();
    }

    [RelayCommand]
    private async Task Save(CancellationToken ct)
    {
        await ExecuteAsync(async ct2 =>
        {
            var current = _settings.Current;
            current.Theme = Theme;
            current.ServerPath = ServerPath;
            current.ServerPort = ServerPort;
            current.AutoStartServer = AutoStartServer;
            current.DefaultOutputFormat = DefaultOutputFormat;
            current.DefaultOutputDirectory = DefaultOutputDirectory;
            current.JpegQuality = JpegQuality;
            current.EmbedMetadata = EmbedMetadata;
            current.ThumbnailSize = ThumbnailSize;
            current.MaxRecentFiles = MaxRecentFiles;
            await _settings.SaveAsync(current, ct2);
            StatusMessage = "Settings saved";
        }, "Save settings", ct);
    }

    [RelayCommand]
    private async Task Reset(CancellationToken ct)
    {
        await _settings.ResetAsync(ct);
        LoadFromCurrent();
        StatusMessage = "Settings reset to defaults";
    }

    [RelayCommand]
    private void BrowseServerPath()
    {
        var dialog = new Microsoft.Win32.OpenFileDialog
        {
            Title = "Select Backend Server",
            Filter = "Executable|*.exe|All files|*.*"
        };
        if (dialog.ShowDialog() == true)
            ServerPath = dialog.FileName;
    }

    [RelayCommand]
    private void BrowseOutputDirectory()
    {
        var dialog = new Microsoft.Win32.OpenFolderDialog
        {
            Title = "Select default output directory"
        };
        if (dialog.ShowDialog() == true)
            DefaultOutputDirectory = dialog.FolderName;
    }

    public void LoadFrom(Models.AppSettings s)
    {
        Theme = s.Theme;
        ServerPath = s.ServerPath;
        ServerPort = s.ServerPort;
        AutoStartServer = s.AutoStartServer;
        DefaultOutputFormat = s.DefaultOutputFormat;
        DefaultOutputDirectory = s.DefaultOutputDirectory;
        JpegQuality = s.JpegQuality;
        EmbedMetadata = s.EmbedMetadata;
        ThumbnailSize = s.ThumbnailSize;
        MaxRecentFiles = s.MaxRecentFiles;
    }

    private void LoadFromCurrent()
    {
        LoadFrom(_settings.Current);
    }
}
