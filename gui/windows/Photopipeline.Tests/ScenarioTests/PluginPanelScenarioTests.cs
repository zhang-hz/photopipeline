using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class PluginPanelScenarioTests
{
    private static PluginBrowserViewModel Create(Mock<IPluginService>? pluginServiceMock = null)
    {
        var logger = Mock.Of<ILogger<PluginBrowserViewModel>>();
        var pluginService = pluginServiceMock?.Object ?? Mock.Of<IPluginService>();
        return new PluginBrowserViewModel(logger, pluginService);
    }

    [Fact]
    public void SearchAndSelect_RendersParameterDefaults()
    {
        var mock = new Mock<IPluginService>();
        var plugins = new List<PluginInfo>
        {
            new PluginInfo
            {
                Id = "denoise_v1",
                Name = "AI Denoise",
                Category = "Noise",
                Description = "Removes noise",
                ParameterSchema = new Dictionary<string, object>
                {
                    ["strength"] = new Dictionary<string, object>
                    {
                        ["type"] = "float", ["default"] = 0.5, ["min"] = 0.0, ["max"] = 1.0
                    }
                }
            }
        };
        mock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>())).ReturnsAsync(plugins);
        mock.Setup(s => s.GetCategories()).Returns(new[] { "Noise" });

        var vm = Create(mock);
        vm.FilteredPlugins = new System.Collections.ObjectModel.ObservableCollection<PluginInfo>(plugins);

        vm.SelectPluginCommand.Execute(plugins[0]);

        vm.SelectedPlugin.Should().NotBeNull();
        vm.CurrentParameters.Should().ContainKey("strength");
        vm.CurrentParameters["strength"].Should().Be(0.5);
    }

    [Fact]
    public void SelectPlugin_WithBoolDefault()
    {
        var vm = Create();
        var plugin = new PluginInfo
        {
            Id = "test",
            Name = "Test",
            ParameterSchema = new Dictionary<string, object>
            {
                ["enabled"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true }
            }
        };

        vm.SelectPluginCommand.Execute(plugin);

        vm.CurrentParameters["enabled"].Should().Be(true);
    }

    [Fact]
    public void AddPluginToPipeline_FiresPluginAdded()
    {
        var vm = Create();
        var plugin = new PluginInfo { Id = "sharpen", Name = "Sharpen", Category = "Detail" };
        PluginInfo? added = null;
        vm.PluginAdded += p => added = p;

        vm.AddToPipelineCommand.Execute(plugin);

        added.Should().Be(plugin);
    }

    [Fact]
    public void SearchText_TriggersFilterRefresh()
    {
        var mock = new Mock<IPluginService>();
        var plugins = new List<PluginInfo>
        {
            new PluginInfo { Id = "denoise", Name = "Denoise" },
            new PluginInfo { Id = "sharpen", Name = "Sharpen" }
        };
        mock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>())).ReturnsAsync(plugins);
        mock.Setup(s => s.GetCategories()).Returns(new[] { "Noise", "Detail" });
        mock.Setup(s => s.Search("sharpen")).Returns(new[] { plugins[1] });

        var vm = Create(mock);
        vm.FilteredPlugins = new System.Collections.ObjectModel.ObservableCollection<PluginInfo>(plugins);

        vm.SearchText = "sharpen";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Id.Should().Be("sharpen");
    }
}
