using System.Windows.Controls;
using Photopipeline.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;
using SkiaSharp;

namespace Photopipeline.Views;

public partial class PipelineEditorView : UserControl
{
    public PipelineEditorView() => InitializeComponent();

    private PipelineEditorViewModel? Vm => DataContext as PipelineEditorViewModel;

    private void OnNodeDropped(PluginInfo plugin, SKPoint worldPos)
    {
        Vm?.AddNodeAt(plugin, worldPos.X, worldPos.Y);
    }

    private void OnNodeSelected(PipelineNode node)
    {
        if (Vm is not null)
        {
            Vm.SelectedNode = node;
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
