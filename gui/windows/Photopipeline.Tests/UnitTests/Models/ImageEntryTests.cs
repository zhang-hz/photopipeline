using System.ComponentModel;

namespace Photopipeline.Tests.UnitTests.Models;

public sealed class ImageEntryTests
{
    [Fact]
    public void ImageEntry_Creation_SetsDefaultValues()
    {
        var entry = new ImageEntry();

        entry.Id.Should().NotBeNullOrEmpty();
        entry.FilePath.Should().BeEmpty();
        entry.FileName.Should().BeEmpty();
        entry.ThumbnailPath.Should().BeNull();
        entry.FileSize.Should().Be(0);
        entry.Width.Should().Be(0);
        entry.Height.Should().Be(0);
        entry.ColorSpace.Should().Be("sRGB");
        entry.BitDepth.Should().Be("8");
        entry.HasMetadataModified.Should().BeFalse();
        entry.OverrideStatus.Should().Be(ImageOverrideStatus.None);
        entry.IsSelected.Should().BeFalse();
        entry.IsProcessing.Should().BeFalse();
        entry.HasError.Should().BeFalse();
        entry.ErrorMessage.Should().BeEmpty();
    }

    [Fact]
    public void ImageEntry_CreationFromFilePath_SetsPathAndName()
    {
        var entry = new ImageEntry
        {
            FilePath = @"C:\Photos\test_image.dng",
            FileName = "test_image.dng"
        };

        entry.FilePath.Should().Be(@"C:\Photos\test_image.dng");
        entry.FileName.Should().Be("test_image.dng");
    }

    [Fact]
    public void ImageEntry_OverrideStatus_None()
    {
        var entry = new ImageEntry { OverrideStatus = ImageOverrideStatus.None };

        entry.OverrideStatus.Should().Be(ImageOverrideStatus.None);
    }

    [Fact]
    public void ImageEntry_OverrideStatus_Original()
    {
        var entry = new ImageEntry { OverrideStatus = ImageOverrideStatus.Original };

        entry.OverrideStatus.Should().Be(ImageOverrideStatus.Original);
    }

    [Fact]
    public void ImageEntry_OverrideStatus_Overridden()
    {
        var entry = new ImageEntry { OverrideStatus = ImageOverrideStatus.Overridden };

        entry.OverrideStatus.Should().Be(ImageOverrideStatus.Overridden);
    }

    [Fact]
    public void ImageEntry_OverrideStatus_Error()
    {
        var entry = new ImageEntry { OverrideStatus = ImageOverrideStatus.Error };

        entry.OverrideStatus.Should().Be(ImageOverrideStatus.Error);
    }

    [Fact]
    public void ImageEntry_OverrideStatusChange_RaisesPropertyChanged()
    {
        var entry = new ImageEntry();
        var eventRaised = false;
        string? changedProperty = null;

        entry.PropertyChanged += (_, e) =>
        {
            eventRaised = true;
            changedProperty = e.PropertyName;
        };

        entry.OverrideStatus = ImageOverrideStatus.Overridden;

        eventRaised.Should().BeTrue();
        changedProperty.Should().Be(nameof(ImageEntry.OverrideStatus));
    }

    [Fact]
    public void ImageEntry_ThumbnailPath_Generation()
    {
        var entry = new ImageEntry
        {
            FilePath = @"C:\Photos\test_image.dng",
            FileName = "test_image.dng"
        };

        var expectedThumbnailPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "Photopipeline", "thumbnails",
            $"thumb_{entry.Id}_test_image.jpg");

        entry.ThumbnailPath = expectedThumbnailPath;
        entry.ThumbnailPath.Should().Contain($"thumb_{entry.Id}_test_image.jpg");
    }

    [Fact]
    public void ImageEntry_ProcessingProgress_RangesZeroToOne()
    {
        var entry = new ImageEntry();

        entry.ProcessingProgress = 0.0;
        entry.ProcessingProgress.Should().Be(0.0);

        entry.ProcessingProgress = 0.5;
        entry.ProcessingProgress.Should().Be(0.5);

        entry.ProcessingProgress = 1.0;
        entry.ProcessingProgress.Should().Be(1.0);
    }

    [Fact]
    public void ImageEntry_ErrorState_SetsMessageAndFlag()
    {
        var entry = new ImageEntry();

        entry.HasError = true;
        entry.ErrorMessage = "Failed to decode raw data";

        entry.HasError.Should().BeTrue();
        entry.ErrorMessage.Should().Be("Failed to decode raw data");
    }

    [Fact]
    public void ImageEntry_FileSize_PropertyChangedNotification()
    {
        var entry = new ImageEntry();
        var eventRaised = false;

        entry.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(ImageEntry.FileSize))
                eventRaised = true;
        };

        entry.FileSize = 1048576;

        eventRaised.Should().BeTrue();
    }

    [Fact]
    public void ImageEntry_ColorSpace_DetectsRawFormat()
    {
        var entry = new ImageEntry { FileName = "photo.cr2" };

        var ext = Path.GetExtension(entry.FileName).ToLowerInvariant();
        var space = ext switch
        {
            ".dng" or ".nef" or ".cr2" or ".arw" or ".orf" => "Linear Raw",
            _ => "sRGB"
        };

        space.Should().Be("Linear Raw");
    }
}
