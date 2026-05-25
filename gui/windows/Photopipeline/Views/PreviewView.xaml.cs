using System.Windows.Controls;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class PreviewView : UserControl
{
    public PreviewView() => InitializeComponent();

    private void OnPixelInfoChanged(string info)
    {
        if (DataContext is PreviewViewModel vm)
            vm.PixelInfo = info;
    }
}
