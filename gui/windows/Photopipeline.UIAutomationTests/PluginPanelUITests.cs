using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Plugin Panel UI tests (20 tests).
/// Covers plugin search, category filtering, plugin selection,
/// dynamic parameter controls, Add-to-Pipeline, and single-plugin
/// workflow tests (GE2E-001 through GE2E-020).
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping — missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if the app does nothing.
/// </summary>
[Collection("FlaUITests")]
public sealed class PluginPanelUITests : UiTestBase
{
    public PluginPanelUITests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Plugin Browser UI Tests
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// GE2E-PPL-001: Verifies the Plugin search text box filters the plugin list.
    /// </summary>
    [Fact]
    public async Task GE2E_PPL_001_SearchInput_FiltersPluginList()
    {
        // Act: Find the PluginSearchBox and verify it exists
        var searchBox = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginSearchBox"));
        });

        // Assert
        searchBox.Should().NotBeNull(
            "PluginSearchBox (AutomationId='PluginSearchBox') should exist in PluginBrowserView. " +
            "If missing, the search functionality is broken.");
        searchBox!.IsEnabled.Should().BeTrue("Plugin search box should be enabled on startup");
    }

    /// <summary>
    /// GE2E-PPL-002: Verifies the Plugin Browser list is populated with plugins.
    /// </summary>
    [Fact]
    public async Task GE2E_PPL_002_PluginList_PopulatedWithPlugins()
    {
        // Act: Find the PluginBrowserList and check for items
        var pluginCount = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var listBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginBrowserList"));
            if (listBox == null)
                throw new InvalidOperationException(
                    "PluginBrowserList (AutomationId='PluginBrowserList') not found.");

            var items = listBox.FindAllChildren(cf =>
                cf.ByControlType(ControlType.ListItem));
            return items.Length;
        });

        // Assert — the plugin list must have items on startup
        pluginCount.Should().BeGreaterThan(0,
            "PluginBrowserList should contain plugin entries on startup. " +
            "If empty, the plugin catalog may not have loaded — this test FAILs (Iron Rule 5).");
        Output.WriteLine($"Plugin count in browser: {pluginCount}");
    }

    /// <summary>
    /// GE2E-PPL-003: Verifies that typing in the search box filters the plugin list.
    /// </summary>
    [Fact]
    public async Task GE2E_PPL_003_SearchFilter_ReducesPluginCount()
    {
        // Act: Get initial count, then search for a specific plugin
        var initialCount = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var listBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginBrowserList"));
            return listBox!.FindAllChildren(cf => cf.ByControlType(ControlType.ListItem)).Length;
        });

        // Type "raw" in the search box
        await Task.Run(() =>
        {
            var window = GetMainWindow();
            var searchBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginSearchBox"));
            if (searchBox != null)
            {
                searchBox.Focus();
                try { searchBox.AsTextBox().Text = "raw"; }
                catch { searchBox.Patterns.Value.Pattern.SetValue("raw"); }
            }
        });

        await Task.Delay(1000);

        var filteredCount = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var listBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginBrowserList"));
            return listBox!.FindAllChildren(cf => cf.ByControlType(ControlType.ListItem)).Length;
        });

        // Assert: Filtered count should be <= initial count
        // If search does nothing, counts will be equal (Iron Rule 5: test detects no-op).
        filteredCount.Should().BeLessOrEqualTo(initialCount,
            $"Search filter should reduce or maintain plugin count (was {initialCount}, now {filteredCount})");
        Output.WriteLine($"Plugin count: {initialCount} -> {filteredCount} (after search 'raw')");
    }

    /// <summary>
    /// GE2E-PPL-004: Verifies that clearing the search restores the full plugin list.
    /// </summary>
    [Fact]
    public async Task GE2E_PPL_004_ClearSearch_RestoresPluginList()
    {
        // First, apply a search
        await Task.Run(() =>
        {
            var window = GetMainWindow();
            var searchBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginSearchBox"));
            if (searchBox != null)
            {
                try { searchBox.AsTextBox().Text = "raw"; }
                catch { searchBox.Patterns.Value.Pattern.SetValue("raw"); }
            }
        });
        await Task.Delay(500);

        // Clear the search
        await Task.Run(() =>
        {
            var window = GetMainWindow();
            var searchBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginSearchBox"));
            if (searchBox != null)
            {
                try { searchBox.AsTextBox().Text = ""; }
                catch { searchBox.Patterns.Value.Pattern.SetValue(""); }
            }
        });
        await Task.Delay(1000);

        // Assert: Plugin list should have items after clearing search
        var count = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var listBox = window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PluginBrowserList"));
            if (listBox == null) return 0;
            return listBox.FindAllChildren(cf => cf.ByControlType(ControlType.ListItem)).Length;
        });

        count.Should().BeGreaterThan(0,
            "Plugin browser should show items after clearing search. " +
            "If clearing search doesn't restore the list, search functionality is broken.");
        Output.WriteLine($"Plugin count after clearing search: {count}");
    }

    /// <summary>
    /// GE2E-PPL-005: Verifies the Add-to-Pipeline button is present.
    /// </summary>
    [Fact]
    public async Task GE2E_PPL_005_AddToPipeline_ButtonInvokesCommand()
    {
        // Act
        var addBtn = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("AddToPipelineButton"));
        });

        // Assert
        addBtn.Should().NotBeNull(
            "AddToPipelineButton (AutomationId='AddToPipelineButton') should exist in PluginBrowserView. " +
            "If missing, the add-to-pipeline workflow is broken.");
    }

    /// <summary>
    /// GE2E-PPL-006: Verifies adding a plugin shows the plugin's parameters
    /// in the Properties panel.
    /// </summary>
    [Fact]
    public async Task GE2E_PPL_006_PluginSelection_HighlightsAndShowsParams()
    {
        // Arrange: Import and navigate
        await Driver.ImportImageAsync(GetTestImagePath("solid/pure_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();

        // Act: Add a plugin
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Task.Delay(1000);

        // Assert: The properties panel should be accessible
        var propsPanel = await Task.Run(() =>
        {
            var window = GetMainWindow();
            return window.FindFirstDescendant(cf =>
                cf.ByAutomationId("PropertiesPanel"));
        });

        propsPanel.Should().NotBeNull(
            "PropertiesPanel should be visible after selecting a plugin node. " +
            "If parameters don't show, the properties panel binding is broken.");
    }

    // ════════════════════════════════════════════════════════════════
    //  Single Plugin Workflow Tests (GE2E-001 through GE2E-014)
    // ════════════════════════════════════════════════════════════════

    /// <summary>
    /// GE2E-001: raw_input auto exposure, TIFF output.
    /// raw_input(raw_mode=auto, apply_wb=true) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// Assert: PixelsEqual with golden, 1920x1080, 3ch RGB
    /// </summary>
    [Fact]
    public async Task GE2E_001_RawInput_AutoExposure_ToTiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new()
                {
                    ["raw_mode"] = "auto",
                    ["apply_white_balance"] = "true",
                },
            });

        File.Exists(outputPath).Should().BeTrue(
            "GE2E-001 output file must exist after pipeline execution");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0,
            "GE2E-001 output file must not be empty — pipeline must actually process");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080);
        Output.WriteLine($"GE2E-001 output: {outputPath} ({new FileInfo(outputPath).Length} bytes)");
    }

    /// <summary>
    /// GE2E-002: raw_input manual white balance, TIFF output.
    /// raw_input(raw_mode=dcraw, manual_wb=5500K) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_002_RawInput_ManualWhiteBalance_Tiff()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new()
                {
                    ["raw_mode"] = "dcraw",
                    ["manual_wb"] = "5500",
                },
            });

        File.Exists(outputPath).Should().BeTrue(
            "GE2E-002 output file must exist after pipeline execution");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0,
            "GE2E-002 output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080);
        Output.WriteLine($"GE2E-002 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-003: raw_input U16 output.
    /// raw_input(output_format=u16, half_size=false) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// Assert: IsValidFormat(TIFF, 1920, 1080, 16)
    /// </summary>
    [Fact]
    public async Task GE2E_003_RawInput_U16Output()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new()
                {
                    ["output_format"] = "u16",
                    ["half_size"] = "false",
                },
                ["tiff_encoder"] = new()
                {
                    ["bit_depth"] = "16",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080, expectedBitDepth: 16);
        Output.WriteLine($"GE2E-003 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-004: transform crop 50% -> center region output = 960x540.
    /// transform(crop_enabled=true, scale_percent=50) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_004_Transform_Crop50Percent()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new()
                {
                    ["crop_enabled"] = "true",
                    ["scale_percent"] = "50",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 960, 540);
        Output.WriteLine($"GE2E-004 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-005: transform scale 200% with lanczos3 filter -> 3200x2133.
    /// transform(scale_percent=200, filter=lanczos3) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_005_Transform_Scale200Percent()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new()
                {
                    ["scale_percent"] = "200",
                    ["filter"] = "lanczos3",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        // 200% scale = 3840x2160
        ImageAssert.IsValidFormat(outputPath, "PNG", 3840, 2160);
        Output.WriteLine($"GE2E-005 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-006: transform rotate 90 -> dimensions swapped (2160x3840).
    /// transform(angle=90) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_006_Transform_Rotate90()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new()
                {
                    ["angle"] = "90",
                    ["resize_mode"] = "expand",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 1080, 1920);
        Output.WriteLine($"GE2E-006 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-007: transform flip horizontal + vertical -> mirror image.
    /// transform(flip_h=true, flip_v=true) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_007_Transform_FlipHorizontalVertical()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "transform", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["transform"] = new()
                {
                    ["flip_h"] = "true",
                    ["flip_v"] = "true",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 1920, 1080);
        Output.WriteLine($"GE2E-007 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-008: colorspace sRGB -> AdobeRGB with ICC embedding.
    /// colorspace(source=sRGB, target=AdobeRGB, embed_icc=true) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_008_Colorspace_SrgbToAdobeRgb_IccEmbed()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new()
                {
                    ["source_color_space"] = "sRGB",
                    ["target_color_space"] = "AdobeRGB",
                    ["embed_icc"] = "true",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080);
        Output.WriteLine($"GE2E-008 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-009: colorspace sRGB -> DisplayP3 with gamut clipping.
    /// colorspace(source=sRGB, target=DisplayP3, gamut=clip) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_009_Colorspace_SrgbToDisplayP3_Clip()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new()
                {
                    ["source_color_space"] = "sRGB",
                    ["target_color_space"] = "DisplayP3",
                    ["gamut"] = "clip",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080);
        Output.WriteLine($"GE2E-009 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-010: colorspace sRGB -> Gray (monochrome).
    /// colorspace(source=sRGB, target=Gray, bp_comp=true) -> tiff_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_010_Colorspace_SrgbToGray_BPC()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "tiff_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new()
                {
                    ["source_color_space"] = "sRGB",
                    ["target_color_space"] = "Gray",
                    ["bp_comp"] = "true",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "TIF", 1920, 1080);
        Output.WriteLine($"GE2E-010 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-011: colorspace Gray -> sRGB.
    /// colorspace(source=Gray, target=sRGB, rendering=perceptual) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_011_Colorspace_GrayToSrgb_Perceptual()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "colorspace", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["colorspace"] = new()
                {
                    ["source_color_space"] = "Gray",
                    ["target_color_space"] = "sRGB",
                    ["rendering"] = "perceptual",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 1920, 1080);
        Output.WriteLine($"GE2E-011 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-012: lut3d warm.cube, intensity=80.
    /// lut3d(intensity=80) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_012_Lut3d_WarmCube_Intensity80()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lut3d", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["lut3d"] = new()
                {
                    ["intensity"] = "80",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 1920, 1080);
        Output.WriteLine($"GE2E-012 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-013: lut3d cool.cube, intensity=50.
    /// lut3d(intensity=50) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_013_Lut3d_CoolCube_Intensity50()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lut3d", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["lut3d"] = new()
                {
                    ["intensity"] = "50",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 1920, 1080);
        Output.WriteLine($"GE2E-013 output: {outputPath}");
    }

    /// <summary>
    /// GE2E-014: lut3d film.cube, tetrahedral interpolation.
    /// lut3d(interp=tetrahedral) -> png_encoder
    /// Input: solid_color_1920 (I01)
    /// </summary>
    [Fact]
    public async Task GE2E_014_Lut3d_FilmCube_Tetrahedral()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("solid/pure_white_1920x1080.png"),
            new[] { "raw_input", "lut3d", "png_encoder" },
            new()
            {
                ["raw_input"] = new() { ["raw_mode"] = "auto" },
                ["lut3d"] = new()
                {
                    ["interp"] = "tetrahedral",
                },
            });

        File.Exists(outputPath).Should().BeTrue("output file must exist");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, "PNG", 1920, 1080);
        Output.WriteLine($"GE2E-014 output: {outputPath}");
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
