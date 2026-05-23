using Photopipeline.Models;
using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace Photopipeline.Services;

public sealed class ImageService
{
    private readonly GrpcClientService _grpc;
    private readonly string _thumbnailCachePath;

    public ImageService(GrpcClientService grpc)
    {
        _grpc = grpc;
        _thumbnailCachePath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "Photopipeline", "thumbnails");
        Directory.CreateDirectory(_thumbnailCachePath);
    }

    public async Task<ObservableCollection<ImageEntry>> LoadImagesAsync(string[] filePaths, CancellationToken ct = default)
    {
        var images = new ObservableCollection<ImageEntry>();
        foreach (var path in filePaths)
        {
            if (!File.Exists(path)) continue;

            var fileInfo = new FileInfo(path);
            var entry = new ImageEntry
            {
                FilePath = path,
                FileName = fileInfo.Name,
                FileSize = (ulong)fileInfo.Length
            };

            await LoadImageMetadataAsync(entry, ct);
            await GenerateThumbnailAsync(entry, ct);
            images.Add(entry);
        }
        return images;
    }

    public async Task LoadImageMetadataAsync(ImageEntry entry, CancellationToken ct = default)
    {
        var ext = Path.GetExtension(entry.FileName).ToLowerInvariant();
        entry.ColorSpace = ext switch
        {
            ".dng" or ".nef" or ".cr2" or ".arw" or ".orf" => "Linear Raw",
            _ => "sRGB"
        };
        await Task.CompletedTask;
    }

    public async Task GenerateThumbnailAsync(ImageEntry entry, CancellationToken ct = default)
    {
        var thumbnailName = $"thumb_{entry.Id}_{Path.GetFileNameWithoutExtension(entry.FileName)}.jpg";
        var thumbnailPath = Path.Combine(_thumbnailCachePath, thumbnailName);

        if (!File.Exists(thumbnailPath))
        {
            await Task.CompletedTask;
        }

        entry.ThumbnailPath = thumbnailPath;
    }

    public async Task<Stream?> LoadPreviewImageAsync(ImageEntry entry, CancellationToken ct = default)
    {
        if (!File.Exists(entry.FilePath)) return null;
        return File.OpenRead(entry.FilePath);
    }

    public async Task<Stream?> ProcessPreviewImageAsync(ImageEntry entry, PipelineModel pipeline, CancellationToken ct = default)
    {
        var input = await LoadPreviewImageAsync(entry, ct);
        if (input is null) return null;

        var ms = new MemoryStream();
        await input.CopyToAsync(ms, ct);
        ms.Position = 0;
        return ms;
    }

    public async Task<string> ExportImageAsync(ImageEntry entry, string outputPath, PipelineModel pipeline, CancellationToken ct = default)
    {
        await Task.CompletedTask;
        return outputPath;
    }

    public static string FormatFileSize(ulong bytes)
    {
        string[] sizes = { "B", "KB", "MB", "GB", "TB" };
        double len = bytes;
        int order = 0;
        while (len >= 1024 && order < sizes.Length - 1)
        {
            order++;
            len /= 1024;
        }
        return $"{len:0.#} {sizes[order]}";
    }
}
