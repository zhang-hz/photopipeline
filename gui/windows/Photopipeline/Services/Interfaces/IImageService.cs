using Photopipeline.Models;

namespace Photopipeline.Services;

public interface IImageService
{
    Task<ImageInfo> LoadImageInfoAsync(string path, CancellationToken ct = default);
    IAsyncEnumerable<PixelDataChunk> DecodeAsync(string path, DecodeOptions? options = null, CancellationToken ct = default);
    IAsyncEnumerable<EncodeProgress> EncodeAsync(EncodeRequest request, CancellationToken ct = default);
    Task<byte[]> GetThumbnailAsync(string path, int maxSize = 256, CancellationToken ct = default);
}
