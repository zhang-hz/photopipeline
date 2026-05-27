using Microsoft.Extensions.Logging;
using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public abstract class ApiTestBase : IDisposable
{
    protected readonly GrpcClientService GrpcClient;
    protected readonly ImageService ImageService;
    protected readonly PipelineService PipelineService;
    protected readonly PluginService PluginService;
    protected readonly BatchService BatchService;
    protected readonly ITestOutputHelper? _output;
    protected bool _isConnected;

    protected ApiTestBase()
    {
        var address = Environment.GetEnvironmentVariable("PHOTOPIPELINE_API_ADDRESS") ?? "http://localhost:50051";
        var loggerFactory = LoggerFactory.Create(b => b.AddFilter((_, level) => level >= LogLevel.Warning));

        GrpcClient = new GrpcClientService(address);
        ImageService = new ImageService(GrpcClient,
            loggerFactory.CreateLogger<ImageService>());
        PipelineService = new PipelineService(GrpcClient,
            loggerFactory.CreateLogger<PipelineService>());
        PluginService = new PluginService();
        BatchService = new BatchService(GrpcClient,
            loggerFactory.CreateLogger<BatchService>());
    }

    protected ApiTestBase(ITestOutputHelper output) : this()
    {
        _output = output;
    }

    internal async Task EnsureConnectedAsync(CancellationToken ct = default)
    {
        if (!_isConnected)
        {
            await GrpcClient.ConnectAsync(ct);
            _isConnected = true;
        }
    }

    /// <summary>
    /// Connects to the gRPC backend and fails the test with Assert.Fail if the backend
    /// is unavailable. This MUST be used instead of try/catch/return patterns that
    /// silently skip tests.
    /// </summary>
    public async Task RequireBackendAsync(CancellationToken ct = default)
    {
        if (_isConnected) return;

        try
        {
            await GrpcClient.ConnectAsync(ct);
            _isConnected = true;
            _output?.WriteLine("Backend gRPC connected successfully.");
        }
        catch (Exception ex)
        {
            string diagnosticInfo = $"Address={GrpcClient.Address ?? "unknown"}, " +
                                    $"ExceptionType={ex.GetType().Name}";
            _output?.WriteLine($"Backend gRPC connection FAILED: {diagnosticInfo}");

            Assert.Fail(
                $"Backend gRPC unavailable: {ex.Message}\n" +
                $"Diagnostic: {diagnosticInfo}\n" +
                $"Inner: {ex.InnerException?.Message ?? "(none)"}\n" +
                $"Stack: {ex.StackTrace}");
        }
    }

    protected async Task<PipelineSpec> CreateAndValidatePipeline(
        PipelineSpec spec, CancellationToken ct = default)
    {
        await EnsureConnectedAsync(ct);
        var pipelineId = await PipelineService.CreatePipelineAsync(spec, ct);
        if (string.IsNullOrEmpty(pipelineId))
            throw new InvalidOperationException("Failed to create pipeline");
        return spec;
    }

    public async Task<string> ExecuteAndGetOutput(
        PipelineSpec spec, string inputImagePath, string outputPath, CancellationToken ct = default)
    {
        await EnsureConnectedAsync(ct);
        var pipelineId = await PipelineService.CreatePipelineAsync(spec, ct);

        await foreach (var _ in PipelineService.ExecuteAsync(pipelineId, inputImagePath, outputPath, ct))
        {
            ct.ThrowIfCancellationRequested();
        }

        if (!File.Exists(outputPath))
            throw new FileNotFoundException($"Pipeline output not found: {outputPath}");

        return outputPath;
    }

    /// <summary>
    /// Executes a pipeline and returns the output path. If <paramref name="spec"/> is null,
    /// creates an identity pipeline with a single exposure node (ev=0).
    /// </summary>
    protected async Task<string> ExecuteOrIdentity(
        PipelineSpec? spec, string inputPath, string outputPath, CancellationToken ct = default)
    {
        var pipeline = spec ?? TestPipelineBuilder.SingleNode("photopipeline.plugins.transform");
        return await ExecuteAndGetOutput(pipeline, inputPath, outputPath, ct);
    }

    /// <summary>
    /// Executes the same pipeline twice and verifies determinism: both outputs must be
    /// pixel-identical. Returns the first output path.
    /// </summary>
    protected async Task<string> ExecuteTwiceAndAssertDeterministic(
        PipelineSpec spec, string inputPath, string outputPath, TestOutputManager outputMgr, CancellationToken ct = default)
    {
        await ExecuteAndGetOutput(spec, inputPath, outputPath, ct);

        var outputPath2 = outputMgr.GetOutputPath(Path.GetFileName(outputPath) + "_run2.png");
        await ExecuteAndGetOutput(spec, inputPath, outputPath2, ct);

        ImageAssert.PixelsEqual(outputPath, outputPath2, tolerancePerChannel: 0);

        _output?.WriteLine($"Determinism verified: two runs produce pixel-identical output");
        return outputPath;
    }

    /// <summary>
    /// Verifies that the output is a valid image of the expected format and size.
    /// Does NOT compare against golden images.
    /// </summary>
    protected static void AssertValidOutput(
        string outputPath, string expectedFormat, int? expectedWidth = null, int? expectedHeight = null, int? expectedBitDepth = null)
    {
        ImageAssert.IsValidFormat(outputPath, expectedFormat, expectedWidth, expectedHeight, expectedBitDepth);
        var fi = new FileInfo(outputPath);
        if (fi.Length == 0)
            throw new Xunit.Sdk.XunitException($"Output file is empty: {outputPath}");
    }

    public virtual void Dispose()
    {
        GrpcClient.Dispose();
        _isConnected = false;
    }
}
