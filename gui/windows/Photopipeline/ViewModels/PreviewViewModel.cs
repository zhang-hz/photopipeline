using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using SkiaSharp;

namespace Photopipeline.ViewModels;

public sealed partial class PreviewViewModel : ViewModelBase
{
    private readonly IImageService _imageService;
    private readonly IPipelineService _pipelineService;
    private CancellationTokenSource? _loadCts;
    private CancellationTokenSource? _processCts;

    [ObservableProperty] private double _zoomLevel = 1.0;
    [ObservableProperty] private double _splitPosition = 0.5;
    [ObservableProperty] private bool _isSplitView = true;
    [ObservableProperty] private bool _isFitToWindow = true;
    [ObservableProperty] private string _fitLabel = "Fit";

    [ObservableProperty] private SKBitmap? _beforeBitmap;
    [ObservableProperty] private SKBitmap? _afterBitmap;
    [ObservableProperty] private SKPoint _panOffset;
    [ObservableProperty] private string _pixelInfo = string.Empty;
    [ObservableProperty] private bool _isLoading;

    private const double MinZoom = 0.0625;
    private const double MaxZoom = 32.0;
    private static readonly double[] ZoomSteps = { 0.25, 0.33, 0.5, 0.67, 1.0, 1.5, 2.0, 3.0, 4.0, 8.0 };

    public PreviewViewModel(
        ILogger<PreviewViewModel> logger,
        IImageService imageService,
        IPipelineService pipelineService) : base(logger)
    {
        _imageService = imageService;
        _pipelineService = pipelineService;
    }

    [RelayCommand]
    private void ZoomIn()
    {
        var next = ZoomSteps.FirstOrDefault(s => s > ZoomLevel + 0.001);
        ZoomLevel = next > 0 ? next : Math.Min(ZoomLevel * 1.5, MaxZoom);
        IsFitToWindow = false;
    }

    [RelayCommand]
    private void ZoomOut()
    {
        var prev = ZoomSteps.LastOrDefault(s => s < ZoomLevel - 0.001);
        ZoomLevel = prev > 0 ? prev : Math.Max(ZoomLevel / 1.5, MinZoom);
        IsFitToWindow = false;
    }

    [RelayCommand]
    private void ResetZoom() { ZoomLevel = 1.0; IsFitToWindow = false; }

    [RelayCommand]
    private void FitToWindow() { IsFitToWindow = true; ZoomLevel = 1.0; }

    [RelayCommand]
    private void ToggleSplit() => IsSplitView = !IsSplitView;

    [RelayCommand]
    private void OneToOne() { ZoomLevel = 1.0; IsFitToWindow = false; }

    public void Pan(double dx, double dy)
    {
        PanOffset = new SKPoint(PanOffset.X + (float)dx, PanOffset.Y + (float)dy);
    }

    [RelayCommand]
    private async Task Export(CancellationToken ct)
    {
        var bmp = AfterBitmap ?? BeforeBitmap;
        if (bmp == null)
        {
            ErrorMessage = "No image to export";
            return;
        }

        var dialog = new Microsoft.Win32.SaveFileDialog
        {
            Title = "Export Image",
            Filter = "TIFF|*.tif|JPEG|*.jpg|PNG|*.png|WebP|*.webp|AVIF|*.avif",
            DefaultExt = ".tif"
        };
        if (dialog.ShowDialog() != true) return;

        await ExecuteAsync(async ct2 =>
        {
            var pixels = bmp.Bytes;
            var request = new EncodeRequest
            {
                PixelData = pixels,
                Width = (uint)bmp.Width,
                Height = (uint)bmp.Height,
                Layout = bmp.ColorType == SKColorType.Rgba8888 ? "RGBA" : "RGB",
                OutputPath = dialog.FileName,
                Format = System.IO.Path.GetExtension(dialog.FileName).TrimStart('.').ToUpperInvariant()
            };

            await foreach (var _ in _imageService.EncodeAsync(request, ct2)) { }
            StatusMessage = $"Exported to {dialog.FileName}";
        }, "Export image", ct);
    }

    public async Task LoadImageAsync(ImageEntry image, CancellationToken ct = default)
    {
        await CancelLoadAsync();
        _loadCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        var token = _loadCts.Token;

        try
        {
            IsLoading = true;
            ErrorMessage = null;

            var chunks = new List<byte[]>();
            uint totalSize = 0;
            uint width = image.Width;
            uint height = image.Height;

            await foreach (var chunk in _imageService.DecodeAsync(image.FilePath, new DecodeOptions
            {
                MaxWidth = width,
                MaxHeight = height,
                ReadMetadata = false
            }, token))
            {
                token.ThrowIfCancellationRequested();
                chunks.Add(chunk.Data);
                totalSize = chunk.TotalSize;
            }

            if (chunks.Count == 0) return;

            var allBytes = new byte[totalSize];
            uint written = 0;
            foreach (var c in chunks)
            {
                Array.Copy(c, 0, allBytes, written, c.Length);
                written += (uint)c.Length;
            }

            var colorType = ParseColorType(image.PixelFormat);
            var bmp = new SKBitmap((int)width, (int)height, colorType, SKAlphaType.Premul);
            System.Runtime.InteropServices.Marshal.Copy(allBytes, 0, bmp.GetPixels(), allBytes.Length);
            BeforeBitmap = bmp;

            StatusMessage = $"Loaded {image.FileName} ({width}x{height})";
        }
        catch (OperationCanceledException) { }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Failed to load image {Path}", image.FilePath);
            ErrorMessage = $"Failed to load: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    public async Task ProcessPreviewAsync(ImageEntry image, PipelineSpec spec, string? pipelineId = null, CancellationToken ct = default)
    {
        if (string.IsNullOrEmpty(image.FilePath)) return;

        await CancelProcessAsync();
        _processCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        var token = _processCts.Token;

        try
        {
            var tempOut = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"pp_preview_{Guid.NewGuid():N}.tif");
            var pid = pipelineId;

            if (string.IsNullOrEmpty(pid) && spec.Nodes.Count > 0)
            {
                pid = await _pipelineService.CreatePipelineAsync(spec, token);
            }

            if (string.IsNullOrEmpty(pid)) return;

            await foreach (var progress in _pipelineService.ExecuteAsync(pid, image.FilePath, tempOut, token))
            {
                token.ThrowIfCancellationRequested();
                // Progress is consumed but not displayed in preview mode
            }

            if (System.IO.File.Exists(tempOut))
            {
                var info = await _imageService.LoadImageInfoAsync(tempOut, token);
                var chunks = new List<byte[]>();
                uint totalSize = 0;

                await foreach (var chunk in _imageService.DecodeAsync(tempOut, new DecodeOptions(), token))
                {
                    chunks.Add(chunk.Data);
                    totalSize = chunk.TotalSize;
                }

                var allBytes = new byte[totalSize];
                uint written = 0;
                foreach (var c in chunks)
                {
                    Array.Copy(c, 0, allBytes, written, c.Length);
                    written += (uint)c.Length;
                }

                var colorType = ParseColorType(info.PixelFormat);
                var bmp = new SKBitmap((int)info.Width, (int)info.Height, colorType, SKAlphaType.Premul);
                System.Runtime.InteropServices.Marshal.Copy(allBytes, 0, bmp.GetPixels(), allBytes.Length);
                AfterBitmap = bmp;
                StatusMessage = "Preview rendered";
            }
        }
        catch (OperationCanceledException) { }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Preview processing failed");
            ErrorMessage = $"Processing failed: {ex.Message}";
        }
    }

    private async Task CancelLoadAsync()
    {
        if (_loadCts != null)
        {
            await _loadCts.CancelAsync();
            _loadCts.Dispose();
            _loadCts = null;
        }
    }

    private async Task CancelProcessAsync()
    {
        if (_processCts != null)
        {
            await _processCts.CancelAsync();
            _processCts.Dispose();
            _processCts = null;
        }
    }

    public override void Shutdown()
    {
        base.Shutdown();
        try { _loadCts?.Cancel(); _loadCts?.Dispose(); } catch { }
        try { _processCts?.Cancel(); _processCts?.Dispose(); } catch { }
    }

    private static SKColorType ParseColorType(string fmt) => fmt.ToUpperInvariant() switch
    {
        "RGBA8" or "BGRA8" => SKColorType.Rgba8888,
        "RGB8" or "BGR8" => SKColorType.Rgb888x,
        "RGBA16" => SKColorType.Rgba16161616,
        "GRAY8" => SKColorType.Gray8,
        "GRAY16" => SKColorType.Alpha16,
        _ => SKColorType.Rgba8888
    };
}
