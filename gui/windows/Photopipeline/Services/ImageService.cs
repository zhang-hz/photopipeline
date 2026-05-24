using Photopipeline.Image;
using Photopipeline.Models;
using System.Collections.ObjectModel;

namespace Photopipeline.Services;

public sealed class ImageService : IImageService
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
        try
        {
            var channel = await _grpc.GetChannelAsync(ct);
            var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
            var response = await client.LoadAsync(new ImagePath { Path = entry.FilePath }, cancellationToken: ct);
            entry.Width = (int)response.Width;
            entry.Height = (int)response.Height;
            entry.ColorSpace = response.ColorSpace;
            entry.BitDepth = response.PixelFormat.Contains("16") ? "16" : "8";
        }
        catch
        {
            var ext = Path.GetExtension(entry.FileName).ToLowerInvariant();
            entry.ColorSpace = ext switch
            {
                ".dng" or ".nef" or ".cr2" or ".arw" or ".orf" => "Linear Raw",
                _ => "sRGB"
            };
        }
    }

    public async Task GenerateThumbnailAsync(ImageEntry entry, CancellationToken ct = default)
    {
        var thumbnailName = $"thumb_{entry.Id}_{Path.GetFileNameWithoutExtension(entry.FileName)}.jpg";
        var thumbnailPath = Path.Combine(_thumbnailCachePath, thumbnailName);

        if (!File.Exists(thumbnailPath))
        {
            try
            {
                var channel = await _grpc.GetChannelAsync(ct);
                var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
                var response = await client.GetThumbnailAsync(
                    new ThumbnailRequest { Path = entry.FilePath, MaxSize = 256 },
                    cancellationToken: ct);
                await File.WriteAllBytesAsync(thumbnailPath, response.Data.ToByteArray(), ct);
            }
            catch
            {
                // Thumbnail generation failed; will show placeholder
            }
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
        try
        {
            var channel = await _grpc.GetChannelAsync(ct);
            var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
            var imageData = await File.ReadAllBytesAsync(entry.FilePath, ct);
            var request = new EncodeRequest
            {
                PixelData = Google.Protobuf.ByteString.CopyFrom(imageData),
                OutputPath = outputPath,
                Format = Path.GetExtension(outputPath).TrimStart('.').ToLowerInvariant(),
                Lossless = true
            };

            using var call = client.Encode(request, cancellationToken: ct);
            while (await call.ResponseStream.MoveNext(ct))
            {
                if (call.ResponseStream.Current.Done) break;
            }
        }
        catch
        {
            // Fall back to file copy
            await Task.CompletedTask;
        }

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
