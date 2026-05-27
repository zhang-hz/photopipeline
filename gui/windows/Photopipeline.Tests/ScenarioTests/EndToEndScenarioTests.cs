using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class EndToEndScenarioTests
{
    [Fact]
    public void FullWorkflow_ImportToBatch_VerifiesCrossVmWiring()
    {
        // Setup all services
        var logger = Mock.Of<ILogger<MainViewModel>>();
        var settingsMock = new Mock<ISettingsService>();
        settingsMock.Setup(s => s.Current).Returns(new AppSettings());
        var backendMock = new Mock<IBackendService>();
        backendMock.Setup(b => b.IsHealthy).Returns(true);

        var imageService = Mock.Of<IImageService>();
        var pipelineService = Mock.Of<IPipelineService>();
        var pluginService = Mock.Of<IPluginService>();
        var batchService = Mock.Of<IBatchService>();

        var filmstrip = new FilmstripViewModel(Mock.Of<ILogger<FilmstripViewModel>>(), imageService, null!);
        var preview = new PreviewViewModel(Mock.Of<ILogger<PreviewViewModel>>(), imageService, pipelineService, null!);
        var pipelineEditor = new PipelineEditorViewModel(Mock.Of<ILogger<PipelineEditorViewModel>>(), pipelineService);
        var pluginBrowser = new PluginBrowserViewModel(Mock.Of<ILogger<PluginBrowserViewModel>>(), pluginService);
        var batch = new BatchViewModel(Mock.Of<ILogger<BatchViewModel>>(), batchService, null!);
        var settingsVm = new SettingsViewModel(Mock.Of<ILogger<SettingsViewModel>>(), settingsMock.Object);

        var mainVm = new MainViewModel(logger, settingsMock.Object, backendMock.Object,
            filmstrip, preview, pipelineEditor, pluginBrowser, batch, settingsVm);

        // Verify all child VMs are assigned
        mainVm.Filmstrip.Should().Be(filmstrip);
        mainVm.Preview.Should().Be(preview);
        mainVm.PipelineEditor.Should().Be(pipelineEditor);
        mainVm.PluginBrowser.Should().Be(pluginBrowser);
        mainVm.Batch.Should().Be(batch);
        mainVm.Settings.Should().Be(settingsVm);

        // Verify cross-VM wiring: image selection propagates to pipeline editor
        var img = new ImageEntry { FilePath = @"C:\photos\test.dng", FileName = "test.dng" };
        filmstrip.SelectedImage = img;
        pipelineEditor.SelectedImagePath.Should().Be(img.FilePath,
            "Filmstrip image selection should propagate to PipelineEditor.SelectedImagePath");

        // Verify cross-VM wiring: pipeline ID change propagates to batch
        pipelineEditor.PipelineId = "test-pipeline-id";
        batch.PipelineConfigPath.Should().Be("test-pipeline-id",
            "PipelineEditor.PipelineId change should propagate to Batch.PipelineConfigPath");

        // Verify cross-VM wiring: plugin addition notifies pipeline editor
        var plugin = new PluginInfo { Id = "denoise", Name = "Denoise", Category = "Enhance" };
        var nodeCountBefore = pipelineEditor.Nodes.Count;
        pluginBrowser.AddToPipelineCommand.Execute(plugin);
        pipelineEditor.Nodes.Should().HaveCount(nodeCountBefore + 1,
            "PluginBrowser.AddToPipeline should add a node to PipelineEditor");
        pipelineEditor.Nodes.Last().PluginId.Should().Be("denoise");
    }

    [Fact]
    public void BuildPipeline_AddPlugin_SetParams_Validate()
    {
        var pipelineLogger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var editor = new PipelineEditorViewModel(pipelineLogger, pipelineService);

        // Step 1: Add node
        editor.AddNodeAt(new PluginInfo { Id = "denoise", Name = "Denoise" }, 150, 200);
        editor.Nodes.Should().HaveCount(1);

        // Step 2: Set parameters
        editor.UpdateNodeParameterCommand.Execute((editor.Nodes[0].Id, "strength", 0.75));
        editor.Nodes[0].Params["strength"].Should().Be(0.75);

        // Step 3: Validate (empty pipeline validation)
        editor.ValidateCommand.Execute(null);
        editor.ValidationMessage.Should().NotBeNull();
    }

    [Fact]
    public void ImageLifecycle_LoadFilterSelect_Remove()
    {
        var filmstripLogger = Mock.Of<ILogger<FilmstripViewModel>>();
        var imageService = Mock.Of<IImageService>();
        var filmstrip = new FilmstripViewModel(filmstripLogger, imageService, null!);

        // Load images
        filmstrip.Images.Add(new ImageEntry { FileName = "photo1.dng", Format = "DNG" });
        filmstrip.Images.Add(new ImageEntry { FileName = "photo2.jpg", Format = "JPEG" });
        filmstrip.Images.Add(new ImageEntry { FileName = "photo3.dng", Format = "DNG" });

        // Filter
        filmstrip.FilterFormat = "DNG";
        filmstrip.FilteredImages.Should().HaveCount(2);

        // Select
        filmstrip.SelectAllCommand.Execute(null);
        filmstrip.SelectedImages.Should().HaveCount(2);

        // Clear filter
        filmstrip.FilterFormat = "All";
        filmstrip.FilteredImages.Should().HaveCount(3);

        // Clear selection
        filmstrip.ClearSelectionCommand.Execute(null);
        filmstrip.SelectedImages.Should().BeEmpty();
    }

    [Fact]
    public void CanvasNavigation_ZoomPanReset()
    {
        var pipelineLogger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var editor = new PipelineEditorViewModel(pipelineLogger, pipelineService);

        // Zoom in
        editor.ZoomCanvasCommand.Execute(0.5);
        editor.Scale.Should().Be(1.5);

        // Zoom out
        editor.ZoomCanvasCommand.Execute(-0.3);
        editor.Scale.Should().Be(1.2);

        // Reset
        editor.ResetCanvasCommand.Execute(null);
        editor.Scale.Should().Be(1.0);
    }
}
