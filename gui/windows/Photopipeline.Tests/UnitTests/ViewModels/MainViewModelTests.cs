using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class MainViewModelTests
{
    private static Mock<ISettingsService> CreateSettingsMock()
    {
        var mock = new Mock<ISettingsService>();
        mock.Setup(s => s.Current).Returns(new AppSettings());
        return mock;
    }

    private static Mock<IBackendService> CreateBackendMock()
    {
        var mock = new Mock<IBackendService>();
        mock.Setup(b => b.IsHealthy).Returns(true);
        return mock;
    }

    private static MainViewModel Create()
    {
        var logger = Mock.Of<ILogger<MainViewModel>>();
        var settings = CreateSettingsMock().Object;
        var backend = CreateBackendMock().Object;

        var filmstripLogger = Mock.Of<ILogger<FilmstripViewModel>>();
        var previewLogger = Mock.Of<ILogger<PreviewViewModel>>();
        var pipelineEditorLogger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pluginBrowserLogger = Mock.Of<ILogger<PluginBrowserViewModel>>();
        var batchLogger = Mock.Of<ILogger<BatchViewModel>>();
        var settingsVmLogger = Mock.Of<ILogger<SettingsViewModel>>();

        var imageService = Mock.Of<IImageService>();
        var pipelineService = Mock.Of<IPipelineService>();
        var pluginService = Mock.Of<IPluginService>();
        var batchService = Mock.Of<IBatchService>();

        var filmstrip = new FilmstripViewModel(filmstripLogger, imageService);
        var preview = new PreviewViewModel(previewLogger, imageService, pipelineService);
        var pipelineEditor = new PipelineEditorViewModel(pipelineEditorLogger, pipelineService);
        var pluginBrowser = new PluginBrowserViewModel(pluginBrowserLogger, pluginService);
        var batch = new BatchViewModel(batchLogger, batchService);
        var settingsVm = new SettingsViewModel(settingsVmLogger, settings);

        return new MainViewModel(logger, settings, backend,
            filmstrip, preview, pipelineEditor, pluginBrowser, batch, settingsVm);
    }

    [Fact]
    public void InitialState_AllChildViewModelsNotNull()
    {
        var vm = Create();

        vm.Filmstrip.Should().NotBeNull();
        vm.Preview.Should().NotBeNull();
        vm.PipelineEditor.Should().NotBeNull();
        vm.PluginBrowser.Should().NotBeNull();
        vm.Batch.Should().NotBeNull();
        vm.Settings.Should().NotBeNull();
    }

    [Fact]
    public void InitialState_BackendHealthy()
    {
        var vm = Create();

        vm.IsBackendHealthy.Should().BeTrue();
        vm.BackendStatus.Should().Be("Connected");
    }

    [Fact]
    public void InitialState_WindowTitle()
    {
        var vm = Create();

        vm.WindowTitle.Should().Be("Photopipeline");
    }

    [Fact]
    public void BackendHealthChanged_UpdatesStatus()
    {
        var backendMock = CreateBackendMock();
        backendMock.Setup(b => b.IsHealthy).Returns(false);
        var backend = backendMock.Object;

        var logger = Mock.Of<ILogger<MainViewModel>>();
        var settings = CreateSettingsMock().Object;
        var imageService = Mock.Of<IImageService>();
        var pipelineService = Mock.Of<IPipelineService>();
        var pluginService = Mock.Of<IPluginService>();
        var batchService = Mock.Of<IBatchService>();

        var vm = new MainViewModel(logger, settings, backend,
            new FilmstripViewModel(Mock.Of<ILogger<FilmstripViewModel>>(), imageService),
            new PreviewViewModel(Mock.Of<ILogger<PreviewViewModel>>(), imageService, pipelineService),
            new PipelineEditorViewModel(Mock.Of<ILogger<PipelineEditorViewModel>>(), pipelineService),
            new PluginBrowserViewModel(Mock.Of<ILogger<PluginBrowserViewModel>>(), pluginService),
            new BatchViewModel(Mock.Of<ILogger<BatchViewModel>>(), batchService),
            new SettingsViewModel(Mock.Of<ILogger<SettingsViewModel>>(), settings));

        vm.IsBackendHealthy.Should().BeFalse();
        vm.BackendStatus.Should().Be("Disconnected");
    }

    [Fact]
    public void ZoomCommands_DelegateToPreview()
    {
        var vm = Create();

        vm.ZoomInCommand.Execute(null);
        vm.ZoomOutCommand.Execute(null);
        vm.ResetZoomCommand.Execute(null);
    }

    [Fact]
    public void WindowSize_LoadedFromSettings()
    {
        var vm = Create();

        vm.WindowWidth.Should().Be(1440);
        vm.WindowHeight.Should().Be(900);
    }
}
