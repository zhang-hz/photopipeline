using Photopipeline.UIAutomationTests.Framework;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests;

/// <summary>
/// Startup and window smoke tests (10 tests).
/// Covers app launch, window visibility, all panels present,
/// version info, and quick startup checks.
///
/// Iron Rule 1: Each test has at least one FAIL-able assertion.
/// Iron Rule 2: No silent skipping -- missing elements throw exceptions.
/// Iron Rule 4: Real WPF process via FlaUI UIA3.
/// </summary>
[Collection("FlaUITests")]
public sealed class StartupSmokeTests : UiTestBase
{
    public StartupSmokeTests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    // ════════════════════════════════════════════════════════════════
    //  Application Launch Tests (4 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Smoke_App_Launches_AndShowsMainWindow()
    {
        var window = GetMainWindow();
        window.Should().NotBeNull("Main window must exist after successful launch");
        window.IsAvailable.Should().BeTrue("Main window must be available");
        window.IsEnabled.Should().BeTrue("Main window must be enabled");
        CaptureScreenshot("Smoke_AppLaunched");
    }

    [Fact]
    public async Task Smoke_Window_HasCorrectTitle()
    {
        var window = GetMainWindow();
        window.Title.Should().NotBeNull("Window must have a title");
        window.Title.Should().Contain("Photopipeline", "Window title should contain the application name");
        Output.WriteLine($"Window title: '{window.Title}'");
    }

    [Fact]
    public async Task Smoke_Window_HasReasonableSize()
    {
        var window = GetMainWindow();
        var bounds = window.BoundingRectangle;
        bounds.Width.Should().BeGreaterThan(800, "Window should have a reasonable width (>800px)");
        bounds.Height.Should().BeGreaterThan(500, "Window should have a reasonable height (>500px)");
        Output.WriteLine($"Window size: {bounds.Width}x{bounds.Height}");
    }

    [Fact]
    public async Task Smoke_App_StaysAlive_After5Seconds()
    {
        var window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must be available immediately");

        await Task.Delay(5000);

        window = GetMainWindow();
        window.IsAvailable.Should().BeTrue("Window must stay alive after 5 seconds of idle");
    }

    // ════════════════════════════════════════════════════════════════
    //  Panel Presence Tests (3 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Smoke_AllThreePanels_Present()
    {
        var window = GetMainWindow();

        var filmstripListBox = window.FindFirstDescendant(cf => cf.ByAutomationId("FilmstripListBox"));
        var pluginBrowserList = window.FindFirstDescendant(cf => cf.ByAutomationId("PluginBrowserList"));
        var pipelineCanvas = window.FindFirstDescendant(cf => cf.ByAutomationId("PipelineCanvas"));

        filmstripListBox.Should().NotBeNull("Filmstrip panel must be present on startup");
        pluginBrowserList.Should().NotBeNull("Plugin browser panel must be present on startup");
        pipelineCanvas.Should().NotBeNull("Pipeline editor panel must be present on startup");
        CaptureScreenshot("Smoke_ThreePanels");
    }

    [Fact]
    public async Task Smoke_Filmstrip_ImportButton_Exists()
    {
        var window = GetMainWindow();
        var importBtn = window.FindFirstDescendant(cf => cf.ByAutomationId("ImportButton"))
            ?? window.FindFirstDescendant(cf =>
                cf.ByControlType(ControlType.Button).And(cf.ByName("Import")));

        importBtn.Should().NotBeNull("Import button must exist in FilmstripView on startup");
        importBtn!.IsEnabled.Should().BeTrue("Import button should be enabled on startup");
    }

    [Fact]
    public async Task Smoke_PluginBrowser_AddButton_Exists()
    {
        var window = GetMainWindow();
        var addBtn = window.FindFirstDescendant(cf => cf.ByAutomationId("AddToPipelineButton"))
            ?? window.FindFirstDescendant(cf =>
                cf.ByControlType(ControlType.Button).And(cf.ByName("Add")));

        addBtn.Should().NotBeNull("Add to Pipeline button must exist in PluginBrowserView");
        Output.WriteLine($"AddToPipeline found, enabled: {addBtn!.IsEnabled}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Status & Version Tests (2 tests)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Smoke_StatusBar_ShowsReadyState()
    {
        await Task.Delay(2000); // Wait for initialization

        var window = GetMainWindow();
        var hasReady = await Task.Run(() =>
        {
            var allText = window.FindAllDescendants(cf =>
                cf.ByControlType(ControlType.Text)
                    .Or(cf.ByControlType(ControlType.StatusBar)));

            return allText.Any(t =>
                (t.Name ?? "").Contains("Ready", StringComparison.OrdinalIgnoreCase));
        });

        hasReady.Should().BeTrue("Status bar should display 'Ready' state after initialization");
    }

    [Fact]
    public async Task Smoke_BackendStatus_ShowsConnection()
    {
        var window = GetMainWindow();
        var statusElements = (await Task.Run(() =>
        {
            var allText = window.FindAllDescendants(cf =>
                cf.ByControlType(ControlType.Text)
                    .Or(cf.ByControlType(ControlType.StatusBar)));

            return allText.Where(t =>
                (t.Name ?? "").Contains("Connected", StringComparison.OrdinalIgnoreCase)
                || (t.Name ?? "").Contains("Disconnected", StringComparison.OrdinalIgnoreCase)
                || (t.Name ?? "").Contains("Backend", StringComparison.OrdinalIgnoreCase)
                || (t.Name ?? "").Contains("Ready", StringComparison.OrdinalIgnoreCase));
        })).ToList();

        statusElements.Should().NotBeEmpty(
            "Window should display some form of backend connection status or ready state");
        Output.WriteLine($"Status elements found: {string.Join("; ", statusElements.Select(s => s.Name))}");
    }

    // ════════════════════════════════════════════════════════════════
    //  Quick Workflow Check (1 test)
    // ════════════════════════════════════════════════════════════════

    [Fact]
    public async Task Smoke_QuickWorkflow_ImportAndRun()
    {
        // Verify the full quick workflow: import -> select -> add plugin -> run -> export
        await Driver.ImportImageAsync(GetTestImagePath("pure_red_small.png"));
        await Driver.SelectImageAsync(0);
        await Driver.NavigateToPipelineEditorAsync();
        await Driver.AddPluginToPipelineAsync("raw_input");
        await Driver.AddPluginToPipelineAsync("png_encoder");

        await Driver.RunPipelineAsync();
        await Driver.WaitForPipelineCompletionAsync(TimeSpan.FromMinutes(2));

        var outputPath = GetOutputPath("Smoke_QuickWorkflow", "png");
        await Driver.ExportOutputAsync(outputPath);

        File.Exists(outputPath).Should().BeTrue("Quick workflow must produce output file");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "Output file must not be empty");
        SaveEvidence(outputPath, "Smoke_QuickWorkflow");
        CaptureScreenshot("Smoke_QuickWorkflow_Complete");
    }
}
