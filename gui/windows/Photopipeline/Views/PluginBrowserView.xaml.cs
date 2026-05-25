using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using Photopipeline.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class PluginBrowserView : UserControl
{
    private Point _dragStart;

    public PluginBrowserView()
    {
        InitializeComponent();
        DataContextChanged += (_, _) => RegenerateParameterControls();
    }

    private PluginBrowserViewModel? Vm => DataContext as PluginBrowserViewModel;

    private void OnPluginListMouseMove(object sender, MouseEventArgs e)
    {
        if (e.LeftButton != MouseButtonState.Pressed) return;
        if (sender is not ListBox listBox) return;

        var pos = e.GetPosition(listBox);
        if (Math.Abs(pos.X - _dragStart.X) < SystemParameters.MinimumHorizontalDragDistance &&
            Math.Abs(pos.Y - _dragStart.Y) < SystemParameters.MinimumVerticalDragDistance)
            return;

        if (listBox.SelectedItem is not PluginInfo plugin) return;

        DragDrop.DoDragDrop(listBox, plugin, DragDropEffects.Copy);
    }

    private void OnPluginListPreviewMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        if (sender is ListBox listBox)
            _dragStart = e.GetPosition(listBox);
    }

    private void OnPluginSelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        RegenerateParameterControls();
    }

    private void OnResetParameters(object sender, RoutedEventArgs e)
    {
        if (Vm?.SelectedPlugin is null) return;
        Vm.SelectPluginCommand.Execute(Vm.SelectedPlugin);
        RegenerateParameterControls();
    }

    public void RegenerateParameterControls()
    {
        PluginParamPanel.Children.Clear();

        var plugin = Vm?.SelectedPlugin;
        if (plugin is null) return;

        foreach (var kvp in plugin.ParameterSchema)
        {
            var schema = kvp.Value as Dictionary<string, object>
                ?? new Dictionary<string, object> { ["type"] = kvp.Value?.ToString() ?? "string" };

            var values = new Dictionary<string, object>(Vm!.CurrentParameters);
            var control = ParameterControlFactory.CreateControl(kvp.Key, schema, values,
                (key, val) =>
                {
                    Vm.CurrentParameters[key] = val;
                });

            PluginParamPanel.Children.Add(control);
        }
    }
}
