using System.Collections.ObjectModel;

namespace Photopipeline.Tests.UnitTests.Models;

public sealed class PluginInfoTests
{
    [Fact]
    public void PluginInfo_Creation_DefaultValues()
    {
        var plugin = new PluginInfo();

        plugin.Id.Should().BeEmpty();
        plugin.Name.Should().BeEmpty();
        plugin.Category.Should().BeEmpty();
        plugin.Description.Should().BeEmpty();
        plugin.Version.Should().Be("1.0.0");
        plugin.MinInputs.Should().Be(1);
        plugin.MaxInputs.Should().Be(1);
        plugin.Outputs.Should().Be(1);
        plugin.SupportsBatching.Should().BeTrue();
        plugin.ParameterSchemas.Should().BeEmpty();
        plugin.IconGlyph.Should().Be("\uE8B7");
    }

    [Fact]
    public void PluginInfo_FromManifest_SetsAllProperties()
    {
        var plugin = new PluginInfo
        {
            Id = "exif_read",
            Name = "EXIF Reader",
            Category = "Metadata",
            Description = "Reads EXIF metadata from image files",
            Version = "2.1.0",
            MinInputs = 1,
            MaxInputs = 1,
            Outputs = 1,
            SupportsBatching = true
        };

        plugin.Id.Should().Be("exif_read");
        plugin.Name.Should().Be("EXIF Reader");
        plugin.Category.Should().Be("Metadata");
        plugin.Description.Should().Be("Reads EXIF metadata from image files");
        plugin.Version.Should().Be("2.1.0");
    }

    [Fact]
    public void PluginInfo_WithParameterSchemas_HasCorrectCount()
    {
        var plugin = new PluginInfo();
        plugin.ParameterSchemas.Add(new ParameterSchema { Name = "temperature", ParameterType = ParameterType.Integer });
        plugin.ParameterSchemas.Add(new ParameterSchema { Name = "tint", ParameterType = ParameterType.Float });
        plugin.ParameterSchemas.Add(new ParameterSchema { Name = "auto_wb", ParameterType = ParameterType.Boolean });

        plugin.ParameterSchemas.Should().HaveCount(3);
    }

    [Fact]
    public void ParameterSchema_RequiredFields_Present()
    {
        var schema = new ParameterSchema
        {
            Name = "exposure",
            DisplayName = "Exposure (EV)",
            Description = "Exposure compensation in stops",
            ParameterType = ParameterType.Float,
            DefaultValue = 0.0,
            MinValue = -5.0,
            MaxValue = 5.0,
            IsRequired = true,
            Step = 0.01,
            Unit = "EV",
            DecimalPlaces = 2
        };

        schema.Name.Should().Be("exposure");
        schema.DisplayName.Should().Be("Exposure (EV)");
        schema.Description.Should().Be("Exposure compensation in stops");
        schema.ParameterType.Should().Be(ParameterType.Float);
        schema.DefaultValue.Should().Be(0.0);
        schema.MinValue.Should().Be(-5.0);
        schema.MaxValue.Should().Be(5.0);
        schema.IsRequired.Should().BeTrue();
        schema.Step.Should().Be(0.01);
        schema.Unit.Should().Be("EV");
        schema.DecimalPlaces.Should().Be(2);
    }

    [Theory]
    [InlineData(ParameterType.String)]
    [InlineData(ParameterType.Integer)]
    [InlineData(ParameterType.Float)]
    [InlineData(ParameterType.Boolean)]
    [InlineData(ParameterType.Enum)]
    [InlineData(ParameterType.Color)]
    [InlineData(ParameterType.FilePath)]
    [InlineData(ParameterType.DirectoryPath)]
    [InlineData(ParameterType.Percentage)]
    public void ParameterType_EnumConversion_AllValuesSupported(ParameterType type)
    {
        var schema = new ParameterSchema
        {
            Name = "param",
            ParameterType = type
        };

        schema.ParameterType.Should().Be(type);
    }

    [Fact]
    public void ParameterSchema_EnumValues_PopulatedCorrectly()
    {
        var schema = new ParameterSchema
        {
            Name = "algorithm",
            DisplayName = "Algorithm",
            ParameterType = ParameterType.Enum,
            EnumValues = new ObservableCollection<object> { "AMaZE", "LMMSE", "VNG4", "PPG", "Bilinear" },
            DefaultValue = "AMaZE"
        };

        schema.EnumValues.Should().HaveCount(5);
        schema.EnumValues[0].Should().Be("AMaZE");
        schema.EnumValues[4].Should().Be("Bilinear");
        schema.DefaultValue.Should().Be("AMaZE");
    }

    [Fact]
    public void PluginInfo_MultiInputPlugin_HasCorrectMinMaxInputs()
    {
        var plugin = new PluginInfo
        {
            Id = "merge_hdr",
            Name = "HDR Merge",
            Category = "Composite",
            MinInputs = 2,
            MaxInputs = 9,
            Outputs = 1
        };

        plugin.MinInputs.Should().Be(2);
        plugin.MaxInputs.Should().Be(9);
    }

    [Fact]
    public void PluginInfo_MultiOutputPlugin_HasCorrectOutputs()
    {
        var plugin = new PluginInfo
        {
            Id = "split_channels",
            Name = "Split Channels",
            Category = "Color",
            Outputs = 3
        };

        plugin.Outputs.Should().Be(3);
    }

    [Fact]
    public void PluginInfo_Category_GroupingLogic_SameCategory()
    {
        var plugin1 = new PluginInfo { Id = "a", Category = "Color" };
        var plugin2 = new PluginInfo { Id = "b", Category = "Color" };

        plugin1.Category.Should().Be(plugin2.Category);
    }

    [Fact]
    public void PluginInfo_Category_GroupingLogic_DifferentCategories()
    {
        var plugin1 = new PluginInfo { Id = "a", Category = "Color" };
        var plugin2 = new PluginInfo { Id = "b", Category = "Metadata" };
        var plugin3 = new PluginInfo { Id = "c", Category = "Tonal" };

        plugin1.Category.Should().NotBe(plugin2.Category);
        plugin1.Category.Should().NotBe(plugin3.Category);
    }

    [Fact]
    public void ParameterSchema_Integer_StepDefaultsToOne()
    {
        var schema = new ParameterSchema
        {
            Name = "border",
            ParameterType = ParameterType.Integer,
            DefaultValue = 3,
            MinValue = 0,
            MaxValue = 8
        };

        schema.Step.Should().Be(1.0);
    }

    [Fact]
    public void ParameterSchema_DefaultValues_ForAllTypes()
    {
        var stringSchema = new ParameterSchema { Name = "s", ParameterType = ParameterType.String, DefaultValue = "hello" };
        var intSchema = new ParameterSchema { Name = "i", ParameterType = ParameterType.Integer, DefaultValue = 42 };
        var floatSchema = new ParameterSchema { Name = "f", ParameterType = ParameterType.Float, DefaultValue = 3.14 };
        var boolSchema = new ParameterSchema { Name = "b", ParameterType = ParameterType.Boolean, DefaultValue = true };

        stringSchema.DefaultValue.Should().Be("hello");
        intSchema.DefaultValue.Should().Be(42);
        floatSchema.DefaultValue.Should().Be(3.14);
        boolSchema.DefaultValue.Should().Be(true);
    }

    [Fact]
    public void PluginInfo_SupportsBatching_DefaultTrue()
    {
        var plugin = new PluginInfo();

        plugin.SupportsBatching.Should().BeTrue();

        plugin.SupportsBatching = false;
        plugin.SupportsBatching.Should().BeFalse();
    }
}
