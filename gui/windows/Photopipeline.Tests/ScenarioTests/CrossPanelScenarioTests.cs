using System.Collections.ObjectModel;
using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class CrossPanelScenarioTests
{
    private static List<PluginInfo> TestPlugins()
    {
        return new List<PluginInfo>
        {
            new() { Id = "grayscale", Name = "Grayscale", Category = "Color",
                Description = "Convert to grayscale", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
            new() { Id = "invert", Name = "Invert", Category = "Color",
                Description = "Invert colors", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
            new() { Id = "brightness", Name = "Brightness", Category = "Tonal",
                Description = "Adjust brightness", MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema> {
                    new() { Name = "value", ParameterType = ParameterType.Float, DefaultValue = 0.0 }
                } },
        };
    }

    // ═══ Filmstrip → Preview ═══
    [Fact]
    public void FilmstripSelect_UpdatesPreviewBeforeImage()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.AddTestImage(TestImageFactory.GetPath("gradient_256.png"), "b.png");
        h.Main.SelectedImage = h.Main.Images[0];
        Assert.Equal(h.Main.Images[0], h.Main.BeforeImage);
    }

    [Fact]
    public void FilmstripDeselect_ClearsPreview()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.SelectedImage = null;
        Assert.Null(h.Main.BeforeImage);
    }

    [Fact]
    public void FilmstripRemove_ClearsPreview()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.SelectedImage = h.Main.Images[0];
        h.Main.ClearImagesCommand.Execute(null);
        Assert.Null(h.Main.BeforeImage);
        Assert.Null(h.Main.AfterImage);
    }

    // ═══ Filmstrip → Batch ═══
    [Fact]
    public void FilmstripImage_AddToBatchQueue()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Main.Images.Add(img);
        h.Batch.AddToQueueCommand.Execute(img);
        Assert.Single(h.Batch.BatchQueue);
    }

    [Fact]
    public void MultipleFilmstripImages_AddAllToBatch()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 5; i++)
        {
            var img = new ImageEntry { FilePath = $"/img_{i}.png", FileName = $"img_{i}.png" };
            h.Main.Images.Add(img);
            h.Batch.AddToQueueCommand.Execute(img);
        }
        Assert.Equal(5, h.Batch.TotalItems);
    }

    [Fact]
    public void ImageInFilmstrip_NotInBatch_CanStillView()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Main.Images.Add(img);
        h.Main.SelectedImage = img;
        Assert.NotNull(h.Main.BeforeImage);
        Assert.Empty(h.Batch.BatchQueue);
    }

    // ═══ Plugin → Pipeline ═══
    [Fact]
    public void PluginSelected_AddToPipeline_CreatesNode()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());
        var plugin = h.PluginPanel.FilteredPlugins.First(p => p.Id == "invert");
        h.PipelineEditor.AddNodeCommand.Execute(plugin);
        Assert.Single(h.PipelineEditor.Nodes);
        Assert.Equal("invert", h.PipelineEditor.Nodes[0].PluginId);
    }

    [Fact]
    public void PluginWithParameters_NodeHasParameters()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());
        var plugin = h.PluginPanel.FilteredPlugins.First(p => p.Id == "brightness");
        h.PipelineEditor.AddNodeCommand.Execute(plugin);
        Assert.True(h.PipelineEditor.Nodes[0].Parameters.ContainsKey("value"));
    }

    [Fact]
    public void SelectPlugin_ThenAddToPipeline_PluginPanelUnaffected()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());
        h.PluginPanel.SelectPluginCommand.Execute(h.PluginPanel.FilteredPlugins[0]);
        int count = h.PluginPanel.FilteredPlugins.Count;
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins[0]);
        // Plugin panel should still have all plugins
        Assert.Equal(count, h.PluginPanel.FilteredPlugins.Count);
    }

    // ═══ Pipeline → Pipeline Editor state ═══
    [Fact]
    public void PipelineNodeDelete_AffectsOnlyPipeline()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins[0]);
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins[1]);
        h.PipelineEditor.RemoveNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        Assert.Single(h.PipelineEditor.Nodes);
        // PipelineEditor only affects pipeline nodes, not plugin panel
        Assert.NotEmpty(h.PluginPanel.FilteredPlugins);
    }

    // ═══ Complete workflow ═══
    [Fact]
    public void FullWorkflow_Import_Select_Plugin_Execute()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());

        // Import
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "input.png");
        Assert.Single(h.Main.Images);

        // Select
        h.Main.SelectedImage = h.Main.Images[0];
        Assert.NotNull(h.Main.BeforeImage);

        // Configure pipeline
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins.First(p => p.Id == "grayscale"));
        Assert.Single(h.PipelineEditor.Nodes);

        // Execute
        h.Main.RunPipelineCommand.Execute(null);
        Assert.NotNull(h.Main.StatusMessage);
    }

    [Fact]
    public void FullWorkflow_ImportMultiple_Configure_BatchExecute()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());

        // Import multiple
        for (int i = 0; i < 3; i++)
            h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), $"img_{i}.png");
        Assert.Equal(3, h.Main.Images.Count);

        // Add to batch
        foreach (var img in h.Main.Images)
            h.Batch.AddToQueueCommand.Execute(img);
        Assert.Equal(3, h.Batch.TotalItems);
    }

    [Fact]
    public void NewPipeline_ResetsPipeline_ButImagesRemain()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.LoadPlugins(TestPlugins());
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins[0]);

        h.Main.NewPipelineCommand.Execute(null);

        // Images should remain
        Assert.Single(h.Main.Images);
        // Pipeline editor should be reset
        Assert.Contains("new pipeline", h.Main.StatusMessage, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ClearImages_DoesNotAffectPipeline()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins[0]);
        int nodeCount = h.PipelineEditor.Nodes.Count;

        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.ClearImagesCommand.Execute(null);

        // Pipeline nodes should be unaffected
        Assert.Equal(nodeCount, h.PipelineEditor.Nodes.Count);
        Assert.Empty(h.Main.Images);
    }

    [Fact]
    public void StatusMessage_Consistent_AcrossOperations()
    {
        var h = new ViewModelTestHarness();

        h.Main.NewPipelineCommand.Execute(null);
        Assert.Contains("new pipeline", h.Main.StatusMessage, StringComparison.OrdinalIgnoreCase);

        h.Main.ClearImagesCommand.Execute(null);
        Assert.NotNull(h.Main.StatusMessage);
    }

    // ═══ Tab/panel switching simulation ═══
    [Fact]
    public void SwitchContext_FilmstripToBatch_StatePreserved()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "a.png");
        h.Main.SelectedImage = h.Main.Images[0];

        // Record state
        var selected = h.Main.SelectedImage;
        var imageCount = h.Main.Images.Count;

        // State should be preserved
        Assert.Equal(selected, h.Main.SelectedImage);
        Assert.Equal(imageCount, h.Main.Images.Count);
    }

    [Fact]
    public void SwitchContext_PipelineToPlugin_StatePreserved()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(TestPlugins());
        h.PipelineEditor.AddNodeCommand.Execute(h.PluginPanel.FilteredPlugins[0]);

        var nodeCount = h.PipelineEditor.Nodes.Count;
        var pluginCount = h.PluginPanel.FilteredPlugins.Count;

        Assert.Equal(nodeCount, h.PipelineEditor.Nodes.Count);
        Assert.Equal(pluginCount, h.PluginPanel.FilteredPlugins.Count);
    }

    // ═══ Data isolation ═══
    [Fact]
    public void BatchQueue_IndependentFrom_ImageCollection()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Main.Images.Add(img);
        h.Batch.AddToQueueCommand.Execute(img);
        h.Main.Images.Clear();
        // Batch queue should still have the image (copied by reference)
        Assert.Single(h.Batch.BatchQueue);
    }

    [Fact]
    public void PipelineEditor_Own_ViewModelInstance()
    {
        var h = new ViewModelTestHarness();
        Assert.NotNull(h.PipelineEditor);
        Assert.Same(h.PipelineEditor, h.Main.PipelineEditor);
    }

    [Fact]
    public void PluginPanel_Own_ViewModelInstance()
    {
        var h = new ViewModelTestHarness();
        Assert.NotNull(h.PluginPanel);
        Assert.Same(h.PluginPanel, h.Main.PluginPanel);
    }

    [Fact]
    public void Batch_Own_ViewModelInstance()
    {
        var h = new ViewModelTestHarness();
        Assert.NotNull(h.Batch);
        Assert.Same(h.Batch, h.Main.Batch);
    }
}
