using Photopipeline.Models;
using Photopipeline.ViewModels;
using SkiaSharp;
using SkiaSharp.Views.Desktop;
using SkiaSharp.Views.WPF;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;

namespace Photopipeline.Controls;

public sealed class SkiaDAGCanvas : UserControl
{
    private SKElement? _skElement;
    private PipelineEditorViewModel? _viewModel;
    private PropertyChangedEventHandler? _propertyChangedHandler;

    // Color constants (SKColor is a value type — safe to create in constructor)
    private static readonly SKColor NodeBackgroundColor = new(40, 44, 52);
    private static readonly SKColor NodeBorderColor = new(80, 84, 92);
    private static readonly SKColor NodeSelectedBorderColor = new(96, 165, 250);
    private static readonly SKColor PortColor = new(150, 170, 200);
    private static readonly SKColor PortConnectedColor = new(34, 197, 94);
    private static readonly SKColor EdgeColor = new(100, 116, 139);
    private static readonly SKColor TextColor = new(220, 220, 225);
    private static readonly SKColor CategoryColor = new(148, 163, 184);

    // Paint objects — initialized in OnLoaded to avoid early SkiaSharp native calls
    private SKPaint? _nodeFillPaint;
    private SKPaint? _nodeBorderPaint;
    private SKPaint? _nodeSelectedBorderPaint;
    private SKPaint? _portPaint;
    private SKPaint? _portConnectedPaint;
    private SKPaint? _portBorderPaint;
    private SKPaint? _edgePaint;
    private SKPaint? _textPaint;
    private SKPaint? _categoryPaint;
    private SKPaint? _headerFillPaint;
    private SKPaint? _connectionLinePaint;

    private bool _paintsInitialized;
    private bool _loadedFired;

    public SkiaDAGCanvas()
    {
        Loaded += OnLoaded;
        Unloaded += OnUnloaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        if (_loadedFired) return;
        _loadedFired = true;

        try
        {
            InitializePaints();
            InitializeSkElement();
            WireViewModel();
        }
        catch (Exception ex)
        {
            WriteTrace($"SkiaDAGCanvas.OnLoaded failed: {ex.GetType().Name}: {ex.Message}");
            Debug.WriteLine($"[SkiaDAGCanvas] OnLoaded exception: {ex}");
        }
    }

    private void InitializePaints()
    {
        _nodeFillPaint = new SKPaint { Color = NodeBackgroundColor, Style = SKPaintStyle.Fill, IsAntialias = true };
        _nodeBorderPaint = new SKPaint { Color = NodeBorderColor, Style = SKPaintStyle.Stroke, StrokeWidth = 1.5f, IsAntialias = true };
        _nodeSelectedBorderPaint = new SKPaint { Color = NodeSelectedBorderColor, Style = SKPaintStyle.Stroke, StrokeWidth = 2.5f, IsAntialias = true };
        _portPaint = new SKPaint { Color = PortColor, Style = SKPaintStyle.Fill, IsAntialias = true };
        _portConnectedPaint = new SKPaint { Color = PortConnectedColor, Style = SKPaintStyle.Fill, IsAntialias = true };
        _portBorderPaint = new SKPaint { Color = SKColors.White, Style = SKPaintStyle.Stroke, StrokeWidth = 1.0f, IsAntialias = true };
        _edgePaint = new SKPaint { Color = EdgeColor, Style = SKPaintStyle.Stroke, StrokeWidth = 2.5f, IsAntialias = true };

        var defaultTypeface = SKTypeface.Default;
        _textPaint = new SKPaint { Color = TextColor, TextSize = 11, IsAntialias = true, Typeface = defaultTypeface };
        _categoryPaint = new SKPaint { Color = CategoryColor, TextSize = 9, IsAntialias = true, Typeface = defaultTypeface };

        _headerFillPaint = new SKPaint { Color = new SKColor(55, 60, 72), Style = SKPaintStyle.Fill, IsAntialias = true };
        _connectionLinePaint = new SKPaint { Color = new SKColor(96, 165, 250), Style = SKPaintStyle.Stroke, StrokeWidth = 2.0f, IsAntialias = true };

        _paintsInitialized = true;
    }

    private void InitializeSkElement()
    {
        _skElement = new SKElement
        {
            HorizontalAlignment = HorizontalAlignment.Stretch,
            VerticalAlignment = VerticalAlignment.Stretch
        };

        _skElement.PaintSurface += OnPaintSurface;
        _skElement.MouseLeftButtonDown += OnMouseLeftButtonDown;
        _skElement.MouseMove += OnMouseMove;
        _skElement.MouseLeftButtonUp += OnMouseLeftButtonUp;
        _skElement.MouseWheel += OnMouseWheel;

        Content = _skElement;
    }

    private void WireViewModel()
    {
        if (DataContext is PipelineEditorViewModel vm)
        {
            _viewModel = vm;
            _propertyChangedHandler = (_, _) => _skElement?.InvalidateVisual();
            _viewModel.PropertyChanged += _propertyChangedHandler;
        }
    }

    private void OnUnloaded(object sender, RoutedEventArgs e)
    {
        if (_viewModel is not null && _propertyChangedHandler is not null)
        {
            _viewModel.PropertyChanged -= _propertyChangedHandler;
            _propertyChangedHandler = null;
        }

        if (_skElement is not null)
        {
            _skElement.PaintSurface -= OnPaintSurface;
            _skElement.MouseLeftButtonDown -= OnMouseLeftButtonDown;
            _skElement.MouseMove -= OnMouseMove;
            _skElement.MouseLeftButtonUp -= OnMouseLeftButtonUp;
            _skElement.MouseWheel -= OnMouseWheel;
            _skElement = null;
        }

        _viewModel = null;
        _paintsInitialized = false;
    }

    private void OnPaintSurface(object sender, SKPaintSurfaceEventArgs e)
    {
        if (_viewModel is null || !_paintsInitialized) return;

        var canvas = e.Surface.Canvas;
        canvas.Clear(SKColors.Transparent);

        canvas.Save();
        canvas.Translate((float)_viewModel.OffsetX, (float)_viewModel.OffsetY);
        canvas.Scale((float)_viewModel.Scale);

        // Draw edges
        foreach (var edge in _viewModel.Edges)
        {
            var sourceNode = _viewModel.Nodes.FirstOrDefault(n => n.Id == edge.SourceNodeId);
            var targetNode = _viewModel.Nodes.FirstOrDefault(n => n.Id == edge.TargetNodeId);
            if (sourceNode is null || targetNode is null) continue;

            var x1 = (float)(sourceNode.CanvasX + sourceNode.Width);
            var y1 = (float)(sourceNode.CanvasY + sourceNode.Height / 2);
            var x2 = (float)targetNode.CanvasX;
            var y2 = (float)(targetNode.CanvasY + targetNode.Height / 2);
            var midX = (x1 + x2) / 2;

            using var path = new SKPath();
            path.MoveTo(x1, y1);
            path.CubicTo(midX, y1, midX, y2, x2, y2);
            canvas.DrawPath(path, _edgePaint);
        }

        // Draw connection line while dragging
        if (_viewModel.IsDrawingConnection && _connectionLinePaint is not null)
        {
            canvas.DrawLine(
                (float)_viewModel.ConnectionLineX1, (float)_viewModel.ConnectionLineY1,
                (float)_viewModel.ConnectionLineX2, (float)_viewModel.ConnectionLineY2,
                _connectionLinePaint);
        }

        // Draw nodes
        foreach (var node in _viewModel.Nodes)
        {
            DrawNode(canvas, node);
        }

        canvas.Restore();
    }

    private void DrawNode(SKCanvas canvas, PipelineNode node)
    {
        if (!_paintsInitialized) return;

        var x = (float)node.CanvasX;
        var y = (float)node.CanvasY;
        var w = (float)node.Width;
        var h = (float)node.Height;

        var fillColor = node.IsProcessing ? new SKColor(50, 50, 65) : NodeBackgroundColor;
        var borderPaint = node.IsSelected ? _nodeSelectedBorderPaint : _nodeBorderPaint;

        using var nodeFill = new SKPaint { Color = fillColor, Style = SKPaintStyle.Fill, IsAntialias = true };
        canvas.DrawRoundRect(x, y, w, h, 8, 8, nodeFill);
        canvas.DrawRoundRect(x, y, w, h, 8, 8, borderPaint);

        // Draw ports
        var portRadius = 5.0f;
        foreach (var port in node.InputPorts)
        {
            var idx = node.InputPorts.IndexOf(port);
            var py = y + (h / (node.InputPorts.Count + 1)) * (idx + 1);
            var pc = port.IsConnected ? _portConnectedPaint : _portPaint;
            canvas.DrawCircle(x, py, portRadius, pc);
            canvas.DrawCircle(x, py, portRadius, _portBorderPaint);
        }

        foreach (var port in node.OutputPorts)
        {
            var idx = node.OutputPorts.IndexOf(port);
            var py = y + (h / (node.OutputPorts.Count + 1)) * (idx + 1);
            var pc = port.IsConnected ? _portConnectedPaint : _portPaint;
            canvas.DrawCircle(x + w, py, portRadius, pc);
            canvas.DrawCircle(x + w, py, portRadius, _portBorderPaint);
        }

        // Draw header
        var headerHeight = 22f;
        canvas.DrawRoundRect(new SKRect(x, y, x + w, y + headerHeight), 8, 8, _headerFillPaint);
        canvas.DrawRect(x, y + headerHeight - 8, w, 8, _headerFillPaint);

        // Draw text
        canvas.DrawText(node.DisplayName, x + 10, y + 15, _textPaint);
        var pluginCategory = GetCategoryForNode(node);
        if (!string.IsNullOrEmpty(pluginCategory))
        {
            canvas.DrawText(pluginCategory, x + 10, y + 36, _categoryPaint);
        }
    }

    private static string GetCategoryForNode(PipelineNode node)
    {
        return node.PluginId switch
        {
            "demosaic" => "Raw Processing",
            "exposure" => "Tonal",
            "white_balance" => "Color",
            "denoise" => "Noise Reduction",
            "sharpen" => "Detail",
            _ => string.Empty
        };
    }

    // Mouse handlers
    private void OnMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        if (_viewModel is null || _skElement is null) return;
        var point = e.GetPosition(_skElement);
        var node = HitTestNode(point);
        if (node is not null)
        {
            _viewModel.OnNodeMouseDown(node, point.X, point.Y);
            _skElement.CaptureMouse();
        }
        else
        {
            _viewModel.ClearSelectionCommand.Execute(null);
        }
    }

    private void OnMouseMove(object sender, MouseEventArgs e)
    {
        if (_viewModel is null || _skElement is null) return;
        var point = e.GetPosition(_skElement);
        _viewModel.OnNodeMouseMove(point.X, point.Y);
    }

    private void OnMouseLeftButtonUp(object sender, MouseButtonEventArgs e)
    {
        if (_viewModel is null || _skElement is null) return;
        _viewModel.OnNodeMouseUp();
        _skElement.ReleaseMouseCapture();
    }

    private void OnMouseWheel(object sender, MouseWheelEventArgs e)
    {
        if (_viewModel is null) return;
        var delta = e.Delta;
        _viewModel.Scale = Math.Max(0.1, Math.Min(3.0, _viewModel.Scale + delta * 0.001));
    }

    private PipelineNode? HitTestNode(Point point)
    {
        if (_viewModel is null) return null;
        var adjustedX = (point.X - _viewModel.OffsetX) / _viewModel.Scale;
        var adjustedY = (point.Y - _viewModel.OffsetY) / _viewModel.Scale;
        return _viewModel.Nodes.Reverse().FirstOrDefault(n =>
            adjustedX >= n.CanvasX && adjustedX <= n.CanvasX + n.Width &&
            adjustedY >= n.CanvasY && adjustedY <= n.CanvasY + n.Height);
    }

    private static void WriteTrace(string msg)
    {
        try
        {
            var dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Photopipeline", "logs");
            Directory.CreateDirectory(dir);
            File.AppendAllText(Path.Combine(dir, "trace.log"),
                $"{DateTime.Now:HH:mm:ss.fff} [SkiaDAG] {msg}\n");
        }
        catch { }
    }
}
