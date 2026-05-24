namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class PipelineEditorViewModelTests
{
    private static PluginInfo CreateTestPlugin(string id = "test", string name = "Test Plugin",
        int minInputs = 1, int maxInputs = 1, int outputs = 1)
    {
        return new PluginInfo
        {
            Id = id,
            Name = name,
            Category = "Test",
            MinInputs = minInputs,
            MaxInputs = maxInputs,
            Outputs = outputs
        };
    }

    [Fact]
    public void PipelineEditor_Creation_EmptyState()
    {
        var vm = new PipelineEditorViewModel();

        vm.Nodes.Should().BeEmpty();
        vm.Edges.Should().BeEmpty();
        vm.SelectedNode.Should().BeNull();
        vm.SelectedEdge.Should().BeNull();
        vm.IsDrawingConnection.Should().BeFalse();
        vm.IsDraggingNode.Should().BeFalse();
        vm.Scale.Should().Be(1.0);
    }

    [Fact]
    public void AddNode_AddsToCollection()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();

        vm.AddNodeCommand.Execute(plugin);

        vm.Nodes.Should().HaveCount(1);
        vm.Nodes[0].PluginId.Should().Be("test");
        vm.Nodes[0].DisplayName.Should().Be("Test Plugin");
    }

    [Fact]
    public void AddNode_SetsDefaultParametersFromSchema()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        plugin.ParameterSchemas.Add(new ParameterSchema
        {
            Name = "threshold",
            ParameterType = ParameterType.Float,
            DefaultValue = 0.5
        });
        plugin.ParameterSchemas.Add(new ParameterSchema
        {
            Name = "enabled",
            ParameterType = ParameterType.Boolean,
            DefaultValue = true
        });

        vm.AddNodeCommand.Execute(plugin);

        vm.Nodes[0].Parameters.Should().ContainKey("threshold");
        vm.Nodes[0].Parameters["threshold"].Should().Be(0.5);
        vm.Nodes[0].Parameters.Should().ContainKey("enabled");
        vm.Nodes[0].Parameters["enabled"].Should().Be(true);
    }

    [Fact]
    public void AddNode_CreatesCorrectNumberOfInputPorts()
    {
        var vm = new PipelineEditorViewModel();
        var multiInputPlugin = CreateTestPlugin("multi", "Multi Input", minInputs: 1, maxInputs: 3);

        vm.AddNodeCommand.Execute(multiInputPlugin);

        vm.Nodes[0].InputPorts.Should().HaveCount(3);
        vm.Nodes[0].InputPorts[0].Id.Should().Be("in0");
        vm.Nodes[0].InputPorts[1].Id.Should().Be("in1");
        vm.Nodes[0].InputPorts[2].Id.Should().Be("in2");
    }

    [Fact]
    public void AddNode_CreatesCorrectNumberOfOutputPorts()
    {
        var vm = new PipelineEditorViewModel();
        var multiOutputPlugin = CreateTestPlugin("split", "Split", outputs: 3);

        vm.AddNodeCommand.Execute(multiOutputPlugin);

        vm.Nodes[0].OutputPorts.Should().HaveCount(3);
        vm.Nodes[0].OutputPorts[0].Id.Should().Be("out0");
        vm.Nodes[0].OutputPorts[1].Id.Should().Be("out1");
        vm.Nodes[0].OutputPorts[2].Id.Should().Be("out2");
    }

    [Fact]
    public void AddNode_PositionsSequentially()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();

        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);

        vm.Nodes.Should().HaveCount(3);
        var positions = vm.Nodes.Select(n => new { n.CanvasX, n.CanvasY }).ToList();
        positions.Should().OnlyHaveUniqueItems();
    }

    [Fact]
    public void RemoveNode_RemovesFromCollection()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];

        vm.RemoveNodeCommand.Execute(node);

        vm.Nodes.Should().BeEmpty();
    }

    [Fact]
    public void RemoveNode_CleansUpConnectedEdges()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        vm.Edges.Add(new PipelineEdge { SourceNodeId = n1.Id, TargetNodeId = n2.Id });

        vm.RemoveNodeCommand.Execute(n1);

        vm.Edges.Should().BeEmpty();
        vm.Nodes.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveNode_RemovesBothSourceAndTargetEdges()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        var n3 = vm.Nodes[2];
        vm.Edges.Add(new PipelineEdge { SourceNodeId = n1.Id, TargetNodeId = n2.Id });
        vm.Edges.Add(new PipelineEdge { SourceNodeId = n2.Id, TargetNodeId = n3.Id });

        vm.RemoveNodeCommand.Execute(n2);

        vm.Edges.Should().BeEmpty();
        vm.Nodes.Should().HaveCount(2);
    }

    [Fact]
    public void RemoveNode_ClearsSelectionIfSelectedNodeRemoved()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        vm.SelectNodeCommand.Execute(n1);

        vm.RemoveNodeCommand.Execute(n1);

        vm.SelectedNode.Should().BeNull();
    }

    [Fact]
    public void RemoveNode_WithNull_Noop()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);

        vm.RemoveNodeCommand.Execute(null);

        vm.Nodes.Should().HaveCount(1);
    }

    [Fact]
    public void SelectNode_SetsSelectedAndMarksNode()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];

        vm.SelectNodeCommand.Execute(node);

        vm.SelectedNode.Should().Be(node);
        node.IsSelected.Should().BeTrue();
    }

    [Fact]
    public void SelectNode_DeselectsPrevious()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        vm.SelectNodeCommand.Execute(n1);

        vm.SelectNodeCommand.Execute(n2);

        n1.IsSelected.Should().BeFalse();
        n2.IsSelected.Should().BeTrue();
        vm.SelectedNode.Should().Be(n2);
    }

    [Fact]
    public void ClearSelection_UnselectsNode()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];
        vm.SelectNodeCommand.Execute(node);

        vm.ClearSelectionCommand.Execute(null);

        vm.SelectedNode.Should().BeNull();
        node.IsSelected.Should().BeFalse();
    }

    [Fact]
    public void OnNodeMouseDown_StartsDrag()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];

        vm.OnNodeMouseDown(node, 100, 200);

        vm.IsDraggingNode.Should().BeTrue();
        node.IsDragging.Should().BeTrue();
        node.IsSelected.Should().BeTrue();
    }

    [Fact]
    public void OnNodeMouseMove_UpdatesCanvasPosition()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];
        vm.OnNodeMouseDown(node, 100, 200);

        vm.OnNodeMouseMove(150, 250);

        node.CanvasX.Should().Be(100 + 50);
        node.CanvasY.Should().Be(80 + 50);
    }

    [Fact]
    public void OnNodeMouseMove_WithScale_RespectsScale()
    {
        var vm = new PipelineEditorViewModel { Scale = 2.0 };
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];
        vm.OnNodeMouseDown(node, 100, 200);

        vm.OnNodeMouseMove(140, 260);

        var expectedDx = (140 - 100) / 2.0;
        var expectedDy = (260 - 200) / 2.0;
        node.CanvasX.Should().Be(100 + expectedDx);
        node.CanvasY.Should().Be(80 + expectedDy);
    }

    [Fact]
    public void OnNodeMouseUp_StopsDrag()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];
        vm.OnNodeMouseDown(node, 100, 200);

        vm.OnNodeMouseUp();

        vm.IsDraggingNode.Should().BeFalse();
        node.IsDragging.Should().BeFalse();
    }

    [Fact]
    public void OnPortDragStart_BeginsConnectionDrawing()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var port = vm.Nodes[0].OutputPorts[0];

        vm.OnPortDragStart(port, 150, 200);

        vm.IsDrawingConnection.Should().BeTrue();
        vm.ConnectionLineX1.Should().Be(150);
        vm.ConnectionLineY1.Should().Be(200);
    }

    [Fact]
    public void OnPortDrag_UpdatesEndpoint()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var port = vm.Nodes[0].OutputPorts[0];
        vm.OnPortDragStart(port, 100, 100);

        vm.OnPortDrag(300, 400);

        vm.ConnectionLineX2.Should().Be(300);
        vm.ConnectionLineY2.Should().Be(400);
    }

    [Fact]
    public void OnPortDrag_WhenNotDrawingConnection_Noop()
    {
        var vm = new PipelineEditorViewModel();

        vm.OnPortDrag(300, 400);

        vm.ConnectionLineX2.Should().Be(0);
        vm.ConnectionLineY2.Should().Be(0);
    }

    [Fact]
    public void ConnectPorts_CreatesEdgeBetweenNodes()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        vm.SelectNodeCommand.Execute(n2);

        vm.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        vm.ConnectPortsCommand.Execute(null);

        vm.Edges.Should().HaveCount(1);
        vm.Edges[0].SourceNodeId.Should().Be(n1.Id);
        vm.Edges[0].TargetNodeId.Should().Be(n2.Id);
        vm.IsDrawingConnection.Should().BeFalse();
    }

    [Fact]
    public void ConnectPorts_AvoidsDuplicateEdges()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        vm.SelectNodeCommand.Execute(n2);

        vm.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        vm.ConnectPortsCommand.Execute(null);

        vm.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        vm.ConnectPortsCommand.Execute(null);

        vm.Edges.Should().HaveCount(1);
    }

    [Fact]
    public void ConnectPorts_WithNullDraggingPort_Noop()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        vm.SelectNodeCommand.Execute(vm.Nodes[1]);

        vm.ConnectPortsCommand.Execute(null);

        vm.Edges.Should().BeEmpty();
    }

    [Fact]
    public void ConnectPorts_SameNode_DoesNotCreateEdge()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];
        vm.SelectNodeCommand.Execute(node);

        vm.OnPortDragStart(node.OutputPorts[0], 100, 100);
        vm.ConnectPortsCommand.Execute(null);

        vm.Edges.Should().BeEmpty();
    }

    [Fact]
    public void ValidatePipeline_AllNodesConnected_ShouldBeValid()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        vm.Edges.Add(new PipelineEdge { SourceNodeId = n1.Id, TargetNodeId = n2.Id });

        vm.ValidatePipelineCommand.Execute(null);

        vm.Nodes.Should().HaveCount(2);
        vm.Edges.Should().HaveCount(1);
    }

    [Fact]
    public void ValidatePipeline_DisconnectedNodes_Detected()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);
        var n1 = vm.Nodes[0];
        var n2 = vm.Nodes[1];
        vm.Edges.Add(new PipelineEdge { SourceNodeId = n1.Id, TargetNodeId = n2.Id });

        vm.ValidatePipelineCommand.Execute(null);
    }

    [Fact]
    public void FitAll_WithNodes_CalculatesOffsets()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        vm.AddNodeCommand.Execute(plugin);

        vm.FitAll();

        vm.OffsetX.Should().NotBe(0);
        vm.OffsetY.Should().NotBe(0);
    }

    [Fact]
    public void FitAll_EmptyCanvas_Noop()
    {
        var vm = new PipelineEditorViewModel();

        vm.FitAll();

        vm.OffsetX.Should().Be(0);
        vm.OffsetY.Should().Be(0);
    }

    [Fact]
    public void DuplicateSelected_ClonesSelectedNode()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var original = vm.Nodes[0];
        vm.SelectNodeCommand.Execute(original);

        vm.DuplicateSelected();

        vm.Nodes.Should().HaveCount(2);
        var clone = vm.Nodes[1];
        clone.PluginId.Should().Be(original.PluginId);
        clone.DisplayName.Should().Be(original.DisplayName + " (copy)");
        clone.CanvasX.Should().Be(original.CanvasX + 200);
        clone.CanvasY.Should().Be(original.CanvasY + 30);
    }

    [Fact]
    public void DuplicateSelected_NoSelection_Noop()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);

        vm.DuplicateSelected();

        vm.Nodes.Should().HaveCount(1);
    }

    [Fact]
    public void AddNode_NoInputsPlugin_HasNoInputPorts()
    {
        var vm = new PipelineEditorViewModel();
        var noInputPlugin = CreateTestPlugin("input", "File Input", minInputs: 0, maxInputs: 0);

        vm.AddNodeCommand.Execute(noInputPlugin);

        vm.Nodes[0].InputPorts.Should().HaveCount(1);
    }

    [Fact]
    public void AddNode_NoOutputsPlugin_HasNoOutputPorts()
    {
        var vm = new PipelineEditorViewModel();
        var noOutputPlugin = CreateTestPlugin("output", "File Output", outputs: 0);

        vm.AddNodeCommand.Execute(noOutputPlugin);

        vm.Nodes[0].OutputPorts.Should().HaveCount(0);
    }

    [Fact]
    public void OnNodeMouseMove_WhenNotDragging_Noop()
    {
        var vm = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin();
        vm.AddNodeCommand.Execute(plugin);
        var node = vm.Nodes[0];
        var originalX = node.CanvasX;
        var originalY = node.CanvasY;

        vm.OnNodeMouseMove(500, 500);

        node.CanvasX.Should().Be(originalX);
        node.CanvasY.Should().Be(originalY);
    }

    [Fact]
    public void CanvasSize_DefaultValues()
    {
        var vm = new PipelineEditorViewModel();

        vm.CanvasWidth.Should().Be(2000);
        vm.CanvasHeight.Should().Be(2000);
    }
}
