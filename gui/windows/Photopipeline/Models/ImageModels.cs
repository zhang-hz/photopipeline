using CommunityToolkit.Mvvm.ComponentModel;

namespace Photopipeline.Models;

public sealed class ImageInfo
{
    public string Id { get; set; } = string.Empty;
    public string Path { get; set; } = string.Empty;
    public string FileName { get; set; } = string.Empty;
    public string Format { get; set; } = string.Empty;
    public uint Width { get; set; }
    public uint Height { get; set; }
    public ulong FileSizeBytes { get; set; }
    public string PixelFormat { get; set; } = string.Empty;
    public string ColorSpace { get; set; } = string.Empty;
    public ImageMetadata? Metadata { get; set; }
}

public sealed class ImageMetadata
{
    public string? Make { get; set; }
    public string? Model { get; set; }
    public string? LensModel { get; set; }
    public string? DateTimeOriginal { get; set; }
    public string? ExposureTime { get; set; }
    public string? FNumber { get; set; }
    public uint? Iso { get; set; }
    public string? FocalLength { get; set; }
    public double? Latitude { get; set; }
    public double? Longitude { get; set; }
    public double? Altitude { get; set; }
}

public sealed class DecodeOptions
{
    public string? PixelFormat { get; set; }
    public uint? MaxWidth { get; set; }
    public uint? MaxHeight { get; set; }
    public bool ReadMetadata { get; set; } = true;
    public bool ApplyTransfer { get; set; } = true;
}

public sealed class PixelDataChunk
{
    public uint Offset { get; set; }
    public byte[] Data { get; set; } = Array.Empty<byte>();
    public uint TotalSize { get; set; }
    public bool IsLast { get; set; }
}

public sealed class EncodeRequest
{
    public byte[] PixelData { get; set; } = Array.Empty<byte>();
    public uint Width { get; set; }
    public uint Height { get; set; }
    public string Layout { get; set; } = "RGBA";
    public string PixelFormat { get; set; } = "U8";
    public string OutputPath { get; set; } = string.Empty;
    public string Format { get; set; } = "TIFF";
    public float? Quality { get; set; }
    public bool Lossless { get; set; }
    public uint BitDepth { get; set; } = 8;
    public string? ChromaSubsampling { get; set; }
    public string? Encoder { get; set; }
    public uint? Effort { get; set; }
    public ImageMetadata? Metadata { get; set; }
}

public sealed class EncodeProgress
{
    public float Fraction { get; set; }
    public string Message { get; set; } = string.Empty;
    public ulong BytesWritten { get; set; }
    public bool Done { get; set; }
}
