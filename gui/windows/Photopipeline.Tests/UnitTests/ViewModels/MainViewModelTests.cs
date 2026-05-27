using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for MainViewModel.
/// Uses MockBehavior.Strict for service mocks to catch unexpected calls.
/// Every test has at least one FluentAssertions assertion that will FAIL if logic is wrong.
/// </summary>
public sealed class MainViewModelTests : IDisposable
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

    // ── Helpers ──

    private static Mock<ILogger<T>> LoggerMock<T>() => new();

    private static ILogger<T> AnyLogger<T>() => Mock.Of<ILogger<T>>();

    private static AppSettings DefaultSettings() => new()
    {
        Theme = "Dark",
        WindowWidth = 1440,
        WindowHeight = 900
    };

    private static ImageEntry TestImage(string name = "test.jpg") => new()
    {
        FilePath = $"C:\\photos\\{name}",
        FileName = name,
        Format = "JPEG",
        Width = 1920,
        Height = 1080
    };

    private static PluginInfo TestPlugin(string id = "denoise", string name = "Denoise") => new()
    {
        Id = id,
        Name = name,
        Category = "Enhance",
        Version = "1.0",
        Description = "Test plugin"
    };

    // ── Factory: build MainViewModel with real child VMs ──

    private MainViewModel CreateVm(
        out Mock<ISettingsService> settingsMock,
        out Mock<IBackendService> backendMock,
        out FilmstripViewModel filmstrip,
        out PreviewViewModel preview,
        out PipelineEditorViewModel pipelineEditor,
        out PluginBrowserViewModel pluginBrowser,
        out BatchViewModel batch,
        out SettingsViewModel settingsVm)
    {
        settingsMock = Strict<ISettingsService>();
        backendMock = Strict<IBackendService>();

        settingsMock.Setup(s => s.Current).Returns(DefaultSettings());
        backendMock.Setup(b => b.IsHealthy).Returns(true);

        // Loose mocks for services consumed by child VMs
        var imageService = Mock.Of<IImageService>();
        var pipelineService = Mock.Of<IPipelineService>();
        var pluginService = Mock.Of<IPluginService>();
        var batchService = Mock.Of<IBatchService>();
        var dialogService = Mock.Of<IDialogService>();

        filmstrip = new FilmstripViewModel(AnyLogger<FilmstripViewModel>(), imageService, dialogService);
        preview = new PreviewViewModel(AnyLogger<PreviewViewModel>(), imageService, pipelineService, dialogService);
        pipelineEditor = new PipelineEditorViewModel(AnyLogger<PipelineEditorViewModel>(), pipelineService);
        pluginBrowser = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginService);
        batch = new BatchViewModel(AnyLogger<BatchViewModel>(), batchService, dialogService);
        settingsVm = new SettingsViewModel(AnyLogger<SettingsViewModel>(), settingsMock.Object);

        return new MainViewModel(
            AnyLogger<MainViewModel>(),
            settingsMock.Object,
            backendMock.Object,
            filmstrip, preview, pipelineEditor, pluginBrowser, batch, settingsVm);
    }

    // ═════════════════════════════════════════════════════════════
    // Tests
    // ═════════════════════════════════════════════════════════════

    [Fact]
    public void Test_001_InitialState_AllChildViewModelsNotNull()
    {
        var vm = CreateVm(out _, out _, out var f, out var p, out var pe, out var pb, out var b, out var s);

        vm.Filmstrip.Should().NotBeNull();
        vm.Preview.Should().NotBeNull();
        vm.PipelineEditor.Should().NotBeNull();
        vm.PluginBrowser.Should().NotBeNull();
        vm.Batch.Should().NotBeNull();
        vm.Settings.Should().NotBeNull();
    }

    [Fact]
    public void Test_002_InitialState_BackendStatusReflectsHealthy()
    {
        var vm = CreateVm(out _, out _, out _, out _, out _, out _, out _, out _);

        vm.IsBackendHealthy.Should().BeTrue();
        vm.BackendStatus.Should().Be("Connected");
    }

    [Fact]
    public void Test_003_InitialState_BackendUnhealthyShowsDisconnected()
    {
        var settingsMock = Strict<ISettingsService>();
        var backendMock = Strict<IBackendService>();
        settingsMock.Setup(s => s.Current).Returns(DefaultSettings());
        backendMock.Setup(b => b.IsHealthy).Returns(false);

        var imgSvc = Mock.Of<IImageService>();
        var plSvc = Mock.Of<IPipelineService>();
        var pSvc = Mock.Of<IPluginService>();
        var batSvc = Mock.Of<IBatchService>();
        var dlg = Mock.Of<IDialogService>();

        var vm = new MainViewModel(
            AnyLogger<MainViewModel>(), settingsMock.Object, backendMock.Object,
            new FilmstripViewModel(AnyLogger<FilmstripViewModel>(), imgSvc, dlg),
            new PreviewViewModel(AnyLogger<PreviewViewModel>(), imgSvc, plSvc, dlg),
            new PipelineEditorViewModel(AnyLogger<PipelineEditorViewModel>(), plSvc),
            new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pSvc),
            new BatchViewModel(AnyLogger<BatchViewModel>(), batSvc, dlg),
            new SettingsViewModel(AnyLogger<SettingsViewModel>(), settingsMock.Object));

        vm.IsBackendHealthy.Should().BeFalse();
        vm.BackendStatus.Should().Be("Disconnected");
    }

    [Fact]
    public void Test_004_BackendHealthChanged_UpdatesStatusDynamically()
    {
        var settingsMock = Strict<ISettingsService>();
        var backendMock = Strict<IBackendService>();
        settingsMock.Setup(s => s.Current).Returns(DefaultSettings());
        backendMock.Setup(b => b.IsHealthy).Returns(true);

        EventHandler<bool>? healthChangedHandler = null;
        backendMock.SetupAdd(b => b.HealthChanged += It.IsAny<EventHandler<bool>>())
            .Callback<EventHandler<bool>>(h => healthChangedHandler = h);

        var imgSvc = Mock.Of<IImageService>();
        var plSvc = Mock.Of<IPipelineService>();
        var pSvc = Mock.Of<IPluginService>();
        var batSvc = Mock.Of<IBatchService>();
        var dlg = Mock.Of<IDialogService>();

        var vm = new MainViewModel(
            AnyLogger<MainViewModel>(), settingsMock.Object, backendMock.Object,
            new FilmstripViewModel(AnyLogger<FilmstripViewModel>(), imgSvc, dlg),
            new PreviewViewModel(AnyLogger<PreviewViewModel>(), imgSvc, plSvc, dlg),
            new PipelineEditorViewModel(AnyLogger<PipelineEditorViewModel>(), plSvc),
            new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pSvc),
            new BatchViewModel(AnyLogger<BatchViewModel>(), batSvc, dlg),
            new SettingsViewModel(AnyLogger<SettingsViewModel>(), settingsMock.Object));

        vm.IsBackendHealthy.Should().BeTrue();

        // Simulate health change from backend
        healthChangedHandler.Should().NotBeNull();
        healthChangedHandler!.Invoke(this, false);

        vm.IsBackendHealthy.Should().BeFalse();
        vm.BackendStatus.Should().Be("Disconnected");
    }

    [Fact]
    public void Test_005_ZoomInCommand_DelegatesToPreview()
    {
        var vm = CreateVm(out _, out _, out _, out var preview, out _, out _, out _, out _);

        var initialZoom = preview.ZoomLevel;
        vm.ZoomInCommand.Execute(null);

        preview.ZoomLevel.Should().BeGreaterThan(initialZoom);
    }

    [Fact]
    public void Test_006_ZoomOutCommand_DelegatesToPreview()
    {
        var vm = CreateVm(out _, out _, out _, out var preview, out _, out _, out _, out _);

        preview.ZoomLevel = 2.0;
        vm.ZoomOutCommand.Execute(null);

        preview.ZoomLevel.Should().BeLessThan(2.0);
    }

    [Fact]
    public void Test_007_ResetZoomCommand_DelegatesToPreview()
    {
        var vm = CreateVm(out _, out _, out _, out var preview, out _, out _, out _, out _);

        preview.ZoomLevel = 4.0;
        vm.ResetZoomCommand.Execute(null);

        preview.ZoomLevel.Should().Be(1.0);
    }

    [Fact]
    public void Test_008_WindowSize_LoadedFromSettings()
    {
        var settingsMock = Strict<ISettingsService>();
        var backendMock = Strict<IBackendService>();
        settingsMock.Setup(s => s.Current).Returns(new AppSettings { WindowWidth = 1920, WindowHeight = 1080 });
        backendMock.Setup(b => b.IsHealthy).Returns(true);

        var imgSvc = Mock.Of<IImageService>();
        var plSvc = Mock.Of<IPipelineService>();
        var pSvc = Mock.Of<IPluginService>();
        var batSvc = Mock.Of<IBatchService>();
        var dlg = Mock.Of<IDialogService>();

        var vm = new MainViewModel(
            AnyLogger<MainViewModel>(), settingsMock.Object, backendMock.Object,
            new FilmstripViewModel(AnyLogger<FilmstripViewModel>(), imgSvc, dlg),
            new PreviewViewModel(AnyLogger<PreviewViewModel>(), imgSvc, plSvc, dlg),
            new PipelineEditorViewModel(AnyLogger<PipelineEditorViewModel>(), plSvc),
            new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pSvc),
            new BatchViewModel(AnyLogger<BatchViewModel>(), batSvc, dlg),
            new SettingsViewModel(AnyLogger<SettingsViewModel>(), settingsMock.Object));

        vm.WindowWidth.Should().Be(1920);
        vm.WindowHeight.Should().Be(1080);
    }

    [Fact]
    public void Test_009_PluginAddedEvent_TriggersPipelineEditorAddNode()
    {
        var vm = CreateVm(out _, out _, out _, out _, out var pipelineEditor,
            out var pluginBrowser, out _, out _);

        var plugin = TestPlugin();

        // PluginAdded event should invoke PipelineEditor.AddNodeCommand
        pipelineEditor.Nodes.Should().BeEmpty();
        pluginBrowser.AddToPipelineCommand.Execute(plugin);

        pipelineEditor.Nodes.Should().HaveCount(1);
        pipelineEditor.Nodes[0].PluginId.Should().Be(plugin.Id);
    }

    [Fact]
    public void Test_010_FilmstripImageSelected_SetsPipelineEditorSelectedImagePath()
    {
        var vm = CreateVm(out _, out _, out var filmstrip, out _, out var pipelineEditor,
            out _, out _, out _);

        var img = TestImage("photo.dng");

        // Filmstrip.ImageSelected event is wired by MainViewModel constructor
        filmstrip.SelectedImage = img;

        pipelineEditor.SelectedImagePath.Should().Be(img.FilePath);
    }

    [Fact]
    public void Test_011_FilmstripImageSelected_NullImage_DoesNotCrash()
    {
        var vm = CreateVm(out _, out _, out var filmstrip, out _, out var pipelineEditor,
            out _, out _, out _);

        // Should not throw when selecting null/clearing selection
        var act = () => filmstrip.SelectedImage = null;

        act.Should().NotThrow();
        pipelineEditor.SelectedImagePath.Should().BeNull();
    }

    [Fact]
    public void Test_012_FilmstripSendToBatchRequested_DelegatesToBatch()
    {
        var vm = CreateVm(out _, out _, out var filmstrip, out _, out _,
            out _, out var batch, out _);

        var img1 = TestImage("a.jpg");
        var img2 = TestImage("b.jpg");

        batch.BatchQueue.Should().BeEmpty();
        filmstrip.SelectedImages.Add(img1);
        filmstrip.SelectedImages.Add(img2);
        filmstrip.SendToBatchCommand.Execute(null);

        batch.BatchQueue.Should().HaveCount(2);
    }

    [Fact]
    public void Test_013_PipelineEditorPipelineIdChanged_UpdatesBatchPipelineConfigPath()
    {
        var vm = CreateVm(out _, out _, out _, out _, out var pipelineEditor,
            out _, out var batch, out _);

        batch.PipelineConfigPath.Should().BeNull();

        pipelineEditor.PipelineId = "pid-save-001";

        batch.PipelineConfigPath.Should().Be("pid-save-001");
    }

    [Fact]
    public void Test_014_ChildVmStatusMessage_ForwardsToMainStatusMessage()
    {
        var vm = CreateVm(out _, out _, out _, out _, out var pipelineEditor,
            out _, out _, out _);

        vm.StatusMessage.Should().Be("Ready");

        pipelineEditor.StatusMessage = "Pipeline validated";

        vm.StatusMessage.Should().Be("Pipeline validated");
    }

    [Fact]
    public void Test_015_FilmstripSelectedImage_FiresMainImageSelectedEvent()
    {
        var vm = CreateVm(out _, out _, out var filmstrip, out _, out _,
            out _, out _, out _);

        ImageEntry? received = null;
        vm.ImageSelected += img => received = img;

        var img = TestImage("photo.dng");
        filmstrip.SelectedImage = img;

        received.Should().Be(img);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 016: Shutdown_UnsubscribesFromEvents_NoMemoryLeaks
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_016_Shutdown_UnsubscribesFromBackendEvents()
    {
        var settingsMock = Strict<ISettingsService>();
        var backendMock = Strict<IBackendService>();
        settingsMock.Setup(s => s.Current).Returns(DefaultSettings());
        backendMock.Setup(b => b.IsHealthy).Returns(true);

        EventHandler<bool>? healthChangedHandler = null;
        backendMock.SetupAdd(b => b.HealthChanged += It.IsAny<EventHandler<bool>>())
            .Callback<EventHandler<bool>>(h => healthChangedHandler = h);
        backendMock.SetupRemove(b => b.HealthChanged -= It.IsAny<EventHandler<bool>>());

        var imgSvc = Mock.Of<IImageService>();
        var plSvc = Mock.Of<IPipelineService>();
        var pSvc = Mock.Of<IPluginService>();
        var batSvc = Mock.Of<IBatchService>();
        var dlg = Mock.Of<IDialogService>();

        var vm = new MainViewModel(
            AnyLogger<MainViewModel>(), settingsMock.Object, backendMock.Object,
            new FilmstripViewModel(AnyLogger<FilmstripViewModel>(), imgSvc, dlg),
            new PreviewViewModel(AnyLogger<PreviewViewModel>(), imgSvc, plSvc, dlg),
            new PipelineEditorViewModel(AnyLogger<PipelineEditorViewModel>(), plSvc),
            new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pSvc),
            new BatchViewModel(AnyLogger<BatchViewModel>(), batSvc, dlg),
            new SettingsViewModel(AnyLogger<SettingsViewModel>(), settingsMock.Object));

        vm.IsBackendHealthy.Should().BeTrue("initial state should reflect healthy backend");

        // Shutdown should unsubscribe all event handlers
        vm.Shutdown();

        // Verify backend event was unsubscribed
        backendMock.VerifyRemove(
            b => b.HealthChanged -= It.IsAny<EventHandler<bool>>(),
            Times.Once,
            "Shutdown must unsubscribe from BackendService.HealthChanged to prevent memory leaks");

        // Verify child VM StatusMessage forwarding is stopped:
        // after Shutdown, changing child VM status should not propagate to MainViewModel
        vm.StatusMessage = "Ready"; // reset
        vm.Filmstrip.StatusMessage = "Should not forward";
        vm.StatusMessage.Should().Be("Ready",
            "StatusMessage should not forward from child VMs after Shutdown");
    }
}
