using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class FilmstripViewModelTests
{
    private static FilmstripViewModel Create(Mock<IImageService>? imageServiceMock = null)
    {
        var logger = Mock.Of<ILogger<FilmstripViewModel>>();
        var imageService = imageServiceMock?.Object ?? Mock.Of<IImageService>();
        return new FilmstripViewModel(logger, imageService, null!);
    }

    [Fact]
    public void InitialState_EmptyCollections()
    {
        var vm = Create();

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.SelectedImages.Should().BeEmpty();
        vm.IsLoading.Should().BeFalse();
    }

    [Fact]
    public void InitialState_DefaultSettings()
    {
        var vm = Create();

        vm.FilterText.Should().BeEmpty();
        vm.SortBy.Should().Be("Name");
        vm.FilterFormat.Should().Be("All");
        vm.ThumbnailSize.Should().Be(120);
    }

    [Fact]
    public void SortOptions_ContainsExpectedValues()
    {
        var vm = Create();

        vm.SortOptions.Should().Contain(new[] { "Name", "Size", "Format" });
    }

    [Fact]
    public void FormatFilters_ContainsExpectedValues()
    {
        var vm = Create();

        vm.FormatFilters.Should().Contain(new[] { "All", "Raw", "JPEG", "TIFF", "PNG", "HEIF" });
    }

    [Fact]
    public void ThumbnailSizes_HasThreeSizes()
    {
        var vm = Create();

        vm.ThumbnailSizes.Should().Equal(80, 120, 180);
    }

    [Fact]
    public void RemoveImage_Null_Noop()
    {
        var vm = Create();

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().BeEmpty();
    }

    [Fact]
    public void RemoveImage_RemovesFromCollection()
    {
        var vm = Create();
        var img = new ImageEntry { FilePath = "test.jpg", FileName = "test.jpg" };
        vm.Images.Add(img);
        vm.FilteredImages.Add(img);

        vm.RemoveImageCommand.Execute(img);

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
    }

    [Fact]
    public void ClearImages_ResetsAllState()
    {
        var vm = Create();
        vm.Images.Add(new ImageEntry { FileName = "a.jpg" });
        vm.Images.Add(new ImageEntry { FileName = "b.jpg" });

        vm.ClearImagesCommand.Execute(null);

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.SelectedImages.Should().BeEmpty();
    }

    [Fact]
    public void SelectAll_SelectsAllFilteredImages()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "a.jpg", Format = "JPEG" };
        vm.Images.Add(img);
        vm.FilteredImages.Add(img);

        vm.SelectAllCommand.Execute(null);

        vm.SelectedImages.Should().HaveCount(1);
    }

    [Fact]
    public void ClearSelection_RemovesAllSelected()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "a.jpg" };
        vm.Images.Add(img);
        vm.FilteredImages.Add(img);
        vm.SelectAllCommand.Execute(null);

        vm.ClearSelectionCommand.Execute(null);

        vm.SelectedImages.Should().BeEmpty();
    }

    [Fact]
    public void InvertSelection_SelectsUnselected()
    {
        var vm = Create();
        var img1 = new ImageEntry { FileName = "a.jpg" };
        var img2 = new ImageEntry { FileName = "b.jpg" };
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.FilteredImages.Add(img1);
        vm.FilteredImages.Add(img2);

        vm.InvertSelectionCommand.Execute(null);

        vm.SelectedImages.Should().HaveCount(2);
    }

    [Fact]
    public void SetThumbnailSize_ValidSizes()
    {
        var vm = Create();

        vm.SetThumbnailSizeCommand.Execute(80);
        vm.ThumbnailSize.Should().Be(80);

        vm.SetThumbnailSizeCommand.Execute("180");
        vm.ThumbnailSize.Should().Be(180);
    }

    [Fact]
    public void FilterText_FiltersByName()
    {
        var vm = Create();
        vm.Images.Add(new ImageEntry { FileName = "sunset.jpg", Format = "JPEG" });
        vm.Images.Add(new ImageEntry { FileName = "portrait.jpg", Format = "JPEG" });

        vm.FilterText = "sunset";

        vm.FilteredImages.Should().HaveCount(1);
        vm.FilteredImages[0].FileName.Should().Be("sunset.jpg");
    }

    [Fact]
    public void SelectedImageChanged_FiresEvent()
    {
        var vm = Create();
        ImageEntry? received = null;
        vm.ImageSelected += img => received = img;
        var img = new ImageEntry { FileName = "test.jpg" };

        vm.SelectedImage = img;

        received.Should().Be(img);
    }

    [Fact]
    public void CopyPath_NullImage_Noop()
    {
        var vm = Create();

        vm.CopyPathCommand.Execute(null);
    }

    [Fact]
    public void OpenInExplorer_NullImage_Noop()
    {
        var vm = Create();

        vm.OpenInExplorerCommand.Execute(null);
    }
}
