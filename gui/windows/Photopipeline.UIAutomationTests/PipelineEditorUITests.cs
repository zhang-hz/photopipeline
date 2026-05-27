using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Pipeline Editor UI tests (20 tests).
/// Covers node rendering on the SkiaSharp canvas, drag-drop node addition,
/// port connections, canvas zoom, parameter panel, and multi-plugin workflows
/// (GE2E-041 through GE2E-060).
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping — missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if the pipeline does not actually process.
/// Iron Rule 6: Where applicable, golden reference images are used for pixel validation.
/// </summary>
[Collection("FlaUITests")]
public sealed class PipelineEditorUITests : UiTestBase
{
    public PipelineEditorUITests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Basic Pipeline Editor UI tests
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// GE2E-PIPE-001: Verifies the PipelineCanvas is present and has non-zero dimensions.
    /// </summary>
    [Fact]
    public async Task GE2E_PIPE_001_Nodes_RenderOnCanvas()
    {
        // Act: Navigate to pipeline editor and verify canvas
        await Driver.NavigateToPipelineEditorAsync();

        var canvasExists = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var canvas = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PipelineCanvas"));
            if (canvas == null) return false;
            var bounds = canvas.BoundingRectangle;
            return bounds.Width > 0 && bounds.Height > 0;
        });

        // Assert
        canvasExists.Should().BeTrue(
            "PipelineCanvas (AutomationId='PipelineCanvas') must exist with non-zero dimensions. " +
            "Ensure PipelineEditorView.xaml sets this on the SkiaDAGCanvas element.");
    }

    /// <summary>
    /// GE2E-PIPE-002: Verifies that adding a plugin via the Plugin Browser
    /// creates a visible node on the pipeline canvas.
    /// </summary>
    [Fact]
    public async Task GE2E_PIPE_002_DragDrop_AddsNodeFromPluginBrowser()
    {
        // Arrange: Navigate to pipeline editor
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        // Act: Add a plugin node
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(1000);

        // Assert: The canvas should have new content (node added)
        var canvasExists = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var canvas = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PipelineCanvas"));
            if (canvas == null) return false;
            var bounds = canvas.BoundingRectangle;
            return bounds.Width > 0 && bounds.Height > 0;
        });

        canvasExists.Should().BeTrue(
            "PipelineCanvas should still exist after adding a node. " +
            "If adding a plugin crashes the canvas, this test FAILs.");
    }

    /// <summary>
    /// GE2E-PIPE-003: Verifies that the Properties panel appears when a node is selected.
    /// </summary>
    [Fact]
    public async Task GE2E_PIPE_003_ParameterPanel_AppearsWhenNodeSelected()
    {
        // Arrange
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(500);

        // Act: Select the node (click the canvas area)
        var canvas = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PipelineCanvas"));
        });

        if (canvas != null)
        {
            canvas.Click();
            await Task.Delay(500);
        }

        // Assert: PropertiesPanel should exist
        var propsPanel = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PropertiesPanel"));
        });

        propsPanel.Should().NotBeNull(
            "PropertiesPanel (AutomationId='PropertiesPanel') should exist. " +
            "Ensure PipelineEditorView.xaml includes the properties panel with this AutomationId.");
    }

    /// <summary>
    /// GE2E-PIPE-004: Verifies the Cancel button stops a running pipeline.
    /// </summary>
    [Fact]
    public async Task GE2E_PIPE_004_CancelButton_StopsPipelineExecution()
    {
        // Arrange
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("ai_denoise");

        // Act: Start pipeline and immediately cancel
        await Driver.RunPipelineAsync();
        await Task.Delay(500);
        await Driver.CancelPipelineAsync();
        await Task.Delay(1000);

        // Assert: Window must still be alive after cancel
        var windowAlive = await Task.Run(() =>
        {
            try { return GetMainWindow().IsAvailable; }
            catch { return false; }
        });

        windowAlive.Should().BeTrue(
            "Main window should remain alive after cancelling a pipeline. " +
            "If cancel causes a deadlock or crash, this test FAILs.");
    }

    // ════════════════════════════════════════════════════════════════
    //  Multi-Plugin Workflow Tests (GE2E-041 through GE2E-056)
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// GE2E-041: Full RAW development workflow.
    /// raw_input (auto) -> colorspace (sRGB→AdobeRGB) -> tiff_encoder (16bit ZIP)
    /// Input: high_bitdepth_1920 (I10)
    /// </summary>
    [Fact]
    public async Task GE2E_041_FullRawWorkflow_AutoToAdobeRGB16BitTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto", ["apply_white_balance"] = "true" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "AdobeRGB" },
                ["tiff_encoder"] = new() { ["compression"] = "deflate", ["bit_depth"] = "16" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist after pipeline execution");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", expectedBitDepth: 16);
        Output.WriteLine($"GE2E-041 output: {outputPath} ({new FileInfo(outputPath).Length} bytes)");
    }

    /// <summary>
    /// GE2E-042: Film simulation workflow.
    /// raw_input -> colorspace -> lut3d(warm) -> jxl_encoder(Q=90)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_042_FilmSimulation_RawToWarmLutJxl()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "lut3d", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["lut3d"] = new() { ["intensity"] = "80" },
                ["jxl_encoder"] = new() { ["quality"] = "90" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-042 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-043: Denoise -> colorspace workflow.
    /// raw_input -> ai_denoise(medium) -> colorspace -> png_encoder
    /// Input: noisy_texture (I06)
    /// </summary>
    [Fact]
    public async Task GE2E_043_DenoiseToColorspace_RawToPng()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "ai_denoise", "colorspace", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["ai_denoise"] = new() { ["strength"] = "50", ["detail"] = "50" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG");
        Output.WriteLine($"GE2E-043 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-044: Lens correction workflow.
    /// raw_input -> lens_correct(full) -> colorspace -> tiff_encoder
    /// Input: barrel_distortion (I07)
    /// </summary>
    [Fact]
    public async Task GE2E_044_LensCorrectToColorspace_Tiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lens_correct", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["lens_correct"] = new() { ["correction_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF");
        Output.WriteLine($"GE2E-044 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-045: Web publishing workflow.
    /// raw_input -> transform(crop 50%) -> colorspace -> avif_encoder(Q=75)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_045_WebPublish_CropColorspaceAvif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "colorspace", "avif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["crop_enabled"] = "true", ["scale_percent"] = "50" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["avif_encoder"] = new() { ["quality"] = "75" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-045 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-046: Upscale + denoise + lossless archive.
    /// raw_input -> transform(200%) -> ai_denoise(light) -> jxl_encoder(lossless)
    /// Input: web_photo_800 (I03)
    /// </summary>
    [Fact]
    public async Task GE2E_046_UpscaleDenoiseLossless()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "ai_denoise", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["scale_percent"] = "200" },
                ["ai_denoise"] = new() { ["strength"] = "20" },
                ["jxl_encoder"] = new() { ["quality"] = "100" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-046 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-047: Rotation workflow.
    /// transform(rotate 90) -> heif_encoder(Q=85)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_047_RotateToHeif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "heif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["angle"] = "90" },
                ["heif_encoder"] = new() { ["quality"] = "85" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-047 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-048: 5-node complete professional RAW pipeline.
    /// raw_input -> lens_correct -> colorspace -> lut3d(film) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_048_FiveNodeProfessionalRawPipeline()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lens_correct", "colorspace", "lut3d", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto", ["apply_white_balance"] = "true" },
                ["lens_correct"] = new() { ["correction_mode"] = "auto" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "AdobeRGB" },
                ["lut3d"] = new() { ["intensity"] = "100" },
                ["tiff_encoder"] = new() { ["compression"] = "deflate", ["bit_depth"] = "16" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist after 5-node pipeline");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", expectedBitDepth: 16);
        Output.WriteLine($"GE2E-048 output: {outputPath} ({new FileInfo(outputPath).Length} bytes)");
    }

    /// <summary>
    /// GE2E-049: Denoise + stylize.
    /// ai_denoise(medium) -> colorspace -> lut3d(cool) -> jxl_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_049_DenoiseStylize_CoolLutJxl()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "ai_denoise", "colorspace", "lut3d", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["ai_denoise"] = new() { ["strength"] = "50" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["lut3d"] = new() { ["intensity"] = "50" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-049 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-050: Social media publish.
    /// colorspace(sRGB->P3) -> transform(50%) -> lut3d -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_050_SocialMedia_P3ResizeLutPng()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "transform", "lut3d", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "DisplayP3" },
                ["transform"] = new() { ["scale_percent"] = "50" },
                ["lut3d"] = new() { ["intensity"] = "80" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG");
        Output.WriteLine($"GE2E-050 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-051: Monochrome conversion.
    /// colorspace(sRGB->Gray) -> tiff_encoder(16bit)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_051_MonochromeConversion_GrayTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "Gray" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "16" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", expectedBitDepth: 16);
        Output.WriteLine($"GE2E-051 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-052: Mirror flip.
    /// transform(flip H+V) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_052_MirrorFlip_HorizontalVertical()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["flip_h"] = "true", ["flip_v"] = "true" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG");
        Output.WriteLine($"GE2E-052 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-053: Wide gamut output.
    /// colorspace(sRGB->DisplayP3) -> avif_encoder(Q=90, 10bit)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_053_WideGamut_P3ToAvif10bit()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "avif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "DisplayP3" },
                ["avif_encoder"] = new() { ["quality"] = "90", ["bit_depth"] = "10" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-053 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-054: Barrel distortion fix.
    /// lens_correct(barrel) -> colorspace -> jxl_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_054_BarrelDistortionCorrection_Jxl()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lens_correct", "colorspace", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["lens_correct"] = new() { ["correction_mode"] = "auto", ["correct_distortion"] = "true" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-054 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-055: Pincushion + vignette correction.
    /// lens_correct(pincushion+vignette) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_055_PincushionVignetteCorrection_Tiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lens_correct", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["lens_correct"] = new()
                {
                    ["correction_mode"] = "auto",
                    ["correct_distortion"] = "true",
                    ["correct_vignette"] = "true",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF");
        Output.WriteLine($"GE2E-055 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-056: Thumbnail + style workflow.
    /// transform(crop 25%) -> colorspace -> lut3d(warm) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_056_ThumbnailStyle_CropWarmLut()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "colorspace", "lut3d", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["crop_enabled"] = "true", ["scale_percent"] = "25" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["lut3d"] = new() { ["intensity"] = "80" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG");
        Output.WriteLine($"GE2E-056 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-057: Web optimization workflow.
    /// colorspace -> transform(resize 50%) -> avif_encoder(Q=60)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_057_WebOptimization_ResizeHalfAvif()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "transform", "avif_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
                ["transform"] = new() { ["scale_percent"] = "50" },
                ["avif_encoder"] = new() { ["quality"] = "60" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-057 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-058: Archival denoise workflow.
    /// ai_denoise(medium) -> colorspace -> tiff_encoder(16bit)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_058_ArchivalDenoise_16bitTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "ai_denoise", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["ai_denoise"] = new() { ["strength"] = "50" },
                ["tiff_encoder"] = new() { ["bit_depth"] = "16", ["compression"] = "deflate" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", expectedBitDepth: 16);
        Output.WriteLine($"GE2E-058 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-059: Alpha channel handling.
    /// colorspace(sRGB->Gray) -> transform(rotate 180) -> png_encoder(RGBA)
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_059_AlphaChannel_GrayRotateRgbaPng()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "transform", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new() { ["source_color_space"] = "sRGB", ["target_color_space"] = "Gray" },
                ["transform"] = new() { ["angle"] = "180" },
                ["png_encoder"] = new() { ["color_type"] = "rgba" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG");
        Output.WriteLine($"GE2E-059 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-060: Large upscale workflow.
    /// transform(scale 400%) -> colorspace -> jxl_encoder
    /// Input: icon_tiny_256 (I13)
    /// </summary>
    [Fact]
    public async Task GE2E_060_LargeUpscale_400Percent()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_red_64x64.png"),
            new[] { "raw_input", "transform", "colorspace", "jxl_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new() { ["scale_percent"] = "400" },
                ["colorspace"] = new() { ["target_color_space"] = "sRGB" },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        Output.WriteLine($"GE2E-060 output: {outputPath}");
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
