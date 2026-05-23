using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Photopipeline.Models;
using Photopipeline.Services;
using System.Collections.ObjectModel;

namespace Photopipeline.ViewModels;

public sealed partial class MainViewModel : ObservableObject
{
    [ObservableProperty]
    private ObservableCollection<ImageEntry> _images = new();

    [ObservableProperty]
    private ImageEntry? _selectedImage;

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

    public PipelineEditorViewModel PipelineEditor { get; } = new();
    public PluginPanelViewModel PluginPanel { get; } = new();
    public BatchViewModel Batch { get; } = new();

    [RelayCommand]
    private void AddImage()
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
    private void RunPipeline()
    {
        CurrentPipeline.IsExecuting = true;
        StatusMessage = "Executing pipeline...";
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
    private void SavePipeline()
    {
        StatusMessage = $"Saved pipeline: {CurrentPipeline.Name}";
    }

    [RelayCommand]
    private void ExportImage()
    {
        StatusMessage = "Exporting processed image...";
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
}
