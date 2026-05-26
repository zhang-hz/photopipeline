using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class CrossPanelScenarioTests
{
    [Fact]
    public void FilmstripSelection_NotifiesPipelineEditor()
    {
        var logger = Mock.Of<ILogger<FilmstripViewModel>>();
        var imageService = Mock.Of<IImageService>();
        var filmstrip = new FilmstripViewModel(logger, imageService, null!);
        var pipelineLogger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var editor = new PipelineEditorViewModel(pipelineLogger, pipelineService);

        ImageEntry? selected = null;
        filmstrip.ImageSelected += img => selected = img;
        var image = new ImageEntry { FileName = "test.dng", FilePath = @"C:\test.dng" };

        filmstrip.SelectedImage = image;

        selected.Should().Be(image);
    }

    [Fact]
    public void PluginAddedToPipeline_CreatesNode()
    {
        var pipelineLogger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var editor = new PipelineEditorViewModel(pipelineLogger, pipelineService);
        var pluginLogger = Mock.Of<ILogger<PluginBrowserViewModel>>();
        var pluginService = Mock.Of<IPluginService>();
        var browser = new PluginBrowserViewModel(pluginLogger, pluginService);

        browser.PluginAdded += plugin => editor.AddNodeAt(plugin, 200, 300);
        var plugin = new PluginInfo { Id = "sharpen", Name = "Sharpen", Category = "Detail" };

        browser.AddToPipelineCommand.Execute(plugin);

        editor.Nodes.Should().HaveCount(1);
        editor.Nodes[0].PluginId.Should().Be("sharpen");
        editor.Nodes[0].PositionX.Should().Be(200);
        editor.Nodes[0].PositionY.Should().Be(300);
    }

    [Fact]
    public void FilmstripToBatch_AddsImagesToQueue()
    {
        var batchLogger = Mock.Of<ILogger<BatchViewModel>>();
        var batchService = Mock.Of<IBatchService>();
        var batch = new BatchViewModel(batchLogger, batchService, null!);

        var img1 = new ImageEntry { FileName = "a.dng" };
        var img2 = new ImageEntry { FileName = "b.dng" };

        batch.AddToQueueCommand.Execute(img1);
        batch.AddToQueueCommand.Execute(img2);

        batch.BatchQueue.Should().HaveCount(2);
    }

    [Fact]
    public void PipelineNodeCount_AffectsValidation()
    {
        var pipelineLogger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var editor = new PipelineEditorViewModel(pipelineLogger, pipelineService);

        editor.ValidateCommand.Execute(null);

        editor.IsPipelineValid.Should().BeFalse();
        editor.ValidationMessage.Should().Contain("no nodes");
    }
}
