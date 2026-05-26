using Microsoft.Extensions.Logging;
using Photopipeline.Models;

namespace Photopipeline.Services;

public sealed class ImageService : IImageService
{
    private readonly GrpcClientService _grpc;
    private readonly ILogger<ImageService> _logger;

    public ImageService(GrpcClientService grpc, ILogger<ImageService> logger)
    {
        _grpc = grpc;
        _logger = logger;
    }

    public async Task<ImageInfo> LoadImageInfoAsync(string path, CancellationToken ct = default)
    {
        _logger.LogInformation("Loading image info: {Path}", path);
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
        var proto = await client.LoadAsync(
            new global::Photopipeline.Image.ImagePath { Path = path },
            cancellationToken: ct);

        _logger.LogInformation("Image info loaded: {Path} ({Width}x{Height}, {Format})",
            path, proto.Width, proto.Height, proto.Format);
        return new ImageInfo
        {
            Id = proto.Id,
            Path = proto.Path,
            FileName = proto.Filename,
            Format = proto.Format,
            Width = proto.Width,
            Height = proto.Height,
            FileSizeBytes = proto.FileSizeBytes,
            PixelFormat = proto.PixelFormat,
            ColorSpace = proto.ColorSpace,
            Metadata = proto.Metadata is not null ? MapMetadata(proto.Metadata) : null
        };
    }

    public async IAsyncEnumerable<PixelDataChunk> DecodeAsync(
        string path, DecodeOptions? options = null,
        [System.Runtime.CompilerServices.EnumeratorCancellation] CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
        var request = new global::Photopipeline.Image.DecodeRequest
        {
            Path = path,
            ReadMetadata = options?.ReadMetadata ?? true,
            ApplyTransfer = options?.ApplyTransfer ?? true
        };
        if (options?.PixelFormat is not null) request.PixelFormat = options.PixelFormat;
        if (options?.MaxWidth.HasValue == true) request.MaxWidth = options.MaxWidth.Value;
        if (options?.MaxHeight.HasValue == true) request.MaxHeight = options.MaxHeight.Value;

        _logger.LogInformation("Decoding image: {Path}", path);
        using var call = client.Decode(request, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            var chunk = call.ResponseStream.Current;
            yield return new PixelDataChunk
            {
                Offset = chunk.Offset,
                Data = chunk.Data.ToByteArray(),
                TotalSize = chunk.TotalSize,
                IsLast = chunk.IsLast
            };
        }
    }

    public async IAsyncEnumerable<EncodeProgress> EncodeAsync(
        EncodeRequest request,
        [System.Runtime.CompilerServices.EnumeratorCancellation] CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
        var protoReq = new global::Photopipeline.Image.EncodeRequest
        {
            PixelData = Google.Protobuf.ByteString.CopyFrom(request.PixelData),
            Width = request.Width,
            Height = request.Height,
            Layout = request.Layout,
            PixelFormat = request.PixelFormat,
            OutputPath = request.OutputPath,
            Format = request.Format,
            Lossless = request.Lossless,
            BitDepth = request.BitDepth
        };
        if (request.Quality.HasValue) protoReq.Quality = request.Quality.Value;
        if (request.ChromaSubsampling is not null) protoReq.ChromaSubsampling = request.ChromaSubsampling;
        if (request.Encoder is not null) protoReq.Encoder = request.Encoder;
        if (request.Effort.HasValue) protoReq.Effort = request.Effort.Value;
        if (request.Metadata is not null) protoReq.Metadata = MapToProtoMetadata(request.Metadata);

        _logger.LogInformation("Encoding image: {Format} {Width}x{Height} → {OutputPath}",
            protoReq.Format, protoReq.Width, protoReq.Height, protoReq.OutputPath);
        using var call = client.Encode(protoReq, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            var p = call.ResponseStream.Current;
            yield return new EncodeProgress
            {
                Fraction = p.Fraction,
                Message = p.Message,
                BytesWritten = p.BytesWritten,
                Done = p.Done
            };
        }
    }

    public async Task<byte[]> GetThumbnailAsync(string path, int maxSize = 256, CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Image.ImageService.ImageServiceClient(channel);
        _logger.LogDebug("Generating thumbnail: {Path} (max {MaxSize}px)", path, maxSize);
        var response = await client.GetThumbnailAsync(
            new global::Photopipeline.Image.ThumbnailRequest { Path = path, MaxSize = (uint)maxSize },
            cancellationToken: ct);
        _logger.LogDebug("Thumbnail generated: {Path} ({Bytes} bytes)", path, response.Data.Length);
        return response.Data.ToByteArray();
    }

    private static ImageMetadata MapMetadata(global::Photopipeline.Image.MetadataInfo m) => new()
    {
        Make = m.HasMake ? m.Make : null,
        Model = m.HasModel ? m.Model : null,
        LensModel = m.HasLensModel ? m.LensModel : null,
        DateTimeOriginal = m.HasDateTimeOriginal ? m.DateTimeOriginal : null,
        ExposureTime = m.HasExposureTime ? m.ExposureTime : null,
        FNumber = m.HasFNumber ? m.FNumber : null,
        Iso = m.HasIso ? m.Iso : null,
        FocalLength = m.HasFocalLength ? m.FocalLength : null,
        Latitude = m.HasLatitude ? m.Latitude : null,
        Longitude = m.HasLongitude ? m.Longitude : null,
        Altitude = m.HasAltitude ? m.Altitude : null
    };

    private static global::Photopipeline.Image.MetadataInfo MapToProtoMetadata(ImageMetadata m)
    {
        var proto = new global::Photopipeline.Image.MetadataInfo();
        if (m.Make is not null) proto.Make = m.Make;
        if (m.Model is not null) proto.Model = m.Model;
        if (m.LensModel is not null) proto.LensModel = m.LensModel;
        if (m.DateTimeOriginal is not null) proto.DateTimeOriginal = m.DateTimeOriginal;
        if (m.ExposureTime is not null) proto.ExposureTime = m.ExposureTime;
        if (m.FNumber is not null) proto.FNumber = m.FNumber;
        if (m.Iso.HasValue) proto.Iso = m.Iso.Value;
        if (m.FocalLength is not null) proto.FocalLength = m.FocalLength;
        if (m.Latitude.HasValue) proto.Latitude = m.Latitude.Value;
        if (m.Longitude.HasValue) proto.Longitude = m.Longitude.Value;
        if (m.Altitude.HasValue) proto.Altitude = m.Altitude.Value;
        return proto;
    }
}
