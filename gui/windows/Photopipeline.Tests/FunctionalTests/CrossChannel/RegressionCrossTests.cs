using Photopipeline.Models;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

/// <summary>
/// Layer 6 Cross-Channel Regression tests (CCV-054 ~ CCV-060).
/// Golden reference image comparison across API and UI channels.
/// Uses PHOTOPIPELINE_GENERATE_GOLDEN environment variable for golden image generation.
/// </summary>
[Trait("Category", "CrossChannel")]
public sealed class RegressionCrossTests : CrossChannelTestBase
{
    private static readonly bool GenerateGolden =
        Environment.GetEnvironmentVariable("PHOTOPIPELINE_GENERATE_GOLDEN") == "true";

    private static readonly string GoldenDir = Path.Combine(
        AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..",
        "FunctionalTests", "TestData", "golden", "cross_channel");

    public RegressionCrossTests(ITestOutputHelper output) : base(output) { }

    // ════════════════════════════════════════════════════════════════
    //  CCV-054: raw_input -> colorspace -> tiff (I01, golden baseline)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_054_Raw_Colorspace_Tiff_Golden()
    {
        const string testName = "CCV-054";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
                p["apply_white_balance"] = true;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateGolden(result, testName, "ccv054_raw_colorspace_tiff.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-055: transform (crop 50%) -> png (golden)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_055_Transform_Crop50_Png_Golden()
    {
        const string testName = "CCV-055";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["crop_enabled"] = true;
                p["crop_x"] = 25;
                p["crop_y"] = 25;
                p["crop_width"] = 50;
                p["crop_height"] = 50;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "solid_white_large", testName, "PNG");
        ValidateGolden(result, testName, "ccv055_crop50_png.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-056: colorspace (sRGB -> Gray) -> tiff (golden)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_056_Colorspace_SrgbToGray_Tiff_Golden()
    {
        const string testName = "CCV-056";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "Gray";
                p["black_point_compensation"] = true;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateGolden(result, testName, "ccv056_gray_tiff.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-057: lut3d (film) -> png (golden)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_057_Lut3d_Film_Png_Golden()
    {
        const string testName = "CCV-057";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("lut3d", "lut", configureParams: p =>
            {
                p["interpolation"] = "tetrahedral";
                p["intensity"] = 100.0;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateGolden(result, testName, "ccv057_film_lut_png.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-058: ai_denoise (medium) -> png (golden)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_058_AiDenoise_Medium_Png_Golden()
    {
        const string testName = "CCV-058";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 50.0;
                p["detail_preservation"] = 50.0;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateGolden(result, testName, "ccv058_denoise_med_png.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-059: lens_correct -> colorspace -> tiff (golden)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_059_LensCorrect_Colorspace_Tiff_Golden()
    {
        const string testName = "CCV-059";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
                p["correct_distortion"] = true;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "checkerboard_8x8", testName, "TIFF");
        ValidateGolden(result, testName, "ccv059_lens_color_tiff.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-060: 5-node full RAW workflow (golden)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_060_Raw_Lens_Denoise_Colorspace_Tiff_Golden()
    {
        const string testName = "CCV-060";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
                p["apply_white_balance"] = true;
            })
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
                p["correct_distortion"] = true;
                p["correct_vignetting"] = true;
            })
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 30.0;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "AdobeRGB";
                p["embed_icc"] = true;
            })
            .AddNode("tiff_encoder", "tiff", configureParams: p =>
            {
                p["compression"] = "deflate";
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateGolden(result, testName, "ccv060_full_raw_workflow_tiff.png");

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Golden image helpers
    // ════════════════════════════════════════════════════════════════

    private void ValidateGolden(CrossChannelResult result, string testName, string goldenFileName)
    {
        // API channel must succeed (Iron Rule 1)
        if (result.ApiException != null)
            Assert.Fail($"API channel FAILED for {testName}: {result.ApiException}");

        Assert.NotNull(result.ApiOutputPath);
        Assert.True(File.Exists(result.ApiOutputPath),
            $"API output file does not exist: {result.ApiOutputPath}");

        var goldenPath = Path.Combine(GoldenDir, goldenFileName);

        if (GenerateGolden)
        {
            // Generate golden image from API output
            Directory.CreateDirectory(GoldenDir);
            File.Copy(result.ApiOutputPath, goldenPath, overwrite: true);
            _output.WriteLine($"Golden image generated: {goldenPath}");
            return;
        }

        // Iron Rule 6: Must have golden reference image for regression tests
        Assert.True(File.Exists(goldenPath),
            $"Golden reference image not found: {goldenPath}. " +
            "Set PHOTOPIPELINE_GENERATE_GOLDEN=true to generate.");

        // Pixel-perfect comparison with golden
        ImageAssert.PixelsEqual(result.ApiOutputPath, goldenPath, tolerancePerChannel: 0);

        // If both channels succeeded, also verify UI output against golden
        if (result.UiSucceeded && result.UiOutputPath != null)
        {
            CrossChannelVerifier.VerifyEquivalence(result.UiOutputPath, goldenPath,
                $"{testName}_ui_vs_golden");
        }
    }
}
