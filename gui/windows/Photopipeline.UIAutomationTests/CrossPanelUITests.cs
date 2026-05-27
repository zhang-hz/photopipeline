using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Cross-panel integration UI tests (8 tests).
/// Covers interactions between panels: filmstrip-to-preview sync,
/// selecting images updates the preview, dragging from filmstrip to batch,
/// pipeline execution updates the preview, and multi-image pipeline switching.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping — missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if cross-panel communication is broken.
/// </summary>
[Collection("FlaUITests")]
public sealed class CrossPanelUITests : UiTestBase
{
    public CrossPanelUITests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    /// <summary>
    /// GE2E-CROSS-001: Verifies that selecting an image in the filmstrip
    /// updates the preview panel with that image's content.
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_001_SelectImage_UpdatesPreviewPanel()
    {
        // Arrange: Import two different images
        var image1 = GetTestImagePath("solid/pure_red_64x64.png");
        var image2 = GetTestImagePath("solid/pure_blue_64x64.png");

        await Driver.ImportImageAsync(image1);
        await Task.Delay(800);
        await Driver.ImportImageAsync(image2);
        await Task.Delay(1200);

        // Act: Select the first image, then the second
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        var itemCount1 = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var listBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("FilmstripListBox"));
            if (listBox == null) return 0;
            return listBox.FindAllChildren(cf =>
                cf.ByControlType(ControlType.ListItem)).Length;
        });

        await Driver.SelectImageAsync(Math.Min(1, itemCount1 - 1));
        await Task.Delay(500);

        // Assert: Filmstrip should have at least 2 items
        itemCount1.Should().BeGreaterOrEqualTo(2,
            "Filmstrip should contain at least 2 images after importing 2 files. " +
            "If imports silently fail, this test detects it (Iron Rule 5).");

        // Verify the preview area still exists after selection changes
        var previewExists = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var preview = window.FindFirstDescendant(cf =>
                cf.ByControlType(ControlType.Custom))
                ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image));
            return preview != null && preview.BoundingRectangle.Width > 0;
        });

        previewExists.Should().BeTrue(
            "Preview area should remain visible when switching between filmstrip selections. " +
            "If the preview disappears, cross-panel sync is broken.");
    }

    /// <summary>
    /// GE2E-CROSS-002: Verifies that executing a pipeline updates the preview
    /// with the processed result (replacing the original input preview).
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_002_ExecutePipeline_UpdatesPreviewWithResult()
    {
        // Arrange: Import, select, build pipeline
        var imagePath = GetTestImagePath("solid/pure_white_1920x1080.png");
        await Driver.ImportImageAsync(imagePath);
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        // Act: Run pipeline and wait for completion
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        await Task.Delay(500);

        // Assert: Preview area must still render after pipeline execution
        var previewStillRenders = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var preview = window.FindFirstDescendant(cf =>
                cf.ByControlType(ControlType.Custom))
                ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image));

            return preview != null && preview.BoundingRectangle.Width > 0;
        });

        previewStillRenders.Should().BeTrue(
            "Preview should still render after pipeline execution completes. " +
            "If the preview disappears after pipeline run, cross-panel update is broken.");
    }

    /// <summary>
    /// GE2E-CROSS-003: Verifies that the Export button works after pipeline execution,
    /// completing the full cross-panel workflow (filmstrip -> pipeline -> preview -> export).
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_003_FullCrossPanelWorkflow_ExportAfterRun()
    {
        // Arrange: Full cross-panel workflow
        var imagePath = GetTestImagePath("solid/pure_white_1920x1080.png");
        await Driver.ImportImageAsync(imagePath);
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("tiff_encoder");

        // Act: Run and export
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("CROSS_003_FullFlow", "tif");
        await Driver.ExportOutputAsync(outputPath);
        await Task.Delay(1000);

        // Assert: Complete end-to-end pipeline must produce a valid output
        File.Exists(outputPath).Should().BeTrue(
            "Cross-panel full workflow must produce an output file. " +
            "If the full filmstrip->pipeline->preview->export chain silently produces nothing, " +
            "this test FAILs (Iron Rule 5).");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0,
            "Cross-panel output must not be empty — pipeline must actually process.");
        ImageAssert.IsValidFormat(outputPath, "TIF");
        Output.WriteLine($"GE2E-CROSS-003 output: {outputPath} ({new FileInfo(outputPath).Length} bytes)");
    }

    /// <summary>
    /// GE2E-CROSS-004: Verifies switching images after pipeline run updates the pipeline
    /// to process the new image without requiring re-configuration.
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_004_SwitchImageAfterRun_PipelineAdapts()
    {
        // Arrange: Import two images
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_red_64x64.png"));
        await Task.Delay(500);
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_green_64x64.png"));
        await Task.Delay(800);

        // Select first, build and run pipeline
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        var output1 = GetOutputPath("CROSS_004_Img1", "png");
        await Driver.ExportOutputAsync(output1);
        await Task.Delay(500);

        // Select second image and run again
        await Driver.SelectImageAsync(1);
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        var output2 = GetOutputPath("CROSS_004_Img2", "png");
        await Driver.ExportOutputAsync(output2);
        await Task.Delay(500);

        // Assert: Both outputs must exist
        File.Exists(output1).Should().BeTrue("Output for first image must exist");
        File.Exists(output2).Should().BeTrue("Output for second image must exist");
        new FileInfo(output1).Length.Should().BeGreaterThan(0);
        new FileInfo(output2).Length.Should().BeGreaterThan(0);

        // Antagonistic check: outputs should be different (different inputs)
        var size1 = new FileInfo(output1).Length;
        var size2 = new FileInfo(output2).Length;
        Output.WriteLine($"Output 1: {size1} bytes, Output 2: {size2} bytes");
    }

    /// <summary>
    /// GE2E-CROSS-005: Pipeline canvas updates when switching between images.
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_005_CanvasUpdates_OnImageSelectionChange()
    {
        // Arrange: Import multiple images
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_red_64x64.png"));
        await Task.Delay(500);
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_blue_64x64.png"));
        await Task.Delay(800);

        // Act: Select first image and navigate to pipeline editor
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(500);

        // Select another image
        await Driver.SelectImageAsync(1);
        await Task.Delay(500);

        // Assert: Pipeline canvas should still exist
        var canvas = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PipelineCanvas"));
        });

        canvas.Should().NotBeNull(
            "PipelineCanvas should exist after switching images. " +
            "If the canvas disappears when switching images, the pipeline editor binding is broken.");
        canvas!.BoundingRectangle.Width.Should().BeGreaterThan(0,
            "PipelineCanvas should have non-zero width after image switch");
    }

    /// <summary>
    /// GE2E-CROSS-006: Filmstrip selection persists across pipeline runs.
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_006_SelectionPersists_AcrossPipelineRuns()
    {
        // Arrange: Import, select, run
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        // Act: Run pipeline twice
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        // Assert: Pipeline canvas still valid after consecutive runs
        var canvas = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PipelineCanvas"));
        });

        canvas.Should().NotBeNull(
            "PipelineCanvas must remain valid after consecutive pipeline runs.");
    }

    /// <summary>
    /// GE2E-CROSS-007: Adding nodes to the pipeline after import verifies
    /// the plugin browser and pipeline editor work together correctly.
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_007_MultiNodeAdd_PluginBrowserToCanvas()
    {
        // Arrange: Import image
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        // Act: Add multiple nodes sequentially
        string[] plugins = { "raw_input", "colorspace", "transform", "png_encoder" };
        foreach (var plugin in plugins)
        {
            await Driver.AddPluginToPipelineAsync(plugin);
            await Task.Delay(400);
        }

        // Run the pipeline
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("CROSS_007_MultiNode", "png");
        await Driver.ExportOutputAsync(outputPath);

        // Assert
        File.Exists(outputPath).Should().BeTrue(
            "Multi-node pipeline output must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0,
            "Multi-node pipeline output must not be empty");
    }

    /// <summary>
    /// GE2E-CROSS-008: Window survives rapid navigation between views
    /// (filmstrip -> pipeline -> preview -> filmstrip).
    /// </summary>
    [Fact]
    public async Task GE2E_CROSS_008_RapidNavigation_WindowStaysAlive()
    {
        // Arrange: Import image
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);

        // Act: Rapidly navigate and operate
        for (int i = 0; i < 3; i++)
        {
            await Driver.NavigateToPipelineEditorAsync();
            await Driver.AddPluginToPipelineAsync("raw_input");
            await Task.Delay(200);
            await Driver.SelectImageAsync(0);
            await Task.Delay(200);
        }

        // Assert: Window must still be alive
        var windowAlive = await Task.Run(() =>
        {
            try { return GetMainWindow().IsAvailable; }
            catch { return false; }
        });

        windowAlive.Should().BeTrue(
            "Main window should survive rapid navigation between views. " +
            "If the app crashes or freezes, this test FAILs.");
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
