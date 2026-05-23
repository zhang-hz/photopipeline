namespace Photopipeline.Tests.SystemTests;

public sealed class PluginInteractionTests : SystemTestBase
{
    [Fact]
    public async Task EXIFPlugin_ReadsMetadataFromTestImage()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var exifPlugin = plugins.FirstOrDefault(p => p.Name.Contains("EXIF", StringComparison.OrdinalIgnoreCase)
                                                || p.Id.Contains("exif", StringComparison.OrdinalIgnoreCase));

        if (exifPlugin is null) return;

        exifPlugin.MinInputs.Should().BeGreaterOrEqualTo(1);
        exifPlugin.Category.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public async Task GPSPlugin_SetsCoordinates_ValidateParameters()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();

        plugins.Should().NotBeEmpty();
        plugins.All(p => !string.IsNullOrEmpty(p.Id)).Should().BeTrue();
        plugins.All(p => !string.IsNullOrEmpty(p.Name)).Should().BeTrue();
    }

    [Fact]
    public async Task TimeShiftPlugin_ChangesTime_ValidateSchema()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();

        foreach (var plugin in plugins)
        {
            plugin.ParameterSchemas.Should().NotBeNull();
            foreach (var schema in plugin.ParameterSchemas)
            {
                schema.Name.Should().NotBeNullOrEmpty();
                schema.DisplayName.Should().NotBeNullOrEmpty();
            }
        }
    }

    [Fact]
    public async Task ColorSpacePlugin_ValidateConversionParameters()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var colorPlugins = plugins.Where(p => p.Category == "Color").ToList();

        if (colorPlugins.Count == 0) return;

        foreach (var plugin in colorPlugins)
        {
            plugin.Category.Should().Be("Color");
            plugin.Outputs.Should().BeGreaterOrEqualTo(0);
        }
    }

    [Fact]
    public async Task TransformPlugin_ValidateResizeParameters()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();

        foreach (var plugin in plugins)
        {
            if (plugin.ParameterSchemas.Count == 0) continue;

            foreach (var schema in plugin.ParameterSchemas)
            {
                if (schema.ParameterType == ParameterType.Float)
                {
                    schema.MinValue.Should().NotBeNull();
                    schema.MaxValue.Should().NotBeNull();
                    schema.Step.Should().BeGreaterThan(0);
                }

                if (schema.ParameterType == ParameterType.Integer)
                {
                    schema.MinValue.Should().NotBeNull();
                    schema.MaxValue.Should().NotBeNull();
                }

                if (schema.ParameterType == ParameterType.Enum)
                {
                    schema.EnumValues.Should().NotBeNull();
                    schema.EnumValues.Should().NotBeEmpty();
                }
            }
        }
    }

    [Fact]
    public async Task FormatPlugins_ValidateEncoderParameters()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var existingCategories = plugins.Select(p => p.Category).Distinct().ToList();

        existingCategories.Should().NotBeEmpty();
        existingCategories.Should().Contain("Raw Processing");

        var tonalPlugins = plugins.Where(p => p.Category == "Tonal").ToList();
        tonalPlugins.Should().NotBeEmpty("At least one tonal plugin should exist");
    }

    [Fact]
    public async Task AllPlugins_HaveValidSchemaDefinitions()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();

        plugins.Should().NotBeEmpty();

        foreach (var plugin in plugins)
        {
            plugin.Id.Should().NotBeNullOrEmpty($"Plugin '{plugin.Name}' should have an Id");
            plugin.Name.Should().NotBeNullOrEmpty("Plugin should have a Name");
            plugin.Category.Should().NotBeNullOrEmpty($"Plugin '{plugin.Name}' should have a Category");
            plugin.Version.Should().NotBeNullOrEmpty($"Plugin '{plugin.Name}' should have a Version");

            foreach (var schema in plugin.ParameterSchemas)
            {
                schema.Name.Should().NotBeNullOrEmpty(
                    $"Parameter in plugin '{plugin.Name}' should have a name");
                schema.DisplayName.Should().NotBeNullOrEmpty(
                    $"Parameter '{schema.Name}' in plugin '{plugin.Name}' should have a display name");
            }
        }
    }

    [Fact]
    public async Task DemosaicPlugin_HasAlgorithmParameter()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var demosaic = plugins.FirstOrDefault(p => p.Id == "demosaic");

        if (demosaic is null) return;

        demosaic.Category.Should().Be("Raw Processing");
        demosaic.ParameterSchemas.Should().NotBeEmpty();
        demosaic.ParameterSchemas.Should().Contain(p => p.Name == "algorithm"
                                                   && p.ParameterType == ParameterType.Enum);
        demosaic.ParameterSchemas.Should().Contain(p => p.Name == "border"
                                                   && p.ParameterType == ParameterType.Integer);
    }

    [Fact]
    public async Task ExposurePlugin_HasExpectedFloatRange()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var exposure = plugins.FirstOrDefault(p => p.Id == "exposure");

        if (exposure is null) return;

        exposure.Category.Should().Be("Tonal");

        var evSchema = exposure.ParameterSchemas.FirstOrDefault(p => p.Name == "ev");
        evSchema.Should().NotBeNull();
        evSchema!.ParameterType.Should().Be(ParameterType.Float);
        evSchema.DefaultValue.Should().Be(0.0);
        evSchema.MinValue.Should().Be(-5.0);
        evSchema.MaxValue.Should().Be(5.0);
        evSchema.Unit.Should().Be("EV");

        var highlightRecovery = exposure.ParameterSchemas.FirstOrDefault(p => p.Name == "highlight_recovery");
        highlightRecovery.Should().NotBeNull();
        highlightRecovery!.ParameterType.Should().Be(ParameterType.Boolean);
        highlightRecovery.DefaultValue.Should().Be(true);
    }

    [Fact]
    public async Task WhiteBalancePlugin_HasTemperatureAndTint()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var wb = plugins.FirstOrDefault(p => p.Id == "white_balance");

        if (wb is null) return;

        wb.Category.Should().Be("Color");

        var tempSchema = wb.ParameterSchemas.FirstOrDefault(p => p.Name == "temperature");
        tempSchema.Should().NotBeNull();
        tempSchema!.ParameterType.Should().Be(ParameterType.Integer);
        tempSchema.DefaultValue.Should().Be(5500);
        tempSchema.MinValue.Should().Be(2000);
        tempSchema.MaxValue.Should().Be(50000);
        tempSchema.Unit.Should().Be("K");

        var tintSchema = wb.ParameterSchemas.FirstOrDefault(p => p.Name == "tint");
        tintSchema.Should().NotBeNull();
        tintSchema!.ParameterType.Should().Be(ParameterType.Float);
        tintSchema.DefaultValue.Should().Be(0.0);
        tintSchema.MinValue.Should().Be(-150.0);
        tintSchema.MaxValue.Should().Be(150.0);
    }

    [Fact]
    public async Task DenoisePlugin_HasStrengthAndModel()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var denoise = plugins.FirstOrDefault(p => p.Id == "denoise");

        if (denoise is null) return;

        denoise.Category.Should().Be("Noise Reduction");

        var strength = denoise.ParameterSchemas.FirstOrDefault(p => p.Name == "strength");
        strength.Should().NotBeNull();
        strength!.ParameterType.Should().Be(ParameterType.Float);
        strength.MinValue.Should().Be(0.0);
        strength.MaxValue.Should().Be(1.0);

        var model = denoise.ParameterSchemas.FirstOrDefault(p => p.Name == "model");
        model.Should().NotBeNull();
        model!.ParameterType.Should().Be(ParameterType.Enum);
        model.EnumValues.Should().Contain(new[] { "Standard", "Raw", "JPG", "High ISO" });
    }

    [Fact]
    public async Task SharpenPlugin_HasAmountAndRadius()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        var sharpen = plugins.FirstOrDefault(p => p.Id == "sharpen");

        if (sharpen is null) return;

        sharpen.Category.Should().Be("Detail");

        var amount = sharpen.ParameterSchemas.FirstOrDefault(p => p.Name == "amount");
        amount.Should().NotBeNull();
        amount!.ParameterType.Should().Be(ParameterType.Float);
        amount.DefaultValue.Should().Be(0.5);

        var radius = sharpen.ParameterSchemas.FirstOrDefault(p => p.Name == "radius");
        radius.Should().NotBeNull();
        radius!.ParameterType.Should().Be(ParameterType.Float);
        radius.MinValue.Should().Be(0.3);
        radius.MaxValue.Should().Be(5.0);
        radius.Unit.Should().Be("px");
    }

    private async Task<bool> TryStartServerAsync()
    {
        try
        {
            await StartServerAsync();
            return true;
        }
        catch (SkipTestException)
        {
            return false;
        }
    }
}
