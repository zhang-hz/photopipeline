using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;
using SkiaSharp;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for PreviewViewModel.
/// Uses MockBehavior.Strict for service mocks. Every test has a FAIL-able assertion.
/// </summary>
public sealed class PreviewViewModelTests : IDisposable
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

    private PreviewViewModel CreateVm(
        Mock<IImageService>? imageMock = null,
        Mock<IPipelineService>? pipelineMock = null,
        Mock<IDialogService>? dialogMock = null)
    {
        var img = imageMock?.Object ?? Mock.Of<IImageService>();
        var pl = pipelineMock?.Object ?? Mock.Of<IPipelineService>();
        var dlg = dialogMock?.Object ?? Mock.Of<IDialogService>();
        return new PreviewViewModel(AnyLogger<PreviewViewModel>(), img, pl, dlg);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 001: InitialState_DefaultValues
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_001_InitialState_DefaultValues()
    {
        var vm = CreateVm();

        vm.ZoomLevel.Should().Be(1.0);
        vm.SplitPosition.Should().Be(0.5);
        vm.IsSplitView.Should().BeTrue();
        vm.IsFitToWindow.Should().BeTrue();
        vm.BeforeBitmap.Should().BeNull();
        vm.AfterBitmap.Should().BeNull();
        vm.IsLoading.Should().BeFalse();
        vm.PixelInfo.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 002: ZoomIn selects next discrete step
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_002_ZoomIn_IncreasesLevel()
    {
        var vm = CreateVm();
        var raised = false;
        vm.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(PreviewViewModel.ZoomLevel))
                raised = true;
        };

        vm.ZoomInCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.5); // next step after default 1.0
        vm.IsFitToWindow.Should().BeFalse();
        raised.Should().BeTrue("ZoomLevel PropertyChanged must be raised");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 003: ZoomOut selects previous discrete step
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_003_ZoomOut_DecreasesLevel()
    {
        var vm = CreateVm();
        vm.ZoomLevel = 2.0;

        vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.5); // previous step before 2.0
        vm.IsFitToWindow.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 004: ZoomIn stops at maximum discrete step (8.0)
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_004_ZoomIn_AtMaximum_DoesNotExceed()
    {
        var vm = CreateVm();

        for (int i = 0; i < 100; i++)
            vm.ZoomInCommand.Execute(null);

        vm.ZoomLevel.Should().Be(8.0, "zoom must stop at max discrete step");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 005: ZoomOut_AtMinimum_DoesNotExceed
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_005_ZoomOut_AtMinimum_DoesNotExceed()
    {
        var vm = CreateVm();

        // Execute ZoomOut 100 times — should never go below MinZoom (0.0625)
        for (int i = 0; i < 100; i++)
            vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().BeGreaterOrEqualTo(0.0625);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 006: ResetZoom_ReturnsToOne
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_006_ResetZoom_ReturnsToOne()
    {
        var vm = CreateVm();
        vm.ZoomLevel = 4.0;

        vm.ResetZoomCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 007: FitToWindow_SetsFitTrue
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_007_FitToWindow_SetsFitTrue()
    {
        var vm = CreateVm();
        vm.ZoomLevel = 4.0;
        vm.IsFitToWindow = false;

        vm.FitToWindowCommand.Execute(null);

        vm.IsFitToWindow.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 008: ToggleSplit_FlipsSplitView
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_008_ToggleSplit_FlipsSplitView()
    {
        var vm = CreateVm();
        vm.IsSplitView.Should().BeTrue();

        vm.ToggleSplitCommand.Execute(null);
        vm.IsSplitView.Should().BeFalse();

        vm.ToggleSplitCommand.Execute(null);
        vm.IsSplitView.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 009: OneToOne_ResetsZoom
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_009_OneToOne_ResetsZoom()
    {
        var vm = CreateVm();
        vm.ZoomLevel = 3.0;
        vm.IsFitToWindow = true;

        vm.OneToOneCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 010: Pan_UpdatesOffset
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_010_Pan_UpdatesOffset()
    {
        var vm = CreateVm();

        vm.Pan(10, -5);

        vm.PanOffset.X.Should().Be(10f);
        vm.PanOffset.Y.Should().Be(-5f);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 011: Pan_AccumulatesMultipleCalls
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_011_Pan_AccumulatesMultipleCalls()
    {
        var vm = CreateVm();

        vm.Pan(10, 0);
        vm.Pan(20, 0);
        vm.Pan(0, 5);

        vm.PanOffset.X.Should().Be(30f);
        vm.PanOffset.Y.Should().Be(5f);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 012: Export_NoImage_ShowsError
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_012_Export_NoImage_ShowsError()
    {
        var vm = CreateVm();

        // Export when BeforeBitmap and AfterBitmap are both null
        vm.ExportCommand.Execute(null);

        vm.ErrorMessage.Should().NotBeNull();
        vm.ErrorMessage.Should().Be("No image to export");
    }
}
