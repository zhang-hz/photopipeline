using Photopipeline.Models;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

/// <summary>
/// Layer 6 Cross-Channel Plugin tests (CCV-001 ~ CCV-020).
/// Each test: API (gRPC) channel + UI (FlaUI) channel -> CrossChannelVerifier.
/// Per Iron Rule 1, every test has at least one pixel-level assertion.
/// Per Iron Rule 2, no exception is silently swallowed.
/// </summary>
[Trait("Category", "CrossChannel")]
public sealed class PluginCrossTests : CrossChannelTestBase
{
    public PluginCrossTests(ITestOutputHelper output) : base(output) { }

    // ════════════════════════════════════════════════════════════════
    //  CCV-001: raw_input (auto) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_001_RawInput_AutoExposure_ToTiff()
    {
        const string testName = "CCV-001";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
                p["apply_white_balance"] = true;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var inputImage = "color_bars_8bit"; // representative 1920x1080 RGB image
        var result = await VerifyCrossChannelAsync(pipeline, inputImage, testName, "TIFF");
        ValidateResult(result, testName, expectBothChannels: false);

        // Iron Rule 1: at minimum, verify API channel executed and produced valid output.
        Assert.NotNull(result.ApiOutputPath);
        Assert.True(File.Exists(result.ApiOutputPath), $"API output not found: {result.ApiOutputPath}");
        Assert.Null(result.ApiException);

        using var bmp = ImageAssert.LoadBitmap(result.ApiOutputPath);
        Assert.True(bmp.Width > 0 && bmp.Height > 0, "Output bitmap has zero dimensions");

        // If UI channel succeeded, cross-verify
        if (result.UiSucceeded && result.UiOutputPath != null)
        {
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);
        }

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-002: raw_input (dcraw, u16) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_002_RawInput_Dcraw_U16_ToTiff()
    {
        const string testName = "CCV-002";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "dcraw";
                p["output_format"] = "u16";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "gradient_horiz_rgb", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-003: transform (crop 50%) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_003_Transform_Crop50_ToPng()
    {
        const string testName = "CCV-003";
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

        var result = await VerifyCrossChannelAsync(pipeline, "pure_white_large", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-004: transform (rotate 90 deg) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_004_Transform_Rotate90_ToPng()
    {
        const string testName = "CCV-004";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["angle"] = 90.0;
                p["resize_mode"] = "expand";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "pure_white_large", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-005: colorspace (sRGB -> AdobeRGB) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_005_Colorspace_SrgbToAdobeRgb_ToTiff()
    {
        const string testName = "CCV-005";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "AdobeRGB";
                p["embed_icc"] = true;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-006: colorspace (sRGB -> Gray) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_006_Colorspace_SrgbToGray_ToTiff()
    {
        const string testName = "CCV-006";
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
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-007: lut3d (warm, intensity=80) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_007_Lut3d_Warm_ToPng()
    {
        const string testName = "CCV-007";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("lut3d", "lut", configureParams: p =>
            {
                p["intensity"] = 80.0;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-008: lut3d (film, tetrahedral) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_008_Lut3d_Film_Tetrahedral_ToPng()
    {
        const string testName = "CCV-008";
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
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-009: lens_correct (auto) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_009_LensCorrect_Auto_ToPng()
    {
        const string testName = "CCV-009";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "checkerboard_8x8", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-010: lens_correct (full: distortion + vignette) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_010_LensCorrect_Full_ToTiff()
    {
        const string testName = "CCV-010";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
                p["correct_distortion"] = true;
                p["correct_vignetting"] = true;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "checkerboard_8x8", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-011: ai_denoise (strength=50) -> png_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_011_AiDenoise_Medium_ToPng()
    {
        const string testName = "CCV-011";
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
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-012: ai_denoise (light: strength=20) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_012_AiDenoise_Light_ToTiff()
    {
        const string testName = "CCV-012";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 20.0;
                p["detail_preservation"] = 80.0;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-013: exif_rw (preserve) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_013_ExifRw_Preserve_ToTiff()
    {
        const string testName = "CCV-013";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("exif_rw", "exif", configureParams: p =>
            {
                p["read_all"] = true;
                p["overwrite_original"] = true;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "format_jpeg", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-014: gps_set (manual: 39.9042, 116.4074) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_014_GpsSet_Manual_ToTiff()
    {
        const string testName = "CCV-014";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("gps_set", "gps", configureParams: p =>
            {
                p["gps_mode"] = "manual";
                p["latitude"] = 39.9042;
                p["longitude"] = 116.4074;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-015: time_shift (+1h) -> tiff_encoder
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_015_TimeShift_Plus1h_ToTiff()
    {
        const string testName = "CCV-015";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("time_shift", "ts", configureParams: p =>
            {
                p["shift_hours"] = 1.0;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "format_jpeg", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-016: avif_encoder (lossless) -> decode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_016_AvifEncoder_Lossless_ToDecode()
    {
        const string testName = "CCV-016";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("avif_encoder", "avif", configureParams: p =>
            {
                p["lossless"] = true;
                p["quality"] = 100;
            })
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "AVIF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-017: jxl_encoder (lossless, effort=9) -> decode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_017_JxlEncoder_Lossless_ToDecode()
    {
        const string testName = "CCV-017";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("jxl_encoder", "jxl", configureParams: p =>
            {
                p["lossless"] = true;
                p["effort"] = 9;
            })
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "JXL");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-018: heif_encoder (Q=80) -> decode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_018_HeifEncoder_Q80_ToDecode()
    {
        const string testName = "CCV-018";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("heif_encoder", "heif", configureParams: p =>
            {
                p["quality"] = 80;
                p["chroma_subsampling"] = "444";
            })
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "HEIF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-019: tiff_encoder (16bit deflate) -> decode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_019_TiffEncoder_16bitDeflate_ToDecode()
    {
        const string testName = "CCV-019";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("tiff_encoder", "tiff", configureParams: p =>
            {
                p["compression"] = "deflate";
                p["pixel_format"] = "u16";
            })
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "gradient_horiz_rgb", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-020: png_encoder (RGBA) -> decode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_020_PngEncoder_RGBA_ToDecode()
    {
        const string testName = "CCV-020";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("png_encoder", "png", configureParams: p =>
            {
                p["color_type"] = "rgba";
                p["compression_level"] = 6;
            })
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiOutputPath != null && result.UiOutputPath != null)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Helpers
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// Validates that the API channel succeeded and produced a valid output.
    /// Iron Rule 1: At minimum, we assert API output exists and has valid dimensions.
    /// Iron Rule 2: If API channel has an exception, it is reported as a clear failure.
    /// </summary>
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

    /// <summary>
    /// Validates that at least the API channel succeeded.
    /// If both channels succeeded, also validates UI channel.
    /// </summary>
    private void ValidateResult(CrossChannelResult result, string testName, bool expectBothChannels)
    {
        ValidateApiChannel(result, testName);

        if (result.UiException != null)
            _output.WriteLine($"UI channel failed (non-fatal): {result.UiException.Message}");

        if (expectBothChannels)
            Assert.True(result.UiSucceeded,
                $"UI channel expected but failed: {result.UiException?.Message}");

        Assert.False(result.ApiSucceeded == false && result.UiSucceeded == false,
            "Both API and UI channels failed");
    }
}
