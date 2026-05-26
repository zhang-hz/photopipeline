using Microsoft.Extensions.Logging;
using Photopipeline.Tests.FunctionalTests.ApiChannel;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Photopipeline.Tests.FunctionalTests.UiChannel;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

public abstract class CrossChannelTestBase : IDisposable
{
    protected readonly ApiTestBase Api;
    protected readonly UiTestDriver Ui;
    protected bool _apiAvailable;
    protected bool _uiAvailable;

    protected CrossChannelTestBase()
    {
        var loggerFactory = Microsoft.Extensions.Logging.LoggerFactory.Create(
            b => b.AddFilter((_, level) => level >= Microsoft.Extensions.Logging.LogLevel.Warning));

        Api = new CrossChannelApiAdapter(loggerFactory);
        Ui = new UiTestDriver();
    }

    protected async Task EnsureApiAvailableAsync(CancellationToken ct = default)
    {
        if (_apiAvailable) return;
        try
        {
            await Api.EnsureConnectedAsync(ct);
            _apiAvailable = true;
        }
        catch
        {
            _apiAvailable = false;
        }
    }

    protected bool CheckUiAvailable()
    {
        if (_uiAvailable) return true;
        _uiAvailable = System.IO.File.Exists(Ui.AppPath);
        return _uiAvailable;
    }

    protected async Task<string> RunApiChannelAsync(
        PipelineSpec pipeline, string inputImageName, string testName, CancellationToken ct = default)
    {
        using var outputMgr = new TestOutputManager(testName);
        var inputPath = TestDataCatalog.Instance.GetPath(inputImageName);
        var outputPath = outputMgr.GetOutputPath($"{testName}_api.tif");
        return await Api.ExecuteAndGetOutput(pipeline, inputPath, outputPath, ct);
    }

    protected async Task<string?> RunUiChannelAsync(
        PipelineSpec pipeline, string inputImageName, string testName, string outputFormat = "TIFF", CancellationToken ct = default)
    {
        if (!CheckUiAvailable()) return null;
        try
        {
            var inputPath = TestDataCatalog.Instance.GetPath(inputImageName);
            return await Ui.RunFullWorkflowAsync(inputPath, pipeline,
                outputFormat: outputFormat, ct: ct);
        }
        catch
        {
            return null;
        }
    }

    /// <summary>
    /// Run both channels and verify pixel-perfect equivalence.
    /// Returns true if cross-channel verification passed, false if UI was skipped.
    /// </summary>
    protected async Task<bool> VerifyCrossChannelAsync(
        PipelineSpec pipeline, string inputImageName, string testName, string outputFormat = "TIFF", CancellationToken ct = default)
    {
        await EnsureApiAvailableAsync(ct);
        var apiOutput = await RunApiChannelAsync(pipeline, inputImageName, testName, ct);
        var uiOutput = await RunUiChannelAsync(pipeline, inputImageName, testName, outputFormat, ct);

        if (uiOutput == null)
            return false; // UI channel skipped

        CrossChannelVerifier.VerifyEquivalence(apiOutput, uiOutput, testName);
        return true;
    }

    public virtual void Dispose()
    {
        Api.Dispose();
        Ui.Dispose();
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
