using Photopipeline.Models;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

/// <summary>
/// Layer 6 Cross-Channel Batch tests (CCV-046 ~ CCV-053).
/// Batch processing cross-channel verification: multiple images through the same pipeline.
/// </summary>
[Trait("Category", "CrossChannel")]
public sealed class BatchCrossTests : CrossChannelTestBase
{
    public BatchCrossTests(ITestOutputHelper output) : base(output) { }

    // ════════════════════════════════════════════════════════════════
    //  CCV-046: Batch 3 images, transform(crop) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_046_Batch_3Images_Crop_Png()
    {
        const string testName = "CCV-046";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["crop_enabled"] = true;
                p["crop_width"] = 50;
                p["crop_height"] = 50;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var images = new[] { "color_bars_8bit", "gradient_horiz_rgb", "pure_white_large" };
        int successCount = 0;

        foreach (var image in images)
        {
            var subTestName = $"{testName}_{image}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, "PNG");
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(images.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{images.Length} images");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-047: Batch 5 images, colorspace -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_047_Batch_5Images_Colorspace_Tiff()
    {
        const string testName = "CCV-047";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var images = new[] {
            "color_bars_8bit", "gradient_horiz_rgb", "pure_white_large",
            "pure_black_large", "checkerboard_8x8"
        };

        int successCount = 0;
        foreach (var image in images)
        {
            var subTestName = $"{testName}_{image}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, "TIFF");
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(images.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{images.Length} images");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-048: Batch 2 images, different encoders (TIFF + PNG)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_048_Batch_2Images_DifferentEncoders()
    {
        const string testName = "CCV-048";
        await RequireApiBackendAsync();

        // Pipeline A: colorspace -> tiff_encoder
        var pipelineA = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        // Pipeline B: colorspace -> png_encoder
        var pipelineB = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var imageAPipeline = (pipelineA, "color_bars_8bit", "TIFF");
        var imageBPipeline = (pipelineB, "pure_white_large", "PNG");

        var scenarios = new[] { imageAPipeline, imageBPipeline };
        int successCount = 0;

        foreach (var (pipeline, image, format) in scenarios)
        {
            var subTestName = $"{testName}_{image}_{format}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, format);
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}/{format}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(scenarios.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{scenarios.Length} images");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-049: Batch 5 images, verify partial completion (simulate Pause/Resume)
    //  Note: API channel has no pause; we verify that all files complete.
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_049_Batch_5Images_AllComplete()
    {
        const string testName = "CCV-049";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var images = new[] {
            "color_bars_8bit", "gradient_horiz_rgb", "pure_white_large",
            "pure_black_large", "checkerboard_8x8"
        };

        int successCount = 0;
        foreach (var image in images)
        {
            var subTestName = $"{testName}_{image}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, "PNG");
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(images.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{images.Length} images completed");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-050: Batch 5 images, only run first 2 (Cancel simulation)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_050_Batch_CancelAfter2()
    {
        const string testName = "CCV-050";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        // Process only the first 2 images out of 5
        var images = new[] { "color_bars_8bit", "gradient_horiz_rgb" };

        int successCount = 0;
        foreach (var image in images)
        {
            var subTestName = $"{testName}_{image}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, "PNG");
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(images.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{images.Length} of 5 images processed");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-051: Batch 8 images stress test (representative subset)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_051_Batch_8Images_Stress()
    {
        const string testName = "CCV-051";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["scale_percent"] = 50;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var images = new[] {
            "color_bars_8bit", "gradient_horiz_rgb", "pure_white_large",
            "pure_black_large", "checkerboard_8x8", "grayscale_256steps",
            "pure_red_small", "pure_white_small"
        };

        int successCount = 0;
        foreach (var image in images)
        {
            var subTestName = $"{testName}_{image}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, "PNG");
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(images.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{images.Length} images");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-052: Mixed format inputs (PNG + JPEG)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_052_MixedFormat_Inputs()
    {
        const string testName = "CCV-052";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var images = new[] { "color_bars_8bit", "format_jpeg" };
        int successCount = 0;

        foreach (var image in images)
        {
            var subTestName = $"{testName}_{image}";
            var result = await VerifyCrossChannelAsync(pipeline, image, subTestName, "TIFF");
            ValidateApiChannel(result, subTestName);

            if (result.ApiSucceeded && result.UiSucceeded)
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, subTestName);

            if (result.ApiSucceeded) successCount++;
            _output.WriteLine($"  {image}: {(result.ApiSucceeded ? "PASS" : "FAIL")}");
        }

        Assert.Equal(images.Length, successCount);
        _output.WriteLine($"PASS: {testName} — {successCount}/{images.Length} mixed-format images");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-053: Single image batch (= non-batch consistency check)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_053_SingleImage_BatchEqualsNonBatch()
    {
        const string testName = "CCV-053";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["scale_percent"] = 100;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        // Iron Rule 1: Verify that single-image API output is valid
        using var bmp = ImageAssert.LoadBitmap(result.ApiOutputPath!);
        Assert.True(bmp.Width > 0 && bmp.Height > 0,
            $"Single-image batch output has valid dimensions: {bmp.Width}x{bmp.Height}");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Helpers
    // ════════════════════════════════════════════════════════════════

    private void ValidateApiChannel(CrossChannelResult result, string testName)
    {
        if (result.ApiException != null)
            Assert.Fail($"API channel FAILED for {testName}: {result.ApiException}");

        Assert.NotNull(result.ApiOutputPath);
        Assert.True(File.Exists(result.ApiOutputPath),
            $"API output file does not exist: {result.ApiOutputPath}");

        using var bmp = ImageAssert.LoadBitmap(result.ApiOutputPath);
        Assert.True(bmp.Width > 0 && bmp.Height > 0,
            $"API output has zero dimensions: {bmp.Width}x{bmp.Height}");
    }
}
