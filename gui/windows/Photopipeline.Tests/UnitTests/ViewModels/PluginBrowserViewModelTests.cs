using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class PluginBrowserViewModelTests
{
    private static PluginBrowserViewModel Create(Mock<IPluginService>? pluginServiceMock = null)
    {
        var logger = Mock.Of<ILogger<PluginBrowserViewModel>>();
        var pluginService = pluginServiceMock?.Object ?? Mock.Of<IPluginService>();
        return new PluginBrowserViewModel(logger, pluginService);
    }

    private static PluginInfo CreatePlugin(string id, string name, string category)
    {
        return new PluginInfo
        {
            Id = id,
            Name = name,
            Category = category,
            Version = "1.0",
            Description = $"Plugin {name}",
            ParameterSchema = new Dictionary<string, object>
            {
                ["strength"] = new Dictionary<string, object>
                {
                    ["type"] = "float",
                    ["default"] = 0.5,
                    ["min"] = 0.0,
                    ["max"] = 1.0
                }
            }
        };
    }

    [Fact]
    public void InitialState_DefaultValues()
    {
        var vm = Create();

        vm.Plugins.Should().BeEmpty();
        vm.FilteredPlugins.Should().BeEmpty();
        vm.Categories.Should().BeEmpty();
        vm.SelectedPlugin.Should().BeNull();
        vm.SearchText.Should().BeEmpty();
        vm.SelectedCategory.Should().Be("All");
        vm.CurrentParameters.Should().BeEmpty();
    }

    [Fact]
    public void SelectPlugin_ExtractsDefaultParameters()
    {
        var vm = Create();
        var plugin = CreatePlugin("denoise", "Denoise", "Noise");

        vm.SelectPluginCommand.Execute(plugin);

        vm.SelectedPlugin.Should().Be(plugin);
        vm.CurrentParameters.Should().ContainKey("strength");
        vm.CurrentParameters["strength"].Should().Be(0.5);
    }

    [Fact]
    public void AddToPipeline_FiresEvent()
    {
        var vm = Create();
        PluginInfo? received = null;
        vm.PluginAdded += p => received = p;
        var plugin = CreatePlugin("sharpen", "Sharpen", "Detail");

        vm.AddToPipelineCommand.Execute(plugin);

        received.Should().Be(plugin);
    }

    [Fact]
    public void SearchText_TriggersFilter()
    {
        var pluginServiceMock = new Mock<IPluginService>();
        var plugins = new List<PluginInfo>
        {
            CreatePlugin("denoise", "Denoise", "Noise"),
            CreatePlugin("sharpen", "Sharpen", "Detail")
        };
        pluginServiceMock.Setup(s => s.GetAllAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(plugins);
        pluginServiceMock.Setup(s => s.GetCategories()).Returns(new[] { "Noise", "Detail" });

        var vm = Create(pluginServiceMock);
        vm.FilteredPlugins = new System.Collections.ObjectModel.ObservableCollection<PluginInfo>(plugins);

        vm.SearchText = "denoise";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Id.Should().Be("denoise");
    }
}
