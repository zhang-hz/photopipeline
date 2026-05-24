using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.TestInfrastructure;

public class ViewModelTestHarness
{
    private readonly IPipelineService _pipelineService;
    private readonly IImageService _imageService;

    public MainViewModel Main { get; }
    public PipelineEditorViewModel PipelineEditor => Main.PipelineEditor;
    public PluginPanelViewModel PluginPanel => Main.PluginPanel;
    public BatchViewModel Batch => Main.Batch;

    public ViewModelTestHarness(
        IPipelineService? pipelineService = null,
        IImageService? imageService = null)
    {
        _pipelineService = pipelineService ?? new LocalPipelineService();
        _imageService = imageService ?? new LocalImageService();

        Main = new MainViewModel(_pipelineService, _imageService,
            new PipelineEditorViewModel(_pipelineService),
            new PluginPanelViewModel(_pipelineService),
            new BatchViewModel());
    }

    public void LoadPlugins(IEnumerable<PluginInfo> plugins)
    {
        PluginPanel.LoadPlugins(new System.Collections.ObjectModel.ObservableCollection<PluginInfo>(plugins));
    }

    public void AddTestImage(string filePath, string fileName = "")
    {
        if (string.IsNullOrEmpty(fileName))
            fileName = Path.GetFileName(filePath);
        Main.Images.Add(new ImageEntry { FilePath = filePath, FileName = fileName });
    }
}
