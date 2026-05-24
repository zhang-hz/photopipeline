using Photopipeline.Models;
using System.Collections.ObjectModel;
using System.IO;

namespace Photopipeline.Services;

public interface IImageService
{
    Task<ObservableCollection<ImageEntry>> LoadImagesAsync(string[] filePaths, CancellationToken ct = default);
    Task LoadImageMetadataAsync(ImageEntry entry, CancellationToken ct = default);
    Task GenerateThumbnailAsync(ImageEntry entry, CancellationToken ct = default);
    Task<Stream?> LoadPreviewImageAsync(ImageEntry entry, CancellationToken ct = default);
    Task<Stream?> ProcessPreviewImageAsync(ImageEntry entry, PipelineModel pipeline, CancellationToken ct = default);
    Task<string> ExportImageAsync(ImageEntry entry, string outputPath, PipelineModel pipeline, CancellationToken ct = default);
}
