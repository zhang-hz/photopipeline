using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;

namespace Photopipeline.Controls;

public partial class BeforeAfterView : UserControl
{
    private bool _isDragging;
    private double _startMouseX;
    private double _startBeforeWidth;

    public BeforeAfterView()
    {
        InitializeComponent();
    }

    private void OnSplitHandleMouseDown(object sender, MouseButtonEventArgs e)
    {
        _isDragging = true;
        _startMouseX = e.GetPosition(RootLayout).X;
        _startBeforeWidth = BeforeColumn.ActualWidth;
        SplitHandle.CaptureMouse();
    }

    private void OnSplitHandleMouseMove(object sender, MouseEventArgs e)
    {
        if (!_isDragging || BeforeColumn is null || AfterColumn is null)
            return;

        var currentX = e.GetPosition(RootLayout).X;
        var delta = currentX - _startMouseX;
        var beforeWidth = _startBeforeWidth + delta;
        var totalWidth = RootLayout.ActualWidth;

        if (totalWidth > 0 && beforeWidth > 50 && beforeWidth < totalWidth - 50)
        {
            BeforeColumn.Width = new GridLength(beforeWidth / totalWidth, GridUnitType.Star);
            AfterColumn.Width = new GridLength(1.0 - beforeWidth / totalWidth, GridUnitType.Star);
        }
    }

    private void OnSplitHandleMouseUp(object sender, MouseButtonEventArgs e)
    {
        _isDragging = false;
        SplitHandle.ReleaseMouseCapture();
    }
}
