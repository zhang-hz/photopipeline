using CommunityToolkit.Mvvm.ComponentModel;
using System.Collections.ObjectModel;

namespace Photopipeline.Models;

public sealed partial class PipelineModel : ObservableObject
{
    [ObservableProperty]
    private string _id = Guid.NewGuid().ToString("N");

    [ObservableProperty]
    private string _name = "New Pipeline";

    [ObservableProperty]
    private string _description = string.Empty;

    [ObservableProperty]
    private ObservableCollection<PipelineNode> _nodes = new();

    [ObservableProperty]
    private ObservableCollection<PipelineEdge> _edges = new();

    [ObservableProperty]
    private bool _isValid;

    [ObservableProperty]
    private bool _isExecuting;

    [ObservableProperty]
    private string _validationError = string.Empty;
}

public sealed partial class PipelineNode : ObservableObject
{
    [ObservableProperty]
    private string _id = Guid.NewGuid().ToString("N");

    [ObservableProperty]
    private string _pluginId = string.Empty;

    [ObservableProperty]
    private string _displayName = string.Empty;

    [ObservableProperty]
    private double _canvasX;

    [ObservableProperty]
    private double _canvasY;

    [ObservableProperty]
    private double _width = 160;

    [ObservableProperty]
    private double _height = 80;

    [ObservableProperty]
    private bool _isSelected;

    [ObservableProperty]
    private bool _isDragging;

    [ObservableProperty]
    private bool _isProcessing;

    [ObservableProperty]
    private Dictionary<string, object> _parameters = new();

    [ObservableProperty]
    private ObservableCollection<Port> _inputPorts = new();

    [ObservableProperty]
    private ObservableCollection<Port> _outputPorts = new();

    public PipelineNode()
    {
        _inputPorts.Add(new Port { Id = "in", Name = "Input", Direction = PortDirection.Input, ParentNodeId = _id });
        _outputPorts.Add(new Port { Id = "out", Name = "Output", Direction = PortDirection.Output, ParentNodeId = _id });
    }

    partial void OnIdChanged(string value)
    {
        foreach (var port in InputPorts)
            port.ParentNodeId = value;
        foreach (var port in OutputPorts)
            port.ParentNodeId = value;
    }
}

public sealed partial class Port : ObservableObject
{
    [ObservableProperty]
    private string _id = string.Empty;

    [ObservableProperty]
    private string _name = string.Empty;

    [ObservableProperty]
    private PortDirection _direction;

    [ObservableProperty]
    private string _parentNodeId = string.Empty;

    [ObservableProperty]
    private double _relativeX;

    [ObservableProperty]
    private double _relativeY;

    [ObservableProperty]
    private bool _isConnected;
}

public sealed partial class PipelineEdge : ObservableObject
{
    [ObservableProperty]
    private string _id = Guid.NewGuid().ToString("N");

    [ObservableProperty]
    private string _sourceNodeId = string.Empty;

    [ObservableProperty]
    private string _sourcePortId = string.Empty;

    [ObservableProperty]
    private string _targetNodeId = string.Empty;

    [ObservableProperty]
    private string _targetPortId = string.Empty;
}

public enum PortDirection
{
    Input,
    Output
}
