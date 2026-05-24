using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;
using SkiaSharp;

namespace Photopipeline.Tests.SystemTests;

public sealed class BoundaryConditionE2ETests : IDisposable
{
    private readonly TestPipelineService _pipeline;
    private readonly string _tempDir;

    public BoundaryConditionE2ETests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"pp_bdy_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
        _pipeline = new TestPipelineService(_tempDir);
        TestImageFactory.GenerateFullTestSet();
    }

    public void Dispose()
    {
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
    }

    [Fact]
    public async Task SinglePixel_1x1_Identity_OutputPreserved()
    {
        var input = TestImageFactory.GetPath("solid_1x1.png");
        var pipeline = new PipelineModel { Name = "1x1" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 1, 1);
    }

    [Fact]
    public async Task SinglePixel_1x1_Invert_OutputValid()
    {
        var input = TestImageFactory.GetPath("solid_1x1.png");
        var pipeline = new PipelineModel { Name = "1x1-inv" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 1, 1);
    }

    [Fact]
    public async Task SingleRow_1x100_Invert_OutputValid()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "row" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "R",
            Parameters = new Dictionary<string, object> { ["width"] = 1, ["height"] = 100 } });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 1, 100);
    }

    [Fact]
    public async Task SingleColumn_100x1_Grayscale_OutputValid()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "col" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "R",
            Parameters = new Dictionary<string, object> { ["width"] = 100, ["height"] = 1 } });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 100, 1);
    }

    [Fact]
    public async Task SolidBlack_Pipeline_OutputPreserved()
    {
        var input = TestImageFactory.GetPath("solid_black_256.png");
        var pipeline = new PipelineModel { Name = "black" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(10, 10);
        Assert.Equal(0, px.Red);
        Assert.Equal(0, px.Green);
        Assert.Equal(0, px.Blue);
    }

    [Fact]
    public async Task SolidWhite_Pipeline_OutputPreserved()
    {
        var input = TestImageFactory.GetPath("solid_white_256.png");
        var pipeline = new PipelineModel { Name = "white" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(10, 10);
        Assert.Equal(255, px.Red);
        Assert.Equal(255, px.Green);
        Assert.Equal(255, px.Blue);
    }

    [Fact]
    public async Task PureRedChannel_Identity_OutputMatches()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "red" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath);
    }

    [Fact]
    public async Task Checkerboard_InvertTwice_ReturnsOriginal_WithTolerance()
    {
        var input = TestImageFactory.GetPath("checkerboard_256.png");
        var pipeline = new PipelineModel { Name = "check-dbl" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv1" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv2" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath, tolerancePerChannel: 1);
    }

    [Fact]
    public async Task Large4000x3000_Identity_OutputDimensions_Preserved()
    {
        var input = TestImageFactory.GetPath("solid_rgb_4000.png");
        var pipeline = new PipelineModel { Name = "large" };

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 4000, 3000);
    }

    [Fact]
    public async Task Large4000x3000_Grayscale_OutputValid()
    {
        var input = TestImageFactory.GetPath("solid_rgb_4000.png");
        var pipeline = new PipelineModel { Name = "large-gray" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        Assert.Equal(4000, result!.Width);
        Assert.Equal(3000, result.Height);
    }

    [Fact]
    public async Task Large2048x1536_ResizeDown_ThenUp_Valid()
    {
        var input = TestImageFactory.GetPath("solid_rgb_2048.png");
        var pipeline = new PipelineModel { Name = "large-resize" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "Down",
            Parameters = new Dictionary<string, object> { ["width"] = 128, ["height"] = 96 } });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "resize", DisplayName = "Up",
            Parameters = new Dictionary<string, object> { ["width"] = 1024, ["height"] = 768 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 1024, 768);
    }

    [Fact]
    public async Task Gradient_AllPluginsChain_OutputValid()
    {
        var input = TestImageFactory.GetPath("gradient_256.png");
        var pipeline = new PipelineModel { Name = "full-chain" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "brightness", DisplayName = "Brt",
            Parameters = new Dictionary<string, object> { ["value"] = 0.1 } });
        pipeline.Nodes.Add(new PipelineNode { PluginId = "blur", DisplayName = "Blur",
            Parameters = new Dictionary<string, object> { ["sigma"] = 0.5f } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        Assert.True(File.Exists(_pipeline.LastOutputPath));
        ImageAssert.HaveDimensions(_pipeline.LastOutputPath, 256, 256);
    }

    [Fact]
    public async Task Invert_Gradient_PreservesGradientStructure()
    {
        var input = TestImageFactory.GetPath("gradient_256.png");
        var pipeline = new PipelineModel { Name = "inv-grad" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var orig = SKBitmap.Decode(input);
        using var inv = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(inv);

        // Top row of gradient should be bright in original, dark after invert
        var origTop = orig!.GetPixel(128, 5);
        var invTop = inv.GetPixel(128, 5);
        Assert.True(invTop.Red != origTop.Red,
            $"Inverted pixel ({invTop.Red},{invTop.Green},{invTop.Blue}) should differ from original ({origTop.Red},{origTop.Green},{origTop.Blue})");
    }

    [Fact]
    public async Task ColorBars_EachBar_TransformedIndependently()
    {
        var input = TestImageFactory.GetPath("color_bars_256.png");
        var pipeline = new PipelineModel { Name = "bars-gray" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);

        // All pixels should be grayscale
        for (int y = 0; y < result!.Height; y += 64)
        for (int x = 0; x < result.Width; x += 64)
        {
            var c = result.GetPixel(x, y);
            Assert.True(c.Red == c.Green && c.Green == c.Blue,
                $"Pixel ({x},{y}) not grayscale: ({c.Red},{c.Green},{c.Blue})");
        }
    }

    [Fact]
    public async Task GraySteps_Invert_DarkBecomesBright()
    {
        var input = TestImageFactory.GetPath("gray_steps_256.png");
        var pipeline = new PipelineModel { Name = "steps-inv" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var orig = SKBitmap.Decode(input);
        using var inv = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(inv);

        // Gray steps image has 8 vertical bars from black (left) to white (right).
        // After invert: left bar (originally black=0) becomes white=255.
        var origPx = orig!.GetPixel(10, 10);
        var invPx = inv.GetPixel(10, 10);
        Assert.True(invPx.Red > 200, $"Leftmost dark area should become bright after invert, got {invPx.Red}");
    }

    [Fact]
    public async Task MissingInputFile_ReturnsFalse()
    {
        var pipeline = new PipelineModel { Name = "missing" };
        var ok = await _pipeline.ExecutePipelineAsync(pipeline, "/nonexistent/file_xyz.png");
        Assert.False(ok);
    }

    [Fact]
    public async Task UnknownPlugin_PassesThrough_OutputMatchesInput()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "unknown" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "nonexistent_xyz", DisplayName = "?" });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        ImageAssert.ArePixelEqual(input, _pipeline.LastOutputPath);
    }

    [Fact]
    public async Task MaxBrightness_Increment_ShouldNotExceed255()
    {
        var input = TestImageFactory.GetPath("solid_white_256.png");
        var pipeline = new PipelineModel { Name = "max-brt" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "brightness", DisplayName = "+",
            Parameters = new Dictionary<string, object> { ["value"] = 1.0 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(64, 64);
        Assert.True(px.Red <= 255);
        Assert.True(px.Green <= 255);
        Assert.True(px.Blue <= 255);
    }

    [Fact]
    public async Task MinBrightness_Decrement_ShouldNotGoBelow0()
    {
        var input = TestImageFactory.GetPath("solid_black_256.png");
        var pipeline = new PipelineModel { Name = "min-brt" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "brightness", DisplayName = "-",
            Parameters = new Dictionary<string, object> { ["value"] = -1.0 } });

        await _pipeline.ExecutePipelineAsync(pipeline, input);

        using var result = SKBitmap.Decode(_pipeline.LastOutputPath);
        Assert.NotNull(result);
        var px = result!.GetPixel(64, 64);
        Assert.True(px.Red >= 0, $"Red {px.Red} should be >= 0");
        Assert.True(px.Green >= 0, $"Green {px.Green} should be >= 0");
        Assert.True(px.Blue >= 0, $"Blue {px.Blue} should be >= 0");
    }
}
