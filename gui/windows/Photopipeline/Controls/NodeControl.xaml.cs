using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace Photopipeline.Controls;

public sealed partial class NodeControl : UserControl
{
    private bool _isDragging;
    private double _startX;
    private double _startY;

    public NodeControl()
    {
        this.InitializeComponent();
    }

    private void OnNodePointerPressed(object sender, PointerRoutedEventArgs e)
    {
        _isDragging = true;
        var point = e.GetCurrentPoint(this);
        _startX = point.Position.X;
        _startY = point.Position.Y;
    }

    private void OnNodePointerMoved(object sender, PointerRoutedEventArgs e)
    {
        if (!_isDragging) return;
    }

    private void OnNodePointerReleased(object sender, PointerRoutedEventArgs e)
    {
        _isDragging = false;
    }

    private void OnNodeRightTapped(object sender, RightTappedRoutedEventArgs e)
    {
    }
}
