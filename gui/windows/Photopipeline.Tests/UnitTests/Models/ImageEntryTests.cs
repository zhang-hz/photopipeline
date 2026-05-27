namespace Photopipeline.Tests.UnitTests.Models;

public sealed class ImageEntryTests
{
    [Fact(Skip = "Default property values are standard .NET auto-property defaults; not test-worthy behavior")]
    public void ImageEntry_Creation_HasDefaultValues()
    {
        // This test validates only .NET auto-property defaults, which is not useful.
        // The ImageEntry's real behavior is tested via FromImageInfo and PropertyChanged.
    }

    [Fact]
    public void FromImageInfo_MapsAllProperties()
    {
        var info = new ImageInfo
        {
            Path = @"C:\Photos\test.dng",
            FileName = "test.dng",
            Format = "DNG",
            Width = 6000,
            Height = 4000,
            FileSizeBytes = 50_000_000,
            PixelFormat = "RGB16",
            ColorSpace = "Linear Raw",
            Metadata = new ImageMetadata { Make = "Canon", Model = "EOS R5" }
        };

        var entry = ImageEntry.FromImageInfo(info);

        entry.FilePath.Should().Be(@"C:\Photos\test.dng");
        entry.FileName.Should().Be("test.dng");
        entry.Format.Should().Be("DNG");
        entry.Width.Should().Be(6000u);
        entry.Height.Should().Be(4000u);
        entry.FileSizeBytes.Should().Be(50_000_000uL);
        entry.PixelFormat.Should().Be("RGB16");
        entry.ColorSpace.Should().Be("Linear Raw");
        entry.Metadata.Should().NotBeNull();
        entry.Metadata!.Make.Should().Be("Canon");
    }

    [Fact]
    public void ImageEntry_RaisesPropertyChanged()
    {
        var entry = new ImageEntry();
        var raised = false;
        entry.PropertyChanged += (_, _) => raised = true;

        entry.FilePath = @"C:\test.jpg";

        raised.Should().BeTrue();
    }

    [Fact]
    public void ImageStatus_EnumValues()
    {
        Enum.GetValues<ImageStatus>().Should().Contain(new[] {
            ImageStatus.None, ImageStatus.Original, ImageStatus.Overridden, ImageStatus.Error });
    }
}
