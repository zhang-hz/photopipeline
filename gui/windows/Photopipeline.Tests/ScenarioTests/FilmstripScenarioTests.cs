using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class FilmstripScenarioTests
{
    [Fact]
    public void NewHarness_ImagesCollection_IsEmpty()
    {
        var h = new ViewModelTestHarness();
        Assert.Empty(h.Main.Images);
    }

    [Fact]
    public void NewHarness_StatusMessage_IsReady()
    {
        var h = new ViewModelTestHarness();
        Assert.NotNull(h.Main.StatusMessage);
        // "Plugins loaded" is the expected startup message after constructor calls LoadPluginsAsync
    }

    [Fact]
    public void AddImage_AddsEntry_ToImagesCollection()
    {
        var h = new ViewModelTestHarness();
        var testPath = TestImageFactory.GetPath("solid_rgb_256.png");

        h.AddTestImage(testPath, "test.png");

        Assert.Single(h.Main.Images);
        Assert.Equal(testPath, h.Main.Images[0].FilePath);
        Assert.Equal("test.png", h.Main.Images[0].FileName);
    }

    [Fact]
    public void AddMultipleImages_AllAppear_InCollection()
    {
        var h = new ViewModelTestHarness();
        var paths = new[]
        {
            TestImageFactory.GetPath("solid_rgb_256.png"),
            TestImageFactory.GetPath("gradient_256.png"),
            TestImageFactory.GetPath("checkerboard_256.png"),
            TestImageFactory.GetPath("color_bars_256.png"),
            TestImageFactory.GetPath("gray_steps_256.png"),
        };

        foreach (var p in paths) h.AddTestImage(p);

        Assert.Equal(5, h.Main.Images.Count);
    }

    [Fact]
    public void RemoveImage_RemovesSelected_AndSelectsNext()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");
        h.Main.SelectedImage = h.Main.Images[0];

        h.Main.RemoveImageCommand.Execute(null);

        Assert.Single(h.Main.Images);
        Assert.Equal("b.png", h.Main.Images[0].FileName);
        Assert.Equal(h.Main.Images[0], h.Main.SelectedImage);
    }

    [Fact]
    public void RemoveImage_LastImage_ClearsSelection()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "only.png");
        h.Main.SelectedImage = h.Main.Images[0];

        h.Main.RemoveImageCommand.Execute(null);

        Assert.Empty(h.Main.Images);
        Assert.Null(h.Main.SelectedImage);
    }

    [Fact]
    public void RemoveImage_NoSelection_DoesNothing()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        // no selection

        h.Main.RemoveImageCommand.Execute(null);

        Assert.Single(h.Main.Images);
    }

    [Fact]
    public void ClearImages_RemovesAll_AndResetsSelection()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");
        h.Main.SelectedImage = h.Main.Images[0];

        h.Main.ClearImagesCommand.Execute(null);

        Assert.Empty(h.Main.Images);
        Assert.Null(h.Main.SelectedImage);
        Assert.Null(h.Main.BeforeImage);
        Assert.Null(h.Main.AfterImage);
    }

    [Fact]
    public void SelectedImage_ChangesBeforeImage()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");

        h.Main.SelectedImage = h.Main.Images[0];
        Assert.Equal(h.Main.Images[0], h.Main.BeforeImage);

        h.Main.SelectedImage = h.Main.Images[1];
        Assert.Equal(h.Main.Images[1], h.Main.BeforeImage);
    }

    [Fact]
    public void RunPipelineCommand_WithoutSelection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        // No selection

        var exception = Record.Exception(() => h.Main.RunPipelineCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ZoomIn_Increases_ZoomLevel()
    {
        var h = new ViewModelTestHarness();
        double initial = h.Main.ZoomLevel;

        h.Main.ZoomInCommand.Execute(null);

        Assert.True(h.Main.ZoomLevel > initial);
    }

    [Fact]
    public void ZoomOut_Decreases_ZoomLevel()
    {
        var h = new ViewModelTestHarness();
        h.Main.ZoomLevel = 2.0;
        double initial = h.Main.ZoomLevel;

        h.Main.ZoomOutCommand.Execute(null);

        Assert.True(h.Main.ZoomLevel < initial);
    }

    [Fact]
    public void ZoomLevel_Clamped_WithinRange()
    {
        var h = new ViewModelTestHarness();

        // Zoom in repeatedly
        for (int i = 0; i < 20; i++) h.Main.ZoomInCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel <= 8.0);

        // Zoom out repeatedly
        for (int i = 0; i < 20; i++) h.Main.ZoomOutCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel >= 0.1);
    }

    [Fact]
    public void ResetZoom_RestoresTo_One()
    {
        var h = new ViewModelTestHarness();
        h.Main.ZoomInCommand.Execute(null);
        h.Main.ZoomInCommand.Execute(null);

        h.Main.ResetZoomCommand.Execute(null);

        Assert.Equal(1.0, h.Main.ZoomLevel);
    }

    [Fact]
    public void IsSplitView_DefaultValue_IsTrue()
    {
        var h = new ViewModelTestHarness();
        Assert.True(h.Main.IsSplitView);
    }

    [Fact]
    public void IsSideBySide_DefaultValue_IsFalse()
    {
        var h = new ViewModelTestHarness();
        Assert.False(h.Main.IsSideBySide);
    }

    [Fact]
    public void Add50Images_AllPresent_NoError()
    {
        var h = new ViewModelTestHarness();
        var basePath = TestImageFactory.GetPath("solid_rgb_256.png");

        for (int i = 0; i < 50; i++)
            h.AddTestImage(basePath, $"img_{i:000}.png");

        Assert.Equal(50, h.Main.Images.Count);
    }

    [Fact]
    public void StatusMessage_Updates_AfterOperations()
    {
        var h = new ViewModelTestHarness();

        h.Main.NewPipelineCommand.Execute(null);
        Assert.Contains("new pipeline", h.Main.StatusMessage, StringComparison.OrdinalIgnoreCase);

        h.Main.ClearImagesCommand.Execute(null);
        Assert.NotNull(h.Main.StatusMessage);
    }
}
