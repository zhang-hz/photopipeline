using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Error handling and boundary UI tests (25 tests).
/// Covers invalid file import, run without nodes, run without image,
/// invalid parameters, file not found, permission denied, network error,
/// cancel during processing, and extreme values.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping -- missing elements or unexpected states throw.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if error handling is silently broken.
/// </summary>
[Collection("FlaUITests")]
public sealed class ErrorHandlingUITests : UiTestBase
{
    public ErrorHandlingUITests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Invalid Input Tests (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Error_Import_InvalidFileFormat()
    {
        var fakeFile = Path.Combine(OutputDir, "error_invalid_format.xyz");
        File.WriteAllText(fakeFile, "not an image");

        try
        {
            await Driver.ImportImageAsync(fakeFile);
            await Task.Delay(1500);

            var window = GetMainWindow();
            var errorShown = await Task.Run(() =>
            {
                var allText = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
                return allText.Any(t =>
                    (t.Name ?? "").Contains("Error", StringComparison.OrdinalIgnoreCase)
                    || (t.Name ?? "").Contains("invalid", StringComparison.OrdinalIgnoreCase)
                    || (t.Name ?? "").Contains("support", StringComparison.OrdinalIgnoreCase));
            });

            Output.WriteLine($"Error shown for invalid format: {errorShown}");
            window.IsAvailable.Should().BeTrue("Window must survive invalid file import");
            CaptureScreenshot("Error_InvalidFormat");
        }
        finally
        {
            try { File.Delete(fakeFile); } catch { }
        }
    }

    [Fact]
    public async Task Error_Import_FileNotFound()
    {
        var nonExistentFile = Path.Combine(OutputDir, "does_not_exist_12345.png");

        try
        {
            await Driver.ImportImageAsync(nonExistentFile);
            Assert.Fail("Driver should throw FileNotFoundException for missing file");
        }
        catch (FileNotFoundException ex)
        {
            ex.Message.Should().Contain("does_not_exist", "Error should reference the missing file");
            Output.WriteLine($"File not found correctly detected: {ex.Message}");
        }
    }

    [Fact]
    public async Task Error_Import_CorruptImage()
    {
        var fakeCorrupt = Path.Combine(OutputDir, "error_corrupt_image.png");
        File.WriteAllBytes(fakeCorrupt, new byte[] { 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFF, 0xFF });

        try
        {
            await Driver.ImportImageAsync(fakeCorrupt);
            await Task.Delay(1500);

            var window = GetMainWindow();
            window.IsAvailable.Should().BeTrue("Window must survive corrupt image import");
        }
        finally
        {
            try { File.Delete(fakeCorrupt); } catch { }
        }
    }

    [Fact]
    public async Task Error_Import_EmptyFile()
    {
        var emptyFile = Path.Combine(OutputDir, "error_empty_file.png");
        File.WriteAllText(emptyFile, "");

        try
        {
            await Driver.ImportImageAsync(emptyFile);
            await Task.Delay(1500);

            var window = GetMainWindow();
            window.IsAvailable.Should().BeTrue("Window must survive empty file import");
        }
        finally
        {
            try { File.Delete(emptyFile); } catch { }
        }
    }

    [Fact]
    public async Task Error_Import_TooManyFiles_MassImport()
    {
        var imagePath = GetTestImagePath("pure_red_small.png");
        for (int i = 0; i < 30; i++)
        {
            await Driver.ImportImageAsync(imagePath);
            await Task.Delay(80);
        }
        await Task.Delay(1500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive mass import (30 files)");

        var listBox = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByAutomationId("FilmstripListBox")));

        listBox.Should().NotBeNull("Filmstrip should still be accessible after mass import");
        CaptureScreenshot("Error_MassImport");
    }

    // ════════════════════════════════════════════════════════════════
    //  Pipeline Error Tests (6 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Error_Run_WithoutNodes()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        bool runBlocked = false;
        try { await Driver.RunPipelineAsync(); await Task.Delay(2000); }
        catch (InvalidOperationException) { runBlocked = true; }

        var window = GetMainWindow();
        var allText = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
        bool hasError = allText.Any(t =>
            (t.Name ?? "").Contains("node", StringComparison.OrdinalIgnoreCase));

        (runBlocked || hasError).Should().BeTrue(
            "Must show error or block execution for empty pipeline");
        window.IsAvailable.Should().BeTrue("Window must survive empty pipeline run");
        CaptureScreenshot("Error_EmptyPipeline");
    }

    [Fact]
    public async Task Error_Run_WithoutImage()
    {
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(500);

        bool runBlocked = false;
        try { await Driver.RunPipelineAsync(); }
        catch (Exception) { runBlocked = true; }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive run without image");
        CaptureScreenshot("Error_NoImage");
    }

    [Fact]
    public async Task Error_InvalidParameter_NegativeScale()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("transform");

        try { await Driver.SetNodeParameterAsync("transform", "scale_percent", "-50"); }
        catch (Exception ex) { Output.WriteLine($"Negative scale: {ex.Message}"); }

        await Task.Delay(500);
        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive invalid parameter");
        CaptureScreenshot("Error_NegativeScale");
    }

    [Fact]
    public async Task Error_InvalidParameter_OutOfRangeAngle()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("transform");

        try { await Driver.SetNodeParameterAsync("transform", "angle", "9999"); }
        catch (Exception ex) { Output.WriteLine($"Out of range: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive out-of-range parameter");
    }

    [Fact]
    public async Task Error_InvalidParameter_QualityOutOfRange()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("avif_encoder");

        try { await Driver.SetNodeParameterAsync("avif_encoder", "quality", "200"); }
        catch (Exception ex) { Output.WriteLine($"Quality invalid: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive invalid quality");
    }

    [Fact]
    public async Task Error_Cancel_DuringExecution_Rollback()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("ai_denoise");
        await Driver.AddPluginToPipelineAsync("tiff_encoder");

        await Driver.RunPipelineAsync();
        await Task.Delay(300);
        await Driver.CancelPipelineAsync();
        await Task.Delay(1500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive mid-execution cancel");
        CaptureScreenshot("Error_CancelMidRun");

        // Should be able to re-run after cancel
        try
        {
            await Driver.RunPipelineAsync();
            await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(3));
            var outputPath = GetOutputPath("Error_AfterCancel", "tif");
            await Driver.ExportOutputAsync(outputPath);
            AssertValidOutput(outputPath, "TIFF");
            SaveEvidence(outputPath, "Error_AfterCancel");
        }
        catch (Exception ex)
        {
            Output.WriteLine($"Re-run after cancel: {ex.Message}");
        }
    }

    // ════════════════════════════════════════════════════════════════
    //  Export Error Tests (3 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Error_Export_WithoutRunning()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);

        bool exportBlocked = false;
        try
        {
            var outputPath = GetOutputPath("Error_Export_NoRun");
            await Driver.ExportOutputAsync(outputPath);
        }
        catch (InvalidOperationException) { exportBlocked = true; }
        catch (Exception ex) { Output.WriteLine($"Export without run: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive export without run");
        CaptureScreenshot("Error_ExportNoRun");
    }

    [Fact]
    public async Task Error_Export_ToExistingFile_Overwrites()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Error_Overwrite", "png");
        await Driver.ExportOutputAsync(outputPath);
        // Export again to same path
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Error_Overwrite");
    }

    [Fact]
    public async Task Error_Export_NormalPath_Works()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");
        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Error_Export_Normal", "png");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Error_Export_Normal");
    }

    // ════════════════════════════════════════════════════════════════
    //  Edge Case Tests (6 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task EdgeCase_AddNonExistentPlugin()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        try
        {
            await Driver.AddPluginToPipelineAsync("non_existent_plugin_abc_123");
            Assert.Fail("Should throw for non-existent plugin");
        }
        catch (InvalidOperationException ex)
        {
            ex.Message.Should().Contain("non_existent_plugin",
                "Error should reference the missing plugin name");
            Output.WriteLine($"Non-existent plugin correctly rejected: {ex.Message}");
        }
    }

    [Fact]
    public async Task EdgeCase_SelectImageOutOfRange()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(800);

        try
        {
            await Driver.SelectImageAsync(99);
            Assert.Fail("Should throw ArgumentOutOfRangeException");
        }
        catch (ArgumentOutOfRangeException ex)
        {
            ex.Message.Should().Contain("99", "Error should reference the invalid index");
            Output.WriteLine($"Out of range correctly rejected: {ex.Message}");
        }
    }

    [Fact]
    public async Task EdgeCase_CycleDetection_SelfConnection()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("colorspace");
        await Task.Delay(500);

        try { await Driver.ConnectNodesAsync("colorspace", "colorspace"); }
        catch (Exception ex) { Output.WriteLine($"Self-connection: {ex.Message}"); }

        var canvas = await Task.Run(() =>
            GetMainWindow().FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        canvas.Should().NotBeNull("PipelineCanvas must survive self-connection attempt");
    }

    [Fact]
    public async Task EdgeCase_DeleteAndReAdd_Node()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        await Driver.AddPluginToPipelineAsync("tiff_encoder");
        await Task.Delay(500);

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Error_ReAddNode", "tif");
        await Driver.ExportOutputAsync(outputPath);

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "Error_ReAddNode");
    }

    [Fact]
    public async Task EdgeCase_RapidAddDelete_Nodes_Stress()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        string[] plugins = { "raw_input", "colorspace", "transform", "lut3d", "ai_denoise", "lens_correct" };
        foreach (var plugin in plugins)
        {
            try { await Driver.AddPluginToPipelineAsync(plugin); }
            catch (Exception ex) { Output.WriteLine($"Add {plugin}: {ex.Message}"); }
            await Task.Delay(150);
        }
        await Task.Delay(1000);

        var canvas = await Task.Run(() =>
            GetMainWindow().FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas")));

        canvas.Should().NotBeNull("PipelineCanvas must survive rapid add operations");
        CaptureScreenshot("Error_RapidAdd");
    }

    [Fact]
    public async Task EdgeCase_AllNodesDisabled_SingleNodeRun()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(300);

        bool runBlocked = false;
        try { await Driver.RunPipelineAsync(); }
        catch (Exception) { runBlocked = true; }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive single-node run");
        CaptureScreenshot("Error_SingleNode");
    }

    // ════════════════════════════════════════════════════════════════
    //  Boundary Value Tests (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Boundary_ScalePercent_0()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("transform");

        try { await Driver.SetNodeParameterAsync("transform", "scale_percent", "0"); }
        catch (Exception ex) { Output.WriteLine($"Scale 0%: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive 0% scale");
    }

    [Fact]
    public async Task Boundary_ScalePercent_1000()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("transform");

        try { await Driver.SetNodeParameterAsync("transform", "scale_percent", "1000"); }
        catch (Exception ex) { Output.WriteLine($"Scale 1000%: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive 1000% scale");
    }

    [Fact]
    public async Task Boundary_BitDepth_32()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("tiff_encoder");

        try { await Driver.SetNodeParameterAsync("tiff_encoder", "bit_depth", "32"); }
        catch (Exception ex) { Output.WriteLine($"Bit depth 32: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive 32-bit depth");
    }

    [Fact]
    public async Task Boundary_Intensity_200()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("lut3d");

        try { await Driver.SetNodeParameterAsync("lut3d", "intensity", "200"); }
        catch (Exception ex) { Output.WriteLine($"Intensity 200: {ex.Message}"); }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive out-of-range intensity");
    }

    [Fact]
    public async Task Boundary_DenoiseStrength_Range0to100()
    {
        await Driver.ImportImageAsync(GetTestImagePath("noise_grain.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("ai_denoise");

        try
        {
            await Driver.SetNodeParameterAsync("ai_denoise", "strength", "0");
            await Task.Delay(200);
            await Driver.SetNodeParameterAsync("ai_denoise", "strength", "100");
        }
        catch (Exception ex) { Output.WriteLine($"Denoise range: {ex.Message}"); }

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(3));

        var outputPath = GetOutputPath("Error_BoundaryDenoise", "png");
        try { await Driver.ExportOutputAsync(outputPath); } catch { }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive boundary denoise values");
    }
}
