using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using System.Collections.ObjectModel;
using System.IO;
using System.Windows.Threading;

namespace Photopipeline.ViewModels;

public sealed partial class BatchViewModel : ViewModelBase
{
    private readonly IBatchService _batchService;
    private readonly IDialogService _dialogService;
    private CancellationTokenSource? _batchCts;
    private string? _batchId;

    [ObservableProperty] private ObservableCollection<ImageEntry> _batchQueue = new();
    [ObservableProperty] private int _totalItems;
    [ObservableProperty] private int _completedItems;
    [ObservableProperty] private int _failedItems;
    [ObservableProperty] private double _overallProgress;
    [ObservableProperty] private bool _isRunning;
    [ObservableProperty] private bool _isPaused;
    [ObservableProperty] private string _elapsedTime = "00:00:00";
    [ObservableProperty] private string _estimatedRemaining = "--:--:--";
    [ObservableProperty] private string _speedInfo = string.Empty;
    [ObservableProperty] private ObservableCollection<string> _outputFormats = new() { "TIFF", "JPEG", "PNG", "WebP", "HEIF", "AVIF", "JPEG XL" };
    [ObservableProperty] private string _selectedOutputFormat = "TIFF";
    [ObservableProperty] private string _outputDirectory = string.Empty;
    [ObservableProperty] private string _fileNameTemplate = "{name}_processed";
    [ObservableProperty] private int _jpegQuality = 95;
    [ObservableProperty] private bool _embedMetadata = true;
    [ObservableProperty] private int _parallelCount = Math.Max(1, Environment.ProcessorCount - 1);
    [ObservableProperty] private string? _pipelineConfigPath;

    private DispatcherTimer? _timer;
    private DateTime _startTime;

    public BatchViewModel(ILogger<BatchViewModel> logger, IBatchService batchService,
        IDialogService dialogService) : base(logger)
    {
        _batchService = batchService;
        _dialogService = dialogService;
    }

    [RelayCommand]
    private async Task StartBatch(CancellationToken ct)
    {
        if (BatchQueue.Count == 0)
        {
            ErrorMessage = "No items in queue";
            return;
        }
        if (string.IsNullOrWhiteSpace(OutputDirectory))
        {
            ErrorMessage = "Select an output directory";
            return;
        }
        if (string.IsNullOrWhiteSpace(PipelineConfigPath))
        {
            ErrorMessage = "Create a pipeline in the Pipeline Editor first";
            return;
        }

        CancelInternal();
        _batchCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        var token = _batchCts.Token;

        try
        {
            IsRunning = true;
            IsPaused = false;
            ErrorMessage = null;

            // Only process pending items
            var pending = BatchQueue.Where(i => i.Status != ImageStatus.Overridden).ToList();
            if (pending.Count == 0)
            {
                StatusMessage = "All items already processed";
                return;
            }

            // Reset pending item status
            foreach (var item in pending)
                item.Status = ImageStatus.None;

            CompletedItems = 0;
            FailedItems = 0;
            TotalItems = pending.Count;
            OverallProgress = 0;
            _startTime = DateTime.Now;
            StartTimer();

            var spec = new BatchSpec
            {
                OutputDir = OutputDirectory,
                Parallel = ParallelCount,
                Resume = false,
                PipelineConfigPath = PipelineConfigPath,
                FilePattern = string.Join(";", pending.Select(i => i.FilePath))
            };
            _batchId = await _batchService.SubmitAsync(spec, token);
            Logger.LogInformation("Batch submitted: {BatchId} ({FileCount} files, {Parallel}x parallel)",
                _batchId, pending.Count, ParallelCount);
            StatusMessage = $"Batch {_batchId} started";

            await foreach (var progress in _batchService.GetProgressAsync(_batchId, token))
            {
                token.ThrowIfCancellationRequested();
                if (IsPaused) continue;

                OverallProgress = progress.Fraction * 100;
                CompletedItems = progress.CompletedFiles;
                FailedItems = progress.FailedFiles;

                if (progress.Status == BatchStatus.Done || progress.Status == BatchStatus.Error)
                    break;
            }

            OverallProgress = 100;
            IsRunning = false;

            foreach (var item in BatchQueue) item.Status = ImageStatus.Overridden;

            StatusMessage = FailedItems > 0
                ? $"Done: {CompletedItems} succeeded, {FailedItems} failed"
                : $"Batch complete: {CompletedItems} items";
        }
        catch (OperationCanceledException)
        {
            StatusMessage = IsPaused ? "Paused" : "Stopped";
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Batch processing failed");
            ErrorMessage = $"Batch failed: {ex.Message}";
        }
        finally
        {
            IsRunning = false;
            StopTimer();
            ElapsedTime = (DateTime.Now - _startTime).ToString(@"hh\:mm\:ss");
            UpdateSpeedInfo();
        }
    }

    [RelayCommand]
    private void PauseBatch()
    {
        Logger.LogInformation("Batch paused: {Completed}/{Total}", CompletedItems, TotalItems);
        IsPaused = true;
        StatusMessage = "Paused";
        StopTimer();
        // Cancel the CTS to stop consuming the gRPC stream.
        // _batchId is preserved so that the batch state can be queried when resuming.
        if (_batchCts is not null)
        {
            _batchCts.Cancel();
            _batchCts.Dispose();
            _batchCts = null;
        }
    }

    [RelayCommand]
    private void ResumeBatch()
    {
        Logger.LogInformation("Batch resumed: {BatchId}", _batchId ?? "new");
        IsPaused = false;
        StatusMessage = "Resuming...";
        // Re-submit the remaining pending items or resume from the saved _batchId
        if (!string.IsNullOrEmpty(_batchId))
        {
            // Resume by restarting the batch; _batchId is tracked from previous SubmitAsync
            StatusMessage = "Resuming batch...";
        }
        StartTimer();
        StartBatchCommand.Execute(null);
    }

    [RelayCommand]
    private void StopBatch()
    {
        Logger.LogInformation("Batch stopped: {Completed}/{Total} completed", CompletedItems, TotalItems);
        CancelInternal();
        IsRunning = false;
        IsPaused = false;
        StatusMessage = "Stopped";
        StopTimer();
        ElapsedTime = (DateTime.Now - _startTime).ToString(@"hh\:mm\:ss");
        UpdateSpeedInfo();
    }

    [RelayCommand]
    private void ClearCompleted()
    {
        var completed = BatchQueue.Where(i => i.Status == ImageStatus.Overridden || i.Status == ImageStatus.Error).ToList();
        foreach (var item in completed) BatchQueue.Remove(item);
    }

    [RelayCommand]
    private void AddToQueue(ImageEntry image)
    {
        if (image is null) return;
        if (!BatchQueue.Contains(image))
            BatchQueue.Add(image);
    }

    [RelayCommand]
    private void RemoveFromQueue(ImageEntry image)
    {
        if (image is null) return;
        BatchQueue.Remove(image);
    }

    [RelayCommand]
    private void BrowseOutputDirectory()
    {
        var folder = _dialogService.ShowOpenFolderDialog("Select output directory");
        if (folder is not null)
            OutputDirectory = folder;
    }

    private void CancelInternal()
    {
        if (_batchCts is not null)
        {
            _batchCts.Cancel();
            _batchCts.Dispose();
            _batchCts = null;
        }

        if (_batchId is not null)
        {
            _ = _batchService.CancelAsync(_batchId);
            _batchId = null;
        }
    }

    private void StartTimer()
    {
        StopTimer();
        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _timer.Tick += (_, _) =>
        {
            if (!IsRunning) return;
            var elapsed = DateTime.Now - _startTime;
            ElapsedTime = elapsed.ToString(@"hh\:mm\:ss");
            if (CompletedItems > 0)
            {
                var rate = elapsed.TotalSeconds / CompletedItems;
                var remaining = rate * (TotalItems - CompletedItems);
                EstimatedRemaining = TimeSpan.FromSeconds(remaining).ToString(@"hh\:mm\:ss");
                UpdateSpeedInfo();
            }
        };
        _timer.Start();
    }

    private void StopTimer() { _timer?.Stop(); _timer = null; }

    private void UpdateSpeedInfo()
    {
        var elapsed = DateTime.Now - _startTime;
        if (elapsed.TotalMinutes > 0.01 && CompletedItems > 0)
        {
            var rate = CompletedItems / elapsed.TotalMinutes;
            SpeedInfo = $"{rate:F1} images/min";
        }
    }

    public override void Shutdown()
    {
        base.Shutdown();
        StopTimer();
        try { _batchCts?.Cancel(); _batchCts?.Dispose(); } catch { }
    }
}
