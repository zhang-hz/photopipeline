using System.Windows.Controls;
using Photopipeline.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;
using SkiaSharp;

namespace Photopipeline.Views;

public partial class PipelineEditorView : UserControl
{
    private PipelineEditorViewModel? _vm;

    public PipelineEditorView()
    {
        InitializeComponent();
        DataContextChanged += OnDataContextChanged;
    }

    private void OnDataContextChanged(object sender, System.Windows.DependencyPropertyChangedEventArgs e)
    {
        if (e.OldValue is PipelineEditorViewModel oldVm)
            oldVm.PreviewUpdateRequested -= OnPreviewUpdateRequested;
        if (e.NewValue is PipelineEditorViewModel newVm)
        {
            _vm = newVm;
            newVm.PreviewUpdateRequested += OnPreviewUpdateRequested;
            // Watch for SelectedNode changes that may come from outside (e.g., DAG canvas click)
            newVm.PropertyChanged += (s, args) =>
            {
                if (args.PropertyName == nameof(PipelineEditorViewModel.SelectedNode) &&
                    newVm.SelectedNode is { } node)
                {
                    RegenerateParameterControls(node);
                }
            };
        }
    }

    private PipelineEditorViewModel? Vm => _vm ?? DataContext as PipelineEditorViewModel;

    private void OnPreviewUpdateRequested()
    {
        // Forward to MainViewModel so it can reload the preview
        if (System.Windows.Application.Current.MainWindow is MainWindow mw &&
            mw.DataContext is MainViewModel mvm)
        {
            var img = mvm.Filmstrip.SelectedImage;
            if (img != null && Vm != null && Vm.IsPipelineValid)
                _ = mvm.Preview.ProcessPreviewAsync(img, Vm.CurrentPipeline, Vm.PipelineId);
        }
    }

    private void OnNodeDropped(PluginInfo plugin, SKPoint worldPos)
    {
        Vm?.AddNodeAt(plugin, worldPos.X, worldPos.Y);
    }

    private void OnNodeSelected(PipelineNode? node)
    {
        if (Vm is not null)
        {
            Vm.SelectedNode = node;
            ParameterPanel.Children.Clear();
            if (node is not null)
                RegenerateParameterControls(node);
        }
    }

    private void OnPortsConnected(string fromId, string toId)
    {
        Vm?.ConnectNodesCommand.Execute((fromId, toId));
    }

    private void OnNodeMoved(PipelineNode node, SKPoint pos)
    {
        Vm?.UpdateNodePosition(node.Id, pos.X, pos.Y);
    }

    private void RegenerateParameterControls(PipelineNode node)
    {
        ParameterPanel.Children.Clear();

        if (node.Params.Count == 0)
        {
            ParameterPanel.Children.Add(new TextBlock
            {
                Text = "No parameters",
                FontSize = 11,
                Foreground = System.Windows.Media.Brushes.Gray
            });
            return;
        }

        foreach (var kvp in node.Params)
        {
            var schema = new Dictionary<string, object>
            {
                ["type"] = kvp.Value switch
                {
                    bool => "bool",
                    int or long => "int",
                    float or double => "float",
                    _ => "string"
                }
            };
            var values = new Dictionary<string, object> { [kvp.Key] = kvp.Value };
            var control = ParameterControlFactory.CreateControl(kvp.Key, schema, values,
                (key, val) =>
                {
                    Vm?.UpdateNodeParameterCommand.Execute((node.Id, key, val));
                });
            ParameterPanel.Children.Add(control);
        }
    }
}
