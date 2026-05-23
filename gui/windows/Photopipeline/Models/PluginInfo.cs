using CommunityToolkit.Mvvm.ComponentModel;
using System.Collections.ObjectModel;

namespace Photopipeline.Models;

public sealed partial class PluginInfo : ObservableObject
{
    [ObservableProperty]
    private string _id = string.Empty;

    [ObservableProperty]
    private string _name = string.Empty;

    [ObservableProperty]
    private string _category = string.Empty;

    [ObservableProperty]
    private string _description = string.Empty;

    [ObservableProperty]
    private string _version = "1.0.0";

    [ObservableProperty]
    private int _minInputs = 1;

    [ObservableProperty]
    private int _maxInputs = 1;

    [ObservableProperty]
    private int _outputs = 1;

    [ObservableProperty]
    private bool _supportsBatching = true;

    [ObservableProperty]
    private ObservableCollection<ParameterSchema> _parameterSchemas = new();

    [ObservableProperty]
    private string _iconGlyph = "\uE8B7";
}

public sealed partial class ParameterSchema : ObservableObject
{
    [ObservableProperty]
    private string _name = string.Empty;

    [ObservableProperty]
    private string _displayName = string.Empty;

    [ObservableProperty]
    private string _description = string.Empty;

    [ObservableProperty]
    private ParameterType _parameterType;

    [ObservableProperty]
    private object? _defaultValue;

    [ObservableProperty]
    private object? _minValue;

    [ObservableProperty]
    private object? _maxValue;

    [ObservableProperty]
    private ObservableCollection<object> _enumValues = new();

    [ObservableProperty]
    private bool _isRequired;

    [ObservableProperty]
    private double _step = 1.0;

    [ObservableProperty]
    private string _unit = string.Empty;

    [ObservableProperty]
    private int _decimalPlaces = 2;
}

public enum ParameterType
{
    String,
    Integer,
    Float,
    Boolean,
    Enum,
    Color,
    FilePath,
    DirectoryPath,
    Percentage
}
