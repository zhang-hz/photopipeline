using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.SystemTests;

public sealed class ImagePipelineOutputE2ETests : IDisposable
{
    private readonly TestPipelineService _pipeline;
    private readonly string _tempDir;

    public ImagePipelineOutputE2ETests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"pp_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
        _pipeline = new TestPipelineService(_tempDir);
        TestImageFactory.GenerateFullTestSet();
    }

    public void Dispose()
    {
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
    }

    // ═══ Identity pipeline (no nodes) ═══
    [Fact]
    public async Task IdentityPipeline_SolidRGB_OutputMatchesInput()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "identity" };

        var ok = await _pipeline.ExecutePipelineAsync(pipeline, input);
        Assert.True(ok);
        Assert.True(File.Exists(_pipeline.LastOutputPath));

        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath);
    }

    [Fact]
    public async Task IdentityPipeline_Gradient_OutputMatchesInput()
    {
        var input = TestImageFactory.GetPath("gradient_256.png");
        var pipeline = new PipelineModel { Name = "identity" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);
        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath);
    }

    [Fact]
    public async Task IdentityPipeline_Checkerboard_OutputMatchesInput()
    {
        var input = TestImageFactory.GetPath("checkerboard_256.png");
        var pipeline = new PipelineModel { Name = "identity" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);
        ImageAssert.HaveSameHash(input, _pipeline.LastOutputPath);
    }

    // ═══ Single-plugin operations ═══
    [Fact]
    public async Task InvertPlugin_Applies_InvertsAllPixels()
    {
        var input = TestImageFactory.GetPath("solid_white_256.png");
        var pipeline = new PipelineModel { Name = "invert" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);
        Assert.True(File.Exists(_pipeline.LastOutputPath));

        // White inverted → black
        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(10, 10);
        Assert.Equal(0, px.Red);
        Assert.Equal(0, px.Green);
        Assert.Equal(0, px.Blue);
    }

    [Fact]
    public async Task InvertPlugin_SolidBlack_BecomesWhite()
    {
        var input = TestImageFactory.GetPath("solid_black_256.png");
        var pipeline = new PipelineModel { Name = "invert" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(10, 10);
        Assert.Equal(255, px.Red);
        Assert.Equal(255, px.Green);
        Assert.Equal(255, px.Blue);
    }

    [Fact]
    public async Task InvertPlugin_DoubleInvert_ReturnsOriginal()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "double-invert" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert1" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert2" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        // Double invert should match original (tolerance 1 due to intermediate rounding)
        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath, tolerancePerChannel: 1);
    }

    [Fact]
    public async Task GrayscalePlugin_ColoredInput_AllChannelsEqual()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "grayscale" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        // Every pixel should have R==G==B
        for (int y = 0; y < result!.Height; y += 32)
        for (int x = 0; x < result.Width; x += 32)
        {
            var c = result.GetPixel(x, y);
            Assert.True(c.Red == c.Green && c.Green == c.Blue,
                $"Pixel ({x},{y}) not grayscale: ({c.Red},{c.Green},{c.Blue})");
        }
    }

    [Fact]
    public async Task BrightnessPlugin_Positive_IncreasesLuminance()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png"); // (128,64,32)
        var pipeline = new PipelineModel { Name = "brightness" };
        var node = new PipelineNode { PluginId = "brightness", DisplayName = "Bright" };
        node.Parameters["value"] = 0.5; // +50% brightness
        pipeline.Nodes.Add(node);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(64, 64);
        // Original (128,64,32) + ~127 → (255,191,159)
        Assert.True(px.Red > 128, $"Red {px.Red} should be >128");
        Assert.True(px.Green > 64, $"Green {px.Green} should be >64");
        Assert.True(px.Blue > 32, $"Blue {px.Blue} should be >32");
    }

    [Fact]
    public async Task BrightnessPlugin_Negative_DecreasesLuminance()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png"); // (128,64,32)
        var pipeline = new PipelineModel { Name = "brightness" };
        var node = new PipelineNode { PluginId = "brightness", DisplayName = "Dark" };
        node.Parameters["value"] = -0.5;
        pipeline.Nodes.Add(node);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(64, 64);
        Assert.True(px.Red < 128, $"Red {px.Red} should be <128");
    }

    [Fact]
    public async Task ResizePlugin_256to512_DimensionsCorrect()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "resize" };
        var node = new PipelineNode { PluginId = "resize", DisplayName = "Resize" };
        node.Parameters["width"] = 512;
        node.Parameters["height"] = 512;
        pipeline.Nodes.Add(node);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 512, 512);
    }

    [Fact]
    public async Task ResizePlugin_256to128_Downscales()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "resize" };
        var node = new PipelineNode { PluginId = "resize", DisplayName = "Resize" };
        node.Parameters["width"] = 128;
        node.Parameters["height"] = 128;
        pipeline.Nodes.Add(node);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 128, 128);
    }

    // ═══ Multi-node chains ═══
    [Fact]
    public async Task TwoNodeChain_InvertThenGrayscale_CorrectPipeline()
    {
        var input = TestImageFactory.GetPath("solid_black_256.png");
        var pipeline = new PipelineModel { Name = "invert+gray" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(10, 10);
        Assert.Equal(255, px.Red);    // black→white→white
        Assert.Equal(255, px.Green);
        Assert.Equal(255, px.Blue);
    }

    [Fact]
    public async Task ThreeNodeChain_GrayscaleInvertBrightness_PipelineOrder()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "3-chain" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert" });
        var brightNode = new PipelineNode { PluginId = "brightness", DisplayName = "Bright" };
        brightNode.Parameters["value"] = 0.2;
        pipeline.Nodes.Add(brightNode);

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SkiaSharp.SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(64, 64);
        // All channels equal (grayscale) ✓
        Assert.True(px.Red == px.Green && px.Green == px.Blue,
            $"Expected grayscale, got ({px.Red},{px.Green},{px.Blue})");
    }

    [Fact]
    public async Task FiveNodeChain_AllPlugins_ExecutesSuccessfully()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "5-chain" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "brightness", DisplayName = "Bright",
            Parameters = new Dictionary<string, object> { ["value"] = 0.1 } });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "blur", DisplayName = "Blur",
            Parameters = new Dictionary<string, object> { ["sigma"] = 1.0f } });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "Resize",
            Parameters = new Dictionary<string, object> { ["width"] = 128, ["height"] = 128 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 128, 128);
    }

    // ═══ Boundary conditions ═══
    [Fact]
    public async Task EmptyPipeline_NoNodes_ReturnsInputUnchanged()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "empty" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath);
    }

    [Fact]
    public async Task SinglePixelImage_ProcessesWithoutError()
    {
        var input = TestImageFactory.GetPath("solid_1x1.png");
        var pipeline = new PipelineModel { Name = "1x1" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Invert" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 1, 1);
    }

    [Fact]
    public async Task MissingInputFile_ReturnsFalse()
    {
        var pipeline = new PipelineModel { Name = "test" };
        var ok = await _pipeline.ExecutePipelineAsync(pipeline, "/nonexistent/path.png");
        Assert.False(ok);
    }

    [Fact]
    public async Task UnknownPluginType_ReturnsInputUnchanged()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "unknown" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "nonexistent_plugin", DisplayName = "???" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        // Unknown plugin should pass through
        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath);
    }
}
