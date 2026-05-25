using Photopipeline.Models;

namespace Photopipeline.Services;

public interface IBatchService
{
    Task<string> SubmitAsync(BatchSpec spec, CancellationToken ct = default);
    IAsyncEnumerable<BatchProgress> GetProgressAsync(string batchId, CancellationToken ct = default);
    Task CancelAsync(string batchId, CancellationToken ct = default);
}
