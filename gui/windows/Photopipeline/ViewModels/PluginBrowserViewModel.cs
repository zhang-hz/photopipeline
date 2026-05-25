using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using System.Collections.ObjectModel;

namespace Photopipeline.ViewModels;

public sealed partial class PluginBrowserViewModel : ViewModelBase
{
    private readonly IPluginService _pluginService;

    [ObservableProperty] private ObservableCollection<PluginInfo> _plugins = new();
    [ObservableProperty] private ObservableCollection<PluginInfo> _filteredPlugins = new();
    [ObservableProperty] private ObservableCollection<string> _categories = new();
    [ObservableProperty] private PluginInfo? _selectedPlugin;
    [ObservableProperty] private string _searchText = string.Empty;
    [ObservableProperty] private string _selectedCategory = "All";
    [ObservableProperty] private Dictionary<string, object> _currentParameters = new();

    public event Action<PluginInfo>? PluginAdded;

    public PluginBrowserViewModel(ILogger<PluginBrowserViewModel> logger, IPluginService pluginService)
        : base(logger)
    {
        _pluginService = pluginService;
        _ = LoadPluginsAsync();
    }

    partial void OnSearchTextChanged(string value) => ApplyFilters();
    partial void OnSelectedCategoryChanged(string value) => ApplyFilters();

    public async Task LoadPluginsAsync()
    {
        try
        {
            var plugins = await _pluginService.GetAllAsync();
            Plugins = new ObservableCollection<PluginInfo>(plugins);
            FilteredPlugins = new ObservableCollection<PluginInfo>(plugins);
            Categories = new ObservableCollection<string>(
                new[] { "All" }.Concat(_pluginService.GetCategories()));
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Failed to load plugins");
        }
    }

    [RelayCommand]
    private void SelectPlugin(PluginInfo plugin)
    {
        SelectedPlugin = plugin;
        CurrentParameters = new Dictionary<string, object>(plugin.ParameterSchema
            .ToDictionary(kv => kv.Key, kv => ExtractDefault(kv.Value)));
    }

    [RelayCommand]
    private void AddToPipeline(PluginInfo plugin) => PluginAdded?.Invoke(plugin);

    [RelayCommand]
    private void ApplyFilters()
    {
        var results = SelectedCategory == "All"
            ? Plugins.AsEnumerable()
            : _pluginService.FilterByCategory(SelectedCategory);

        if (!string.IsNullOrWhiteSpace(SearchText))
        {
            var term = SearchText;
            results = results.Where(p =>
                p.Name.Contains(term, StringComparison.OrdinalIgnoreCase) ||
                p.Description.Contains(term, StringComparison.OrdinalIgnoreCase));
        }

        FilteredPlugins = new ObservableCollection<PluginInfo>(results);
    }

    private static object ExtractDefault(object schemaObj)
    {
        if (schemaObj is Dictionary<string, object> schema && schema.TryGetValue("default", out var dv))
            return dv;
        return new object();
    }
}
