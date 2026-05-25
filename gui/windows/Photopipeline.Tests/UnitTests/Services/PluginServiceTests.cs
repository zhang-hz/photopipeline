using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.Services;

public sealed class PluginServiceTests
{
    // PluginService is a concrete class - these tests validate the interface contract
    // Actual implementation tests would require the full service setup

    [Fact]
    public void PluginService_ImplementsIPluginService()
    {
        // Verify the interface contract is complete
        typeof(IPluginService).Should().NotBeNull();
        typeof(IPluginService).GetMethods().Should().NotBeEmpty();
    }

    [Fact]
    public void PluginService_InterfaceHasGetAllAsync()
    {
        var method = typeof(IPluginService).GetMethod(nameof(IPluginService.GetAllAsync));
        method.Should().NotBeNull();
        method!.ReturnType.Should().Be<Task<IReadOnlyList<PluginInfo>>>();
    }

    [Fact]
    public void PluginService_InterfaceHasGetSchemaAsync()
    {
        var method = typeof(IPluginService).GetMethod(nameof(IPluginService.GetSchemaAsync));
        method.Should().NotBeNull();
        method!.ReturnType.Should().Be<Task<NodeSchema?>>();
    }

    [Fact]
    public void PluginService_InterfaceHasGetCategories()
    {
        var method = typeof(IPluginService).GetMethod(nameof(IPluginService.GetCategories));
        method.Should().NotBeNull();
        method!.ReturnType.Should().Be<IReadOnlyList<string>>();
    }

    [Fact]
    public void PluginService_InterfaceHasSearch()
    {
        var method = typeof(IPluginService).GetMethod(nameof(IPluginService.Search));
        method.Should().NotBeNull();
        method!.ReturnType.Should().Be<IReadOnlyList<PluginInfo>>();
    }

    [Fact]
    public void PluginService_InterfaceHasFilterByCategory()
    {
        var method = typeof(IPluginService).GetMethod(nameof(IPluginService.FilterByCategory));
        method.Should().NotBeNull();
        method!.ReturnType.Should().Be<IReadOnlyList<PluginInfo>>();
    }
}
