using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using Photopipeline.Models;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class PluginPanelView : UserControl
{
    public PluginPanelView()
    {
        InitializeComponent();
    }

    private void OnPluginClicked(object sender, MouseButtonEventArgs e)
    {
        if (sender is FrameworkElement fe && fe.DataContext is PluginInfo plugin)
        {
            if (DataContext is PluginPanelViewModel vm)
            {
                vm.SelectPluginCommand.Execute(plugin);
            }
        }
    }
}
