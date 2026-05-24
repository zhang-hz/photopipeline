using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace Photopipeline.Controls;

public sealed partial class BeforeAfterView : UserControl
{
    public BeforeAfterView()
    {
        this.InitializeComponent();
    }

    private void OnSplitHandleDrag(object sender, ManipulationDeltaRoutedEventArgs e)
    {
        if (BeforeColumn == null || AfterColumn == null)
            return;

        var delta = e.Delta.Translation.X;
        var beforeWidth = BeforeColumn.ActualWidth + delta;
        var afterWidth = AfterColumn.ActualWidth - delta;
        var totalWidth = beforeWidth + afterWidth;

        if (totalWidth > 0)
        {
            BeforeColumn.Width = new GridLength(beforeWidth / totalWidth, GridUnitType.Star);
            AfterColumn.Width = new GridLength(afterWidth / totalWidth, GridUnitType.Star);
        }
    }
}
