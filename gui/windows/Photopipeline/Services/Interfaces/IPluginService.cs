using Photopipeline.Models;

namespace Photopipeline.Services;

public interface IPluginService
{
    Task<IReadOnlyList<PluginInfo>> GetAllAsync(CancellationToken ct = default);
    Task<NodeSchema?> GetSchemaAsync(string pluginId, CancellationToken ct = default);
    IReadOnlyList<string> GetCategories();
    IReadOnlyList<PluginInfo> Search(string query);
    IReadOnlyList<PluginInfo> FilterByCategory(string category);
}
