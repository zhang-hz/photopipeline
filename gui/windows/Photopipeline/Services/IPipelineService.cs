using Photopipeline.Models;
using System.Collections.ObjectModel;

namespace Photopipeline.Services;

public interface IPipelineService
{
    Task<PipelineModel> CreatePipelineAsync(string name, string description = "", CancellationToken ct = default);
    Task<bool> ValidatePipelineAsync(PipelineModel pipeline, CancellationToken ct = default);
    Task<bool> ExecutePipelineAsync(PipelineModel pipeline, string imageId, CancellationToken ct = default);
    Task<ObservableCollection<PluginInfo>> GetAvailablePluginsAsync(CancellationToken ct = default);
    Task UpdateNodeParametersAsync(string nodeId, Dictionary<string, object> parameters, CancellationToken ct = default);
}
