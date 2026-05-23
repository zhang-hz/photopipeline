using Microsoft.Graphics.Canvas;
using Microsoft.Graphics.Canvas.UI.Xaml;
using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Photopipeline.Models;
using Photopipeline.ViewModels;
using System;
using System.Linq;
using Windows.Foundation;
using Windows.UI;

namespace Photopipeline.Controls;

public sealed class DAGCanvas : UserControl
{
    private CanvasControl? _canvasControl;
    private PipelineEditorViewModel? _viewModel;

    private static readonly Color NodeBackgroundColor = Color.FromArgb(255, 40, 44, 52);
    private static readonly Color NodeBorderColor = Color.FromArgb(255, 80, 84, 92);
    private static readonly Color NodeSelectedBorderColor = Color.FromArgb(255, 96, 165, 250);
    private static readonly Color PortColor = Color.FromArgb(255, 150, 170, 200);
    private static readonly Color PortConnectedColor = Color.FromArgb(255, 34, 197, 94);
    private static readonly Color EdgeColor = Color.FromArgb(255, 100, 116, 139);
    private static readonly Color TextColor = Color.FromArgb(255, 220, 220, 225);
    private static readonly Color CategoryColor = Color.FromArgb(255, 148, 163, 184);

    public DAGCanvas()
    {
        this.Loaded += OnLoaded;
        this.Unloaded += OnUnloaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        _canvasControl = new CanvasControl
        {
            HorizontalAlignment = HorizontalAlignment.Stretch,
            VerticalAlignment = VerticalAlignment.Stretch,
            ClearColor = Colors.Transparent
        };

        _canvasControl.Draw += OnDraw;
        _canvasControl.PointerPressed += OnPointerPressed;
        _canvasControl.PointerMoved += OnPointerMoved;
        _canvasControl.PointerReleased += OnPointerReleased;
        _canvasControl.PointerWheelChanged += OnPointerWheelChanged;

        this.Content = _canvasControl;

        if (this.DataContext is PipelineEditorViewModel vm)
        {
            _viewModel = vm;
            _viewModel.PropertyChanged += (s, args) => _canvasControl?.Invalidate();
        }
    }

    private void OnUnloaded(object sender, RoutedEventArgs e)
    {
        if (_canvasControl is not null)
        {
            _canvasControl.Draw -= OnDraw;
            _canvasControl.PointerPressed -= OnPointerPressed;
            _canvasControl.PointerMoved -= OnPointerMoved;
            _canvasControl.PointerReleased -= OnPointerReleased;
            _canvasControl.PointerWheelChanged -= OnPointerWheelChanged;
            _canvasControl.RemoveFromVisualTree();
            _canvasControl = null;
        }
    }

    private void OnDraw(CanvasControl sender, CanvasDrawEventArgs args)
    {
        if (_viewModel is null) return;
        var ds = args.DrawingSession;

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

            ds.DrawBezier(
                new System.Numerics.Vector2(x1, y1),
                new System.Numerics.Vector2(midX, y1),
                new System.Numerics.Vector2(midX, y2),
                new System.Numerics.Vector2(x2, y2),
                EdgeColor, 2.5f);
        }

        if (_viewModel.IsDrawingConnection)
        {
            ds.DrawLine(
                new System.Numerics.Vector2((float)_viewModel.ConnectionLineX1, (float)_viewModel.ConnectionLineY1),
                new System.Numerics.Vector2((float)_viewModel.ConnectionLineX2, (float)_viewModel.ConnectionLineY2),
                Color.FromArgb(255, 96, 165, 250), 2.0f,
                new CanvasStrokeStyle { DashStyle = CanvasDashStyle.Dash });
        }

        foreach (var node in _viewModel.Nodes)
        {
            DrawNode(ds, node);
        }
    }

    private void DrawNode(CanvasDrawingSession ds, PipelineNode node)
    {
        var x = (float)node.CanvasX;
        var y = (float)node.CanvasY;
        var w = (float)node.Width;
        var h = (float)node.Height;

        var fillColor = node.IsProcessing
            ? Color.FromArgb(255, 50, 50, 65)
            : NodeBackgroundColor;

        var borderColor = node.IsSelected
            ? NodeSelectedBorderColor
            : NodeBorderColor;

        var borderWidth = node.IsSelected ? 2.5f : 1.5f;

        ds.FillRoundedRectangle(x, y, w, h, 8, 8, fillColor);
        ds.DrawRoundedRectangle(x, y, w, h, 8, 8, borderColor, borderWidth);

        var portRadius = 5.0f;
        foreach (var port in node.InputPorts)
        {
            var py = y + ((h / (node.InputPorts.Count + 1)) * (node.InputPorts.IndexOf(port) + 1));
            ds.FillCircle(x, py, portRadius,
                port.IsConnected ? PortConnectedColor : PortColor);
            ds.DrawCircle(x, py, portRadius, Colors.White, 1.0f);
        }

        foreach (var port in node.OutputPorts)
        {
            var py = y + ((h / (node.OutputPorts.Count + 1)) * (node.OutputPorts.IndexOf(port) + 1));
            ds.FillCircle(x + w, py, portRadius,
                port.IsConnected ? PortConnectedColor : PortColor);
            ds.DrawCircle(x + w, py, portRadius, Colors.White, 1.0f);
        }

        var headerHeight = 22f;
        ds.FillRoundedRectangle(new Rect(x, y, w, headerHeight), 8, 8, Color.FromArgb(255, 55, 60, 72));
        ds.FillRectangle(x, y + headerHeight - 8, w, 8, Color.FromArgb(255, 55, 60, 72));

        ds.DrawText(node.DisplayName,
            new System.Numerics.Vector2(x + 10, y + 3),
            TextColor,
            new CanvasTextFormat
            {
                FontSize = 11,
                FontWeight = Windows.UI.Text.FontWeights.SemiBold,
                WordWrapping = CanvasWordWrapping.NoWrap,
                VerticalAlignment = CanvasVerticalAlignment.Top,
            });

        var pluginCategory = GetCategoryForNode(node);
        if (!string.IsNullOrEmpty(pluginCategory))
        {
            ds.DrawText(pluginCategory,
                new System.Numerics.Vector2(x + 10, y + 26),
                CategoryColor,
                new CanvasTextFormat
                {
                    FontSize = 9,
                    FontWeight = Windows.UI.Text.FontWeights.Normal,
                    WordWrapping = CanvasWordWrapping.NoWrap,
                });
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

    private void OnPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        if (_viewModel is null || _canvasControl is null) return;
        var point = e.GetCurrentPoint(_canvasControl).Position;
        var node = HitTestNode(point);
        if (node is not null)
        {
            _viewModel.OnNodeMouseDown(node, point.X, point.Y);
            _canvasControl.CapturePointer(e.Pointer);
        }
        else
        {
            _viewModel.ClearSelection();
        }
    }

    private void OnPointerMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_viewModel is null || _canvasControl is null) return;
        var point = e.GetCurrentPoint(_canvasControl).Position;
        _viewModel.OnNodeMouseMove(point.X, point.Y);
    }

    private void OnPointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (_viewModel is null || _canvasControl is null) return;
        _viewModel.OnNodeMouseUp();
        _canvasControl?.ReleasePointerCapture(e.Pointer);
    }

    private void OnPointerWheelChanged(object sender, PointerRoutedEventArgs e)
    {
        if (_viewModel is null) return;
        var props = e.GetCurrentPoint(null).Properties;
        var delta = props.MouseWheelDelta;
        _viewModel.Scale = Math.Max(0.1, Math.Min(3.0, _viewModel.Scale + delta * 0.001));
    }

    private PipelineNode? HitTestNode(Point point)
    {
        if (_viewModel is null) return null;
        return _viewModel.Nodes.Reverse().FirstOrDefault(n =>
            point.X >= n.CanvasX && point.X <= n.CanvasX + n.Width &&
            point.Y >= n.CanvasY && point.Y <= n.CanvasY + n.Height);
    }
}
