using SkiaSharp;
using SkiaSharp.Views.Desktop;
using SkiaSharp.Views.WPF;
using System.Windows;
using System.Windows.Input;

namespace Photopipeline.Controls;

public sealed class SkiaPreviewCanvas : SKElement
{
    // ── Dependency Properties ──

    public static readonly DependencyProperty BeforeBitmapProperty =
        DependencyProperty.Register(nameof(BeforeBitmap), typeof(SKBitmap), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender, OnBeforeBitmapChanged));

    public static readonly DependencyProperty AfterBitmapProperty =
        DependencyProperty.Register(nameof(AfterBitmap), typeof(SKBitmap), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender, OnAfterBitmapChanged));

    public static readonly DependencyProperty ZoomLevelProperty =
        DependencyProperty.Register(nameof(ZoomLevel), typeof(double), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(1.0, FrameworkPropertyMetadataOptions.AffectsRender, OnZoomChanged));

    public static readonly DependencyProperty SplitPositionProperty =
        DependencyProperty.Register(nameof(SplitPosition), typeof(double), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(0.5, FrameworkPropertyMetadataOptions.AffectsRender, OnSplitChanged));

    public static readonly DependencyProperty IsSplitViewProperty =
        DependencyProperty.Register(nameof(IsSplitView), typeof(bool), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(true, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty IsFitToWindowProperty =
        DependencyProperty.Register(nameof(IsFitToWindow), typeof(bool), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(true, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty PanOffsetProperty =
        DependencyProperty.Register(nameof(PanOffset), typeof(SKPoint), typeof(SkiaPreviewCanvas),
            new FrameworkPropertyMetadata(SKPoint.Empty, FrameworkPropertyMetadataOptions.AffectsRender));

    // ── CLR wrappers ──

    public SKBitmap? BeforeBitmap
    {
        get => (SKBitmap?)GetValue(BeforeBitmapProperty);
        set => SetValue(BeforeBitmapProperty, value);
    }

    public SKBitmap? AfterBitmap
    {
        get => (SKBitmap?)GetValue(AfterBitmapProperty);
        set => SetValue(AfterBitmapProperty, value);
    }

    public double ZoomLevel
    {
        get => (double)GetValue(ZoomLevelProperty);
        set => SetValue(ZoomLevelProperty, value);
    }

    public double SplitPosition
    {
        get => (double)GetValue(SplitPositionProperty);
        set => SetValue(SplitPositionProperty, Math.Clamp(value, 0.05, 0.95));
    }

    public bool IsSplitView
    {
        get => (bool)GetValue(IsSplitViewProperty);
        set => SetValue(IsSplitViewProperty, value);
    }

    public bool IsFitToWindow
    {
        get => (bool)GetValue(IsFitToWindowProperty);
        set => SetValue(IsFitToWindowProperty, value);
    }

    public SKPoint PanOffset
    {
        get => (SKPoint)GetValue(PanOffsetProperty);
        set => SetValue(PanOffsetProperty, value);
    }

    // ── Events ──

    public event Action<string>? PixelInfoChanged;

    // ── Interaction state ──

    private bool _isPanning;
    private bool _isDraggingSplit;
    private SKPoint _lastMouse;
    private DateTime _lastSampleTime;
    private static readonly TimeSpan SampleThrottle = TimeSpan.FromMilliseconds(50);

    private const double MinZoom = 0.0625;
    private const double MaxZoom = 32.0;
    private const float SplitHandleWidth = 6;

    public SkiaPreviewCanvas()
    {
        Focusable = true;
    }

    // ── Property change callbacks ──

    private static void OnBeforeBitmapChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (e.OldValue is SKBitmap oldBmp && oldBmp != e.NewValue)
        {
            var canvas = (SkiaPreviewCanvas)d;
            if (!ReferenceEquals(oldBmp, canvas.AfterBitmap))
                oldBmp.Dispose();
        }
        ((SkiaPreviewCanvas)d).InvalidateVisual();
    }

    private static void OnAfterBitmapChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (e.OldValue is SKBitmap oldBmp && oldBmp != e.NewValue)
        {
            var canvas = (SkiaPreviewCanvas)d;
            if (!ReferenceEquals(oldBmp, canvas.BeforeBitmap))
                oldBmp.Dispose();
        }
        ((SkiaPreviewCanvas)d).InvalidateVisual();
    }

    private static void OnZoomChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var newVal = (double)e.NewValue;
        if (newVal < MinZoom || newVal > MaxZoom)
            ((SkiaPreviewCanvas)d).ZoomLevel = Math.Clamp(newVal, MinZoom, MaxZoom);
        else
            ((SkiaPreviewCanvas)d).InvalidateVisual();
    }

    private static void OnSplitChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        ((SkiaPreviewCanvas)d).InvalidateVisual();
    }

    // ── Rendering ──

    protected override void OnPaintSurface(SKPaintSurfaceEventArgs e)
    {
        var canvas = e.Surface.Canvas;
        var info = e.Info;
        canvas.Clear(SKColors.Black);

        var before = BeforeBitmap;
        var after = AfterBitmap;

        if (before == null && after == null)
        {
            DrawEmptyState(canvas, info);
            return;
        }

        if (!IsSplitView)
        {
            DrawSingle(canvas, info, after ?? before!);
        }
        else
        {
            DrawSplit(canvas, info, before, after);
        }
    }

    private void DrawEmptyState(SKCanvas canvas, SKImageInfo info)
    {
        using var stroke = new SKPaint
        {
            Color = new SKColor(80, 80, 80),
            Style = SKPaintStyle.Stroke,
            StrokeWidth = 2,
            PathEffect = SKPathEffect.CreateDash(new[] { 8f, 4f }, 0)
        };
        var rect = new SKRect(40, 40, info.Width - 40, info.Height - 40);
        canvas.DrawRoundRect(rect, 8, 8, stroke);

        using var textPaint = new SKPaint
        {
            Color = new SKColor(120, 120, 120),
            TextSize = 16,
            IsAntialias = true,
            TextAlign = SKTextAlign.Center
        };
        canvas.DrawText("No Image", info.Width / 2f, info.Height / 2f + 6, textPaint);
    }

    private void DrawSingle(SKCanvas canvas, SKImageInfo info, SKBitmap bitmap)
    {
        if (bitmap == null || bitmap.Handle == IntPtr.Zero) return;

        var matrix = ComputeDisplayMatrix(info, bitmap.Width, bitmap.Height);
        canvas.SetMatrix(matrix);
        canvas.DrawBitmap(bitmap, 0, 0);
        canvas.ResetMatrix();
    }

    private void DrawSplit(SKCanvas canvas, SKImageInfo info, SKBitmap? before, SKBitmap? after)
    {
        float splitX = (float)(info.Width * SplitPosition);

        // Left side (Before)
        if (before != null && before.Handle != IntPtr.Zero)
        {
            canvas.Save();
            canvas.ClipRect(new SKRect(0, 0, splitX, info.Height));
            var matrix = ComputeDisplayMatrix(info, before.Width, before.Height);
            canvas.SetMatrix(matrix);
            canvas.DrawBitmap(before, 0, 0);
            canvas.Restore();
        }

        // Right side (After)
        if (after != null && after.Handle != IntPtr.Zero)
        {
            canvas.Save();
            canvas.ClipRect(new SKRect(splitX, 0, info.Width, info.Height));
            var matrix = ComputeDisplayMatrix(info, after.Width, after.Height);
            canvas.SetMatrix(matrix);
            canvas.DrawBitmap(after, 0, 0);
            canvas.Restore();
        }

        // Divider line
        using var linePaint = new SKPaint
        {
            Color = SKColors.White,
            StrokeWidth = 2,
            IsAntialias = true
        };
        canvas.DrawLine(splitX, 0, splitX, info.Height, linePaint);

        // Handle
        float handleTop = info.Height / 2f - 24;
        using var handleBg = new SKPaint { Color = new SKColor(255, 255, 255, 200), IsAntialias = true };
        var handleRect = new SKRect(splitX - SplitHandleWidth / 2, handleTop,
            splitX + SplitHandleWidth / 2, handleTop + 48);
        canvas.DrawRoundRect(handleRect, 3, 3, handleBg);

        // Labels
        using var labelPaint = new SKPaint
        {
            Color = new SKColor(255, 255, 255, 180),
            TextSize = 13,
            IsAntialias = true,
            TextAlign = SKTextAlign.Center
        };
        canvas.DrawText("Before", info.Width / 4f, 28, labelPaint);
        canvas.DrawText("After", info.Width * 3f / 4f, 28, labelPaint);
    }

    private SKMatrix ComputeDisplayMatrix(SKImageInfo canvasInfo, int bitmapW, int bitmapH)
    {
        float zoom = (float)ZoomLevel;

        if (IsFitToWindow)
        {
            float scaleX = (float)canvasInfo.Width / bitmapW;
            float scaleY = (float)canvasInfo.Height / bitmapH;
            zoom = Math.Min(scaleX, scaleY);
        }

        var pan = PanOffset;
        float cx = canvasInfo.Width / 2f + (float)pan.X;
        float cy = canvasInfo.Height / 2f + (float)pan.Y;

        return SKMatrix.CreateScaleTranslation(zoom, zoom, cx - bitmapW * zoom / 2f, cy - bitmapH * zoom / 2f);
    }

    // ── Mouse input ──

    protected override void OnMouseWheel(MouseWheelEventArgs e)
    {
        base.OnMouseWheel(e);

        var pos = e.GetPosition(this);
        float mouseX = (float)pos.X;
        float mouseY = (float)pos.Y;

        double oldZoom = ZoomLevel;
        double delta = e.Delta > 0 ? 1.15 : 1.0 / 1.15;
        double newZoom = Math.Clamp(oldZoom * delta, MinZoom, MaxZoom);

        var pan = PanOffset;
        double newX = mouseX - (mouseX - pan.X) * (newZoom / oldZoom);
        double newY = mouseY - (mouseY - pan.Y) * (newZoom / oldZoom);

        IsFitToWindow = false;
        ZoomLevel = newZoom;
        PanOffset = new SKPoint((float)newX, (float)newY);
    }

    protected override void OnMouseDown(MouseButtonEventArgs e)
    {
        base.OnMouseDown(e);
        Focus();
        var pos = e.GetPosition(this);

        if (IsSplitView && e.LeftButton == MouseButtonState.Pressed)
        {
            float splitX = (float)(ActualWidth * SplitPosition);
            if (Math.Abs(pos.X - splitX) < SplitHandleWidth + 4)
            {
                _isDraggingSplit = true;
                CaptureMouse();
                return;
            }
        }

        if (e.LeftButton == MouseButtonState.Pressed)
        {
            _isPanning = true;
            _lastMouse = new SKPoint((float)pos.X, (float)pos.Y);
            CaptureMouse();
        }
    }

    protected override void OnMouseMove(MouseEventArgs e)
    {
        base.OnMouseMove(e);
        var pos = e.GetPosition(this);
        float mx = (float)pos.X;
        float my = (float)pos.Y;

        if (_isDraggingSplit)
        {
            SplitPosition = mx / ActualWidth;
            return;
        }

        if (_isPanning)
        {
            float dx = mx - _lastMouse.X;
            float dy = my - _lastMouse.Y;
            var pan = PanOffset;
            PanOffset = new SKPoint(pan.X + dx, pan.Y + dy);
            _lastMouse = new SKPoint(mx, my);
            return;
        }

        // Pixel sampling on hover (throttled to ~20 samples/sec)
        if (DateTime.UtcNow - _lastSampleTime >= SampleThrottle)
        {
            _lastSampleTime = DateTime.UtcNow;
            var bmp = IsSplitView ? BeforeBitmap : (AfterBitmap ?? BeforeBitmap);
            if (bmp != null && bmp.Handle != IntPtr.Zero)
            {
                var info = new SKImageInfo((int)ActualWidth, (int)ActualHeight);
                var matrix = ComputeDisplayMatrix(info, bmp.Width, bmp.Height);
                matrix.TryInvert(out var inv);

                var imgPt = inv.MapPoint(mx, my);
                int px = (int)imgPt.X;
                int py = (int)imgPt.Y;

                if (px >= 0 && px < bmp.Width && py >= 0 && py < bmp.Height)
                {
                    var color = bmp.GetPixel(px, py);
                    PixelInfoChanged?.Invoke(
                        $"X:{px} Y:{py} | R:{color.Red} G:{color.Green} B:{color.Blue}");
                }
                else
                {
                    PixelInfoChanged?.Invoke(string.Empty);
                }
            }
        }
    }

    protected override void OnMouseUp(MouseButtonEventArgs e)
    {
        base.OnMouseUp(e);
        _isPanning = false;
        _isDraggingSplit = false;
        ReleaseMouseCapture();
    }

    // Split handle cursor
    protected override void OnMouseEnter(MouseEventArgs e)
    {
        base.OnMouseEnter(e);
        if (IsSplitView)
        {
            var pos = e.GetPosition(this);
            float splitX = (float)(ActualWidth * SplitPosition);
            if (Math.Abs(pos.X - splitX) < SplitHandleWidth + 4)
                Cursor = Cursors.SizeWE;
        }
    }

    protected override void OnMouseLeave(MouseEventArgs e)
    {
        base.OnMouseLeave(e);
        Cursor = Cursors.Arrow;
        PixelInfoChanged?.Invoke(string.Empty);
    }
}
