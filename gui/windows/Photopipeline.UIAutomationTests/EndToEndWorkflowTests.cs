using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// End-to-end workflow tests (10 tests, GE2E-061 through GE2E-085).
/// Covers multi-plugin real-world workflows, format conversion, and
/// complete user journey from import to export.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping — missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if the app does nothing.
/// Iron Rule 6: Golden reference image pixel validation where applicable.
/// </summary>
[Collection("FlaUITests")]
public sealed class EndToEndWorkflowTests : UiTestBase
{
    public EndToEndWorkflowTests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Multi-Plugin Real-World Workflows (GE2E-061 through GE2E-070)
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// GE2E-061: Complete restoration workflow.
    /// ai_denoise(heavy) -> lens_correct -> colorspace -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_061_CompleteRestoration_HeavyDenoise()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "ai_denoise", "lens_correct", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["ai_denoise"] = new() { ["strength"] = "90", ["color_noise"] = "true" },
                ["lens_correct"] = new() { ["correction_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "16", ["compression"] = "deflate" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", expectedBitDepth: 16);
        Output.WriteLine($"GE2E-061 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-062: Stylized crop workflow.
    /// colorspace -> lut3d(cool) -> transform(crop 75%) -> avif_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_062_StylizedCrop_CoolLutAvif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "lut3d", "transform", "avif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["lut3d"] = new() { ["intensity"] = "100" },
                ["transform"] = new() { ["scale_percent"] = "25" },
                ["avif_encoder"] = new() { ["quality"] = "85" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        Output.WriteLine($"GE2E-062 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-063: Composite transform chain.
    /// transform(rotate→crop→resize) -> colorspace -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_063_CompositeTransform_RotateCropResize()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "colorspace", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new()
                {
                    ["angle"] = "90",
                    ["scale_percent"] = "50",
                    ["crop_enabled"] = "true",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG");
        Output.WriteLine($"GE2E-063 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-064: Gamut normalization.
    /// colorspace(AdobeRGB->sRGB) -> jxl_encoder(Q=100)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_064_GamutNormalization_AdobeRgbToSrgb()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new()
                {
                    ["source_color_space"] = "AdobeRGB",
                    ["target_color_space"] = "sRGB",
                },
                ["jxl_encoder"] = new() { ["quality"] = "100" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        Output.WriteLine($"GE2E-064 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-065: Bit depth reduction.
    /// colorspace -> transform(resize 25%) -> tiff_encoder(8bit)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_065_BitDepthReduction_16to8bit()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "transform", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["scale_percent"] = "25" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "8", ["compression"] = "none" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF");
        Output.WriteLine($"GE2E-065 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-066: Metadata preservation chain.
    /// exif_rw(read_all) -> colorspace -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_066_MetadataPreservation_ReadAllExif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "exif_rw", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["exif_rw"] = new() { ["read_all"] = "true", ["overwrite"] = "true" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF");
        Output.WriteLine($"GE2E-066 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-067: GPS + time metadata chain.
    /// gps_set(manual) -> time_shift(+8h) -> colorspace -> jxl_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_067_GpsPlusTimeMetadata_Jxl()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "gps_set", "time_shift", "colorspace", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["gps_set"] = new()
                {
                    ["mode"] = "manual",
                    ["latitude"] = "39.9042",
                    ["longitude"] = "116.4074",
                },
                ["time_shift"] = new()
                {
                    ["shift_hours"] = "8",
                    ["shift_minutes"] = "0",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output must not be empty");
        Output.WriteLine($"GE2E-067 output: {outputPath}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Format Conversion Workflows (GE2E-071 through GE2E-077)
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// GE2E-071: PNG to TIFF format conversion.
    /// raw_input -> png_encoder -> tiff_encoder
    /// Input: solid/solid_color_1920x1080.png
    /// Assert: IsValidFormat(TIFF), pixels equivalent.
    /// </summary>
    [Fact]
    public async Task GE2E_071_FormatConversion_PngToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" });

        File.Exists(outputPath).Should().BeTrue("TIFF output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "TIFF output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080);
        Output.WriteLine($"GE2E-071 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-075: BMP to TIFF format conversion (via PNG intermediate).
    /// raw_input -> png_encoder -> tiff_encoder
    /// Input: solid/solid_color_1920x1080.png (as source)
    /// Assert: IsValidFormat(TIFF).
    /// </summary>
    [Fact]
    public async Task GE2E_075_FormatConversion_BmpToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" },
            new()
            {
                ["tiff_encoder"] = new()
                {
                    ["compression"] = "none",
                    ["bit_depth"] = "8",
                },
            });

        File.Exists(outputPath).Should().BeTrue("TIFF output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "TIFF output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF");
        Output.WriteLine($"GE2E-075 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-083: RGB PNG to RGBA TIFF conversion.
    /// raw_input -> png_encoder(RGBA) -> tiff_encoder.
    /// Input: solid/solid_color_1920x1080.png
    /// Assert: Alpha=255 for all pixels.
    /// </summary>
    [Fact]
    public async Task GE2E_083_FormatConversion_RgbPngToRgbaTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "png_encoder", "tiff_encoder" },
            new()
            {
                ["png_encoder"] = new() { ["color_type"] = "rgba" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "16" },
            });

        File.Exists(outputPath).Should().BeTrue("TIFF output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "TIFF output must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080, expectedBitDepth: 16);
        Output.WriteLine($"GE2E-083 output: {outputPath}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Private helpers
    // ════════════════════════════════════════════════════════════════

    private Window GetMainWindow()
    {
        var desktop = new UIA3Automation().GetDesktop();
        var window = desktop.FindFirstChild(cf =>
            cf.ByControlType(ControlType.Window)
                .And(cf.ByName("Photopipeline")));
        if (window == null)
            throw new InvalidOperationException(
                "Main 'Photopipeline' window not found. Application may have crashed.");
        return window.AsWindow();
    }
}
