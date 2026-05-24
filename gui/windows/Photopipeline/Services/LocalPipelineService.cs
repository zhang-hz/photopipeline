using Photopipeline.Models;
using System.Collections.ObjectModel;

namespace Photopipeline.Services;

public sealed class LocalPipelineService : IPipelineService
{
    public Task<PipelineModel> CreatePipelineAsync(string name, string description = "", CancellationToken ct = default)
    {
        return Task.FromResult(new PipelineModel
        {
            Id = Guid.NewGuid().ToString(),
            Name = name,
            Description = description
        });
    }

    public Task<bool> ValidatePipelineAsync(PipelineModel pipeline, CancellationToken ct = default)
    {
        pipeline.IsValid = true;
        pipeline.ValidationError = string.Empty;
        return Task.FromResult(true);
    }

    public Task<bool> ExecutePipelineAsync(PipelineModel pipeline, string imageId, CancellationToken ct = default)
    {
        return Task.FromResult(true);
    }

    public Task<ObservableCollection<PluginInfo>> GetAvailablePluginsAsync(CancellationToken ct = default)
    {
        return Task.FromResult(new ObservableCollection<PluginInfo>());
    }

    public Task UpdateNodeParametersAsync(string nodeId, Dictionary<string, object> parameters, CancellationToken ct = default)
    {
        return Task.CompletedTask;
    }
}
