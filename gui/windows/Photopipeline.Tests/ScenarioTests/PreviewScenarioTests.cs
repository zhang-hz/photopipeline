using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class PreviewScenarioTests
{
    private static PreviewViewModel Create()
    {
        var logger = Mock.Of<ILogger<PreviewViewModel>>();
        var imageService = Mock.Of<IImageService>();
        var pipelineService = Mock.Of<IPipelineService>();
        return new PreviewViewModel(logger, imageService, pipelineService);
    }

    [Fact]
    public void ZoomIn_StepsThroughPresetValues()
    {
        var vm = Create();

        vm.ZoomInCommand.Execute(null); // 1.0 → 1.5
        vm.ZoomLevel.Should().Be(1.5);

        vm.ZoomInCommand.Execute(null); // 1.5 → 2.0
        vm.ZoomLevel.Should().Be(2.0);

        vm.ZoomInCommand.Execute(null); // 2.0 → 3.0
        vm.ZoomLevel.Should().Be(3.0);
    }

    [Fact]
    public void ZoomOut_StepsThroughPresetValues()
    {
        var vm = Create();
        vm.ZoomLevel = 4.0;

        vm.ZoomOutCommand.Execute(null); // 4.0 → 3.0
        vm.ZoomLevel.Should().Be(3.0);

        vm.ZoomOutCommand.Execute(null); // 3.0 → 2.0
        vm.ZoomLevel.Should().Be(2.0);
    }

    [Fact]
    public void PanAndZoom_WorksTogether()
    {
        var vm = Create();

        vm.ZoomInCommand.Execute(null);
        vm.Pan(50, -30);

        vm.ZoomLevel.Should().Be(1.5);
        vm.PanOffset.X.Should().Be(50f);
        vm.PanOffset.Y.Should().Be(-30f);
    }

    [Fact]
    public void SplitView_ToggleAndPosition()
    {
        var vm = Create();

        vm.ToggleSplitCommand.Execute(null);
        vm.IsSplitView.Should().BeFalse();
        vm.SplitPosition = 0.75;

        vm.ToggleSplitCommand.Execute(null);
        vm.IsSplitView.Should().BeTrue();
        vm.SplitPosition.Should().Be(0.75);
    }

    [Fact]
    public void FitToWindow_ResetZoom_Interaction()
    {
        var vm = Create();
        vm.ZoomInCommand.Execute(null);
        vm.ZoomInCommand.Execute(null);

        vm.FitToWindowCommand.Execute(null);
        vm.IsFitToWindow.Should().BeTrue();

        vm.ResetZoomCommand.Execute(null);
        vm.ZoomLevel.Should().Be(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }

    [Fact]
    public void OneToOne_AtHighZoom_Resets()
    {
        var vm = Create();
        vm.ZoomLevel = 8.0;
        vm.IsFitToWindow = true;

        vm.OneToOneCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
        vm.IsFitToWindow.Should().BeFalse();
    }
}
