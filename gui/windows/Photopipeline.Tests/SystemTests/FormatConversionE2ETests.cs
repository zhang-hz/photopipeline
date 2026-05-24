using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;
using SkiaSharp;

namespace Photopipeline.Tests.SystemTests;

public sealed class FormatConversionE2ETests : IDisposable
{
    private readonly TestPipelineService _pipeline;
    private readonly string _tempDir;

    public FormatConversionE2ETests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"pp_conv_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
        _pipeline = new TestPipelineService(_tempDir);
        TestImageFactory.GenerateFullTestSet();
    }

    public void Dispose()
    {
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
    }

    [Fact]
    public async Task PngToJpeg_IdentityPipeline_ProducesValidOutput()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "pn" +
"g→jpeg" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);
        Assert.True(File.Exists(_pipeline.LastOutputPath));

        // Output should be decodable
        using var decoded = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(decoded);
        Assert.Equal(256, decoded!.Width);
        Assert.Equal(256, decoded.Height);
    }

    [Fact]
    public async Task IdentityPipeline_InputDimensions_Preserved()
    {
        var input = TestImageFactory.GetPath("solid_rgb_2048.png");
        var pipeline = new PipelineModel { Name = "identity" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 2048, 1536);
    }

    [Fact]
    public async Task IdentityPipeline_Large4000x3000_Preserved()
    {
        var input = TestImageFactory.GetPath("solid_rgb_4000.png");
        var pipeline = new PipelineModel { Name = "identity" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 4000, 3000);
    }

    [Fact]
    public async Task GrayscalePlugin_OutputStillDecodable_PNG()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "gray" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var decoded = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(decoded);
        Assert.True(decoded!.ColorType is SKColorType.Rgba8888 or SKColorType.Bgra8888);
    }

    [Fact]
    public async Task ResizePlugin_AspectRatioChange_OutputCorrectDimensions()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "resize" };
        var node = new PipelineNode { PluginId = "resize", DisplayName = "Resize" };
        node.Parameters["width"] = 200;
        node.Parameters["height"] = 100;
        pipeline.Nodes.Add(node);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 200, 100);
    }

    [Fact]
    public async Task MultipleConsecutiveResizes_FinalDimensions_Correct()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "double-resize" };
        var n1 = new PipelineNode { PluginId = "resize", DisplayName = "R1" };
        n1.Parameters["width"] = 512; n1.Parameters["height"] = 512;
        pipeline.Nodes.Add(n1);
        var n2 = new PipelineNode { PluginId = "resize", DisplayName = "R2" };
        n2.Parameters["width"] = 128; n2.Parameters["height"] = 128;
        pipeline.Nodes.Add(n2);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 128, 128);
    }
}
