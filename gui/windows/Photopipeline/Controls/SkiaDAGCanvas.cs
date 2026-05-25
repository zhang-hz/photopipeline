using Photopipeline.Models;
using SkiaSharp;
using SkiaSharp.Views.Desktop;
using SkiaSharp.Views.WPF;
using System.Windows;
using System.Windows.Input;

namespace Photopipeline.Controls;

public sealed class SkiaDAGCanvas : SKElement
{
    // ── Data ──

    private IReadOnlyList<PipelineNode> _nodes = Array.Empty<PipelineNode>();
    private IReadOnlyList<PipelineEdge> _edges = Array.Empty<PipelineEdge>();

    // Node layout cache: nodeId → (bounds rect, input port center, output port center)
    private readonly Dictionary<string, NodeLayout> _layoutCache = new();

    // ── Interaction state ──

    private enum InteractionState { Idle, DraggingNode, ConnectingPort, BoxSelecting, Panning }

    private InteractionState _state;
    private string? _dragNodeId;
    private SKPoint _dragOffset;
    private SKPoint _lastMouse;
    private string? _connectFromNodeId;
    private SKPoint _connectFromPos;
    private SKPoint _connectMousePos;

    // ── Dependency Properties ──

    public static readonly DependencyProperty ScaleProperty =
        DependencyProperty.Register(nameof(Scale), typeof(double), typeof(SkiaDAGCanvas),
            new FrameworkPropertyMetadata(1.0, FrameworkPropertyMetadataOptions.AffectsRender, OnScaleChanged));

    public static readonly DependencyProperty OffsetXProperty =
        DependencyProperty.Register(nameof(OffsetX), typeof(double), typeof(SkiaDAGCanvas),
            new FrameworkPropertyMetadata(0.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty OffsetYProperty =
        DependencyProperty.Register(nameof(OffsetY), typeof(double), typeof(SkiaDAGCanvas),
            new FrameworkPropertyMetadata(0.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public double Scale
    {
        get => (double)GetValue(ScaleProperty);
        set => SetValue(ScaleProperty, Math.Clamp(value, 0.1, 5.0));
    }

    public double OffsetX
    {
        get => (double)GetValue(OffsetXProperty);
        set => SetValue(OffsetXProperty, value);
    }

    public double OffsetY
    {
        get => (double)GetValue(OffsetYProperty);
        set => SetValue(OffsetYProperty, value);
    }

    // ── Dependency Properties for collections ──

    public static readonly DependencyProperty NodesProperty =
        DependencyProperty.Register(nameof(Nodes), typeof(IReadOnlyList<PipelineNode>), typeof(SkiaDAGCanvas),
            new FrameworkPropertyMetadata(Array.Empty<PipelineNode>(), FrameworkPropertyMetadataOptions.AffectsRender, OnNodesChanged));

    public static readonly DependencyProperty EdgesProperty =
        DependencyProperty.Register(nameof(Edges), typeof(IReadOnlyList<PipelineEdge>), typeof(SkiaDAGCanvas),
            new FrameworkPropertyMetadata(Array.Empty<PipelineEdge>(), FrameworkPropertyMetadataOptions.AffectsRender, OnEdgesChanged));

    public static readonly DependencyProperty SelectedNodeProperty =
        DependencyProperty.Register(nameof(SelectedNode), typeof(PipelineNode), typeof(SkiaDAGCanvas),
            new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    private static void OnNodesChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var canvas = (SkiaDAGCanvas)d;
        canvas._nodes = e.NewValue as IReadOnlyList<PipelineNode> ?? Array.Empty<PipelineNode>();
    }

    private static void OnEdgesChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var canvas = (SkiaDAGCanvas)d;
        canvas._edges = e.NewValue as IReadOnlyList<PipelineEdge> ?? Array.Empty<PipelineEdge>();
    }

    public IReadOnlyList<PipelineNode> Nodes
    {
        get => (IReadOnlyList<PipelineNode>)GetValue(NodesProperty);
        set => SetValue(NodesProperty, value ?? Array.Empty<PipelineNode>());
    }

    public IReadOnlyList<PipelineEdge> Edges
    {
        get => (IReadOnlyList<PipelineEdge>)GetValue(EdgesProperty);
        set => SetValue(EdgesProperty, value ?? Array.Empty<PipelineEdge>());
    }

    public PipelineNode? SelectedNode
    {
        get => (PipelineNode?)GetValue(SelectedNodeProperty);
        set => SetValue(SelectedNodeProperty, value);
    }

    // ── Events ──

    public event Action<PluginInfo, SKPoint>? NodeDropped;
    public event Action<PipelineNode>? NodeSelected;
    public event Action<string, string>? PortsConnected;
    public event Action<PipelineNode, SKPoint>? NodeMoved;

    // ── Visual constants ──

    private const float NodeWidth = 150;
    private const float NodeHeight = 56;
    private const float PortRadius = 6;
    private const float CornerRadius = 8;
    private const float GridSpacing = 40;
    private const float MiniMapWidth = 160;
    private const float MiniMapHeight = 100;
    private const float MiniMapMargin = 12;

    private static readonly SKColor BgColor = new(30, 30, 30);
    private static readonly SKColor GridColor = new(50, 50, 50);
    private static readonly SKColor NodeBgColor = new(45, 45, 48);
    private static readonly SKColor NodeBorderColor = new(80, 80, 85);
    private static readonly SKColor NodeSelectedBorder = new(0, 120, 212);
    private static readonly SKColor InputPortColor = new(70, 130, 220);
    private static readonly SKColor OutputPortColor = new(60, 180, 100);
    private static readonly SKColor EdgeColor = new(160, 160, 170);
    private static readonly SKColor EdgeSelectedColor = new(0, 160, 255);
    private static readonly SKColor TextColor = SKColors.White;
    private static readonly SKColor SubTextColor = new(170, 170, 180);

    public SkiaDAGCanvas()
    {
        Focusable = true;
        AllowDrop = true;
    }

    // ── Property callbacks ──

    private static void OnScaleChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var newVal = (double)e.NewValue;
        if (newVal is < 0.1 or > 5.0)
            ((SkiaDAGCanvas)d).Scale = Math.Clamp(newVal, 0.1, 5.0);
        else
            ((SkiaDAGCanvas)d).InvalidateVisual();
    }

    // ── Coordinate transforms ──

    private SKPoint ScreenToWorld(SKPoint screen) => new(
        (screen.X - (float)OffsetX) / (float)Scale,
        (screen.Y - (float)OffsetY) / (float)Scale);

    private SKPoint WorldToScreen(SKPoint world) => new(
        world.X * (float)Scale + (float)OffsetX,
        world.Y * (float)Scale + (float)OffsetY);

    // ── Rendering ──

    protected override void OnPaintSurface(SKPaintSurfaceEventArgs e)
    {
        var canvas = e.Surface.Canvas;
        var info = e.Info;
        canvas.Clear(BgColor);

        DrawGrid(canvas, info);
        DrawEdges(canvas);
        DrawNodes(canvas);
        DrawTempConnection(canvas);
        DrawMiniMap(canvas, info);
    }

    private void DrawGrid(SKCanvas canvas, SKImageInfo info)
    {
        float scaledSpacing = GridSpacing * (float)Scale;
        float ox = (float)OffsetX % scaledSpacing;
        float oy = (float)OffsetY % scaledSpacing;

        using var paint = new SKPaint { Color = GridColor, StrokeWidth = 0.5f };
        for (float x = ox; x < info.Width; x += scaledSpacing)
            canvas.DrawLine(x, 0, x, info.Height, paint);
        for (float y = oy; y < info.Height; y += scaledSpacing)
            canvas.DrawLine(0, y, info.Width, y, paint);
    }

    private void DrawEdges(SKCanvas canvas)
    {
        foreach (var edge in _edges)
        {
            if (!_layoutCache.TryGetValue(edge.From, out var fromLayout)) continue;
            if (!_layoutCache.TryGetValue(edge.To, out var toLayout)) continue;

            var start = WorldToScreen(fromLayout.OutputPort);
            var end = WorldToScreen(toLayout.InputPort);
            bool selected = SelectedNode != null &&
                (SelectedNode.Id == edge.From || SelectedNode.Id == edge.To);

            using var paint = new SKPaint
            {
                Color = selected ? EdgeSelectedColor : EdgeColor,
                StrokeWidth = selected ? 2.5f : 1.5f,
                IsAntialias = true,
                Style = SKPaintStyle.Stroke
            };

            var path = new SKPath();
            path.MoveTo(start);
            float dx = Math.Abs(end.X - start.X) * 0.5f;
            path.CubicTo(start.X + dx, start.Y, end.X - dx, end.Y, end.X, end.Y);
            canvas.DrawPath(path, paint);
            path.Dispose();
        }
    }

    private void DrawNodes(SKCanvas canvas)
    {
        _layoutCache.Clear();

        float startX = 80;
        float startY = 60;
        float spacingY = NodeHeight + 30;

        for (int i = 0; i < _nodes.Count; i++)
        {
            var node = _nodes[i];
            float nx = (float)(node.PositionX > 0 || node.PositionY > 0 ? node.PositionX : startX);
            float ny = (float)(node.PositionX > 0 || node.PositionY > 0 ? node.PositionY : startY + i * spacingY);
            var worldRect = new SKRect(nx, ny, nx + NodeWidth, ny + NodeHeight);
            var screenRect = new SKRect(
                WorldToScreen(new SKPoint(worldRect.Left, worldRect.Top)).X,
                WorldToScreen(new SKPoint(worldRect.Left, worldRect.Top)).Y,
                WorldToScreen(new SKPoint(worldRect.Right, worldRect.Bottom)).X,
                WorldToScreen(new SKPoint(worldRect.Right, worldRect.Bottom)).Y);

            bool selected = SelectedNode?.Id == node.Id;

            // Shadow
            using var shadowPaint = new SKPaint
            {
                Color = new SKColor(0, 0, 0, 40),
                IsAntialias = true,
                MaskFilter = SKMaskFilter.CreateBlur(SKBlurStyle.Normal, 4)
            };
            canvas.DrawRoundRect(new SKRect(screenRect.Left + 2, screenRect.Top + 2,
                screenRect.Right + 2, screenRect.Bottom + 2), CornerRadius, CornerRadius, shadowPaint);

            // Body
            using var bodyPaint = new SKPaint
            {
                Color = NodeBgColor,
                IsAntialias = true,
                Style = SKPaintStyle.Fill
            };
            canvas.DrawRoundRect(screenRect, CornerRadius, CornerRadius, bodyPaint);

            // Border
            using var borderPaint = new SKPaint
            {
                Color = selected ? NodeSelectedBorder : NodeBorderColor,
                StrokeWidth = selected ? 2f : 1f,
                IsAntialias = true,
                Style = SKPaintStyle.Stroke
            };
            canvas.DrawRoundRect(screenRect, CornerRadius, CornerRadius, borderPaint);

            // Label
            using var labelPaint = new SKPaint
            {
                Color = TextColor,
                TextSize = 12,
                IsAntialias = true
            };
            canvas.DrawText(node.Label, screenRect.Left + 36, screenRect.Top + 20, labelPaint);

            // Plugin ID subtitle
            using var subPaint = new SKPaint
            {
                Color = SubTextColor,
                TextSize = 10,
                IsAntialias = true
            };
            canvas.DrawText(node.PluginId, screenRect.Left + 36, screenRect.Top + 38, subPaint);

            // Input port (left side)
            var inputPortWorld = new SKPoint(worldRect.Left, worldRect.Top + NodeHeight / 2);
            var inputPortScreen = WorldToScreen(inputPortWorld);
            using var inputPaint = new SKPaint
            {
                Color = InputPortColor,
                IsAntialias = true
            };
            canvas.DrawCircle(inputPortScreen, PortRadius, inputPaint);

            // Output port (right side)
            var outputPortWorld = new SKPoint(worldRect.Right, worldRect.Top + NodeHeight / 2);
            var outputPortScreen = WorldToScreen(outputPortWorld);
            using var outputPaint = new SKPaint
            {
                Color = OutputPortColor,
                IsAntialias = true
            };
            canvas.DrawCircle(outputPortScreen, PortRadius, outputPaint);

            // Enable/disable indicator
            if (!node.Enabled)
            {
                using var dimOverlay = new SKPaint
                {
                    Color = new SKColor(0, 0, 0, 100),
                    IsAntialias = true
                };
                canvas.DrawRoundRect(screenRect, CornerRadius, CornerRadius, dimOverlay);
            }

            // Cache layout
            _layoutCache[node.Id] = new NodeLayout(worldRect, inputPortWorld, outputPortWorld);

            // Selection highlight ring
            if (selected)
            {
                using var selPaint = new SKPaint
                {
                    Color = NodeSelectedBorder,
                    StrokeWidth = 2,
                    IsAntialias = true,
                    Style = SKPaintStyle.Stroke
                };
                canvas.DrawRoundRect(new SKRect(screenRect.Left - 3, screenRect.Top - 3,
                    screenRect.Right + 3, screenRect.Bottom + 3), CornerRadius + 2, CornerRadius + 2, selPaint);
            }
        }
    }

    private void DrawTempConnection(SKCanvas canvas)
    {
        if (_state != InteractionState.ConnectingPort || _connectFromNodeId == null) return;

        var end = new SKPoint(_connectMousePos.X, _connectMousePos.Y);
        using var paint = new SKPaint
        {
            Color = new SKColor(255, 255, 255, 150),
            StrokeWidth = 2,
            IsAntialias = true,
            Style = SKPaintStyle.Stroke,
            PathEffect = SKPathEffect.CreateDash(new[] { 6f, 4f }, 0)
        };

        var path = new SKPath();
        path.MoveTo(_connectFromPos);
        float dx = Math.Abs(end.X - _connectFromPos.X) * 0.5f;
        path.CubicTo(_connectFromPos.X + dx, _connectFromPos.Y,
            end.X - dx, end.Y, end.X, end.Y);
        canvas.DrawPath(path, paint);
        path.Dispose();
    }

    private void DrawMiniMap(SKCanvas canvas, SKImageInfo info)
    {
        float mx = info.Width - MiniMapWidth - MiniMapMargin;
        float my = info.Height - MiniMapHeight - MiniMapMargin;
        var mapRect = new SKRect(mx, my, mx + MiniMapWidth, my + MiniMapHeight);

        // Background
        using var bgPaint = new SKPaint
        {
            Color = new SKColor(20, 20, 20, 200),
            IsAntialias = true
        };
        canvas.DrawRoundRect(mapRect, 6, 6, bgPaint);

        // Border
        using var borderPaint = new SKPaint
        {
            Color = new SKColor(80, 80, 80),
            StrokeWidth = 1,
            IsAntialias = true,
            Style = SKPaintStyle.Stroke
        };
        canvas.DrawRoundRect(mapRect, 6, 6, borderPaint);

        if (_layoutCache.Count == 0) return;

        // Dynamic world bounding box (with padding)
        var worldBounds = ComputeWorldBounds();
        float padding = 200;
        worldBounds = new SKRect(
            worldBounds.Left - padding, worldBounds.Top - padding,
            worldBounds.Right + padding, worldBounds.Bottom + padding);

        float worldW = worldBounds.Width > 0 ? worldBounds.Width : 1;
        float worldH = worldBounds.Height > 0 ? worldBounds.Height : 1;

        SKPoint MapToMiniMap(SKPoint worldPt) => new(
            mx + (worldPt.X - worldBounds.Left) / worldW * MiniMapWidth,
            my + (worldPt.Y - worldBounds.Top) / worldH * MiniMapHeight);

        // Node dots
        using var dotPaint = new SKPaint { Color = new SKColor(150, 150, 150), IsAntialias = true };
        foreach (var layout in _layoutCache.Values)
        {
            var dot = MapToMiniMap(new SKPoint(layout.Bounds.MidX, layout.Bounds.MidY));
            canvas.DrawCircle(dot, 2, dotPaint);
        }

        // Viewport rectangle
        var worldViewTL = ScreenToWorld(SKPoint.Empty);
        var worldViewBR = ScreenToWorld(new SKPoint(info.Width, info.Height));
        var vpTL = MapToMiniMap(worldViewTL);
        var vpBR = MapToMiniMap(worldViewBR);
        float vx = Math.Clamp(vpTL.X, mx, mx + MiniMapWidth);
        float vy = Math.Clamp(vpTL.Y, my, my + MiniMapHeight);
        float vw = Math.Max(vpBR.X - vpTL.X, 3);
        float vh = Math.Max(vpBR.Y - vpTL.Y, 3);

        using var vpPaint = new SKPaint
        {
            Color = new SKColor(255, 80, 80, 100),
            IsAntialias = true
        };
        canvas.DrawRect(vx, vy, vw, vh, vpPaint);
    }

    private SKRect ComputeWorldBounds()
    {
        if (_layoutCache.Count == 0)
            return new SKRect(0, 0, 800, 600);

        float minX = float.MaxValue, minY = float.MaxValue;
        float maxX = float.MinValue, maxY = float.MinValue;

        foreach (var layout in _layoutCache.Values)
        {
            var b = layout.Bounds;
            if (b.Left < minX) minX = b.Left;
            if (b.Top < minY) minY = b.Top;
            if (b.Right > maxX) maxX = b.Right;
            if (b.Bottom > maxY) maxY = b.Bottom;
        }

        if (minX == maxX) maxX += 100;
        if (minY == maxY) maxY += 100;

        return new SKRect(minX, minY, maxX, maxY);
    }

    // ── Hit testing ──

    private enum HitTarget { None, InputPort, OutputPort, NodeBody, Edge }

    private (HitTarget Target, string? NodeId, object? Data) HitTest(SKPoint screenPt)
    {
        var worldPt = ScreenToWorld(screenPt);
        float portHitRadius = PortRadius * 2;

        // Test ports (priority)
        foreach (var (nodeId, layout) in _layoutCache)
        {
            float dInput = SKPoint.Distance(worldPt, layout.InputPort);
            if (dInput <= portHitRadius)
                return (HitTarget.InputPort, nodeId, null);

            float dOutput = SKPoint.Distance(worldPt, layout.OutputPort);
            if (dOutput <= portHitRadius)
                return (HitTarget.OutputPort, nodeId, null);
        }

        // Test node bodies
        foreach (var (nodeId, layout) in _layoutCache)
        {
            if (layout.Bounds.Contains(worldPt))
                return (HitTarget.NodeBody, nodeId, null);
        }

        return (HitTarget.None, null, null);
    }

    // ── Mouse input ──

    protected override void OnMouseDown(MouseButtonEventArgs e)
    {
        base.OnMouseDown(e);
        Focus();

        var pos = e.GetPosition(this);
        var screenPt = new SKPoint((float)pos.X, (float)pos.Y);
        _lastMouse = screenPt;

        // Middle button — pan
        if (e.MiddleButton == MouseButtonState.Pressed)
        {
            _state = InteractionState.Panning;
            CaptureMouse();
            return;
        }

        if (e.LeftButton != MouseButtonState.Pressed) return;

        var hit = HitTest(screenPt);

        switch (hit.Target)
        {
            case HitTarget.OutputPort when hit.NodeId != null:
                _state = InteractionState.ConnectingPort;
                _connectFromNodeId = hit.NodeId;
                if (_layoutCache.TryGetValue(hit.NodeId, out var layout))
                    _connectFromPos = WorldToScreen(layout.OutputPort);
                _connectMousePos = screenPt;
                CaptureMouse();
                break;

            case HitTarget.NodeBody when hit.NodeId != null:
                var node = _nodes.FirstOrDefault(n => n.Id == hit.NodeId);
                if (node != null)
                {
                    // Ctrl+click for multi-select — single select for now
                    SelectedNode = node;
                    NodeSelected?.Invoke(node);
                }
                _state = InteractionState.DraggingNode;
                _dragNodeId = hit.NodeId;
                if (_layoutCache.TryGetValue(hit.NodeId, out var dragLayout))
                {
                    var worldPt = ScreenToWorld(screenPt);
                    _dragOffset = new SKPoint(worldPt.X - dragLayout.Bounds.Left, worldPt.Y - dragLayout.Bounds.Top);
                }
                CaptureMouse();
                break;

            default:
                SelectedNode = null;
                NodeSelected?.Invoke(null!);
                _state = InteractionState.Panning;
                CaptureMouse();
                break;
        }

        InvalidateVisual();
    }

    protected override void OnMouseMove(MouseEventArgs e)
    {
        base.OnMouseMove(e);
        var pos = e.GetPosition(this);
        var screenPt = new SKPoint((float)pos.X, (float)pos.Y);

        switch (_state)
        {
            case InteractionState.Panning:
                float dx = screenPt.X - _lastMouse.X;
                float dy = screenPt.Y - _lastMouse.Y;
                OffsetX += dx;
                OffsetY += dy;
                break;

            case InteractionState.DraggingNode when _dragNodeId != null:
                var worldPt = ScreenToWorld(screenPt);
                var nodeData = _nodes.FirstOrDefault(n => n.Id == _dragNodeId);
                if (nodeData != null)
                {
                    NodeMoved?.Invoke(nodeData, new SKPoint(worldPt.X - _dragOffset.X, worldPt.Y - _dragOffset.Y));
                    InvalidateVisual();
                }
                break;

            case InteractionState.ConnectingPort:
                _connectMousePos = screenPt;
                InvalidateVisual();
                break;
        }

        _lastMouse = screenPt;

        // Update cursor
        var hoverHit = HitTest(screenPt);
        Cursor = hoverHit.Target switch
        {
            HitTarget.OutputPort => Cursors.Cross,
            HitTarget.NodeBody => Cursors.SizeAll,
            _ => Cursors.Arrow
        };
    }

    protected override void OnMouseUp(MouseButtonEventArgs e)
    {
        base.OnMouseUp(e);

        if (_state == InteractionState.ConnectingPort && _connectFromNodeId != null)
        {
            var pos = e.GetPosition(this);
            var screenPt = new SKPoint((float)pos.X, (float)pos.Y);
            var hit = HitTest(screenPt);

            if (hit.Target == HitTarget.InputPort && hit.NodeId != null && hit.NodeId != _connectFromNodeId)
                PortsConnected?.Invoke(_connectFromNodeId, hit.NodeId);
            // else: released over empty space — tentative connection is silently discarded.
            // The dashed line is cleared on the next render pass (DrawTempConnection skips when state != ConnectingPort).
        }

        _state = InteractionState.Idle;
        _dragNodeId = null;
        _connectFromNodeId = null;
        ReleaseMouseCapture();
        InvalidateVisual();
    }

    protected override void OnMouseWheel(MouseWheelEventArgs e)
    {
        base.OnMouseWheel(e);

        var pos = e.GetPosition(this);
        float mouseX = (float)pos.X;
        float mouseY = (float)pos.Y;

        // Calculate world position under mouse before zoom
        var worldBefore = ScreenToWorld(new SKPoint(mouseX, mouseY));

        double oldScale = Scale;
        double newScale = Math.Clamp(oldScale * (e.Delta > 0 ? 1.15 : 1.0 / 1.15), 0.1, 5.0);
        Scale = newScale;

        // Keep world position under mouse stable
        var worldAfter = ScreenToWorld(new SKPoint(mouseX, mouseY));
        OffsetX += (worldAfter.X - worldBefore.X) * newScale;
        OffsetY += (worldAfter.Y - worldBefore.Y) * newScale;
    }

    // ── Drag & Drop ──

    protected override void OnDragOver(DragEventArgs e)
    {
        base.OnDragOver(e);
        if (e.Data.GetDataPresent("Photopipeline.Models.PluginInfo"))
            e.Effects = DragDropEffects.Copy;
        else
            e.Effects = DragDropEffects.None;
        e.Handled = true;
    }

    protected override void OnDrop(DragEventArgs e)
    {
        base.OnDrop(e);
        if (e.Data.GetData("Photopipeline.Models.PluginInfo") is PluginInfo plugin)
        {
            var pos = e.GetPosition(this);
            var worldPt = ScreenToWorld(new SKPoint((float)pos.X, (float)pos.Y));
            NodeDropped?.Invoke(plugin, worldPt);
        }
    }

    // ── Layout cache struct ──

    private readonly record struct NodeLayout(SKRect Bounds, SKPoint InputPort, SKPoint OutputPort);
}
