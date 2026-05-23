using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Photopipeline.Models;
using System.Collections.ObjectModel;
using System.Windows.Input;

namespace Photopipeline.ViewModels;

public sealed partial class PipelineEditorViewModel : ObservableObject
{
    [ObservableProperty]
    private ObservableCollection<PipelineNode> _nodes = new();

    [ObservableProperty]
    private ObservableCollection<PipelineEdge> _edges = new();

    [ObservableProperty]
    private PipelineNode? _selectedNode;

    [ObservableProperty]
    private PipelineEdge? _selectedEdge;

    [ObservableProperty]
    private double _canvasWidth = 2000;

    [ObservableProperty]
    private double _canvasHeight = 2000;

    [ObservableProperty]
    private double _offsetX;

    [ObservableProperty]
    private double _offsetY;

    [ObservableProperty]
    private double _scale = 1.0;

    [ObservableProperty]
    private Port? _draggingPort;

    [ObservableProperty]
    private double _connectionLineX1;

    [ObservableProperty]
    private double _connectionLineY1;

    [ObservableProperty]
    private double _connectionLineX2;

    [ObservableProperty]
    private double _connectionLineY2;

    [ObservableProperty]
    private bool _isDrawingConnection;

    [ObservableProperty]
    private bool _isDraggingNode;

    private double _lastMouseX;
    private double _lastMouseY;

    [RelayCommand]
    private void AddNode(PluginInfo plugin)
    {
        var node = new PipelineNode
        {
            PluginId = plugin.Id,
            DisplayName = plugin.Name,
            CanvasX = 100 + (_nodes.Count * 220) % 1500,
            CanvasY = 80 + (_nodes.Count / 7) * 180
        };

        foreach (var schema in plugin.ParameterSchemas)
        {
            node.Parameters[schema.Name] = schema.DefaultValue ?? new object();
        }

        if (plugin.MinInputs > 0)
        {
            node.InputPorts.Clear();
            for (int i = 0; i < plugin.MaxInputs; i++)
            {
                node.InputPorts.Add(new Port
                {
                    Id = $"in{i}",
                    Name = plugin.MaxInputs > 1 ? $"Input {i + 1}" : "Input",
                    Direction = PortDirection.Input,
                    ParentNodeId = node.Id
                });
            }
        }

        if (plugin.Outputs > 0)
        {
            node.OutputPorts.Clear();
            for (int i = 0; i < plugin.Outputs; i++)
            {
                node.OutputPorts.Add(new Port
                {
                    Id = $"out{i}",
                    Name = plugin.Outputs > 1 ? $"Output {i + 1}" : "Output",
                    Direction = PortDirection.Output,
                    ParentNodeId = node.Id
                });
            }
        }

        Nodes.Add(node);
    }

    [RelayCommand]
    private void RemoveNode(PipelineNode? node)
    {
        if (node is null) return;
        var edgesToRemove = Edges.Where(e => e.SourceNodeId == node.Id || e.TargetNodeId == node.Id).ToList();
        foreach (var edge in edgesToRemove)
            Edges.Remove(edge);
        Nodes.Remove(node);
        if (SelectedNode == node) SelectedNode = null;
    }

    [RelayCommand]
    private void SelectNode(PipelineNode node)
    {
        if (SelectedNode is not null)
            SelectedNode.IsSelected = false;
        SelectedNode = node;
        node.IsSelected = true;
    }

    [RelayCommand]
    private void ClearSelection()
    {
        if (SelectedNode is not null)
        {
            SelectedNode.IsSelected = false;
            SelectedNode = null;
        }
    }

    [RelayCommand]
    private void ConnectPorts()
    {
        if (_draggingPort is null) return;

        if (SelectedNode is not null && _draggingPort.ParentNodeId != SelectedNode.Id)
        {
            var sourceNodeId = _draggingPort.Direction == PortDirection.Output
                ? _draggingPort.ParentNodeId : SelectedNode.Id;
            var sourcePortId = _draggingPort.Direction == PortDirection.Output
                ? _draggingPort.Id : "in0";
            var targetNodeId = _draggingPort.Direction == PortDirection.Input
                ? _draggingPort.ParentNodeId : SelectedNode.Id;
            var targetPortId = _draggingPort.Direction == PortDirection.Input
                ? _draggingPort.Id : "out0";

            var existing = Edges.FirstOrDefault(e =>
                e.SourceNodeId == sourceNodeId && e.TargetNodeId == targetNodeId);
            if (existing is null)
            {
                Edges.Add(new PipelineEdge
                {
                    SourceNodeId = sourceNodeId,
                    SourcePortId = sourcePortId,
                    TargetNodeId = targetNodeId,
                    TargetPortId = targetPortId
                });
            }
        }

        _draggingPort = null;
        IsDrawingConnection = false;
    }

    public void OnNodeMouseDown(PipelineNode node, double x, double y)
    {
        _lastMouseX = x;
        _lastMouseY = y;
        SelectNodeCommand.Execute(node);
        node.IsDragging = true;
        IsDraggingNode = true;
    }

    public void OnNodeMouseMove(double x, double y)
    {
        if (!IsDraggingNode || SelectedNode is null) return;
        var dx = (x - _lastMouseX) / Scale;
        var dy = (y - _lastMouseY) / Scale;
        SelectedNode.CanvasX += dx;
        SelectedNode.CanvasY += dy;
        _lastMouseX = x;
        _lastMouseY = y;
    }

    public void OnNodeMouseUp()
    {
        if (SelectedNode is not null)
            SelectedNode.IsDragging = false;
        IsDraggingNode = false;
    }

    public void OnPortDragStart(Port port, double canvasX, double canvasY)
    {
        _draggingPort = port;
        IsDrawingConnection = true;
        ConnectionLineX1 = canvasX;
        ConnectionLineY1 = canvasY;
        ConnectionLineX2 = canvasX;
        ConnectionLineY2 = canvasY;
    }

    public void OnPortDrag(double toX, double toY)
    {
        if (!IsDrawingConnection) return;
        ConnectionLineX2 = toX;
        ConnectionLineY2 = toY;
    }

    [RelayCommand]
    private void ValidatePipeline()
    {
        bool valid = true;
        string error = string.Empty;

        var connectedNodeIds = new HashSet<string>();
        foreach (var edge in Edges)
        {
            connectedNodeIds.Add(edge.SourceNodeId);
            connectedNodeIds.Add(edge.TargetNodeId);
        }

        if (connectedNodeIds.Count < Nodes.Count && Nodes.Count > 1)
        {
            valid = false;
            error = "Some nodes are not connected";
        }

        foreach (var node in Nodes)
        {
            if (node.OutputPorts.Count > 0 && !Edges.Any(e => e.SourceNodeId == node.Id) &&
                !Edges.Any(e => e.TargetNodeId == node.Id) && Edges.Count > 0 && node != Nodes[^1])
            {
                valid = false;
                error = $"Node '{node.DisplayName}' has no connections";
                break;
            }
        }
    }

    public void FitAll()
    {
        if (Nodes.Count == 0) return;
        var minX = Nodes.Min(n => n.CanvasX);
        var minY = Nodes.Min(n => n.CanvasY);
        var maxX = Nodes.Max(n => n.CanvasX + n.Width);
        var maxY = Nodes.Max(n => n.CanvasY + n.Height);
        OffsetX = -minX + 40;
        OffsetY = -minY + 40;
    }

    public void DuplicateSelected()
    {
        if (SelectedNode is null) return;
        var clone = new PipelineNode
        {
            PluginId = SelectedNode.PluginId,
            DisplayName = SelectedNode.DisplayName + " (copy)",
            CanvasX = SelectedNode.CanvasX + 200,
            CanvasY = SelectedNode.CanvasY + 30,
            Parameters = new Dictionary<string, object>(SelectedNode.Parameters)
        };
        Nodes.Add(clone);
    }
}
