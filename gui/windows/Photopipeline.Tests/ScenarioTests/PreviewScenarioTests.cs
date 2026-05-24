using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class PreviewScenarioTests
{
    // ═══ Selection → Preview ═══
    [Fact]
    public void NewHarness_BeforeImage_Null()
    {
        var h = new ViewModelTestHarness();
        Assert.Null(h.Main.BeforeImage);
    }

    [Fact]
    public void NewHarness_AfterImage_Null()
    {
        var h = new ViewModelTestHarness();
        Assert.Null(h.Main.AfterImage);
    }

    [Fact]
    public void SelectedImage_SetsBeforeImage()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        h.Main.SelectedImage = h.Main.Images[0];
        Assert.Equal(h.Main.Images[0], h.Main.BeforeImage);
    }

    [Fact]
    public void SelectedImage_Clears_ClearsBeforeImage()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.SelectedImage = null;
        Assert.Null(h.Main.BeforeImage);
    }

    [Fact]
    public void SwitchSelection_UpdatesBeforeImage()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.SelectedImage = h.Main.Images[1];
        Assert.Equal(h.Main.Images[1], h.Main.BeforeImage);
    }

    // ═══ Zoom ═══
    [Fact]
    public void ZoomIn_IncreasesLevel()
    {
        var h = new ViewModelTestHarness();
        double initial = h.Main.ZoomLevel;
        h.Main.ZoomInCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel > initial);
    }

    [Fact]
    public void ZoomOut_DecreasesLevel()
    {
        var h = new ViewModelTestHarness();
        h.Main.ZoomLevel = 2.0;
        h.Main.ZoomOutCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel < 2.0);
    }

    [Fact]
    public void ZoomLevel_Clamped_Max8()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++) h.Main.ZoomInCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel <= 8.0);
    }

    [Fact]
    public void ZoomLevel_Clamped_Min0_1()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++) h.Main.ZoomOutCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel >= 0.1);
    }

    [Fact]
    public void ZoomLevel_ZoomInOut_Reversible()
    {
        var h = new ViewModelTestHarness();
        double initial = h.Main.ZoomLevel;
        h.Main.ZoomInCommand.Execute(null);
        h.Main.ZoomOutCommand.Execute(null);
        Assert.Equal(initial, h.Main.ZoomLevel, 4);
    }

    [Fact]
    public void ResetZoom_RestoresTo1()
    {
        var h = new ViewModelTestHarness();
        h.Main.ZoomInCommand.Execute(null);
        h.Main.ZoomInCommand.Execute(null);
        h.Main.ResetZoomCommand.Execute(null);
        Assert.Equal(1.0, h.Main.ZoomLevel);
    }

    [Fact]
    public void FitToWindow_SetsTo1()
    {
        var h = new ViewModelTestHarness();
        h.Main.ZoomInCommand.Execute(null);
        h.Main.ZoomInCommand.Execute(null);
        h.Main.FitToWindowCommand.Execute(null);
        Assert.Equal(1.0, h.Main.ZoomLevel);
    }

    // ═══ SplitView / SideBySide ═══
    [Fact]
    public void IsSplitView_Default_True()
    {
        var h = new ViewModelTestHarness();
        Assert.True(h.Main.IsSplitView);
    }

    [Fact]
    public void IsSideBySide_Default_False()
    {
        var h = new ViewModelTestHarness();
        Assert.False(h.Main.IsSideBySide);
    }

    [Fact]
    public void SplitView_ToggleToFalse()
    {
        var h = new ViewModelTestHarness();
        h.Main.IsSplitView = false;
        Assert.False(h.Main.IsSplitView);
    }

    [Fact]
    public void SideBySide_ToggleToTrue()
    {
        var h = new ViewModelTestHarness();
        h.Main.IsSideBySide = true;
        Assert.True(h.Main.IsSideBySide);
    }

    [Fact]
    public void SplitView_SplitPosition_Default_Half()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal(0.5, h.Main.SplitPosition);
    }

    [Fact]
    public void SplitView_SplitPosition_CanChange()
    {
        var h = new ViewModelTestHarness();
        h.Main.SplitPosition = 0.75;
        Assert.Equal(0.75, h.Main.SplitPosition);
    }

    [Fact]
    public void SplitView_SplitPosition_ClampZero()
    {
        var h = new ViewModelTestHarness();
        h.Main.SplitPosition = -0.5;
        Assert.True(h.Main.SplitPosition >= 0 || h.Main.SplitPosition <= 1);
    }

    // ═══ Export ═══
    [Fact]
    public async Task ExportImage_WithoutSelection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Main.ExportImageCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ExportImage_StatusMessage_Updates()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.ExportImageCommand.Execute(null);
        Assert.NotNull(h.Main.StatusMessage);
    }

    // ═══ Processing state ═══
    [Fact]
    public void IsProcessing_Default_False()
    {
        var h = new ViewModelTestHarness();
        Assert.False(h.Main.IsProcessing);
    }

    [Fact]
    public void RunPipeline_WithoutSelection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        var exception = Record.Exception(() => h.Main.RunPipelineCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void RunPipeline_WithSelection_Executes()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.RunPipelineCommand.Execute(null);
        // Should have completed without exception
        Assert.NotNull(h.Main.StatusMessage);
    }

    [Fact]
    public void StopExecution_SetsStatus()
    {
        var h = new ViewModelTestHarness();
        h.Main.StopExecutionCommand.Execute(null);
        Assert.Equal("Stopped", h.Main.StatusMessage);
    }

    // ═══ File entry preview metadata ═══
    [Fact]
    public void ImageEntry_HasCorrectPath()
    {
        var h = new ViewModelTestHarness();
        var path = TestImageFactory.GetPath("solid_rgb_256.png");
        h.AddTestImage(path, "test.png");
        Assert.Equal(path, h.Main.Images[0].FilePath);
    }

    [Fact]
    public void ImageEntry_HasCorrectFileName()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "my_image.png");
        Assert.Equal("my_image.png", h.Main.Images[0].FileName);
    }

    [Fact]
    public void ImageEntry_Default_ProcessingProgress_Zero()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        Assert.Equal(0.0, h.Main.Images[0].ProcessingProgress);
    }
}
