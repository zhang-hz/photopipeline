using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Services;

namespace Photopipeline.ViewModels;

public sealed partial class MainViewModel : ViewModelBase
{
    private readonly ISettingsService _settings;
    private readonly IBackendService _backend;

    // ── Child ViewModels ──
    public FilmstripViewModel Filmstrip { get; }
    public PreviewViewModel Preview { get; }
    public PipelineEditorViewModel PipelineEditor { get; }
    public PluginBrowserViewModel PluginBrowser { get; }
    public BatchViewModel Batch { get; }
    public SettingsViewModel Settings { get; }

    // ── Window state ──
    [ObservableProperty] private string _windowTitle = "Photopipeline";
    [ObservableProperty] private double _windowWidth = 1440;
    [ObservableProperty] private double _windowHeight = 900;
    [ObservableProperty] private bool _isBackendHealthy;
    [ObservableProperty] private string _backendStatus = "Unknown";

    // ── Mediator events ──
    public event Action<Models.ImageEntry?>? ImageSelected;

    public MainViewModel(
        ILogger<MainViewModel> logger,
        ISettingsService settings,
        IBackendService backend,
        FilmstripViewModel filmstrip,
        PreviewViewModel preview,
        PipelineEditorViewModel pipelineEditor,
        PluginBrowserViewModel pluginBrowser,
        BatchViewModel batch,
        SettingsViewModel settingsVm) : base(logger)
    {
        _settings = settings;
        _backend = backend;

        Filmstrip = filmstrip;
        Preview = preview;
        PipelineEditor = pipelineEditor;
        PluginBrowser = pluginBrowser;
        Batch = batch;
        Settings = settingsVm;

        WindowWidth = settings.Current.WindowWidth;
        WindowHeight = settings.Current.WindowHeight;

        _backend.HealthChanged += OnBackendHealthChanged;
        IsBackendHealthy = _backend.IsHealthy;
        BackendStatus = IsBackendHealthy ? "Connected" : "Disconnected";

        Filmstrip.ImageSelected += OnImageSelected;
        Filmstrip.SendToBatchRequested += OnSendToBatch;
    }

    [RelayCommand]
    private void ZoomIn() => Preview?.ZoomInCommand.Execute(null);

    [RelayCommand]
    private void ZoomOut() => Preview?.ZoomOutCommand.Execute(null);

    [RelayCommand]
    private void ResetZoom() => Preview?.ResetZoomCommand.Execute(null);

    [RelayCommand]
    private void ShowSettings()
    {
        var dialog = App.Services.GetRequiredService<Views.SettingsDialog>();
        dialog.Owner = System.Windows.Application.Current.MainWindow;
        dialog.ShowDialog();
    }

    private void OnBackendHealthChanged(object? sender, bool healthy)
    {
        IsBackendHealthy = healthy;
        BackendStatus = healthy ? "Connected" : "Disconnected";
    }

    private void OnSendToBatch(IReadOnlyList<Models.ImageEntry> images)
    {
        foreach (var img in images)
            Batch.AddToQueueCommand.Execute(img);
    }

    public override void Shutdown()
    {
        base.Shutdown();
        Filmstrip.Shutdown();
        Preview.Shutdown();
        PipelineEditor.Shutdown();
        PluginBrowser.Shutdown();
        Batch.Shutdown();
        Settings.Shutdown();
    }

    private async void OnImageSelected(Models.ImageEntry? image)
    {
        StatusMessage = image is not null
            ? $"Selected: {image.FileName} ({image.Width}x{image.Height})"
            : "Ready";
        ImageSelected?.Invoke(image);

        if (image is null) return;

        try
        {
            await Preview.LoadImageAsync(image);

            if (PipelineEditor.IsPipelineValid)
                await Preview.ProcessPreviewAsync(image, PipelineEditor.CurrentPipeline, PipelineEditor.PipelineId);
        }
        catch (OperationCanceledException) { }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Preview update failed for {Path}", image.FilePath);
        }
    }
}
