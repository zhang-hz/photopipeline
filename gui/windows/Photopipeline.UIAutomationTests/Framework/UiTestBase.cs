using FlaUI.Core;
using FlaUI.UIA3;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.Tests.FunctionalTests.UiChannel;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests.Framework;

/// <summary>
/// Abstract base class for all FlaUI-based GUI E2E tests.
///
/// Lifecycle (per test method):
///   InitializeAsync -> LaunchApp -> [test method] -> DisposeAsync -> CloseApp + SaveEvidence
///
/// Iron Rule 2: No silent skipping. If the app fails to launch, the test FAILs immediately.
/// Iron Rule 4: Real WPF process via FlaUI UIA3.
/// </summary>
public abstract class UiTestBase : IAsyncLifetime
{
    protected TestAppFixture Fixture { get; }
    protected ITestOutputHelper Output { get; }
    protected UiTestDriver Driver { get; private set; } = null!;

    /// <summary>Directory for test output files (images, logs). Unique per test run.</summary>
    protected string OutputDir => Fixture.OutputDir;

    /// <summary>Directory for failure screenshots.</summary>
    protected string ScreenshotDir => Fixture.ScreenshotDir;

    /// <summary>Directory for evidence (copies of output images for audit).</summary>
    protected string EvidenceDir => Fixture.EvidenceDir;

    /// <summary>Root directory containing test input images.</summary>
    protected string TestDataDir => Fixture.TestDataDir;

    /// <summary>Lazy-loaded main window reference, shared across test methods.</summary>
    protected Window? _mainWindow;

    protected UiTestBase(TestAppFixture fixture, ITestOutputHelper output)
    {
        Fixture = fixture;
        Output = output;
    }

    // ── IAsyncLifetime: per-test-method lifecycle ──

    public virtual async Task InitializeAsync()
    {
        Driver = Fixture.CreateDriver(Output);
        try
        {
            await Driver.LaunchAppAsync();
        }
        catch (Exception ex)
        {
            Output.WriteLine($"FATAL: App launch failed: {ex}");
            throw;
        }
    }

    public virtual async Task DisposeAsync()
    {
        try
        {
            if (Driver != null!)
                await Driver.CloseAppAsync();
        }
        catch (Exception ex)
        {
            Output.WriteLine($"Cleanup warning: {ex.Message}");
        }
        finally
        {
            Driver?.Dispose();
        }
    }

    // ── Convenience helpers ──

    protected string GetTestImagePath(string filename)
    {
        var fullPath = Path.Combine(TestDataDir, filename);
        if (!File.Exists(fullPath))
        {
            GenerateTestImagesIfNeeded();
            fullPath = Path.Combine(TestDataDir, filename);
        }
        if (!File.Exists(fullPath))
            throw new FileNotFoundException(
                $"Test image '{filename}' not found in {TestDataDir}. " +
                "Ensure TestImageGenerator has populated the directory.");
        return fullPath;
    }

    protected string GetOutputPath(string testName, string extension = "tif")
    {
        var safeName = testName.Replace(' ', '_').Replace('.', '_');
        var fileName = $"{safeName}_{DateTime.Now:yyyyMMdd_HHmmss}.{extension.TrimStart('.')}";
        return Path.Combine(OutputDir, fileName);
    }

    protected async Task<string> RunStandardWorkflowAsync(
        string inputImageFilename,
        string[] pluginIds,
        Dictionary<string, Dictionary<string, string>>? nodeParams = null,
        string? outputFormat = null,
        CancellationToken ct = default)
    {
        var inputPath = GetTestImagePath(inputImageFilename);
        return await Driver.RunFullWorkflowAsync(
            inputPath, pluginIds, nodeParams, outputFormat, ct);
    }

    // ── Evidence collection ──

    /// <summary>
    /// Copies the output file to the evidence directory for audit trail.
    /// </summary>
    protected void SaveEvidence(string outputPath, string testName)
    {
        if (!File.Exists(outputPath)) return;
        var evidencePath = Path.Combine(EvidenceDir, $"{testName}_{Path.GetFileName(outputPath)}");
        Directory.CreateDirectory(EvidenceDir);
        File.Copy(outputPath, evidencePath, overwrite: true);
        Output.WriteLine($"Evidence saved: {evidencePath}");
    }

    /// <summary>
    /// Takes a screenshot and saves it to the evidence directory.
    /// </summary>
    protected void CaptureScreenshot(string label)
    {
        try
        {
            var screenshotPath = Path.Combine(ScreenshotDir, $"{label}_{DateTime.Now:yyyyMMdd_HHmmss}.png");
            Directory.CreateDirectory(ScreenshotDir);
            UiElementLocator.CaptureScreenshot(_mainWindow!, screenshotPath);
            Output.WriteLine($"Screenshot: {screenshotPath}");
        }
        catch (Exception ex)
        {
            Output.WriteLine($"Screenshot failed: {ex.Message}");
        }
    }

    // ── Shared MainWindow access (eliminates duplication) ──

    /// <summary>
    /// Returns the current main window, lazy-loading the UIA3 automation.
    /// </summary>
    protected Window GetMainWindow()
    {
        if (_mainWindow != null) return _mainWindow;
        var automation = new UIA3Automation();
        var desktop = automation.GetDesktop();
        var element = desktop.FindFirstChild(cf => cf.ByClassName("Window"))
            ?? throw new InvalidOperationException("Main window not found");
        _mainWindow = element.AsWindow();
        return _mainWindow;
    }

    /// <summary>
    /// Verified output validation: checks file exists, non-empty, valid format.
    /// </summary>
    protected void AssertValidOutput(string outputPath, string expectedFormat = "TIFF")
    {
        File.Exists(outputPath).Should().BeTrue($"Output file must exist: {outputPath}");
        new FileInfo(outputPath).Length.Should().BeGreaterThan(0, "Output file must not be empty");
        ImageAssert.IsValidFormat(outputPath, expectedFormat);
    }

    // ── Test data generation (idempotent, lazy) ──

    private static readonly object _generateLock = new();
    private static bool _imagesGenerated;

    private void GenerateTestImagesIfNeeded()
    {
        if (_imagesGenerated) return;
        lock (_generateLock)
        {
            if (_imagesGenerated) return;
            TestImageGenerator.GenerateAll(TestDataDir);
            _imagesGenerated = true;
            Output.WriteLine($"Generated test images in {TestDataDir}");
        }
    }
}
