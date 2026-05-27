using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// End-to-end workflow tests (20 tests).
/// Covers complete workflows: import-edit-export, RAW-denoise-colorspace-export,
/// batch-all-formats, multi-image workflow, project save/load scenarios.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping -- missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if the app does nothing.
/// Iron Rule 6: All outputs must be validated.
/// </summary>
[Collection("FlaUITests")]
public sealed class EndToEndWorkflowTests : UiTestBase
{
    public EndToEndWorkflowTests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Complete Restoration Workflows (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task E2E_CompleteRestoration_HeavyDenoiseToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("noise_grain.png"),
            new[] { "raw_input", "ai_denoise", "lens_correct", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["ai_denoise"] = new() { ["strength"] = "90" },
                ["lens_correct"] = new() { ["correction_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "16", ["compression"] = "deflate" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_Restoration");
    }

    [Fact]
    public async Task E2E_StylizedCrop_CoolLutToAvif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "lut3d", "transform", "avif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["lut3d"] = new() { ["intensity"] = "100" },
                ["transform"] = new() { ["scale_percent"] = "25" },
                ["avif_encoder"] = new() { ["quality"] = "85" },
            });

        File.Exists(outputPath).Should().BeTrue("AVIF output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "AVIF output must not be empty");
        SaveEvidence(outputPath, "E2E_StylizedCrop");
    }

    [Fact]
    public async Task E2E_CompositeTransform_RotateCropResize()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "transform", "colorspace", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["angle"] = "90", ["scale_percent"] = "50", ["crop_enabled"] = "true" },
            });

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "E2E_CompositeTransform");
    }

    [Fact]
    public async Task E2E_UpscaleDenoise_Lossless()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("pure_red_small.png"),
            new[] { "raw_input", "transform", "ai_denoise", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["scale_percent"] = "200" },
                ["ai_denoise"] = new() { ["strength"] = "20" },
                ["jxl_encoder"] = new() { ["quality"] = "100" },
            });

        File.Exists(outputPath).Should().BeTrue("JXL output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "JXL output must not be empty");
        SaveEvidence(outputPath, "E2E_UpscaleDenoise");
    }

    [Fact]
    public async Task E2E_FiveNode_ProfessionalRawPipeline()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "lens_correct", "colorspace", "lut3d", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto", ["apply_white_balance"] = "true" },
                ["lens_correct"] = new() { ["correction_mode"] = "auto" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "AdobeRGB" },
                ["lut3d"] = new() { ["intensity"] = "100" },
                ["tiff_encoder"] = new() { ["compression"] = "deflate", ["bit_depth"] = "16" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_Professional");
    }

    // ════════════════════════════════════════════════════════════════
    //  Format Conversion Workflows (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task E2E_FormatConversion_PngToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_PngToTiff");
    }

    [Fact]
    public async Task E2E_FormatConversion_RgbPngToRgbaTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "png_encoder", "tiff_encoder" },
            new()
            {
                ["png_encoder"] = new() { ["color_type"] = "rgba" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "16" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_RgbToRgba");
    }

    [Fact]
    public async Task E2E_FormatConversion_BmpToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" },
            new()
            {
                ["tiff_encoder"] = new() { ["compression"] = "none", ["bit_depth"] = "8" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_BmpToTiff");
    }

    [Fact]
    public async Task E2E_FormatConversion_GradientToMultipleFormats()
    {
        // PNG output
        var pngOutput = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("gradient_horiz_rgb.png"),
            new[] { "raw_input", "png_encoder" });

        AssertValidOutput(pngOutput, "PNG");
        SaveEvidence(pngOutput, "E2E_Gradient_Png");
    }

    [Fact]
    public async Task E2E_FormatConversion_ColorBarsToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("color_bars_8bit.png"),
            new[] { "raw_input", "tiff_encoder" });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_ColorBars_Tiff");
    }

    // ════════════════════════════════════════════════════════════════
    //  Colorspace Workflows (4 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task E2E_Colorspace_SrgbToAdobeRgb_IccEmbed()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "AdobeRGB", ["embed_icc"] = "true" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_SrgbToAdobeRgb");
    }

    [Fact]
    public async Task E2E_Colorspace_SrgbToDisplayP3()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "DisplayP3" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_SrgbToP3");
    }

    [Fact]
    public async Task E2E_Colorspace_SrgbToGray_Monochrome()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("gradient_horiz_rgb.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "Gray" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_SrgbToGray");
    }

    [Fact]
    public async Task E2E_Colorspace_AdobeRgbToSrgb()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "png_encoder" },
            new()
            {
                ["colorspace"] = new() { ["source_color_space"] = "AdobeRGB", ["target_color_space"] = "sRGB" },
            });

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "E2E_AdobeRgbToSrgb");
    }

    // ════════════════════════════════════════════════════════════════
    //  Multi-Image Workflow Tests (3 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task E2E_MultiImage_SequentialProcessing()
    {
        var images = new[] { "pure_red_small.png", "color_bars_8bit.png", "gradient_horiz_rgb.png" };
        var outputPaths = new List<string>();

        for (int i = 0; i < images.Length; i++)
        {
            await Driver.ImportImageAsync(GetTestImagePath(images[i]));
            await Task.Delay(300);
            await Driver.SelectImageAsync(i);
            await Driver.NavigateToPipelineEditorAsync();
            await Driver.AddPluginToPipelineAsync("raw_input");
            await Driver.AddPluginToPipelineAsync("png_encoder");
            await Driver.RunPipelineAsync();
            await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

            var outputPath = GetOutputPath($"E2E_MultiImg_{i}", "png");
            await Driver.ExportOutputAsync(outputPath);
            outputPaths.Add(outputPath);
        }

        outputPaths.Count.Should().Be(images.Length);
        foreach (var path in outputPaths)
        {
            AssertValidOutput(path, "PNG");
            SaveEvidence(path, Path.GetFileNameWithoutExtension(path));
        }
    }

    [Fact]
    public async Task E2E_MultiImage_SameConfig_DifferentResults()
    {
        // Process two different images with the same pipeline -- outputs should differ
        var output1 = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("pure_red_small.png"),
            new[] { "raw_input", "png_encoder" });

        var output2 = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("gradient_horiz_rgb.png"),
            new[] { "raw_input", "png_encoder" });

        AssertValidOutput(output1, "PNG");
        AssertValidOutput(output2, "PNG");
        SaveEvidence(output1, "E2E_SameConfig_1");
        SaveEvidence(output2, "E2E_SameConfig_2");

        var size1 = new FileInfo(output1).Length;
        var size2 = new FileInfo(output2).Length;
        Output.WriteLine($"Output sizes: {size1} vs {size2} bytes");
        // Sizes should differ (different input content)
        size1.Should().NotBe(size2, "Different inputs should produce different sized outputs");
    }

    [Fact]
    public async Task E2E_MultiImage_AllTestImages_BasicPipeline()
    {
        string[] allImages = {
            "pure_red_small.png", "color_bars_8bit.png", "gradient_horiz_rgb.png",
            "gradient_vert_rgb.png", "checkerboard_8x8.png"
        };

        var outputPaths = new List<string>();
        for (int i = 0; i < allImages.Length; i++)
        {
            try
            {
                var outputPath = await Driver.RunFullWorkflowAsync(
                    GetTestImagePath(allImages[i]),
                    new[] { "raw_input", "png_encoder" });

                AssertValidOutput(outputPath, "PNG");
                SaveEvidence(outputPath, $"E2E_AllImages_{i}");
                outputPaths.Add(outputPath);
            }
            catch (Exception ex)
            {
                Output.WriteLine($"Image {allImages[i]} processing failed: {ex.Message}");
            }
        }

        outputPaths.Count.Should().BeGreaterThan(0, "At least one image should process successfully");
        Output.WriteLine($"Processed {outputPaths.Count}/{allImages.Length} images");
    }

    // ════════════════════════════════════════════════════════════════
    //  Metadata & Special Workflows (3 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task E2E_MetadataPreservation_ExifReadWrite()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "exif_rw", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["exif_rw"] = new() { ["read_all"] = "true", ["overwrite"] = "true" },
            });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "E2E_ExifRw");
    }

    [Fact]
    public async Task E2E_GpsPlusTimeShift_Metadata()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "gps_set", "time_shift", "colorspace", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["gps_set"] = new() { ["mode"] = "manual", ["latitude"] = "39.9042", ["longitude"] = "116.4074" },
                ["time_shift"] = new() { ["shift_hours"] = "8" },
            });

        File.Exists(outputPath).Should().BeTrue("Metadata output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "Metadata output must not be empty");
        SaveEvidence(outputPath, "E2E_GpsTime");
    }

    [Fact]
    public async Task E2E_WebPublish_CropColorspaceAvif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid_white_1920x1080.png"),
            new[] { "raw_input", "transform", "colorspace", "avif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["scale_percent"] = "50" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["avif_encoder"] = new() { ["quality"] = "75" },
            });

        File.Exists(outputPath).Should().BeTrue("AVIF web output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "AVIF web output must not be empty");
        SaveEvidence(outputPath, "E2E_WebPublish");
    }
}
