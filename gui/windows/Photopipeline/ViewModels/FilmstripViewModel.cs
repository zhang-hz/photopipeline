using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.IO;
using System.Windows;
using System.Windows.Data;

namespace Photopipeline.ViewModels;

public sealed partial class FilmstripViewModel : ViewModelBase
{
    private readonly IImageService _imageService;
    private readonly SemaphoreSlim _thumbnailSemaphore = new(4);

    [ObservableProperty] private ObservableCollection<ImageEntry> _images = new();
    [ObservableProperty] private ObservableCollection<ImageEntry> _filteredImages = new();
    [ObservableProperty] private ImageEntry? _selectedImage;
    [ObservableProperty] private ObservableCollection<ImageEntry> _selectedImages = new();
    [ObservableProperty] private string _filterText = string.Empty;
    [ObservableProperty] private string _sortBy = "Name";
    [ObservableProperty] private string _filterFormat = "All";
    [ObservableProperty] private int _thumbnailSize = 120;
    [ObservableProperty] private bool _isLoading;

    public IReadOnlyList<string> SortOptions { get; } = new[] { "Name", "Date", "Size", "Format" };
    public IReadOnlyList<string> FormatFilters { get; } = new[] { "All", "Raw", "JPEG", "TIFF", "PNG", "HEIF" };
    public IReadOnlyList<int> ThumbnailSizes { get; } = new[] { 80, 120, 180 };

    public event Action<ImageEntry?>? ImageSelected;
    public event Action<IReadOnlyList<ImageEntry>>? SendToBatchRequested;

    public FilmstripViewModel(ILogger<FilmstripViewModel> logger, IImageService imageService)
        : base(logger)
    {
        _imageService = imageService;
        FilteredImages = Images;
    }

    partial void OnSelectedImageChanged(ImageEntry? value) => ImageSelected?.Invoke(value);
    partial void OnFilterTextChanged(string value) => ApplyFilter();
    partial void OnSortByChanged(string value) => ApplyFilter();
    partial void OnFilterFormatChanged(string value) => ApplyFilter();

    [RelayCommand]
    private async Task AddImages(CancellationToken ct)
    {
        // TODO: File dialogs should be abstracted behind a service interface to avoid violating MVVM
        await ExecuteAsync(async ct2 =>
        {
            var dialog = new Microsoft.Win32.OpenFileDialog
            {
                Title = "Select images",
                Multiselect = true,
                Filter = "Image files|*.dng;*.nef;*.cr2;*.arw;*.orf;*.tif;*.tiff;*.jpg;*.jpeg;*.png;*.heic;*.heif;*.avif;*.jxl;*.exr;*.bmp;*.ppm|All files|*.*"
            };
            if (dialog.ShowDialog() == true)
            {
                foreach (var path in dialog.FileNames)
                {
                    ct2.ThrowIfCancellationRequested();
                    if (Images.Any(i => string.Equals(i.FilePath, path, StringComparison.OrdinalIgnoreCase)))
                        continue;
                    try
                    {
                        var info = await _imageService.LoadImageInfoAsync(path, ct2);
                        Images.Add(ImageEntry.FromImageInfo(info));
                    }
                    catch (Exception ex)
                    {
                        Logger.LogWarning(ex, "Failed to load image info for {Path}", path);
                    }
                }
                ApplyFilter();
                StatusMessage = $"{Images.Count} images loaded";
                _ = RefreshThumbnailsAsync().ContinueWith(t =>
                {
                    if (t.IsFaulted && t.Exception != null)
                        Logger.LogWarning(t.Exception, "Background thumbnail refresh failed");
                }, TaskScheduler.Default);
            }
        }, "AddImages", ct);
    }

    [RelayCommand]
    private void RemoveImage(ImageEntry? image)
    {
        if (image is null) return;
        Images.Remove(image);
        FilteredImages.Remove(image);
        if (SelectedImage == image) SelectedImage = null;
    }

    [RelayCommand]
    private void ClearImages()
    {
        Images.Clear();
        FilteredImages.Clear();
        SelectedImage = null;
        SelectedImages.Clear();
    }

    [RelayCommand]
    private void SendToBatch()
    {
        var items = SelectedImages.Count > 0 ? SelectedImages : FilteredImages;
        if (items.Count > 0)
            SendToBatchRequested?.Invoke(items.ToList());
    }

    [RelayCommand]
    private void SelectAll()
    {
        SelectedImages.Clear();
        foreach (var img in FilteredImages)
            SelectedImages.Add(img);
    }

    [RelayCommand]
    private void ClearSelection() => SelectedImages.Clear();

    [RelayCommand]
    private void InvertSelection()
    {
        var toSelect = FilteredImages.Except(SelectedImages).ToList();
        SelectedImages.Clear();
        foreach (var img in toSelect)
            SelectedImages.Add(img);
    }

    [RelayCommand]
    private void SetThumbnailSize(object size)
    {
        if (!TryParseInt(size, out var s)) return;
        ThumbnailSize = s;
        _ = RefreshThumbnailsAsync();
    }

    [RelayCommand]
    private void OpenInExplorer(ImageEntry? image)
    {
        if (image is null) return;
        var path = image.FilePath;
        if (File.Exists(path))
            System.Diagnostics.Process.Start("explorer.exe", $"/select,\"{path}\"");
    }

    [RelayCommand]
    private void CopyPath(ImageEntry? image)
    {
        if (image is null) return;
        try { Clipboard.SetText(image.FilePath); }
        catch { /* clipboard may be busy */ }
    }

    public async Task RefreshThumbnailsAsync()
    {
        var size = ThumbnailSize;
        var tasks = Images
            .Where(img => img.ThumbnailData is null)
            .Select(async img =>
            {
                await _thumbnailSemaphore.WaitAsync();
                try
                {
                    img.ThumbnailData = await _imageService.GetThumbnailAsync(img.FilePath, size);
                }
                catch (Exception ex)
                {
                    Logger.LogWarning(ex, "Thumbnail load failed for {Path}", img.FilePath);
                }
                finally
                {
                    _thumbnailSemaphore.Release();
                }
            });

        await Task.WhenAll(tasks);
    }

    private void ApplyFilter()
    {
        var source = Images.AsEnumerable();

        if (!string.IsNullOrWhiteSpace(FilterText))
        {
            var term = FilterText.Trim();
            source = source.Where(i =>
                i.FileName.Contains(term, StringComparison.OrdinalIgnoreCase) ||
                i.Format.Contains(term, StringComparison.OrdinalIgnoreCase));
        }

        if (FilterFormat != "All")
        {
            source = source.Where(i =>
                i.Format.Equals(FilterFormat, StringComparison.OrdinalIgnoreCase));
        }

        source = SortBy switch
        {
            "Size" => source.OrderByDescending(i => i.FileSizeBytes),
            "Format" => source.OrderBy(i => i.Format),
            _ => source.OrderBy(i => i.FileName)
        };

        FilteredImages = new ObservableCollection<ImageEntry>(source);
        SelectedImages.Clear();
    }

    private static bool TryParseInt(object? value, out int result)
    {
        if (value is int i) { result = i; return true; }
        if (value is string s && int.TryParse(s, out var parsed)) { result = parsed; return true; }
        result = 0;
        return false;
    }

    public override void Shutdown()
    {
        base.Shutdown();
        try { _thumbnailSemaphore.Dispose(); } catch { }
    }
}
