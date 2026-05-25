namespace Photopipeline.Tests.UnitTests.Models;

public sealed class PluginInfoTests
{
    [Fact]
    public void PluginInfo_Creation_DefaultValues()
    {
        var plugin = new PluginInfo();

        plugin.Id.Should().BeEmpty();
        plugin.Name.Should().BeEmpty();
        plugin.Version.Should().BeEmpty();
        plugin.Category.Should().BeEmpty();
        plugin.Description.Should().BeEmpty();
        plugin.ParameterSchema.Should().BeEmpty();
        plugin.Icon.Should().BeNull();
        plugin.Color.Should().BeNull();
    }

    [Fact]
    public void PluginInfo_SetAllProperties()
    {
        var plugin = new PluginInfo
        {
            Id = "denoise_v1",
            Name = "AI Denoise",
            Version = "2.0.0",
            Category = "Noise Reduction",
            Description = "Deep learning denoising",
            Icon = "denoise_icon",
            Color = "#FF5500"
        };

        plugin.Id.Should().Be("denoise_v1");
        plugin.Name.Should().Be("AI Denoise");
        plugin.Version.Should().Be("2.0.0");
        plugin.Category.Should().Be("Noise Reduction");
        plugin.Description.Should().Be("Deep learning denoising");
        plugin.Icon.Should().Be("denoise_icon");
        plugin.Color.Should().Be("#FF5500");
    }

    [Fact]
    public void PluginInfo_ParameterSchema_JsonStyleDict()
    {
        var plugin = new PluginInfo();
        plugin.ParameterSchema["strength"] = new Dictionary<string, object>
        {
            ["type"] = "float",
            ["default"] = 0.5,
            ["min"] = 0.0,
            ["max"] = 1.0
        };
        plugin.ParameterSchema["enabled"] = new Dictionary<string, object>
        {
            ["type"] = "bool",
            ["default"] = true
        };

        plugin.ParameterSchema.Should().HaveCount(2);
        var strength = plugin.ParameterSchema["strength"] as Dictionary<string, object>;
        strength.Should().NotBeNull();
        strength!["type"].Should().Be("float");
    }

    [Fact]
    public void BatchSpec_Defaults()
    {
        var spec = new BatchSpec();

        spec.PipelineConfigPath.Should().BeEmpty();
        spec.FilePattern.Should().BeEmpty();
        spec.OutputDir.Should().BeEmpty();
        spec.Parallel.Should().Be(1);
        spec.Resume.Should().BeFalse();
    }

    [Fact]
    public void BatchProgress_TracksState()
    {
        var progress = new BatchProgress
        {
            Status = BatchStatus.Running,
            TotalFiles = 10,
            CompletedFiles = 5,
            FailedFiles = 1,
            CurrentFile = "photo_006.dng",
            Fraction = 0.55f,
            ProgressDetails = "Denoising..."
        };

        progress.Status.Should().Be(BatchStatus.Running);
        progress.TotalFiles.Should().Be(10);
        progress.CompletedFiles.Should().Be(5);
        progress.FailedFiles.Should().Be(1);
        progress.CurrentFile.Should().Be("photo_006.dng");
        progress.Fraction.Should().Be(0.55f);
    }

    [Fact]
    public void BatchStatus_AllEnumValues()
    {
        var values = Enum.GetValues<BatchStatus>();
        values.Should().Contain(new[] {
            BatchStatus.Pending, BatchStatus.Running,
            BatchStatus.Done, BatchStatus.Canceled, BatchStatus.Error });
    }
}
