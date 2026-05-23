using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Photopipeline.Models;
using System.Collections.ObjectModel;

namespace Photopipeline.ViewModels;

public sealed partial class BatchViewModel : ObservableObject
{
    [ObservableProperty]
    private ObservableCollection<ImageEntry> _batchQueue = new();

    [ObservableProperty]
    private int _totalItems;

    [ObservableProperty]
    private int _completedItems;

    [ObservableProperty]
    private int _failedItems;

    [ObservableProperty]
    private double _overallProgress;

    [ObservableProperty]
    private bool _isRunning;

    [ObservableProperty]
    private bool _isPaused;

    [ObservableProperty]
    private string _statusText = "Idle";

    [ObservableProperty]
    private string _elapsedTime = "00:00:00";

    [ObservableProperty]
    private string _estimatedRemaining = "--:--:--";

    [ObservableProperty]
    private ObservableCollection<string> _outputFormats = new() { "TIFF", "JPEG", "PNG", "WebP", "HEIF" };

    [ObservableProperty]
    private string _selectedOutputFormat = "TIFF";

    [ObservableProperty]
    private string _outputDirectory = string.Empty;

    [ObservableProperty]
    private int _jpegQuality = 95;

    [ObservableProperty]
    private bool _embedMetadata = true;

    private System.Timers.Timer? _timer;
    private DateTime _startTime;
    private int _workerCount = Environment.ProcessorCount - 1;

    [RelayCommand]
    private void StartBatch()
    {
        if (BatchQueue.Count == 0) return;
        IsRunning = true;
        IsPaused = false;
        CompletedItems = 0;
        FailedItems = 0;
        TotalItems = BatchQueue.Count;
        OverallProgress = 0;
        _startTime = DateTime.Now;
        StartTimer();
        StatusText = "Processing...";
    }

    [RelayCommand]
    private void PauseBatch()
    {
        IsPaused = true;
        StatusText = "Paused";
        StopTimer();
    }

    [RelayCommand]
    private void ResumeBatch()
    {
        IsPaused = false;
        StatusText = "Processing...";
        StartTimer();
    }

    [RelayCommand]
    private void StopBatch()
    {
        IsRunning = false;
        IsPaused = false;
        StatusText = "Stopped";
        StopTimer();
        ElapsedTime = (DateTime.Now - _startTime).ToString(@"hh\:mm\:ss");
    }

    [RelayCommand]
    private void ClearCompleted()
    {
        var completed = BatchQueue.Where(i => i.ProcessingProgress >= 1.0).ToList();
        foreach (var item in completed)
            BatchQueue.Remove(item);
        TotalItems = BatchQueue.Count;
    }

    [RelayCommand]
    private void AddToQueue(ImageEntry image)
    {
        if (!BatchQueue.Contains(image))
            BatchQueue.Add(image);
        TotalItems = BatchQueue.Count;
    }

    [RelayCommand]
    private void RemoveFromQueue(ImageEntry image)
    {
        BatchQueue.Remove(image);
        TotalItems = BatchQueue.Count;
    }

    private void StartTimer()
    {
        StopTimer();
        _timer = new System.Timers.Timer(1000);
        _timer.Elapsed += (_, _) =>
        {
            ElapsedTime = (DateTime.Now - _startTime).ToString(@"hh\:mm\:ss");
            if (CompletedItems > 0 && IsRunning)
            {
                var rate = (DateTime.Now - _startTime).TotalSeconds / CompletedItems;
                var remaining = rate * (TotalItems - CompletedItems);
                EstimatedRemaining = TimeSpan.FromSeconds(remaining).ToString(@"hh\:mm\:ss");
            }
        };
        _timer.Start();
    }

    private void StopTimer()
    {
        _timer?.Stop();
        _timer?.Dispose();
        _timer = null;
    }
}
