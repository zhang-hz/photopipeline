using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;
using SkiaSharp;

namespace Photopipeline.Tests.SystemTests;

public sealed class MetadataRoundtripE2ETests : IDisposable
{
    private readonly TestPipelineService _pipeline;
    private readonly string _tempDir;

    public MetadataRoundtripE2ETests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"pp_meta_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
        _pipeline = new TestPipelineService(_tempDir);
        TestImageFactory.GenerateFullTestSet();
    }

    public void Dispose()
    {
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
    }

    [Fact]
    public async Task IdentityPipeline_DimensionsMetadata_Preserved()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "dims" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var orig = SKBitmap.Decode(input);
        using var outBmp = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(outBmp);
        Assert.Equal(orig!.Width, outBmp!.Width);
        Assert.Equal(orig.Height, outBmp.Height);
    }

    [Fact]
    public async Task IdentityPipeline_ColorType_Preserved()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "color" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var decoded = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(decoded);
        Assert.True(decoded!.ColorType is SKColorType.Rgba8888 or SKColorType.Bgra8888);
    }

    [Fact]
    public async Task GrayscalePipeline_OutputStillRgba()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "gray-meta" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var decoded = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(decoded);
        Assert.True(decoded!.ColorType is SKColorType.Rgba8888 or SKColorType.Bgra8888);
    }

    [Fact]
    public async Task ResizePipeline_OutputDimensions_MatchRequestedSize()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "resize-meta" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "R",
            Parameters = new Dictionary<string, object> { ["width"] = 800, ["height"] = 600 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 800, 600);
    }

    [Fact]
    public async Task BrightnessPipeline_Dimensions_Unchanged()
    {
        var input = TestImageFactory.GetPath("solid_rgb_2048.png");
        var pipeline = new PipelineModel { Name = "brt-meta" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "brightness", DisplayName = "Brt",
            Parameters = new Dictionary<string, object> { ["value"] = 0.3 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 2048, 1536);
    }

    [Fact]
    public async Task BlurPipeline_Dimensions_Unchanged()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "blur-meta" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "blur", DisplayName = "Blur",
            Parameters = new Dictionary<string, object> { ["sigma"] = 2.0f } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 256, 256);
    }

    [Fact]
    public async Task MultiNodeChain_OutputDimensions_MatchLastResize()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "multi-meta" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "R",
            Parameters = new Dictionary<string, object> { ["width"] = 400, ["height"] = 300 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 400, 300);
    }

    [Fact]
    public async Task JpegOutput_PixelValues_ApproximatelyPreserved()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "jpeg-meta" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "identity", DisplayName = "Id",
            Parameters = new Dictionary<string, object> { ["format"] = "jpeg", ["quality"] = 95 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        // After JPEG encoding, PSNR should be high
        var psnr = ImageAssert.ComputePSNR(input, _pipeline.LastOutputPath);
        Assert.True(psnr > 35, $"JPEG Q95 PSNR {psnr:F1}dB should be >35dB");
    }

    [Fact]
    public async Task PipelineService_ExecutionCount_Increments()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var initial = _pipeline.ExecutionCount;

        await _pipeline.ExecutePipelineAsync(new PipelineModel { Name = "test1" }, input);
        Assert.Equal(initial + 1, _pipeline.ExecutionCount);

        await _pipeline.ExecutePipelineAsync(new PipelineModel { Name = "test2" }, input);
        Assert.Equal(initial + 2, _pipeline.ExecutionCount);
    }

    [Fact]
    public async Task PipelineService_LastOutputPath_UpdatedEachRun()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");

        await _pipeline.ExecutePipelineAsync(new PipelineModel { Name = "run1" }, input);
        var path1 = _pipeline.LastOutputPath;

        await _pipeline.ExecutePipelineAsync(new PipelineModel { Name = "run2" }, input);
        var path2 = _pipeline.LastOutputPath;

        Assert.NotEqual(path1, path2);
        Assert.True(File.Exists(path1));
        Assert.True(File.Exists(path2));
    }
}
