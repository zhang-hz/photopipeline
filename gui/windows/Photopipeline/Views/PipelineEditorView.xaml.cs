using System.Windows;
using System.Windows.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class PipelineEditorView : UserControl
{
    public PipelineEditorView()
    {
        InitializeComponent();
    }

    private void OnAddNodeClick(object sender, RoutedEventArgs e)
    {
        if (DataContext is not PipelineEditorViewModel pipelineVm)
            return;

        var window = Window.GetWindow(this);
        if (window?.DataContext is not MainViewModel mainVm)
            return;

        if (mainVm.AvailablePlugins.Count == 0)
            return;

        var menu = new ContextMenu();
        foreach (var plugin in mainVm.AvailablePlugins)
        {
            var item = new MenuItem { Header = $"{plugin.Name} ({plugin.Category})" };
            var captured = plugin;
            item.Click += (_, _) => pipelineVm.AddNodeCommand.Execute(captured);
            menu.Items.Add(item);
        }
        menu.IsOpen = true;
    }
}
