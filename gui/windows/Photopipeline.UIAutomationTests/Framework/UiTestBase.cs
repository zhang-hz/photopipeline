using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.Tests.FunctionalTests.UiChannel;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests.Framework;

/// <summary>
/// Abstract base class for all FlaUI-based GUI E2E tests.
///
/// Lifecycle (per test method):
///   InitializeAsync -> LaunchApp -> [test method] -> DisposeAsync -> CloseApp
///
/// Iron Rule 2: No silent skipping. If the app fails to launch, the test FAILs immediately.
/// Iron Rule 4: Real WPF process via FlaUI UIA3.
/// </summary>
public abstract class UiTestBase : IAsyncLifetime
{
    protected TestAppFixture Fixture { get; }
    protected ITestOutputHelper Output { get; }
    protected UiTestDriver Driver { get; private set; } = null!;

    /// <summary>
    /// Directory for test output files (images, logs). Unique per test run.
    /// </summary>
    protected string OutputDir => Fixture.OutputDir;

    /// <summary>
    /// Directory for failure screenshots.
    /// </summary>
    protected string ScreenshotDir => Fixture.ScreenshotDir;

    /// <summary>
    /// Root directory containing test input images.
    /// </summary>
    protected string TestDataDir => Fixture.TestDataDir;

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
            // Iron Rule 2: No silent skipping — rethrow immediately
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

    /// <summary>
    /// Returns the full path to a test input image by its filename.
    /// </summary>
    protected string GetTestImagePath(string filename)
    {
        var fullPath = Path.Combine(TestDataDir, filename);
        if (!File.Exists(fullPath))
        {
            // Auto-generate test images on first use
            GenerateTestImagesIfNeeded();
            fullPath = Path.Combine(TestDataDir, filename);
        }
        if (!File.Exists(fullPath))
            throw new FileNotFoundException(
                $"Test image '{filename}' not found in {TestDataDir}. " +
                "Ensure TestImageGenerator has populated the directory.");
        return fullPath;
    }

    /// <summary>
    /// Returns an output path unique to this test.
    /// Format: OutputDir/{testName}_output.{extension}
    /// </summary>
    protected string GetOutputPath(string testName, string extension = "tif")
    {
        var safeName = testName.Replace(' ', '_').Replace('.', '_');
        var fileName = $"{safeName}_{DateTime.Now:yyyyMMdd_HHmmss}.{extension.TrimStart('.')}";
        return Path.Combine(OutputDir, fileName);
    }

    /// <summary>
    /// Runs the standard 11-step workflow and returns the output file path.
    /// This is the "happy path" for single-pipeline tests.
    /// </summary>
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
