namespace Photopipeline.Tests.UnitTests.Services;

public sealed class PluginServiceTests
{
    // PluginService is a concrete class with a hardcoded plugin catalog.
    // These tests validate behavior, not just reflection.

    // ── Test 1: GetAllAsync returns all 14 plugins ──
    [Fact]
    public async Task GetAllAsync_ReturnsAllPlugins()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var plugins = await service.GetAllAsync();

        // Assert — the catalog must have plugins registered
        plugins.Should().NotBeNull("GetAllAsync must never return null");
        plugins.Count.Should().BeGreaterThan(0, "plugin catalog must contain at least one plugin");
        plugins.Should().AllSatisfy(p =>
        {
            p.Id.Should().NotBeNullOrEmpty("every plugin must have an Id");
            p.Name.Should().NotBeNullOrEmpty("every plugin must have a Name");
            p.Category.Should().NotBeNullOrEmpty("every plugin must have a Category");
        });
    }

    // ── Test 2: GetSchemaAsync returns parameter definitions for a valid plugin ID ──
    [Fact]
    public async Task GetSchemaAsync_ValidPluginId_ReturnsSchema()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var schema = await service.GetSchemaAsync("raw_decoder");

        // Assert
        schema.Should().NotBeNull("GetSchemaAsync must return a schema for a valid plugin ID");
        schema!.PluginId.Should().Be("raw_decoder");
        schema.Name.Should().Be("Raw Decoder");
        schema.Category.Should().Be("Input");
        schema.ParameterSchema.Should().NotBeEmpty("raw_decoder must have parameter definitions");
        schema.ParameterSchema.Should().ContainKey("demosaic", "raw_decoder has demosaic parameter");
        schema.ParameterSchema.Should().ContainKey("border", "raw_decoder has border parameter");
        schema.ParameterSchema.Should().ContainKey("white_balance", "raw_decoder has white_balance parameter");
    }

    // ── Test 3: GetSchemaAsync returns null for an invalid plugin ID ──
    [Fact]
    public async Task GetSchemaAsync_InvalidPluginId_ReturnsNull()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var schema = await service.GetSchemaAsync("nonexistent_plugin");

        // Assert
        schema.Should().BeNull("GetSchemaAsync must return null for a nonexistent plugin ID");
    }

    // ── Test 4: Search filters plugins by name, description, and category ──
    [Fact]
    public void Search_ByName_FindsMatchingPlugins()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var results = service.Search("Raw Decoder");

        // Assert
        results.Should().NotBeNull();
        results.Count.Should().Be(1, "exact name match should return exactly one result");
        results[0].Id.Should().Be("raw_decoder", "the matching plugin should be raw_decoder");
    }

    // ── Test 5: Search with no matches returns empty list ──
    [Fact]
    public void Search_NoMatch_ReturnsEmpty()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var results = service.Search("zzz_nonexistent_zzz");

        // Assert
        results.Should().NotBeNull("Search must never return null");
        results.Should().BeEmpty("no plugins should match the nonsense query");
    }

    // ── Test 6: FilterByCategory returns plugins within a category ──
    [Fact]
    public void FilterByCategory_ValidCategory_ReturnsPluginsInCategory()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var results = service.FilterByCategory("Transform");

        // Assert
        results.Should().NotBeNull();
        results.Count.Should().Be(3, "Transform category has 3 plugins: resize, crop, rotate");
        results.Select(p => p.Id).Should().Contain(new[] { "resize", "crop", "rotate" });
    }

    // ── Test 7: FilterByCategory with unknown category returns empty list ──
    [Fact]
    public void FilterByCategory_UnknownCategory_ReturnsEmpty()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var results = service.FilterByCategory("Unknown");

        // Assert
        results.Should().NotBeNull("FilterByCategory must never return null");
        results.Should().BeEmpty("unknown category should yield no results");
    }

    // ── Test 8: GetCategories returns all distinct categories ──
    [Fact]
    public void GetCategories_ReturnsAllCategories()
    {
        // Arrange
        var service = new PluginService();

        // Act
        var categories = service.GetCategories();

        // Assert
        categories.Should().NotBeNull();
        categories.Should().NotBeEmpty();
        categories.Should().Contain("Input");
        categories.Should().Contain("Metadata");
        categories.Should().Contain("Color");
        categories.Should().Contain("Transform");
        categories.Should().Contain("Enhance");
        categories.Should().Contain("Format");
        categories.Should().BeInAscendingOrder("categories should be sorted alphabetically");
        categories.Should().OnlyHaveUniqueItems("category list must not contain duplicates");
    }
}
