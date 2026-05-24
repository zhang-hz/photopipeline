using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.DependencyInjection;
using Photopipeline.Models;
using Photopipeline.Services;
using System.Collections.ObjectModel;

namespace Photopipeline.ViewModels;

public sealed partial class MainViewModel : ObservableObject
{
    private readonly IPipelineService _pipelineService;
    private readonly IImageService _imageService;

    [ObservableProperty]
    private ObservableCollection<ImageEntry> _images = new();

    [ObservableProperty]
    private ImageEntry? _selectedImage;

    partial void OnSelectedImageChanged(ImageEntry? value)
    {
        BeforeImage = value;
    }

    [ObservableProperty]
    private PipelineModel _currentPipeline = new() { Name = "Default Pipeline" };

    [ObservableProperty]
    private ImageEntry? _beforeImage;

    [ObservableProperty]
    private ImageEntry? _afterImage;

    [ObservableProperty]
    private double _splitPosition = 0.5;

    [ObservableProperty]
    private bool _isProcessing;

    [ObservableProperty]
    private string _statusMessage = "Ready";

    [ObservableProperty]
    private double _zoomLevel = 1.0;

    [ObservableProperty]
    private ObservableCollection<PluginInfo> _availablePlugins = new();

    [ObservableProperty]
    private PluginInfo? _selectedPlugin;

    [ObservableProperty]
    private ObservableCollection<string> _logMessages = new();

    [ObservableProperty]
    private bool _isSplitView = true;

    [ObservableProperty]
    private bool _isSideBySide;

    public PipelineEditorViewModel PipelineEditor { get; }
    public PluginPanelViewModel PluginPanel { get; }
    public BatchViewModel Batch { get; }

    public MainViewModel(
        IPipelineService pipelineService,
        IImageService imageService,
        PipelineEditorViewModel pipelineEditor,
        PluginPanelViewModel pluginPanel,
        BatchViewModel batch)
    {
        _pipelineService = pipelineService;
        _imageService = imageService;
        PipelineEditor = pipelineEditor;
        PluginPanel = pluginPanel;
        Batch = batch;

        _ = LoadPluginsAsync();
    }

    public MainViewModel() : this(
        App.Services?.GetRequiredService<IPipelineService>() ?? new LocalPipelineService(),
        App.Services?.GetRequiredService<IImageService>() ?? new LocalImageService(),
        new PipelineEditorViewModel(),
        new PluginPanelViewModel(),
        new BatchViewModel())
    { }

    [RelayCommand]
    private async Task AddImage()
    {
        StatusMessage = "Opening file picker...";
    }

    [RelayCommand]
    private void RemoveImage()
    {
        if (SelectedImage is not null)
        {
            Images.Remove(SelectedImage);
            SelectedImage = Images.Count > 0 ? Images[0] : null;
        }
    }

    [RelayCommand]
    private void ClearImages()
    {
        Images.Clear();
        SelectedImage = null;
        BeforeImage = null;
        AfterImage = null;
    }

    [RelayCommand]
    private async Task RunPipeline()
    {
        if (SelectedImage is null) return;
        CurrentPipeline.IsExecuting = true;
        StatusMessage = "Executing pipeline...";
        try
        {
            await _pipelineService.ExecutePipelineAsync(CurrentPipeline, SelectedImage.FilePath);
            StatusMessage = "Pipeline completed";
        }
        catch
        {
            StatusMessage = "Pipeline execution failed";
        }
        finally
        {
            CurrentPipeline.IsExecuting = false;
        }
    }

    [RelayCommand]
    private void StopExecution()
    {
        CurrentPipeline.IsExecuting = false;
        StatusMessage = "Stopped";
    }

    [RelayCommand]
    private void NewPipeline()
    {
        CurrentPipeline = new PipelineModel { Name = "New Pipeline" };
        StatusMessage = "Created new pipeline";
    }

    [RelayCommand]
    private void LoadPipeline()
    {
        StatusMessage = "Loading pipeline...";
    }

    [RelayCommand]
    private async Task SavePipeline()
    {
        await _pipelineService.CreatePipelineAsync(CurrentPipeline.Name, CurrentPipeline.Description);
        StatusMessage = $"Saved pipeline: {CurrentPipeline.Name}";
    }

    [RelayCommand]
    private async Task ExportImage()
    {
        if (SelectedImage is null) return;
        StatusMessage = "Exporting processed image...";
        await _imageService.ExportImageAsync(SelectedImage, "output.tif", CurrentPipeline);
        StatusMessage = "Export complete";
    }

    [RelayCommand]
    private void ZoomIn()
    {
        ZoomLevel = Math.Min(ZoomLevel * 1.25, 8.0);
    }

    [RelayCommand]
    private void ZoomOut()
    {
        ZoomLevel = Math.Max(ZoomLevel / 1.25, 0.1);
    }

    [RelayCommand]
    private void ResetZoom()
    {
        ZoomLevel = 1.0;
    }

    [RelayCommand]
    private void FitToWindow()
    {
        ZoomLevel = 1.0;
    }

    [RelayCommand]
    private void SendToPhotoshop()
    {
        StatusMessage = "Sending current image to Photoshop...";
    }

    public void Log(string message)
    {
        LogMessages.Add($"[{DateTime.Now:HH:mm:ss}] {message}");
        StatusMessage = message;
    }

    private async Task LoadPluginsAsync()
    {
        try
        {
            var plugins = await _pipelineService.GetAvailablePluginsAsync();
            PluginPanel.LoadPlugins(new ObservableCollection<PluginInfo>(plugins));
            Log("Plugins loaded");
        }
        catch (Exception ex)
        {
            Log($"Plugin load failed: {ex.Message}");
        }
    }
}
