using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class ErrorRecoveryScenarioTests
{
    // ═══ Missing/invalid input ═══
    [Fact]
    public void RemoveImage_EmptyCollection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Main.RemoveImageCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void RemoveImage_NoSelection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        var exception = Record.Exception(() => h.Main.RemoveImageCommand.Execute(null));
        Assert.Null(exception);
        Assert.Single(h.Main.Images);
    }

    [Fact]
    public void ClearImages_EmptyCollection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Main.ClearImagesCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void RunPipeline_NoSelection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Main.RunPipelineCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ExportImage_NoSelection_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Main.ExportImageCommand.Execute(null));
        Assert.Null(exception);
    }

    // ═══ Batch edge cases ═══
    [Fact]
    public void StartBatch_EmptyQueue_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Batch.StartBatchCommand.Execute(null));
        Assert.Null(exception);
        Assert.False(h.Batch.IsRunning);
    }

    [Fact]
    public void PauseBatch_NotRunning_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Batch.PauseBatchCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void StopBatch_NotRunning_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Batch.StopBatchCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ResumeBatch_NotPaused_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        var exception = Record.Exception(() => h.Batch.ResumeBatchCommand.Execute(null));
        Assert.Null(exception);
    }

    // ═══ Plugin panel error handling ═══
    [Fact]
    public void LoadPlugins_EmptyList_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.LoadPlugins(new List<PluginInfo>()));
        Assert.Null(exception);
        Assert.Empty(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void ApplyParameters_NoSelectedNode_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PluginPanel.ApplyParametersCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ResetParameters_NoSelectedPlugin_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PluginPanel.ResetParametersCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void SearchText_SpecialCharacters_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(new List<PluginInfo>
        {
            new() { Id = "test", Name = "Test (v1.0)", Category = "Test", Description = "A *test* plugin",
                MinInputs = 1, MaxInputs = 1, Outputs = 1 }
        });
        var exception = Record.Exception(() => h.PluginPanel.SearchText = "^$.*+?{}[]()|\\");
        Assert.Null(exception);
    }

    // ═══ Pipeline editor error handling ═══
    [Fact]
    public void RemoveNode_Null_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.RemoveNodeCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void DuplicateSelected_NothingSelected_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.DuplicateSelectedCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void FitAll_NoNodes_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.FitAllCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ConnectPorts_NoDraggingPort_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.ConnectPortsCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void NodeMouseMove_NoDragging_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(new PluginInfo { Id = "a", Name = "A", Category = "T",
            Description = "", MinInputs = 1, MaxInputs = 1, Outputs = 1 });
        var exception = Record.Exception(() => h.PipelineEditor.OnNodeMouseMove(100, 100));
        Assert.Null(exception);
    }

    // ═══ Recovery after error ═══
    [Fact]
    public void AfterRemoveImage_CanStillAdd()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.RemoveImageCommand.Execute(null);
        Assert.Empty(h.Main.Images);
        // Can still add
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");
        Assert.Single(h.Main.Images);
    }

    [Fact]
    public void AfterClearImages_CanStillAdd()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.ClearImagesCommand.Execute(null);
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");
        Assert.Single(h.Main.Images);
    }

    [Fact]
    public void AfterRunPipeline_Error_CanRetry()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.RunPipelineCommand.Execute(null); // First attempt
        h.Main.RunPipelineCommand.Execute(null); // Retry
        Assert.NotNull(h.Main.StatusMessage);
    }

    [Fact]
    public void AfterStopExecution_CanRestart()
    {
        var h = new ViewModelTestHarness();
        h.Main.StopExecutionCommand.Execute(null);
        Assert.Equal("Stopped", h.Main.StatusMessage);
        // Status can still be updated
        h.Main.NewPipelineCommand.Execute(null);
        Assert.NotEqual("Stopped", h.Main.StatusMessage);
    }

    // ═══ Rapid operations ═══
    [Fact]
    public void RapidAddRemoveImages_NoError()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++)
        {
            h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), $"img_{i}.png");
            if (h.Main.Images.Count > 5)
            {
                h.Main.SelectedImage = h.Main.Images[0];
                h.Main.RemoveImageCommand.Execute(null);
            }
        }
        Assert.NotEmpty(h.Main.Images);
    }

    [Fact]
    public void RapidStartStopBatch_NoError()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        for (int i = 0; i < 10; i++)
        {
            h.Batch.StartBatchCommand.Execute(null);
            h.Batch.StopBatchCommand.Execute(null);
        }
    }

    [Fact]
    public void RapidPauseResume_NoError()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        for (int i = 0; i < 10; i++)
        {
            h.Batch.PauseBatchCommand.Execute(null);
            h.Batch.ResumeBatchCommand.Execute(null);
        }
    }

    // ═══ Zoom recovery ═══
    [Fact]
    public void AfterMaxZoomIn_ZoomOut_Works()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++) h.Main.ZoomInCommand.Execute(null);
        h.Main.ZoomOutCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel < 8.0);
    }

    [Fact]
    public void AfterMinZoomOut_ZoomIn_Works()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++) h.Main.ZoomOutCommand.Execute(null);
        h.Main.ZoomInCommand.Execute(null);
        Assert.True(h.Main.ZoomLevel > 0.1);
    }
}
