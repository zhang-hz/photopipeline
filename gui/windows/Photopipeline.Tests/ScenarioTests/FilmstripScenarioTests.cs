using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class FilmstripScenarioTests
{
    private static FilmstripViewModel Create()
    {
        var logger = Mock.Of<ILogger<FilmstripViewModel>>();
        var imageService = Mock.Of<IImageService>();
        return new FilmstripViewModel(logger, imageService, null!);
    }

    [Fact]
    public void BatchImport_Images_AppearInCollection()
    {
        var vm = Create();
        vm.Images.Add(new ImageEntry { FileName = "img1.dng", Format = "DNG" });
        vm.Images.Add(new ImageEntry { FileName = "img2.jpg", Format = "JPEG" });
        vm.Images.Add(new ImageEntry { FileName = "img3.tif", Format = "TIFF" });

        vm.Images.Should().HaveCount(3);
    }

    [Fact]
    public void FilterByText_PartialNameMatch()
    {
        var vm = Create();
        vm.Images.Add(new ImageEntry { FileName = "sunset_beach.jpg" });
        vm.Images.Add(new ImageEntry { FileName = "sunrise_mountain.jpg" });
        vm.Images.Add(new ImageEntry { FileName = "portrait.jpg" });

        vm.FilterText = "sun";

        vm.FilteredImages.Should().HaveCount(2);
    }

    [Fact]
    public void FilterAndSort_CombinedOperation()
    {
        var vm = Create();
        vm.Images.Add(new ImageEntry { FileName = "c.dng", FileSizeBytes = 3000 });
        vm.Images.Add(new ImageEntry { FileName = "a.dng", FileSizeBytes = 1000 });
        vm.Images.Add(new ImageEntry { FileName = "b.dng", FileSizeBytes = 2000 });

        vm.SortBy = "Size";

        vm.FilteredImages.Should().HaveCount(3);
        vm.FilteredImages[0].FileSizeBytes.Should().Be(3000);
        vm.FilteredImages[2].FileSizeBytes.Should().Be(1000);
    }

    [Fact]
    public void MultiSelect_ThenRemove()
    {
        var vm = Create();
        var img1 = new ImageEntry { FileName = "a.dng" };
        var img2 = new ImageEntry { FileName = "b.dng" };
        var img3 = new ImageEntry { FileName = "c.dng" };
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.Images.Add(img3);
        vm.FilteredImages.Add(img1);
        vm.FilteredImages.Add(img2);
        vm.FilteredImages.Add(img3);

        vm.SelectAllCommand.Execute(null);
        vm.SelectedImages.Should().HaveCount(3);

        vm.ClearSelectionCommand.Execute(null);
        vm.SelectedImages.Should().BeEmpty();
    }

    [Fact]
    public void RemoveImage_AlsoRemovesFromFiltered()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "test.dng" };
        vm.Images.Add(img);

        vm.RemoveImageCommand.Execute(img);

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
    }

    [Fact]
    public void ClearImages_ResetsFilteredView()
    {
        var vm = Create();
        vm.Images.Add(new ImageEntry { FileName = "test.jpg" });
        vm.FilterText = "test";

        vm.ClearImagesCommand.Execute(null);

        vm.FilteredImages.Should().BeEmpty();
    }
}
