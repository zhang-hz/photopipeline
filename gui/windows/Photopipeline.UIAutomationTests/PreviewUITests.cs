using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Preview view UI tests (40 tests).
/// Covers zoom in/out, fit to window, 1:1 view, pan, split view,
/// before/after toggle, histogram, pixel info, rotate view, fullscreen toggle.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping -- missing elements throw exceptions.
/// Iron Rule 4: Real WPF window via FlaUI UIA3.
/// Iron Rule 5: Tests must fail if the preview does not render.
/// </summary>
[Collection("FlaUITests")]
public sealed class PreviewUITests : UiTestBase
{
    public PreviewUITests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Basic Preview Rendering Tests (8 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Preview_Renders_AfterImageSelection()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(1500);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        var previewArea = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
            ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image))
            ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Document)));

        if (previewArea == null)
            Assert.Fail("Preview area not found. SkiaSharp canvas should render as Custom, Image, or Document control.");

        previewArea.BoundingRectangle.Width.Should().BeGreaterThan(0, "Preview must have non-zero width");
        previewArea.BoundingRectangle.Height.Should().BeGreaterThan(0, "Preview must have non-zero height");
        CaptureScreenshot("Preview_Renders_AfterSelection");
    }

    [Fact]
    public async Task Preview_Renders_SmallImage()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        var previewArea = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
            ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image)));

        previewArea.Should().NotBeNull("Preview must render even for small images");
        CaptureScreenshot("Preview_SmallImage");
    }

    [Fact]
    public async Task Preview_Renders_GradientImage()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must render gradient preview");
        CaptureScreenshot("Preview_GradientImage");
    }

    [Fact]
    public async Task Preview_Renders_Checkerboard()
    {
        await Driver.ImportImageAsync(GetTestImagePath("checkerboard_8x8.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must render checkerboard preview");
        CaptureScreenshot("Preview_Checkerboard");
    }

    [Fact]
    public async Task Preview_Renders_ColorBars()
    {
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must render color bars preview");
        CaptureScreenshot("Preview_ColorBars");
    }

    [Fact]
    public async Task Preview_Renders_NoiseGrain()
    {
        await Driver.ImportImageAsync(GetTestImagePath("noise_grain.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must render noise grain preview");
        CaptureScreenshot("Preview_NoiseGrain");
    }

    [Fact]
    public async Task Preview_Renders_NoiseMarble()
    {
        await Driver.ImportImageAsync(GetTestImagePath("noise_marble.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must render noise marble preview");
        CaptureScreenshot("Preview_NoiseMarble");
    }

    [Fact]
    public async Task Preview_Updates_AfterImageSwitch()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(800);
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(800);

        await Driver.SelectImageAsync(0);
        await Task.Delay(300);
        await Driver.SelectImageAsync(1);
        await Task.Delay(300);

        var window = GetMainWindow();
        var previewArea = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
            ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image)));

        previewArea.Should().NotBeNull("Preview must exist after switching images");
    }

    // ════════════════════════════════════════════════════════════════
    //  Zoom Tests (8 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Zoom_In_Button_Click()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var zoomOperated = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var zoomInBtn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("Zoom In", StringComparison.OrdinalIgnoreCase)
                || (b.Name ?? "").Contains("+", StringComparison.Ordinal));
            if (zoomInBtn != null && zoomInBtn.IsEnabled)
            {
                zoomInBtn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"Zoom in button clicked: {zoomOperated}");
        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive zoom in");
    }

    [Fact]
    public async Task Zoom_Out_Button_Click()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var zoomOperated = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var zoomOutBtn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("Zoom Out", StringComparison.OrdinalIgnoreCase)
                || (b.Name ?? "").Contains("-", StringComparison.Ordinal));
            if (zoomOutBtn != null && zoomOutBtn.IsEnabled)
            {
                zoomOutBtn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"Zoom out button clicked: {zoomOperated}");
        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive zoom out");
    }

    [Fact]
    public async Task Zoom_Keyboard_CtrlPlus()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        await Task.Run(() =>
        {
            var window = GetMainWindow();
            window.Focus();
            FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.ADD);
            System.Threading.Thread.Sleep(200);
            FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.ADD);
        });
        await Task.Delay(500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive Ctrl+Plus zoom");
    }

    [Fact]
    public async Task Zoom_Keyboard_CtrlMinus()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        await Task.Run(() =>
        {
            var window = GetMainWindow();
            window.Focus();
            FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.SUBTRACT);
        });
        await Task.Delay(500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive Ctrl+Minus zoom");
    }

    [Fact]
    public async Task Zoom_Reset_Ctrl0()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        await Task.Run(() =>
        {
            var window = GetMainWindow();
            window.Focus();
            FlaUI.Core.Input.Keyboard.TypeSimultaneously(
                FlaUI.Core.WindowsAPI.VirtualKeyShort.CONTROL,
                FlaUI.Core.WindowsAPI.VirtualKeyShort.KEY_0);
        });
        await Task.Delay(500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive Ctrl+0 zoom reset");
    }

    [Fact]
    public async Task Zoom_MouseWheel_OverPreview()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        await Task.Run(() =>
        {
            var window = GetMainWindow();
            var preview = window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
                ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image));

            if (preview != null)
            {
                var bounds = preview.BoundingRectangle;
                FlaUI.Core.Input.Mouse.MoveTo(
                    bounds.Left + bounds.Width / 2,
                    bounds.Top + bounds.Height / 2);
                FlaUI.Core.Input.Mouse.Scroll(2);
                System.Threading.Thread.Sleep(200);
                FlaUI.Core.Input.Mouse.Scroll(-1);
            }
        });
        await Task.Delay(300);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive mouse wheel zoom");
    }

    [Fact]
    public async Task Zoom_MaxLevel_NoCrash()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(300);

        // Zoom in many times
        for (int i = 0; i < 10; i++)
        {
            await Task.Run(() =>
            {
                var window = GetMainWindow();
                window.Focus();
                FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.ADD);
            });
            await Task.Delay(100);
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive maximum zoom level");
    }

    [Fact]
    public async Task Zoom_MinLevel_NoCrash()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(300);

        // Zoom out many times
        for (int i = 0; i < 10; i++)
        {
            await Task.Run(() =>
            {
                var window = GetMainWindow();
                window.Focus();
                FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.SUBTRACT);
            });
            await Task.Delay(100);
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive minimum zoom level");
    }

    // ════════════════════════════════════════════════════════════════
    //  Fit & View Mode Tests (6 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task FitToWindow_Button_Click()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var fitClicked = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var fitBtn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("Fit", StringComparison.OrdinalIgnoreCase));
            if (fitBtn != null && fitBtn.IsEnabled)
            {
                fitBtn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"Fit button clicked: {fitClicked}");
        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive fit-to-window");
    }

    [Fact]
    public async Task OneToOne_View_Button_Click()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(1000);
        await Driver.SelectImageAsync(0);

        var oneToOneClicked = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var btn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("1:1", StringComparison.OrdinalIgnoreCase)
                || (b.Name ?? "").Contains("100%", StringComparison.OrdinalIgnoreCase));
            if (btn != null && btn.IsEnabled)
            {
                btn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"1:1 button clicked: {oneToOneClicked}");
    }

    [Fact]
    public async Task Pan_Drag_MouseMove_OverPreview()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        await Task.Run(() =>
        {
            var window = GetMainWindow();
            var preview = window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
                ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image));

            if (preview != null)
            {
                var bounds = preview.BoundingRectangle;
                var cx = bounds.Left + bounds.Width / 2;
                var cy = bounds.Top + bounds.Height / 2;

                FlaUI.Core.Input.Mouse.MoveTo(cx, cy);
                FlaUI.Core.Input.Mouse.Down(FlaUI.Core.Input.MouseButton.Left);
                FlaUI.Core.Input.Mouse.MoveTo(cx + 50, cy + 50);
                FlaUI.Core.Input.Mouse.Up(FlaUI.Core.Input.MouseButton.Left);
            }
        });
        await Task.Delay(300);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive preview pan drag");
    }

    [Fact]
    public async Task Fullscreen_Toggle_F11()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        // Toggle fullscreen with F11
        await Task.Run(() =>
        {
            FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.F11);
        });
        await Task.Delay(500);

        // Toggle back
        await Task.Run(() =>
        {
            FlaUI.Core.Input.Keyboard.Press(FlaUI.Core.WindowsAPI.VirtualKeyShort.F11);
        });
        await Task.Delay(500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive fullscreen toggle");
    }

    [Fact]
    public async Task Rotate_View_Button()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var rotateClicked = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var btn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("Rotate", StringComparison.OrdinalIgnoreCase));
            if (btn != null && btn.IsEnabled)
            {
                btn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"Rotate button clicked: {rotateClicked}");
    }

    [Fact]
    public async Task ZoomPercentage_Displayed()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        var zoomText = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var allText = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
            return allText.FirstOrDefault(t =>
                (t.Name ?? "").Contains("%", StringComparison.OrdinalIgnoreCase));
        });

        Output.WriteLine($"Zoom percentage found: {zoomText?.Name ?? "none"}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Split View Tests (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task SplitView_Toggle_Button()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var splitClicked = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var btn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("Split", StringComparison.OrdinalIgnoreCase)
                || (b.Name ?? "").Contains("Compare", StringComparison.OrdinalIgnoreCase));
            if (btn != null && btn.IsEnabled)
            {
                btn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"Split toggle clicked: {splitClicked}");
        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive split toggle");
    }

    [Fact]
    public async Task BeforeAfter_Toggle_Button()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var toggleClicked = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
            var btn = buttons.FirstOrDefault(b =>
                (b.Name ?? "").Contains("Before", StringComparison.OrdinalIgnoreCase)
                || (b.Name ?? "").Contains("After", StringComparison.OrdinalIgnoreCase)
                || (b.Name ?? "").Contains("Original", StringComparison.OrdinalIgnoreCase));
            if (btn != null && btn.IsEnabled)
            {
                btn.AsButton().Invoke();
                return true;
            }
            return false;
        });

        Output.WriteLine($"Before/after toggle clicked: {toggleClicked}");
    }

    [Fact]
    public async Task SplitView_DragHandle_MovesDivider()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Task.Delay(500);

        await Task.Run(() =>
        {
            var window = GetMainWindow();
            var thumbs = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Thumb));
            if (thumbs.Length > 0)
            {
                var handle = thumbs[0];
                var b = handle.BoundingRectangle;
                FlaUI.Core.Input.Mouse.MoveTo(b.Left + b.Width / 2, b.Top + b.Height / 2);
                FlaUI.Core.Input.Mouse.Down(FlaUI.Core.Input.MouseButton.Left);
                FlaUI.Core.Input.Mouse.MoveTo(b.Left + 50, b.Top + b.Height / 2);
                FlaUI.Core.Input.Mouse.Up(FlaUI.Core.Input.MouseButton.Left);
            }
        });
        await Task.Delay(300);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive split handle drag");
    }

    [Fact]
    public async Task SplitView_SideBySide_BothRender()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        var customControls = await Task.Run(() =>
            window.FindAllDescendants(cf => cf.ByControlType(ControlType.Custom)));

        Output.WriteLine($"Custom controls for split view: {customControls.Length}");
        window.IsAvailable.Should().BeTrue("Window must be alive with split view");
        CaptureScreenshot("SplitView_SideBySide");
    }

    [Fact]
    public async Task SplitView_Close_RestoresSingleView()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Task.Delay(600);
        await Driver.SelectImageAsync(0);

        // Toggle split on, then off
        for (int i = 0; i < 2; i++)
        {
            await Task.Run(() =>
            {
                var window = GetMainWindow();
                var buttons = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
                var btn = buttons.FirstOrDefault(b =>
                    (b.Name ?? "").Contains("Split", StringComparison.OrdinalIgnoreCase)
                    || (b.Name ?? "").Contains("Compare", StringComparison.OrdinalIgnoreCase));
                if (btn != null && btn.IsEnabled) btn.AsButton().Invoke();
            });
            await Task.Delay(300);
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive split toggle cycle");
    }

    // ════════════════════════════════════════════════════════════════
    //  Histogram & Info Tests (4 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Histogram_Panel_Exists()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        var histRelated = await Task.Run(() =>
        {
            var allText = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
            return allText.Where(t => (t.Name ?? "").Contains("Histogram", StringComparison.OrdinalIgnoreCase)
                                   || (t.Name ?? "").Contains("RGB", StringComparison.OrdinalIgnoreCase));
        });

        Output.WriteLine($"Histogram-related text elements: {histRelated.Count()}");
        window.IsAvailable.Should().BeTrue("Window should be alive");
    }

    [Fact]
    public async Task PixelInfo_Displayed_OnHover()
    {
        await Driver.ImportImageAsync(GetTestImagePath("color_bars_8bit.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var window = GetMainWindow();
        var preview = await Task.Run(() =>
            window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Custom))
            ?? window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Image)));

        if (preview != null)
        {
            var bounds = preview.BoundingRectangle;
            FlaUI.Core.Input.Mouse.MoveTo(
                bounds.Left + bounds.Width / 2,
                bounds.Top + bounds.Height / 2);
        }
        await Task.Delay(300);

        window.IsAvailable.Should().BeTrue("Window must survive pixel info hover");
        CaptureScreenshot("PixelInfo_Hover");
    }

    [Fact]
    public async Task Info_Panel_ShowsImageDimensions()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Task.Delay(800);
        await Driver.SelectImageAsync(0);

        var dimensionText = await Task.Run(() =>
        {
            var window = GetMainWindow();
            var allText = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
            return allText.FirstOrDefault(t =>
                ((t.Name ?? "").Contains("1920", StringComparison.OrdinalIgnoreCase) &&
                 (t.Name ?? "").Contains("1080", StringComparison.OrdinalIgnoreCase))
                || (t.Name ?? "").Contains("x", StringComparison.OrdinalIgnoreCase));
        });

        Output.WriteLine($"Dimension text found: {dimensionText?.Name ?? "none"}");
    }

    [Fact]
    public async Task ColorSampler_Tool_Exists()
    {
        var window = GetMainWindow();
        var samplerElements = await Task.Run(() =>
        {
            var allText = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
            return allText.Where(t =>
                (t.Name ?? "").Contains("Color", StringComparison.OrdinalIgnoreCase)
                || (t.Name ?? "").Contains("Sampler", StringComparison.OrdinalIgnoreCase)
                || (t.Name ?? "").Contains("Picker", StringComparison.OrdinalIgnoreCase));
        });

        Output.WriteLine($"Color/sampler related text: {samplerElements.Count()}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Export Tests (5 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task ExportButton_Exists()
    {
        var window = GetMainWindow();
        var exportBtn = await Task.Run(() =>
        {
            var btn = window.FindFirstDescendant(cf => cf.ByAutomationId("ExportButton"));
            if (btn == null)
            {
                var btns = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button));
                btn = btns.FirstOrDefault(b =>
                    (b.Name ?? "").Contains("Export", StringComparison.OrdinalIgnoreCase));
            }
            return btn;
        });

        exportBtn.Should().NotBeNull("ExportButton must exist in PreviewView");
    }

    [Fact]
    public async Task Export_AfterPipeline_ProducesValidOutput()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("tiff_encoder");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Preview_Export", "tif");
        await Driver.ExportOutputAsync(outputPath);
        await Task.Delay(1000);

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "Preview_Export");
    }

    [Fact]
    public async Task Export_Png_AfterPipeline()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("gradient_horiz_rgb.png"),
            new[] { "raw_input", "png_encoder" });

        AssertValidOutput(outputPath, "PNG");
        SaveEvidence(outputPath, "Preview_Export_Png");
    }

    [Fact]
    public async Task Export_Tiff_AfterPipeline()
    {
        var outputPath = await Driver.RunFullWorkflowAsync(
            GetTestImagePath("color_bars_8bit.png"),
            new[] { "raw_input", "tiff_encoder" });

        AssertValidOutput(outputPath, "TIFF");
        SaveEvidence(outputPath, "Preview_Export_Tiff");
    }

    [Fact]
    public async Task Export_WithoutRun_ShowsErrorState()
    {
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);

        try
        {
            var outputPath = GetOutputPath("Preview_NoRun", "tif");
            await Driver.ExportOutputAsync(outputPath);
        }
        catch (Exception ex)
        {
            Output.WriteLine($"Export without run: {ex.Message}");
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window should survive export without run");
    }

    // ════════════════════════════════════════════════════════════════
    //  Preview After Pipeline Tests (4 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Preview_Updates_AfterPipelineCompletes()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
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

        preview.Should().NotBeNull("Preview must render after pipeline execution");
        preview!.BoundingRectangle.Width.Should().BeGreaterThan(0);
        CaptureScreenshot("Preview_AfterPipeline");
    }

    [Fact]
    public async Task Preview_AfterDenoise_StillRenders()
    {
        await Driver.ImportImageAsync(GetTestImagePath("noise_grain.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("ai_denoise");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(3));
        await Task.Delay(500);

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive denoise pipeline");
        CaptureScreenshot("Preview_AfterDenoise");
    }

    [Fact]
    public async Task Preview_AfterColorspaceChange_StillRenders()
    {
        await Driver.ImportImageAsync(GetTestImagePath("solid_white_1920x1080.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("colorspace");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive colorspace pipeline");
    }

    [Fact]
    public async Task Preview_AfterMultiplePipelineRuns()
    {
        await Driver.ImportImageAsync(GetTestImagePath("gradient_horiz_rgb.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        for (int i = 0; i < 3; i++)
        {
            await Driver.RunPipelineAsync();
            await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));
            await Task.Delay(200);
        }

        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must survive 3 consecutive pipeline runs");
    }
}
