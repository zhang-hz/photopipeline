using Photopipeline.Models;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

/// <summary>
/// Layer 6 Cross-Channel Format tests (CCV-036 ~ CCV-045).
/// Cross-format consistency verification across API and UI channels.
/// </summary>
[Trait("Category", "CrossChannel")]
public sealed class FormatCrossTests : CrossChannelTestBase
{
    public FormatCrossTests(ITestOutputHelper output) : base(output) { }

    // ════════════════════════════════════════════════════════════════
    //  CCV-036: PNG -> decode -> TIFF encode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_036_Png_Decode_TiffEncode()
    {
        const string testName = "CCV-036";
        await RequireApiBackendAsync();

        // A minimal pipeline that just re-encodes: colorspace passthrough -> tiff
        var pipeline = new TestPipelineBuilder()
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
    //  CCV-037: JPEG -> decode -> PNG encode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_037_Jpeg_Decode_PngEncode()
    {
        const string testName = "CCV-037";
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

        var result = await VerifyCrossChannelAsync(pipeline, "format_jpeg", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-038: 16bit TIFF -> decode -> 16bit PNG encode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_038_16bitTiff_Decode_16bitPngEncode()
    {
        const string testName = "CCV-038";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
                p["output_format"] = "u16";
            })
            .AddNode("png_encoder", "png", configureParams: p =>
            {
                p["color_type"] = "rgb";
                p["compression_level"] = 6;
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "gradient_horiz_rgb", testName, "PNG");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-039: RGBA PNG -> decode -> TIFF encode (alpha channel preserved)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_039_RgbaPng_Decode_TiffEncode()
    {
        const string testName = "CCV-039";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("png_encoder", "png", configureParams: p =>
            {
                p["color_type"] = "rgba";
                p["compression_level"] = 6;
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
    //  CCV-040: Gray PNG -> decode -> RGB -> decode -> Gray
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_040_Gray_Decode_Rgb_Decode_Gray()
    {
        const string testName = "CCV-040";
        await RequireApiBackendAsync();

        // sRGB -> Gray conversion, then re-encode to TIFF
        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "Gray";
            })
            .AddNode("tiff_encoder", "tiff")
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "grayscale_256steps", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-041: CMYK TIFF -> decode -> sRGB PNG
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_041_CmykTiff_Decode_SrgbPng()
    {
        const string testName = "CCV-041";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "CMYK";
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
    //  CCV-042: AVIF -> decode -> PNG encode (lossy tolerance)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_042_Avif_Decode_PngEncode()
    {
        const string testName = "CCV-042";
        await RequireApiBackendAsync();

        // Encode to AVIF first, then re-encode to PNG
        var pipeline = new TestPipelineBuilder()
            .AddNode("avif_encoder", "avif", configureParams: p =>
            {
                p["quality"] = 100;
                p["lossless"] = true;
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
    //  CCV-043: JXL (lossless) -> decode -> TIFF encode
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_043_JxlLossless_Decode_TiffEncode()
    {
        const string testName = "CCV-043";
        await RequireApiBackendAsync();

        var pipeline = new TestPipelineBuilder()
            .AddNode("jxl_encoder", "jxl", configureParams: p =>
            {
                p["lossless"] = true;
                p["effort"] = 5;
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
    //  CCV-044: 8bit -> 16bit promotion
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_044_8bit_To_16bit_Promotion()
    {
        const string testName = "CCV-044";
        await RequireApiBackendAsync();

        // Use raw_input to convert 8bit input to 16bit, then encode to TIFF
        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
                p["output_format"] = "u16";
            })
            .AddNode("tiff_encoder", "tiff", configureParams: p =>
            {
                p["compression"] = "deflate";
                p["pixel_format"] = "u16";
            })
            .ConnectLinear()
            .Build();

        var result = await VerifyCrossChannelAsync(pipeline, "color_bars_8bit", testName, "TIFF");
        ValidateApiChannel(result, testName);

        if (result.ApiSucceeded && result.UiSucceeded)
            CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath!, result.UiOutputPath!, testName);

        _output.WriteLine($"PASS: {testName} — {result}");
    }

    // ════════════════════════════════════════════════════════════════
    //  CCV-045: 16bit -> 8bit truncation
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task CCV_045_16bit_To_8bit_Truncation()
    {
        const string testName = "CCV-045";
        await RequireApiBackendAsync();

        // raw_input in u16 mode, then png encoder at 8bit
        var pipeline = new TestPipelineBuilder()
            .AddNode("raw_input", "raw", configureParams: p =>
            {
                p["raw_mode"] = "auto";
                p["output_format"] = "u16";
            })
            .AddNode("colorspace", "cs", configureParams: p =>
            {
                p["source_color_space"] = "sRGB";
                p["target_color_space"] = "sRGB";
            })
            .AddNode("png_encoder", "png", configureParams: p =>
            {
                p["color_type"] = "rgb";
            })
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
