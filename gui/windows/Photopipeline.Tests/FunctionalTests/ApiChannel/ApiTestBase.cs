using Microsoft.Extensions.Logging;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public abstract class ApiTestBase : IDisposable
{
    protected readonly GrpcClientService GrpcClient;
    protected readonly ImageService ImageService;
    protected readonly PipelineService PipelineService;
    protected readonly PluginService PluginService;
    protected readonly BatchService BatchService;
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

    public async Task EnsureConnectedAsync(CancellationToken ct = default)
    {
        if (!_isConnected)
        {
            await GrpcClient.ConnectAsync(ct);
            _isConnected = true;
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

    public virtual void Dispose()
    {
        GrpcClient.Dispose();
        _isConnected = false;
    }
}
