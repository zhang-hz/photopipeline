// NOTE: File dialogs used in FilmstripVM, PreviewVM, and other VMs should be abstracted
// behind a service interface (e.g., IDialogService) to avoid violating MVVM.
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using SkiaSharp;
using System.Collections.ObjectModel;

namespace Photopipeline.ViewModels;

public sealed partial class PipelineEditorViewModel : ViewModelBase
{
    private readonly IPipelineService _pipelineService;
    private CancellationTokenSource? _executeCts;

    [ObservableProperty] private PipelineSpec _currentPipeline = new() { Name = "New Pipeline" };
    [ObservableProperty] private string? _pipelineId;
    [ObservableProperty] private bool _isPipelineValid;
    [ObservableProperty] private string _validationMessage = string.Empty;
    [ObservableProperty] private ObservableCollection<PipelineNode> _nodes = new();
    [ObservableProperty] private ObservableCollection<PipelineEdge> _edges = new();
    [ObservableProperty] private PipelineNode? _selectedNode;
    [ObservableProperty] private double _scale = 1.0;
    [ObservableProperty] private double _offsetX;
    [ObservableProperty] private double _offsetY;
    [ObservableProperty] private bool _isExecuting;
    [ObservableProperty] private string _executionStatus = string.Empty;
    [ObservableProperty] private ObservableCollection<ExecuteProgress> _progressHistory = new();
    [ObservableProperty] private string? _selectedImagePath;

    public event Action? PreviewUpdateRequested;

    public PipelineEditorViewModel(ILogger<PipelineEditorViewModel> logger, IPipelineService pipelineService)
        : base(logger)
    {
        _pipelineService = pipelineService;
    }

    [RelayCommand]
    private void NewPipeline()
    {
        CurrentPipeline = new PipelineSpec { Name = "New Pipeline" };
        Nodes = new ObservableCollection<PipelineNode>();
        Edges = new ObservableCollection<PipelineEdge>();
        PipelineId = null;
        IsPipelineValid = false;
        ValidationMessage = string.Empty;
    }

    [RelayCommand]
    private void AddNode(PluginInfo plugin)
    {
        AddNodeAt(plugin, 80 + Nodes.Count * 30, 60 + Nodes.Count * 86);
    }

    public void AddNodeAt(PluginInfo plugin, double x, double y)
    {
        if (IsExecuting) return;
        var node = new PipelineNode
        {
            Id = Guid.NewGuid().ToString(),
            PluginId = plugin.Id,
            Label = plugin.Name,
            Enabled = true,
            PositionX = x,
            PositionY = y
        };
        Nodes.Add(node);
        CurrentPipeline.Nodes.Add(node);
        IsPipelineValid = false;
    }

    public event Func<PipelineNode?, bool>? ConfirmRemoveNode;

    [RelayCommand]
    private void RemoveNode(PipelineNode? node)
    {
        if (node is null || IsExecuting) return;

        if (ConfirmRemoveNode != null && !ConfirmRemoveNode(node))
            return;

        Nodes.Remove(node);
        CurrentPipeline.Nodes.Remove(node);
        var relatedEdges = Edges.Where(e => e.From == node.Id || e.To == node.Id).ToList();
        foreach (var edge in relatedEdges)
        {
            Edges.Remove(edge);
            CurrentPipeline.Edges.Remove(edge);
        }
        if (SelectedNode?.Id == node.Id)
            SelectedNode = null;
    }

    public void UpdateNodePosition(string nodeId, double x, double y)
    {
        var node = Nodes.FirstOrDefault(n => n.Id == nodeId);
        if (node is null) return;
        node.PositionX = x;
        node.PositionY = y;
    }

    [RelayCommand]
    private void ConnectNodes((string from, string to) connection)
    {
        if (IsExecuting) return;
        if (!CanConnect(connection.from, connection.to)) return;

        var exists = Edges.Any(e => e.From == connection.from && e.To == connection.to);
        if (exists) return;

        var edge = new PipelineEdge { From = connection.from, To = connection.to };
        Edges.Add(edge);
        CurrentPipeline.Edges.Add(edge);
    }

    [RelayCommand]
    private void DisconnectEdge(PipelineEdge? edge)
    {
        if (edge is null) return;
        Edges.Remove(edge);
        CurrentPipeline.Edges.Remove(edge);
    }

    public bool CanConnect(string fromNodeId, string toNodeId)
    {
        if (fromNodeId == toNodeId) return false;
        return !WouldCreateCycle(fromNodeId, toNodeId);
    }

    private bool WouldCreateCycle(string from, string to)
    {
        var visited = new HashSet<string>();
        var queue = new Queue<string>();
        queue.Enqueue(from);

        while (queue.Count > 0)
        {
            var current = queue.Dequeue();
            if (current == to) return true;
            if (!visited.Add(current)) continue;

            foreach (var edge in Edges.Where(e => e.To == current))
                queue.Enqueue(edge.From);
        }
        return false;
    }

    [RelayCommand]
    private async Task Validate(CancellationToken ct)
    {
        if (Nodes.Count == 0)
        {
            ValidationMessage = "Pipeline has no nodes";
            IsPipelineValid = false;
            return;
        }
        await ExecuteAsync(async ct2 =>
        {
            var result = await _pipelineService.ValidateAsync(CurrentPipeline, ct2);
            IsPipelineValid = result.Valid;
            ValidationMessage = result.Valid ? "Valid" : string.Join("; ", result.Issues.Select(i => i.Message));
        }, "Validate pipeline", ct);
    }

    [RelayCommand]
    private async Task Create(CancellationToken ct)
    {
        await ExecuteAsync(async ct2 =>
        {
            PipelineId = await _pipelineService.CreatePipelineAsync(CurrentPipeline, ct2);
            StatusMessage = $"Pipeline created: {PipelineId}";
        }, "Create pipeline", ct);
    }

    [RelayCommand]
    private async Task Execute(CancellationToken ct)
    {
        if (Nodes.Count == 0)
        {
            ErrorMessage = "Add at least one node to the pipeline";
            return;
        }
        if (string.IsNullOrEmpty(SelectedImagePath))
        {
            ErrorMessage = "Select an image in the Filmstrip first";
            return;
        }

        CancelExecute();
        _executeCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        var token = _executeCts.Token;

        try
        {
            IsExecuting = true;
            ExecutionStatus = "Starting...";
            ProgressHistory.Clear();
            ErrorMessage = null;

            var pid = PipelineId;
            if (string.IsNullOrEmpty(pid))
                pid = await _pipelineService.CreatePipelineAsync(CurrentPipeline, token);

            var outputPath = System.IO.Path.Combine(System.IO.Path.GetTempPath(),
                $"photopipeline_preview_{Guid.NewGuid():N}.tif");

            await foreach (var progress in _pipelineService.ExecuteAsync(pid, SelectedImagePath, outputPath, token))
            {
                token.ThrowIfCancellationRequested();
                ProgressHistory.Add(progress);
                ExecutionStatus = $"{progress.Stage}: {progress.NodeLabel} ({progress.Fraction:P0})";
            }

            ExecutionStatus = "Completed";
            StatusMessage = "Pipeline execution completed";
            PreviewUpdateRequested?.Invoke();
        }
        catch (OperationCanceledException)
        {
            ExecutionStatus = "Cancelled";
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Pipeline execution failed");
            ExecutionStatus = "Failed";
            ErrorMessage = $"Execution failed: {ex.Message}";
        }
        finally
        {
            IsExecuting = false;
        }
    }

    [RelayCommand]
    private void CancelExecute()
    {
        if (_executeCts != null)
        {
            _executeCts.Cancel();
            _executeCts.Dispose();
            _executeCts = null;
        }
    }

    [RelayCommand]
    private void UpdateNodeParameter((string nodeId, string key, object value) param)
    {
        var node = Nodes.FirstOrDefault(n => n.Id == param.nodeId);
        if (node is null) return;
        node.Params[param.key] = param.value;
    }

    [RelayCommand]
    private void ZoomCanvas(object delta)
    {
        if (TryParseDouble(delta, out var d))
            Scale = Math.Clamp(Scale + d, 0.1, 5.0);
    }

    private static bool TryParseDouble(object? value, out double result)
    {
        if (value is double d) { result = d; return true; }
        if (value is string s && double.TryParse(s, out var parsed)) { result = parsed; return true; }
        result = 0;
        return false;
    }

    [RelayCommand]
    private void ResetCanvas() { Scale = 1.0; OffsetX = 0; OffsetY = 0; }

    public override void Shutdown()
    {
        base.Shutdown();
        try { _executeCts?.Cancel(); _executeCts?.Dispose(); } catch { }
    }
}
