using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class PreviewViewModelTests
{
    private static PreviewViewModel Create(
        Mock<IImageService>? imageServiceMock = null,
        Mock<IPipelineService>? pipelineServiceMock = null)
    {
        var logger = Mock.Of<ILogger<PreviewViewModel>>();
        var imageService = imageServiceMock?.Object ?? Mock.Of<IImageService>();
        var pipelineService = pipelineServiceMock?.Object ?? Mock.Of<IPipelineService>();
        return new PreviewViewModel(logger, imageService, pipelineService);
    }

    [Fact]
    public void InitialState_DefaultValues()
    {
        var vm = Create();

        vm.ZoomLevel.Should().Be(1.0);
        vm.SplitPosition.Should().Be(0.5);
        vm.IsSplitView.Should().BeTrue();
        vm.IsFitToWindow.Should().BeTrue();
        vm.BeforeBitmap.Should().BeNull();
        vm.AfterBitmap.Should().BeNull();
        vm.IsLoading.Should().BeFalse();
        vm.PixelInfo.Should().BeEmpty();
    }

    [Fact]
    public void ZoomIn_IncreasesLevel()
    {
        var vm = Create();

        vm.ZoomInCommand.Execute(null);

        vm.ZoomLevel.Should().BeGreaterThan(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }

    [Fact]
    public void ZoomOut_DecreasesLevel()
    {
        var vm = Create();
        vm.ZoomLevel = 2.0;

        vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().BeLessThan(2.0);
    }

    [Fact]
    public void ZoomOut_AtMinimum_DoesNotExceed()
    {
        var vm = Create();

        for (int i = 0; i < 100; i++)
            vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().BeGreaterOrEqualTo(0.0625);
    }

    [Fact]
    public void ZoomIn_AtMaximum_DoesNotExceed()
    {
        var vm = Create();

        for (int i = 0; i < 100; i++)
            vm.ZoomInCommand.Execute(null);

        vm.ZoomLevel.Should().BeLessOrEqualTo(32.0);
    }

    [Fact]
    public void ResetZoom_ReturnsToOne()
    {
        var vm = Create();
        vm.ZoomLevel = 4.0;

        vm.ResetZoomCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }

    [Fact]
    public void FitToWindow_SetsFitTrue()
    {
        var vm = Create();
        vm.ZoomLevel = 4.0;
        vm.IsFitToWindow = false;

        vm.FitToWindowCommand.Execute(null);

        vm.IsFitToWindow.Should().BeTrue();
    }

    [Fact]
    public void ToggleSplit_FlipsSplitView()
    {
        var vm = Create();

        vm.ToggleSplitCommand.Execute(null);
        vm.IsSplitView.Should().BeFalse();

        vm.ToggleSplitCommand.Execute(null);
        vm.IsSplitView.Should().BeTrue();
    }

    [Fact]
    public void OneToOne_ResetsZoom()
    {
        var vm = Create();
        vm.ZoomLevel = 3.0;
        vm.IsFitToWindow = true;

        vm.OneToOneCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }

    [Fact]
    public void Pan_UpdatesOffset()
    {
        var vm = Create();

        vm.Pan(10, -5);

        vm.PanOffset.X.Should().Be(10f);
        vm.PanOffset.Y.Should().Be(-5f);
    }

    [Fact]
    public void Pan_AccumulatesMultipleCalls()
    {
        var vm = Create();

        vm.Pan(10, 0);
        vm.Pan(20, 0);

        vm.PanOffset.X.Should().Be(30f);
    }
}
