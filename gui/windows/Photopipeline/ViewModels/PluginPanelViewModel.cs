using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Photopipeline.Models;
using System.Collections.ObjectModel;

namespace Photopipeline.ViewModels;

public sealed partial class PluginPanelViewModel : ObservableObject
{
    [ObservableProperty]
    private PluginInfo? _selectedPlugin;

    [ObservableProperty]
    private ObservableCollection<ParameterControlViewModel> _parameterControls = new();

    [ObservableProperty]
    private string _searchText = string.Empty;

    [ObservableProperty]
    private ObservableCollection<PluginInfo> _filteredPlugins = new();

    [ObservableProperty]
    private string _selectedCategory = "All";

    [ObservableProperty]
    private ObservableCollection<string> _categories = new();

    private ObservableCollection<PluginInfo> _allPlugins = new();

    public void LoadPlugins(ObservableCollection<PluginInfo> plugins)
    {
        _allPlugins = plugins;
        Categories = new ObservableCollection<string>(
            plugins.Select(p => p.Category).Distinct().Prepend("All"));
        ApplyFilters();
    }

    [RelayCommand]
    private void SelectPlugin(PluginInfo plugin)
    {
        SelectedPlugin = plugin;
        BuildParameterControls(plugin);
    }

    [RelayCommand]
    private void ApplyFilters()
    {
        FilteredPlugins = new ObservableCollection<PluginInfo>(
            _allPlugins.Where(p =>
                (SelectedCategory == "All" || p.Category == SelectedCategory) &&
                (string.IsNullOrEmpty(SearchText) ||
                 p.Name.Contains(SearchText, StringComparison.OrdinalIgnoreCase) ||
                 p.Description.Contains(SearchText, StringComparison.OrdinalIgnoreCase))));
    }

    partial void OnSearchTextChanged(string value) => ApplyFiltersCommand.Execute(null);
    partial void OnSelectedCategoryChanged(string value) => ApplyFiltersCommand.Execute(null);

    private void BuildParameterControls(PluginInfo plugin)
    {
        ParameterControls.Clear();
        foreach (var schema in plugin.ParameterSchemas)
        {
            var control = new ParameterControlViewModel
            {
                Schema = schema,
                Value = schema.DefaultValue
            };
            ParameterControls.Add(control);
        }
    }

    [RelayCommand]
    private void ResetParameters()
    {
        if (SelectedPlugin is null) return;
        foreach (var control in ParameterControls)
        {
            control.Value = control.Schema.DefaultValue;
        }
    }

    [RelayCommand]
    private void ApplyParameters()
    {
    }

    public Dictionary<string, object> GetCurrentParameterValues()
    {
        return ParameterControls.ToDictionary(c => c.Schema.Name, c => c.Value ?? new object());
    }
}

public sealed partial class ParameterControlViewModel : ObservableObject
{
    [ObservableProperty]
    private ParameterSchema _schema = new();

    [ObservableProperty]
    private object? _value;

    [ObservableProperty]
    private string _stringValue = string.Empty;

    [ObservableProperty]
    private double _numericValue;

    [ObservableProperty]
    private bool _boolValue;

    [ObservableProperty]
    private int _selectedEnumIndex;

    [ObservableProperty]
    private bool _isValid = true;

    [ObservableProperty]
    private string _validationMessage = string.Empty;

    partial void OnStringValueChanged(string value)
    {
        if (Schema.ParameterType == ParameterType.String ||
            Schema.ParameterType == ParameterType.FilePath ||
            Schema.ParameterType == ParameterType.DirectoryPath)
        {
            Value = value;
        }
    }

    partial void OnNumericValueChanged(double value)
    {
        if (Schema.ParameterType == ParameterType.Float ||
            Schema.ParameterType == ParameterType.Integer)
        {
            Value = value;
        }
    }

    partial void OnBoolValueChanged(bool value)
    {
        if (Schema.ParameterType == ParameterType.Boolean)
        {
            Value = value;
        }
    }

    partial void OnSelectedEnumIndexChanged(int value)
    {
        if (Schema.ParameterType == ParameterType.Enum &&
            value >= 0 && value < Schema.EnumValues.Count)
        {
            Value = Schema.EnumValues[value];
        }
    }
}
