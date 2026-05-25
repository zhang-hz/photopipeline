namespace Photopipeline.Tests.UnitTests.Models;

public sealed class ImageModelsTests
{
    [Fact]
    public void ImageInfo_Defaults()
    {
        var info = new ImageInfo();

        info.Id.Should().BeEmpty();
        info.Path.Should().BeEmpty();
        info.FileName.Should().BeEmpty();
        info.Format.Should().BeEmpty();
        info.Width.Should().Be(0u);
        info.Height.Should().Be(0u);
        info.FileSizeBytes.Should().Be(0uL);
    }

    [Fact]
    public void ImageMetadata_StoresExifFields()
    {
        var meta = new ImageMetadata
        {
            Make = "Sony",
            Model = "A7IV",
            LensModel = "FE 24-70mm F2.8 GM II",
            DateTimeOriginal = "2024-01-15T14:30:00",
            ExposureTime = "1/250",
            FNumber = "f/2.8",
            Iso = 100,
            FocalLength = "50mm",
            Latitude = 37.7749,
            Longitude = -122.4194,
            Altitude = 15.2
        };

        meta.Make.Should().Be("Sony");
        meta.Iso.Should().Be(100u);
        meta.Latitude.Should().Be(37.7749);
        meta.Altitude.Should().Be(15.2);
    }

    [Fact]
    public void DecodeOptions_Defaults()
    {
        var opts = new DecodeOptions();

        opts.PixelFormat.Should().BeNull();
        opts.MaxWidth.Should().BeNull();
        opts.ReadMetadata.Should().BeTrue();
        opts.ApplyTransfer.Should().BeTrue();
    }

    [Fact]
    public void PixelDataChunk_Defaults()
    {
        var chunk = new PixelDataChunk();

        chunk.Offset.Should().Be(0u);
        chunk.Data.Should().BeEmpty();
        chunk.TotalSize.Should().Be(0u);
        chunk.IsLast.Should().BeFalse();
    }

    [Fact]
    public void EncodeRequest_HasRequiredFields()
    {
        var req = new EncodeRequest
        {
            PixelData = new byte[] { 0xFF, 0xD8 },
            Width = 1920,
            Height = 1080,
            Layout = "RGBA",
            PixelFormat = "U8",
            OutputPath = @"C:\out\test.tif",
            Format = "TIFF",
            Quality = 95f,
            Lossless = true,
            BitDepth = 16,
            ChromaSubsampling = "4:4:4"
        };

        req.Width.Should().Be(1920u);
        req.Height.Should().Be(1080u);
        req.Format.Should().Be("TIFF");
        req.Quality.Should().Be(95f);
        req.BitDepth.Should().Be(16u);
    }

    [Fact]
    public void EncodeProgress_TracksEncodingState()
    {
        var progress = new EncodeProgress
        {
            Fraction = 0.8f,
            Message = "Writing TIFF...",
            BytesWritten = 12_000_000uL,
            Done = false
        };

        progress.Fraction.Should().Be(0.8f);
        progress.BytesWritten.Should().Be(12_000_000uL);
        progress.Done.Should().BeFalse();
    }
}
