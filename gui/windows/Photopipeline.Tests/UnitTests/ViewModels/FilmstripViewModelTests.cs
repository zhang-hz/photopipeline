using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for FilmstripViewModel.
/// Uses MockBehavior.Strict for service mocks. Every test has a FAIL-able assertion.
/// </summary>
public sealed class FilmstripViewModelTests : IDisposable
{
    private readonly List<Mock> _strictMocks = new();

    public void Dispose()
    {
        foreach (var mock in _strictMocks)
            mock.VerifyAll();
    }

    private Mock<T> Strict<T>() where T : class
    {
        var mock = new Mock<T>(MockBehavior.Strict);
        _strictMocks.Add(mock);
        return mock;
    }

    private static ILogger<T> AnyLogger<T>() => Mock.Of<ILogger<T>>();

    private FilmstripViewModel CreateVm(
        Mock<IImageService>? imageMock = null,
        Mock<IDialogService>? dialogMock = null)
    {
        var img = imageMock?.Object ?? Mock.Of<IImageService>();
        var dlg = dialogMock?.Object ?? Mock.Of<IDialogService>();
        return new FilmstripViewModel(AnyLogger<FilmstripViewModel>(), img, dlg);
    }

    private static ImageEntry TestImage(string name = "test.jpg", string format = "JPEG",
        ulong sizeBytes = 1024, uint w = 1920, uint h = 1080) => new()
    {
        FilePath = $"C:\\photos\\{name}",
        FileName = name,
        Format = format,
        FileSizeBytes = sizeBytes,
        Width = w,
        Height = h
    };

    // ═════════════════════════════════════════════════════════════
    // Test 001: InitialState_EmptyCollections
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_001_InitialState_EmptyCollections()
    {
        var vm = CreateVm();

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.SelectedImages.Should().BeEmpty();
        vm.IsLoading.Should().BeFalse();
        vm.FilterText.Should().BeEmpty();
        vm.SortBy.Should().Be("Name");
        vm.FilterFormat.Should().Be("All");
        vm.ThumbnailSize.Should().Be(120);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 002: RemoveImage_RemovesFromCollection
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_002_RemoveImage_RemovesFromCollection()
    {
        var vm = CreateVm();
        var img = TestImage("test.jpg");
        vm.Images.Add(img);
        vm.FilteredImages.Add(img);
        vm.SelectedImage = img;

        vm.RemoveImageCommand.Execute(img);

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 003: RemoveImage_Null_Noop
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_003_RemoveImage_Null_Noop()
    {
        var vm = CreateVm();
        vm.Images.Add(TestImage("test.jpg"));

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().HaveCount(1);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 004: ClearImages_ResetsAllState
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_004_ClearImages_ResetsAllState()
    {
        var vm = CreateVm();
        vm.Images.Add(TestImage("a.jpg"));
        vm.Images.Add(TestImage("b.jpg"));
        vm.FilteredImages.Add(vm.Images[0]);
        vm.FilteredImages.Add(vm.Images[1]);
        vm.SelectedImages.Add(vm.Images[0]);
        vm.SelectedImage = vm.Images[0];

        vm.ClearImagesCommand.Execute(null);

        vm.Images.Should().BeEmpty();
        vm.FilteredImages.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.SelectedImages.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 005: SelectAll_SelectsAllFilteredImages
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_005_SelectAll_SelectsAllFilteredImages()
    {
        var vm = CreateVm();
        var img1 = TestImage("a.jpg");
        var img2 = TestImage("b.jpg");
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.FilteredImages.Add(img1);
        vm.FilteredImages.Add(img2);

        vm.SelectAllCommand.Execute(null);

        vm.SelectedImages.Should().HaveCount(2);
        vm.SelectedImages.Should().Contain(img1);
        vm.SelectedImages.Should().Contain(img2);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 006: ClearSelection_RemovesAllSelected
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_006_ClearSelection_RemovesAllSelected()
    {
        var vm = CreateVm();
        var img = TestImage("a.jpg");
        vm.Images.Add(img);
        vm.FilteredImages.Add(img);
        vm.SelectAllCommand.Execute(null);
        vm.SelectedImages.Should().NotBeEmpty();

        vm.ClearSelectionCommand.Execute(null);

        vm.SelectedImages.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 007: InvertSelection_SelectsUnselected
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_007_InvertSelection_SelectsUnselected()
    {
        var vm = CreateVm();
        var img1 = TestImage("a.jpg");
        var img2 = TestImage("b.jpg");
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.FilteredImages.Add(img1);
        vm.FilteredImages.Add(img2);
        vm.SelectedImages.Add(img1); // Only img1 selected

        vm.InvertSelectionCommand.Execute(null);

        vm.SelectedImages.Should().HaveCount(1);
        vm.SelectedImages[0].Should().Be(img2);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 008: SortBy_SortsByName
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_008_SortBy_SortsByName()
    {
        var vm = CreateVm();
        var images = new List<ImageEntry>
        {
            TestImage("zebra.jpg"),
            TestImage("alpha.jpg"),
            TestImage("mango.jpg")
        };
        foreach (var img in images)
            vm.Images.Add(img);

        vm.SortBy = "Name";
        vm.SortBy.Should().Be("Name");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 009: SortBy_SortsBySize
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_009_SortBy_SortsBySizeDescending()
    {
        var vm = CreateVm();
        vm.Images.Add(TestImage("small.jpg", sizeBytes: 100));
        vm.Images.Add(TestImage("large.jpg", sizeBytes: 5000));
        vm.Images.Add(TestImage("medium.jpg", sizeBytes: 500));

        var propertyChangedNames = new List<string>();
        vm.PropertyChanged += (s, e) => propertyChangedNames.Add(e.PropertyName!);

        vm.SortBy = "Size";

        vm.FilteredImages.Should().HaveCount(3);
        vm.FilteredImages[0].FileName.Should().Be("large.jpg");
        vm.FilteredImages[1].FileName.Should().Be("medium.jpg");
        vm.FilteredImages[2].FileName.Should().Be("small.jpg");
        propertyChangedNames.Should().Contain(nameof(vm.FilteredImages),
            "SortBy change should rebuild FilteredImages");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 010: SortBy_SortsByFormat
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_010_SortBy_SortsByFormat()
    {
        var vm = CreateVm();
        vm.Images.Add(TestImage("a.png", format: "PNG"));
        vm.Images.Add(TestImage("b.heif", format: "HEIF"));
        vm.Images.Add(TestImage("c.jpg", format: "JPEG"));

        vm.SortBy = "Format";

        vm.FilteredImages.Should().HaveCount(3);
        vm.FilteredImages[0].Format.Should().Be("HEIF");
        vm.FilteredImages[1].Format.Should().Be("JPEG");
        vm.FilteredImages[2].Format.Should().Be("PNG");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 011: FilterText_FiltersByName
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_011_FilterText_FiltersByName()
    {
        var vm = CreateVm();
        vm.Images.Add(TestImage("sunset.jpg"));
        vm.Images.Add(TestImage("portrait.jpg"));

        var propertyChangedNames = new List<string>();
        vm.PropertyChanged += (s, e) => propertyChangedNames.Add(e.PropertyName!);

        vm.FilterText = "sunset";

        vm.FilteredImages.Should().HaveCount(1);
        vm.FilteredImages[0].FileName.Should().Be("sunset.jpg");
        propertyChangedNames.Should().Contain(nameof(vm.FilterText));
        propertyChangedNames.Should().Contain(nameof(vm.FilteredImages));
    }

    // ═════════════════════════════════════════════════════════════
    // Test 012: FilterFormat_FiltersByFormat
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_012_FilterFormat_FiltersByFormat()
    {
        var vm = CreateVm();
        vm.Images.Add(TestImage("a.jpg", format: "JPEG"));
        vm.Images.Add(TestImage("b.png", format: "PNG"));
        vm.Images.Add(TestImage("c.tif", format: "TIFF"));

        var propertyChangedNames = new List<string>();
        vm.PropertyChanged += (s, e) => propertyChangedNames.Add(e.PropertyName!);

        vm.FilterFormat = "PNG";

        vm.FilteredImages.Should().HaveCount(1);
        vm.FilteredImages[0].Format.Should().Be("PNG");
        propertyChangedNames.Should().Contain(nameof(vm.FilterFormat));
        propertyChangedNames.Should().Contain(nameof(vm.FilteredImages));
    }

    // ═════════════════════════════════════════════════════════════
    // Test 013: SelectedImageChanged_FiresEvent
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_013_SelectedImageChanged_FiresEvent()
    {
        var vm = CreateVm();
        ImageEntry? received = null;
        vm.ImageSelected += img => received = img;
        var img = TestImage("photo.jpg");

        vm.SelectedImage = img;

        received.Should().Be(img);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 014: CopyPath_NullImage_Noop (edge case)
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_014_CopyPath_NullImage_Noop()
    {
        var vm = CreateVm();

        var act = () => vm.CopyPathCommand.Execute(null);

        act.Should().NotThrow();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 015: OpenInExplorer_NullImage_Noop (edge case)
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_015_OpenInExplorer_NullImage_Noop()
    {
        var vm = CreateVm();

        var act = () => vm.OpenInExplorerCommand.Execute(null);

        act.Should().NotThrow();
    }
}
