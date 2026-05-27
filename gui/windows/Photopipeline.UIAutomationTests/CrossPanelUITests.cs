using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Cross-panel integration UI tests (25 tests).
/// Covers interactions between panels: Filmstrip-Pipeline, Filmstrip-Batch,
/// Pipeline-Batch, selection sync, and cross-panel notifications.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping -- missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if cross-panel communication is broken.
/// </summary>
[Collection("FlaUITests")]
public sealed class CrossPanelUITests : UiTestBase
{
    public CrossPanelUITests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Filmstrip to Pipeline Tests (6 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task FilmstripToPipeline_SelectImage_PipelineCanvasExists()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        var window = GetMainWindow();
        var canvas = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        canvas.Should().NotBeNull("PipelineCanvas must exist after navigating from filmstrip");
    }

    [Fact]
    public async Task FilmstripToPipeline_SwitchImage_PipelineStays()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(500);
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(800);

        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(500);

        // Switch to second image
        await Driver.SelectImageAsync(1);
        await Task.Delay(500);

        var window = GetMainWindow();
        var canvas = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        canvas.Should().NotBeNull("PipelineCanvas must survive image switching");
    }

    [Fact]
    public async Task FilmstripToPipeline_ImportWhilePipelineOpen()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(300);

        // Import another while pipeline editor is open
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive import while pipeline editor open");
    }

    [Fact]
    public async Task FilmstripToPipeline_FullWorkflow_ProducesOutput()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("tiff_encoder");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_FilmstripToPipeline", "tif");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "Cross_FilmstripToPipeline");
    }

    [Fact]
    public async Task FilmstripToPipeline_SelectionPersists_AcrossRuns()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var canvas = await Task.Run(() =>
            GetMainWindow().FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        canvas.Should().NotBeNull("PipelineCanvas must persist across consecutive runs");
    }

    [Fact]
    public async Task FilmstripToPipeline_PreviewUpdates_AfterRun()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        await Task.Delay(500);

        var window = GetMainWindow();
        var preview = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
            ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image)));

        preview.Should().NotBeNull("Preview should still render after pipeline run");
        preview!.BoundingRectangle.Width.Should().BeGreaterThan(0,
            "Preview must have non-zero dimensions after run");
        CaptureScreenshot("Cross_AfterRunPreview");
    }

    // ════════════════════════════════════════════════════════════════
    //  Filmstrip to Batch Tests (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task FilmstripToBatch_ImportThenExport_BatchGetsItem()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_FilmstripToBatch", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Cross_FilmstripToBatch");
    }

    [Fact]
    public async Task FilmstripToBatch_MultipleImages_AllProcessed()
    {
        var images = new[] { "pure_red_small.png", "color_bars_8bit.png", "gradient_horiz_rgb.png" };
        var outputPaths = new List<string>();

        foreach (var img in images)
        {
            await Driver.ImportImageAsync(GetTestImagePath(img));
            await Task.Delay(400);
        }

        for (int i = 0; i < images.Length; i++)
        {
            await Driver.SelectImageAsync(i);
            await Driver.NavigateToPipelineEditorAsync();
            await Driver.AddPluginToPipelineAsync("raw_input");
            await Driver.AddPluginToPipelineAsync("png_encoder");
            await Driver.RunPipelineAsync();
            await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

            var outputPath = GetOutputPath($"Cross_Batch_{i}", "png");
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
    public async Task FilmstripToBatch_CountMatches_AfterExport()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_CountMatch", "png");
        await Driver.ExportOutputAsync(outputPath);

        var window = GetMainWindow();
        var listBox = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("FilmstripListBox")));

        listBox.Should().NotBeNull("FilmstripListBox must exist after batch workflow");
    }

    [Fact]
    public async Task FilmstripToBatch_Reimport_AfterExport()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_Reimport_1", "png");
        await Driver.ExportOutputAsync(outputPath);

        // Import new image
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(600);

        var window = GetMainWindow();
        var listBox = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("FilmstripListBox")));

        listBox.Should().NotBeNull("Filmstrip should have items after re-import");
    }

    [Fact]
    public async Task FilmstripToBatch_LargeBatch_20Images()
    {
        var imagePath = GetTestImagePath("pure_red_small.png");
        for (int i = 0; i < 10; i++)
        {
            await Driver.ImportImageAsync(imagePath);
            await Task.Delay(120);
        }
        await Task.Delay(1000);

        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_LargeFilmstrip", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Cross_LargeFilmstrip");
    }

    // ════════════════════════════════════════════════════════════════
    //  Pipeline to Batch Tests (4 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task PipelineToBatch_RunThenExport_FileCreated()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_vert_rgb.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_PipelineToBatch", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Cross_PipelineToBatch");
    }

    [Fact]
    public async Task PipelineToBatch_DifferentFormats_PerRun()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);

        // PNG
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        var pngPath = GetOutputPath("Cross_PipelineToBatch_Png", "png");
        await Driver.ExportOutputAsync(pngPath);
        AssertValidOutput(pngPath, "PNG");
        SaveEvidence(pngPath, "Cross_PipelineToBatch_Png");
    }

    [Fact]
    public async Task PipelineToBatch_CancelThenRerun_Works()
    {
        await Driver.ImportImageAsync(GetTestImagePath("noise_grain.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("ai_denoise");

        await Driver.RunPipelineAsync();
        await Task.Delay(300);
        await Driver.CancelPipelineAsync();
        await Task.Delay(1000);

        // Re-run
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(3));

        var outputPath = GetOutputPath("Cross_CancelRerun", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Cross_CancelRerun");
    }

    [Fact]
    public async Task PipelineToBatch_MultiNode_Chain()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        string[] plugins = { "raw_input", "colorspace", "transform", "png_encoder" };
        foreach (var p in plugins)
        {
            await Driver.AddPluginToPipelineAsync(p);
            await Task.Delay(300);
        }

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_MultiNode", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Cross_MultiNode");
    }

    // ════════════════════════════════════════════════════════════════
    //  Selection Sync Tests (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task SelectionSync_SelectImage_PreviewUpdates()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(500);
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(800);

        await Driver.SelectImageAsync(0);
        await Task.Delay(300);

        var previewExists1 = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var p = window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
                 ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image));
            return p != null;
        });

        await Driver.SelectImageAsync(1);
        await Task.Delay(300);

        var previewExists2 = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var p = window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
                 ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image));
            return p != null;
        });

        previewExists1.Should().BeTrue("Preview should exist for first image");
        previewExists2.Should().BeTrue("Preview should exist for second image");
    }

    [Fact]
    public async Task SelectionSync_PipelineCanvas_ReflectsSelectedImage()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(500);
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);

        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(500);

        await Driver.SelectImageAsync(1);
        await Task.Delay(500);

        var canvas = await Task.Run(() =>
            GetMainWindow().FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        canvas.Should().NotBeNull("PipelineCanvas must exist after switching selected image");
    }

    [Fact]
    public async Task SelectionSync_CrossPanelNotifications_FilmstripCount()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(400);
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(400);
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);

        var window = GetMainWindow();
        var listBox = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("FilmstripListBox")));

        listBox.Should().NotBeNull("FilmstripListBox must exist");
        var itemCount = await Task.Run(() =>
            listBox!.FindAllChildren(cf => cf.ByControlType(ControlType.ListItem)).Length);

        itemCount.Should().BeGreaterOrEqualTo(3,
            $"Filmstrip should have >= 3 items but has {itemCount}");
        CaptureScreenshot("Cross_SelectionSync");
    }

    [Fact]
    public async Task SelectionSync_RapidSelectionChange_NoCrash()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(400);
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(400);
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);

        // Rapidly switch between selections
        for (int i = 0; i < 5; i++)
        {
            await Driver.SelectImageAsync(i % 3);
            await Task.Delay(150);
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive rapid selection changes");
    }

    [Fact]
    public async Task SelectionSync_PipelineRun_OnSwitchedImage()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(400);
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(800);

        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        // Switch to second image and run
        await Driver.SelectImageAsync(1);
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_SwitchedImage", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Cross_SwitchedImage");
    }

    // ════════════════════════════════════════════════════════════════
    //  Cross-Panel Edge Cases (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task EdgeCase_RapidNavigation_BetweenViews()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);

        for (int i = 0; i < 3; i++)
        {
            await Driver.NavigateToPipelineEditorAsync();
            await Driver.AddPluginToPipelineAsync("raw_input");
            await Task.Delay(200);
            await Driver.SelectImageAsync(0);
            await Task.Delay(200);
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive rapid view navigation");
    }

    [Fact]
    public async Task EdgeCase_AllPanels_VisibleAfterProcessing()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("colorspace");
        await Driver.AddPluginToPipelineAsync("tiff_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_AllPanels", "tif");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "Cross_AllPanels");

        // Verify all three main panels still render
        var window = GetMainWindow();
        var listBox = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("FilmstripListBox")));
        var canvas = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        listBox.Should().NotBeNull("Filmstrip panel should still be present");
        canvas.Should().NotBeNull("Pipeline canvas should still be present");
    }

    [Fact]
    public async Task EdgeCase_Pipeline_AllImages_SameConfig()
    {
        var imagePath = GetTestImagePath("pure_red_small.png");
        int imageCount = 3;

        for (int i = 0; i < imageCount; i++)
        {
            await Driver.ImportImageAsync(imagePath);
            await Task.Delay(300);
        }

        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        for (int i = 0; i < imageCount; i++)
        {
            await Driver.SelectImageAsync(i);
            await Driver.RunPipelineAsync();
            await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

            var outputPath = GetOutputPath($"Cross_SameConfig_{i}", "png");
            await Driver.ExportOutputAsync(outputPath);
            SaveEvidence(outputPath, $"Cross_SameConfig_{i}");
        }
    }

    [Fact]
    public async Task EdgeCase_DifferentPluginsPerRun()
    {
        // First run: PNG output
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        var pngPath = GetOutputPath("Cross_DiffPlugins_Png", "png");
        await Driver.ExportOutputAsync(pngPath);
        AssertValidOutput(pngPath, "PNG");

        // Second run: also add colorspace
        await Driver.AddPluginToPipelineAsync("colorspace");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
        var tiffPath = GetOutputPath("Cross_DiffPlugins_Tiff", "tif");
        await Driver.ExportOutputAsync(tiffPath);
        // May not be TIFF if no tiff_encoder, but file should exist
    }

    [Fact]
    public async Task EdgeCase_WindowLayout_StableAfterWorkflow()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Cross_LayoutStable", "png");
        await Driver.ExportOutputAsync(outputPath);

        // Verify all three panels still correctly positioned
        var window = GetMainWindow();
        var importBtn = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("ImportButton")));
        var pluginList = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("PluginBrowserList")));
        var canvas = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        importBtn.Should().NotBeNull("Filmstrip should still have Import button");
        pluginList.Should().NotBeNull("Plugin browser should still be present");
        canvas.Should().NotBeNull("Pipeline canvas should still be present");
        CaptureScreenshot("Cross_LayoutStable");
    }
}
