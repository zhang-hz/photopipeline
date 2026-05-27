using Photopipeline.Models;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

/// <summary>
/// Layer 6 Cross-Channel Pipeline tests (CCV-021 ~ CCV-035).
/// Multi-node pipeline chains verified across API (gRPC) and UI (FlaUI) channels.
/// </summary>
[Trait("Category", "CrossChannel")]
public sealed class PipelineCrossTests : CrossChannelTestBase
{
    public PipelineCrossTests(ITestOutputHelper output) : base(output) { }

    // ════════════════════════════════════════════════════════════════
    //  CCV-021: raw_input -> colorspace -> tiff_encoder (3 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_021_Raw_Colorspace_Tiff_3Nodes()
    {
        const string testName = "CCV-021";
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
                p["target_color_space"] = "AdobeRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-022: raw_input -> ai_denoise -> colorspace -> png_encoder (4 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_022_Raw_Denoise_Colorspace_Png_4Nodes()
    {
        const string testName = "CCV-022";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
            })
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 30.0;
                p["detail_preservation"] = 60.0;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-023: raw_input -> lens_correct -> colorspace -> lut3d -> tiff_encoder (5 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_023_Raw_Lens_Colorspace_Lut_Tiff_5Nodes()
    {
        const string testName = "CCV-023";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
            })
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "AdobeRGB";
            })
            .AddNode("lut3d", "lut", configureParams: p =>
            {
                p["intensity"] = 80.0;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-024: transform -> colorspace -> jxl_encoder (3 nodes, lossless)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_024_Transform_Colorspace_Jxl_3Nodes()
    {
        const string testName = "CCV-024";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["scale_percent"] = 100;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("jxl_encoder", "jxl", configureParams: p =>
            {
                p["lossless"] = true;
                p["effort"] = 9;
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "JXL");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-025: raw_input -> transform -> colorspace -> avif_encoder (4 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_025_Raw_Transform_Colorspace_Avif_4Nodes()
    {
        const string testName = "CCV-025";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
            })
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["crop_enabled"] = true;
                p["crop_width"] = 50;
                p["crop_height"] = 50;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("avif_encoder", "avif", configureParams: p =>
            {
                p["quality"] = 90;
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "AVIF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-026: raw_input -> lens_correct -> ai_denoise -> colorspace -> tiff_encoder (5 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_026_Raw_Lens_Denoise_Colorspace_Tiff_5Nodes()
    {
        const string testName = "CCV-026";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
            })
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
            })
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 40.0;
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
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-027: exif_rw -> gps_set -> time_shift -> colorspace -> tiff_encoder (5 nodes, metadata chain)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_027_Exif_Gps_Time_Colorspace_Tiff_5Nodes()
    {
        const string testName = "CCV-027";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("exif_rw", "exif", configureParams: p =>
            {
                p["read_all"] = true;
                p["overwrite_original"] = true;
            })
            .AddNode("gps_set", "gps", configureParams: p =>
            {
                p["gps_mode"] = "manual";
                p["latitude"] = 39.9042;
                p["longitude"] = 116.4074;
            })
            .AddNode("time_shift", "ts", configureParams: p =>
            {
                p["shift_hours"] = 8.0;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "format_jpeg", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-028: ai_denoise -> colorspace -> lut3d -> tiff_encoder (4 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_028_Denoise_Colorspace_Lut_Tiff_4Nodes()
    {
        const string testName = "CCV-028";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 35.0;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "AdobeRGB";
            })
            .AddNode("lut3d", "lut", configureParams: p =>
            {
                p["intensity"] = 60.0;
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-029: colorspace -> lut3d -> transform -> png_encoder (4 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_029_Colorspace_Lut_Transform_Png_4Nodes()
    {
        const string testName = "CCV-029";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("lut3d", "lut", configureParams: p =>
            {
                p["intensity"] = 70.0;
            })
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

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-030: raw_input -> colorspace -> lut3d -> jxl_encoder (4 nodes, lossless)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_030_Raw_Colorspace_Lut_Jxl_4Nodes()
    {
        const string testName = "CCV-030";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "AdobeRGB";
            })
            .AddNode("lut3d", "lut", configureParams: p =>
            {
                p["intensity"] = 80.0;
            })
            .AddNode("jxl_encoder", "jxl", configureParams: p =>
            {
                p["lossless"] = true;
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "JXL");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-031: transform (crop+resize) -> colorspace -> png_encoder (3 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_031_Transform_CropResize_Colorspace_Png_3Nodes()
    {
        const string testName = "CCV-031";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["crop_enabled"] = true;
                p["crop_x"] = 0;
                p["crop_y"] = 0;
                p["crop_width"] = 50;
                p["crop_height"] = 50;
                p["scale_percent"] = 100;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "pure_white_large", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-032: lens_correct -> ai_denoise -> colorspace -> heif_encoder (4 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_032_Lens_Denoise_Colorspace_Heif_4Nodes()
    {
        const string testName = "CCV-032";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("lens_correct", "lens", configureParams: p =>
            {
                p["correction_mode"] = "auto";
            })
            .AddNode("ai_denoise", "dn", configureParams: p =>
            {
                p["denoise_strength"] = 20.0;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("heif_encoder", "heif", configureParams: p =>
            {
                p["quality"] = 85;
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "HEIF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-033: raw_input -> transform (flip) -> colorspace -> tiff_encoder (4 nodes)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_033_Raw_TransformFlip_Colorspace_Tiff_4Nodes()
    {
        const string testName = "CCV-033";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
            })
            .AddNode("transform", "xf", configureParams: p =>
            {
                p["flip_h"] = true;
                p["flip_v"] = true;
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "pure_white_large", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-034: A (enabled) -> B (disabled) -> C (enabled), B bypassed
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_034_DisabledMiddleNode_Bypassed()
    {
        const string testName = "CCV-034";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs1", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("ai_denoise", "dn", enabled: false, configureParams: p =>
            {
                p["denoise_strength"] = 90.0;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-035: All nodes disabled except encoder (passthrough)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_035_AllDisabled_Passthrough()
    {
        const string testName = "CCV-035";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", enabled: false, configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "Gray";
            })
            .AddNode("ai_denoise", "dn", enabled: false, configureParams: p =>
            {
                p["denoise_strength"] = 90.0;
            })
            .AddNode("png_encoder", "png")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

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
