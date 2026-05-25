using Photopipeline.Models;

namespace Photopipeline.Services;

public interface IPipelineService
{
    Task<string> CreatePipelineAsync(PipelineSpec spec, CancellationToken ct = default);
    Task<ValidationResult> ValidateAsync(PipelineSpec spec, CancellationToken ct = default);
    IAsyncEnumerable<ExecuteProgress> ExecuteAsync(string pipelineId, string imagePath, string outputPath, CancellationToken ct = default);
    Task<NodeSchema> GetNodeSchemaAsync(string pluginId, CancellationToken ct = default);
}
