using Photopipeline.Models;
using System.Collections.ObjectModel;
using System.IO;

namespace Photopipeline.Services;

public sealed class LocalImageService : IImageService
{
    public Task<ObservableCollection<ImageEntry>> LoadImagesAsync(string[] filePaths, CancellationToken ct = default)
    {
        var images = new ObservableCollection<ImageEntry>();
        foreach (var path in filePaths)
        {
            if (!File.Exists(path)) continue;
            var fi = new FileInfo(path);
            images.Add(new ImageEntry
            {
                FilePath = path,
                FileName = fi.Name,
                FileSize = (ulong)fi.Length
            });
        }
        return Task.FromResult(images);
    }

    public Task LoadImageMetadataAsync(ImageEntry entry, CancellationToken ct = default)
    {
        return Task.CompletedTask;
    }

    public Task GenerateThumbnailAsync(ImageEntry entry, CancellationToken ct = default)
    {
        return Task.CompletedTask;
    }

    public Task<Stream?> LoadPreviewImageAsync(ImageEntry entry, CancellationToken ct = default)
    {
        if (!File.Exists(entry.FilePath)) return Task.FromResult<Stream?>(null);
        try { return Task.FromResult<Stream?>(File.OpenRead(entry.FilePath)); }
        catch { return Task.FromResult<Stream?>(null); }
    }

    public Task<Stream?> ProcessPreviewImageAsync(ImageEntry entry, PipelineModel pipeline, CancellationToken ct = default)
    {
        return LoadPreviewImageAsync(entry, ct);
    }

    public Task<string> ExportImageAsync(ImageEntry entry, string outputPath, PipelineModel pipeline, CancellationToken ct = default)
    {
        return Task.FromResult(outputPath);
    }
}
