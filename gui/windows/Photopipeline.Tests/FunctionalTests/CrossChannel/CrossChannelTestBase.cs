using Microsoft.Extensions.Logging;
using Photopipeline.Tests.FunctionalTests.ApiChannel;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.Tests.FunctionalTests.UiChannel;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

/// <summary>
/// Holds the results from executing a test case independently across the three channels
/// (API, UI, Cross). Each channel runs in isolation; exceptions are captured rather than
/// swallowed, so the caller can decide how to aggregate failures.
/// </summary>
public sealed record CrossChannelResult
{
    public string TestName { get; init; } = string.Empty;

    // API channel
    public string? ApiOutputPath { get; init; }
    public Exception? ApiException { get; init; }
    public bool ApiSucceeded => ApiException == null && ApiOutputPath != null;

    // UI channel
    public string? UiOutputPath { get; init; }
    public Exception? UiException { get; init; }
    public int? UiProcessExitCode { get; init; }
    public string? UiProcessStandardOutput { get; init; }
    public string? UiProcessStandardError { get; init; }
    public bool UiSucceeded => UiException == null && UiOutputPath != null;

    // Cross-channel verification (aggregate)
    public bool CrossVerified { get; init; }
    public string? CrossError { get; init; }

    public bool AllChannelsSucceeded => ApiSucceeded && UiSucceeded && CrossVerified;

    public override string ToString()
    {
        return $"CrossChannelResult[{TestName}]: " +
               $"API={(ApiSucceeded ? "PASS" : "FAIL")}, " +
               $"UI={(UiSucceeded ? "PASS" : "FAIL")}, " +
               $"Cross={(CrossVerified ? "PASS" : "FAIL")}";
    }
}

public abstract class CrossChannelTestBase : IDisposable
{
    protected readonly ApiTestBase Api;
    protected readonly ITestOutputHelper _output;
    private readonly string _appPath;
    private readonly string _outputDir;
    protected bool _apiAvailable;
    protected bool _uiAvailable;
    private UiTestDriver? _uiDriver;

    protected CrossChannelTestBase(ITestOutputHelper output)
    {
        _output = output;

        var loggerFactory = Microsoft.Extensions.Logging.LoggerFactory.Create(
            b => b.AddFilter((_, level) => level >= Microsoft.Extensions.Logging.LogLevel.Warning));

        Api = new CrossChannelApiAdapter(loggerFactory);

        // Resolve app path using the same logic as UiTestBase
        var publishDir = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..", "..",
            "Photopipeline", "bin", "x64", "Release", "net9.0-windows", "publish");

        if (!Directory.Exists(publishDir))
        {
            publishDir = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..", "..",
                "Photopipeline", "bin", "x64", "Debug", "net9.0-windows");
        }

        _appPath = Path.Combine(publishDir, "Photopipeline.exe");
        _outputDir = Path.Combine(Path.GetTempPath(), "photopipeline_cross_tests");
        Directory.CreateDirectory(_outputDir);
    }

    /// <summary>
    /// Lazy-initializes the UiTestDriver. Call this only when UI tests are needed.
    /// </summary>
    protected UiTestDriver GetUiDriver()
    {
        if (_uiDriver != null) return _uiDriver;

        if (!File.Exists(_appPath))
            throw new FileNotFoundException(
                $"UI application not found at: {_appPath}. " +
                "Build the UI project before running cross-channel tests.");

        _uiDriver = new UiTestDriver(
            _appPath,
            TestDataCatalog.GetInputDir(),
            _outputDir,
            _output,
            TimeSpan.FromSeconds(60));

        return _uiDriver;
    }

    /// <summary>
    /// Verifies API backend reachability. Throws <see cref="Xunit.Sdk.XunitException"/>
    /// via <see cref="ApiTestBase.RequireBackendAsync"/> if the backend is unavailable.
    /// </summary>
    protected async Task RequireApiBackendAsync(CancellationToken ct = default)
    {
        await Api.RequireBackendAsync(ct);
        _apiAvailable = true;
    }

    /// <summary>
    /// Lightweight connectivity check that throws on failure.
    /// Use <see cref="RequireApiBackendAsync"/> for tests that MUST have a backend.
    /// </summary>
    protected async Task EnsureApiAvailableAsync(CancellationToken ct = default)
    {
        if (_apiAvailable) return;

        await Api.EnsureConnectedAsync(ct);
        _apiAvailable = true;
    }

    /// <summary>
    /// Checks whether the UI application executable exists on disk.
    /// </summary>
    protected bool CheckUiAvailable()
    {
        if (_uiAvailable) return true;
        _uiAvailable = File.Exists(_appPath);
        return _uiAvailable;
    }

    /// <summary>
    /// Executes the pipeline through the API (gRPC) channel.
    /// </summary>
    protected async Task<string> RunApiChannelAsync(
        PipelineSpec pipeline, string inputImageName, string testName, CancellationToken ct = default)
    {
        using var outputMgr = new TestOutputManager(testName);
        var inputPath = TestDataCatalog.Instance.GetPath(inputImageName);
        var outputPath = outputMgr.GetOutputPath($"{testName}_api.tif");
        return await Api.ExecuteAndGetOutput(pipeline, inputPath, outputPath, ct);
    }

    /// <summary>
    /// Executes the pipeline through the UI channel.
    /// Converts the PipelineSpec to plugin IDs for the newer UiTestDriver API.
    /// Exceptions are NEVER silently swallowed. The caller must handle failures.
    /// </summary>
    protected async Task<string> RunUiChannelAsync(
        PipelineSpec pipeline, string inputImageName, string testName,
        string outputFormat = "TIFF", CancellationToken ct = default)
    {
        if (!CheckUiAvailable())
            throw new FileNotFoundException(
                $"UI application not found at: {_appPath}. " +
                "Build the UI project before running cross-channel tests.");

        var uiDriver = GetUiDriver();
        var inputPath = TestDataCatalog.Instance.GetPath(inputImageName);
        var pluginIds = pipeline.Nodes.Select(n => n.PluginId).ToArray();

        if (pluginIds.Length == 0)
            throw new InvalidOperationException("Pipeline has no nodes; cannot run in UI channel.");

        await uiDriver.LaunchAppAsync(ct: ct);
        try
        {
            return await uiDriver.RunFullWorkflowAsync(inputPath, pluginIds, ct: ct);
        }
        finally
        {
            await uiDriver.CloseAppAsync(ct);
        }
    }

    /// <summary>
    /// Runs all three channels independently (API, UI) and performs cross-channel
    /// pixel-equivalence verification. Returns a <see cref="CrossChannelResult"/>
    /// with per-channel success/failure details. No exception is silently swallowed.
    /// </summary>
    protected async Task<CrossChannelResult> VerifyCrossChannelAsync(
        PipelineSpec pipeline, string inputImageName, string testName,
        string outputFormat = "TIFF", CancellationToken ct = default)
    {
        var result = new CrossChannelResult { TestName = testName };

        // ── API channel ──
        try
        {
            result = result with { ApiOutputPath = await RunApiChannelAsync(pipeline, inputImageName, testName, ct) };
        }
        catch (Exception ex)
        {
            result = result with { ApiException = ex };
        }

        // ── UI channel ──
        try
        {
            result = result with { UiOutputPath = await RunUiChannelAsync(pipeline, inputImageName, testName, outputFormat, ct) };
        }
        catch (Exception ex)
        {
            result = result with
            {
                UiException = ex,
                UiProcessStandardError = $"UI channel failed: {ex.GetType().Name}: {ex.Message}"
            };
        }

        // ── Cross-channel verification ──
        if (result.ApiOutputPath != null && result.UiOutputPath != null)
        {
            try
            {
                // Verify pixel equivalence between API and UI output
                CrossChannelVerifier.VerifyEquivalence(result.ApiOutputPath, result.UiOutputPath, testName);

                // Compare output image metadata (dimensions, format) across channels
                VerifyCrossChannelMetadata(result.ApiOutputPath, result.UiOutputPath);

                result = result with { CrossVerified = true };
            }
            catch (Exception ex)
            {
                result = result with { CrossError = ex.Message };
            }
        }
        else
        {
            var missingParts = new List<string>();
            if (result.ApiOutputPath == null) missingParts.Add("API output");
            if (result.UiOutputPath == null) missingParts.Add("UI output");
            result = result with
            {
                CrossError = $"Cannot verify cross-channel equivalence: {string.Join(", ", missingParts)} missing."
            };
        }

        // Assert all channels succeeded — this ensures callers always get the assertion
        result.AllChannelsSucceeded.Should().BeTrue(
            $"all channels must succeed for {testName}: {result}");

        return result;
    }

    /// <summary>
    /// Performs a lightweight integrity check on cross-channel outputs before the
    /// full pixel-level verification. This catches issues like truncated/corrupt files
    /// early, providing a clearer error message than the pixel diffs would give.
    /// The full metadata comparison (EXIF, dimensions, bit depth, format) is handled
    /// by <see cref="CrossChannelVerifier.VerifyEquivalence"/>.
    /// </summary>
    private static void VerifyCrossChannelMetadata(string apiOutputPath, string uiOutputPath)
    {
        // Verify both outputs can be opened as valid bitmaps (not corrupt/truncated)
        using var apiBmp = ImageAssert.LoadBitmap(apiOutputPath);
        using var uiBmp = ImageAssert.LoadBitmap(uiOutputPath);

        apiBmp.Width.Should().BeGreaterThan(0,
            $"API output image at {apiOutputPath} has zero width");
        apiBmp.Height.Should().BeGreaterThan(0,
            $"API output image at {apiOutputPath} has zero height");
        uiBmp.Width.Should().BeGreaterThan(0,
            $"UI output image at {uiOutputPath} has zero width");
        uiBmp.Height.Should().BeGreaterThan(0,
            $"UI output image at {uiOutputPath} has zero height");
    }

    public virtual void Dispose()
    {
        Api.Dispose();
        _uiDriver?.Dispose();
    }

    /// <summary>Adapter to expose ApiTestBase functionality without inheritance.</summary>
    private sealed class CrossChannelApiAdapter : ApiTestBase
    {
        public CrossChannelApiAdapter(Microsoft.Extensions.Logging.ILoggerFactory lf) : base()
        {
            // ApiTestBase creates its own services in constructor
        }
    }
}
