using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for PluginBrowserViewModel.
/// Uses MockBehavior.Strict for service mocks. Every test has a FAIL-able assertion.
/// </summary>
public sealed class PluginBrowserViewModelTests : IDisposable
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

    private static ILogger<T> AnyLogger<T>() => Mock.Of<ILogger<T>>();

    private static PluginInfo TestPlugin(string id, string name, string category,
        string desc = "") => new()
    {
        Id = id,
        Name = name,
        Category = category,
        Version = "1.0",
        Description = desc,
        ParameterSchema = new Dictionary<string, object>
        {
            ["strength"] = new Dictionary<string, object>
            {
                ["type"] = "float", ["default"] = 0.5, ["min"] = 0.0, ["max"] = 1.0
            }
        }
    };

    /// <summary>
    /// Creates a PluginBrowserViewModel with a pre-loaded mock plugin service.
    /// The LoadPluginsAsync call in the constructor is async void, but the mock
    /// GetAllAsync returns synchronously, so Plugins is populated immediately.
    /// </summary>
    private PluginBrowserViewModel CreateWithPlugins()
    {
        var plugins = new List<PluginInfo>
        {
            TestPlugin("denoise", "Denoise", "Enhance", "Noise reduction"),
            TestPlugin("sharpen", "Sharpen", "Enhance", "Image sharpening"),
            TestPlugin("raw_decoder", "Raw Decoder", "Input", "Raw file decoder"),
            TestPlugin("tiff_encoder", "TIFF Encoder", "Format", "TIFF output"),
            TestPlugin("png_encoder", "PNG Encoder", "Format", "PNG output")
        };

        var pluginMock = Strict<IPluginService>();
        pluginMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(plugins);
        pluginMock.Setup(s => s.GetCategories())
            .Returns(new[] { "Enhance", "Format", "Input" });

        var vm = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginMock.Object);
        return vm;
    }

    // ═════════════════════════════════════════════════════════════
    // Test 001: InitialState_PluginsLoaded
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_001_InitialState_PluginsLoaded()
    {
        var vm = CreateWithPlugins();

        vm.Plugins.Should().HaveCount(5);
        vm.FilteredPlugins.Should().HaveCount(5);
        vm.Categories.Should().Contain(new[] { "All", "Enhance", "Format", "Input" });
        vm.SelectedCategory.Should().Be("All");
        vm.SearchText.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 002: SearchText_FiltersByName
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_002_SearchText_FiltersByName()
    {
        var vm = CreateWithPlugins();

        vm.SearchText = "denoise";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Id.Should().Be("denoise");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 003: SearchText_NoMatch_ReturnsEmpty
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_003_SearchText_NoMatch_ReturnsEmpty()
    {
        var vm = CreateWithPlugins();

        vm.SearchText = "zzz_nonexistent";

        vm.FilteredPlugins.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 004: SearchText_Empty_ShowsAll
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_004_SearchText_Empty_ShowsAll()
    {
        var vm = CreateWithPlugins();

        vm.SearchText = "denoise";
        vm.FilteredPlugins.Should().HaveCount(1);

        vm.SearchText = "";

        vm.FilteredPlugins.Should().HaveCount(5);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 005: CategoryFilter_ShowsOnlyFormatPlugins
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_005_CategoryFilter_ShowsOnlyFormatPlugins()
    {
        var pluginMock = Strict<IPluginService>();
        var plugins = new List<PluginInfo>
        {
            TestPlugin("denoise", "Denoise", "Enhance"),
            TestPlugin("sharpen", "Sharpen", "Enhance"),
            TestPlugin("tiff_encoder", "TIFF Encoder", "Format"),
            TestPlugin("png_encoder", "PNG Encoder", "Format")
        };
        pluginMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(plugins);
        pluginMock.Setup(s => s.GetCategories())
            .Returns(new[] { "Enhance", "Format" });
        pluginMock.Setup(s => s.FilterByCategory("Format"))
            .Returns(plugins.Where(p => p.Category == "Format").ToList());

        var vm = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginMock.Object);

        vm.SelectedCategory = "Format";

        vm.FilteredPlugins.Should().HaveCount(2);
        vm.FilteredPlugins.Should().OnlyContain(p => p.Category == "Format");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 006: SelectPlugin_UpdatesSelectedAndParameters
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_006_SelectPlugin_UpdatesSelectedAndParameters()
    {
        var plugin = TestPlugin("denoise", "Denoise", "Enhance");
        var pluginMock = Strict<IPluginService>();
        pluginMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new[] { plugin });
        pluginMock.Setup(s => s.GetCategories())
            .Returns(new[] { "Enhance" });

        var vm = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginMock.Object);

        vm.SelectPluginCommand.Execute(plugin);

        vm.SelectedPlugin.Should().Be(plugin);
        vm.CurrentParameters.Should().ContainKey("strength");
        vm.CurrentParameters["strength"].Should().Be(0.5);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 007: AddToPipeline_FiresPluginAddedEvent
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_007_AddToPipeline_FiresPluginAddedEvent()
    {
        var plugin = TestPlugin("sharpen", "Sharpen", "Enhance");
        var pluginMock = Strict<IPluginService>();
        pluginMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new[] { plugin });
        pluginMock.Setup(s => s.GetCategories())
            .Returns(new[] { "Enhance" });

        var vm = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginMock.Object);
        PluginInfo? received = null;
        vm.PluginAdded += p => received = p;

        vm.AddToPipelineCommand.Execute(plugin);

        received.Should().Be(plugin);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 008: SearchText_FiltersByDescription
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_008_SearchText_FiltersByDescription()
    {
        var vm = CreateWithPlugins();

        vm.SearchText = "sharpening";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Id.Should().Be("sharpen");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 009: Combined_SearchAndCategoryFilter
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_009_CombinedSearchAndCategoryFilter()
    {
        var pluginMock = Strict<IPluginService>();
        var plugins = new List<PluginInfo>
        {
            TestPlugin("denoise", "Denoise", "Enhance"),
            TestPlugin("dn_advanced", "DN Advanced", "Enhance"),
            TestPlugin("tiff_encoder", "TIFF Encoder", "Format")
        };
        pluginMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(plugins);
        pluginMock.Setup(s => s.GetCategories())
            .Returns(new[] { "Enhance", "Format" });
        pluginMock.Setup(s => s.FilterByCategory("Enhance"))
            .Returns(plugins.Where(p => p.Category == "Enhance").ToList());

        var vm = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginMock.Object);

        vm.SelectedCategory = "Enhance";
        vm.SearchText = "dn";

        vm.FilteredPlugins.Should().HaveCount(2);
        vm.FilteredPlugins.Should().OnlyContain(p => p.Id.Contains("dn", StringComparison.OrdinalIgnoreCase));
    }

    // ═════════════════════════════════════════════════════════════
    // Test 010: PluginAdded_MultipleSubscribers_BothNotified
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_010_PluginAdded_MultipleSubscribers_BothNotified()
    {
        var plugin = TestPlugin("crop", "Crop", "Transform");
        var pluginMock = Strict<IPluginService>();
        pluginMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new[] { plugin });
        pluginMock.Setup(s => s.GetCategories())
            .Returns(new[] { "Transform" });

        var vm = new PluginBrowserViewModel(AnyLogger<PluginBrowserViewModel>(), pluginMock.Object);
        PluginInfo? received1 = null;
        PluginInfo? received2 = null;
        vm.PluginAdded += p => received1 = p;
        vm.PluginAdded += p => received2 = p;

        vm.AddToPipelineCommand.Execute(plugin);

        received1.Should().Be(plugin);
        received2.Should().Be(plugin);
    }
}
