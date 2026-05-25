using Microsoft.Extensions.Logging;
using Photopipeline.Models;

namespace Photopipeline.Services;

public sealed class BatchService : IBatchService
{
    private readonly GrpcClientService _grpc;
    private readonly ILogger<BatchService> _logger;

    public BatchService(GrpcClientService grpc, ILogger<BatchService> logger)
    {
        _grpc = grpc;
        _logger = logger;
    }

    public async Task<string> SubmitAsync(BatchSpec spec, CancellationToken ct = default)
    {
        var protoSpec = ToProtoSpec(spec);
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Batch.BatchService.BatchServiceClient(channel);
        var result = await client.SubmitBatchAsync(protoSpec, cancellationToken: ct);
        _logger.LogInformation("Batch submitted: {Id}", result.Id);
        return result.Id;
    }

    public async IAsyncEnumerable<BatchProgress> GetProgressAsync(
        string batchId,
        [System.Runtime.CompilerServices.EnumeratorCancellation] CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Batch.BatchService.BatchServiceClient(channel);
        var request = new global::Photopipeline.Batch.BatchId { Id = batchId };

        using var call = client.GetProgress(request, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            var p = call.ResponseStream.Current;
            yield return new BatchProgress
            {
                Status = p.Status switch
                {
                    global::Photopipeline.Batch.BatchProgress.Types.Status.Pending => BatchStatus.Pending,
                    global::Photopipeline.Batch.BatchProgress.Types.Status.Running => BatchStatus.Running,
                    global::Photopipeline.Batch.BatchProgress.Types.Status.Done => BatchStatus.Done,
                    global::Photopipeline.Batch.BatchProgress.Types.Status.Canceled => BatchStatus.Canceled,
                    global::Photopipeline.Batch.BatchProgress.Types.Status.Error => BatchStatus.Error,
                    _ => BatchStatus.Error
                },
                TotalFiles = p.TotalFiles,
                CompletedFiles = p.CompletedFiles,
                FailedFiles = p.FailedFiles,
                CurrentFile = p.CurrentFile,
                Fraction = p.Fraction,
                ProgressDetails = p.ProgressDetails
            };
        }
    }

    public async Task CancelAsync(string batchId, CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Batch.BatchService.BatchServiceClient(channel);
        await client.CancelAsync(
            new global::Photopipeline.Batch.BatchId { Id = batchId },
            cancellationToken: ct);
        _logger.LogInformation("Batch canceled: {Id}", batchId);
    }

    private static global::Photopipeline.Batch.BatchSpec ToProtoSpec(BatchSpec spec)
    {
        return new global::Photopipeline.Batch.BatchSpec
        {
            PipelineConfigPath = spec.PipelineConfigPath,
            FilePattern = spec.FilePattern,
            OutputDir = spec.OutputDir,
            Parallel = spec.Parallel,
            Resume = spec.Resume
        };
    }
}
