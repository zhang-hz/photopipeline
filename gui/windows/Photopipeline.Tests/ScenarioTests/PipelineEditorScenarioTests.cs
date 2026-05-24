using System.Collections.ObjectModel;
using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class PipelineEditorScenarioTests
{
    private static PluginInfo TestPlugin(string id, string name, string cat = "Basic",
        int minInputs = 1, int maxInputs = 1, int outputs = 1)
    {
        return new PluginInfo
        {
            Id = id, Name = name, Category = cat,
            Description = $"Test plugin {name}",
            MinInputs = minInputs, MaxInputs = maxInputs, Outputs = outputs
        };
    }

    // ═══ Initial state ═══
    [Fact]
    public void NewEditor_Nodes_Empty()
    {
        var h = new ViewModelTestHarness();
        Assert.Empty(h.PipelineEditor.Nodes);
    }

    [Fact]
    public void NewEditor_Edges_Empty()
    {
        var h = new ViewModelTestHarness();
        Assert.Empty(h.PipelineEditor.Edges);
    }

    [Fact]
    public void NewEditor_Scale_IsOne()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal(1.0, h.PipelineEditor.Scale);
    }

    [Fact]
    public void NewEditor_NotValid()
    {
        var h = new ViewModelTestHarness();
        Assert.False(h.PipelineEditor.IsPipelineValid);
    }

    // ═══ Add node ═══
    [Fact]
    public void AddNode_IncreasesNodeCount()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("test", "Test"));
        Assert.Single(h.PipelineEditor.Nodes);
    }

    [Fact]
    public void AddNode_SetsPluginId()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("grayscale", "Grayscale"));
        Assert.Equal("grayscale", h.PipelineEditor.Nodes[0].PluginId);
    }

    [Fact]
    public void AddNode_SetsDisplayName()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("inv", "Invert"));
        Assert.Equal("Invert", h.PipelineEditor.Nodes[0].DisplayName);
    }

    [Fact]
    public void AddNode_CreatesPorts()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("test", "Test"));
        Assert.NotEmpty(h.PipelineEditor.Nodes[0].InputPorts);
        Assert.NotEmpty(h.PipelineEditor.Nodes[0].OutputPorts);
    }

    [Fact]
    public void AddNode_MultipleNodes_PositionNonOverlapping()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 5; i++)
            h.PipelineEditor.AddNodeCommand.Execute(TestPlugin($"p{i}", $"Plugin {i}"));
        Assert.Equal(5, h.PipelineEditor.Nodes.Count);

        // Verify positions differ
        var positions = h.PipelineEditor.Nodes.Select(n => (n.CanvasX, n.CanvasY)).ToHashSet();
        Assert.Equal(5, positions.Count);
    }

    [Fact]
    public void AddNode_MultiInput_CreatesCorrectPortCount()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("mix", "Mixer", minInputs: 1, maxInputs: 3, outputs: 1));
        Assert.Equal(3, h.PipelineEditor.Nodes[0].InputPorts.Count);
        Assert.Single(h.PipelineEditor.Nodes[0].OutputPorts);
    }

    [Fact]
    public void AddNode_NoInputs_CreatesNoInputPorts()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("const", "Constant", minInputs: 0, maxInputs: 0, outputs: 1));
        Assert.Empty(h.PipelineEditor.Nodes[0].InputPorts);
        Assert.Single(h.PipelineEditor.Nodes[0].OutputPorts);
    }

    [Fact]
    public void AddNode_CopiesParameters()
    {
        var h = new ViewModelTestHarness();
        var plugin = TestPlugin("brt", "Brightness");
        plugin.ParameterSchemas = new ObservableCollection<ParameterSchema>
        {
            new() { Name = "value", ParameterType = ParameterType.Float, DefaultValue = 0.5 }
        };
        h.PipelineEditor.AddNodeCommand.Execute(plugin);
        Assert.True(h.PipelineEditor.Nodes[0].Parameters.ContainsKey("value"));
    }

    [Fact]
    public void AddNode_NullPlugin_PortCountSafety()
    {
        var h = new ViewModelTestHarness();
        var plugin = TestPlugin("null-inputs", "NullInputs", minInputs: 1, maxInputs: 0, outputs: 1);
        var exception = Record.Exception(() => h.PipelineEditor.AddNodeCommand.Execute(plugin));
        Assert.Null(exception);
    }

    // ═══ Remove node ═══
    [Fact]
    public void RemoveNode_DecreasesNodeCount()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.RemoveNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        Assert.Empty(h.PipelineEditor.Nodes);
    }

    [Fact]
    public void RemoveNode_Null_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.RemoveNodeCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void RemoveNode_RemovesConnectedEdges()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        // Connect a → b
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[0].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);
        Assert.Single(h.PipelineEditor.Edges);

        h.PipelineEditor.RemoveNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        Assert.Empty(h.PipelineEditor.Edges);
    }

    [Fact]
    public void RemoveNode_ClearsSelectionIfSelected()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        h.PipelineEditor.RemoveNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        Assert.Null(h.PipelineEditor.SelectedNode);
    }

    // ═══ Select node ═══
    [Fact]
    public void SelectNode_SetsSelectedNode()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        Assert.Equal(h.PipelineEditor.Nodes[0], h.PipelineEditor.SelectedNode);
        Assert.True(h.PipelineEditor.Nodes[0].IsSelected);
    }

    [Fact]
    public void SelectNode_SwitchesSelection()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        Assert.Equal(h.PipelineEditor.Nodes[1], h.PipelineEditor.SelectedNode);
        Assert.False(h.PipelineEditor.Nodes[0].IsSelected);
        Assert.True(h.PipelineEditor.Nodes[1].IsSelected);
    }

    [Fact]
    public void ClearSelection_ResetsSelectedNode()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        h.PipelineEditor.ClearSelectionCommand.Execute(null);
        Assert.Null(h.PipelineEditor.SelectedNode);
        Assert.False(h.PipelineEditor.Nodes[0].IsSelected);
    }

    // ═══ Connect ports ═══
    [Fact]
    public void ConnectPorts_CreatesEdge()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[0].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);
        Assert.Single(h.PipelineEditor.Edges);
    }

    [Fact]
    public void ConnectPorts_Duplicate_NotCreated()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[0].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[0].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);
        Assert.Single(h.PipelineEditor.Edges);
    }

    [Fact]
    public void ConnectPorts_WithoutDraggingPort_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        var exception = Record.Exception(() => h.PipelineEditor.ConnectPortsCommand.Execute(null));
        Assert.Null(exception);
    }

    // ═══ Validate pipeline ═══
    [Fact]
    public async Task ValidatePipeline_SingleNode_Valid()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        await h.PipelineEditor.ValidatePipelineCommand.ExecuteAsync(null);
        Assert.True(h.PipelineEditor.IsPipelineValid);
    }

    [Fact]
    public async Task ValidatePipeline_EmptyNodes_Valid()
    {
        var h = new ViewModelTestHarness();
        await h.PipelineEditor.ValidatePipelineCommand.ExecuteAsync(null);
        Assert.True(h.PipelineEditor.IsPipelineValid);
    }

    [Fact]
    public async Task ValidatePipeline_ConnectedChain_Valid()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[0].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);
        await h.PipelineEditor.ValidatePipelineCommand.ExecuteAsync(null);
        Assert.True(h.PipelineEditor.IsPipelineValid);
    }

    [Fact]
    public async Task ValidatePipeline_DisconnectedNodes_Invalid()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        // No connection between them
        await h.PipelineEditor.ValidatePipelineCommand.ExecuteAsync(null);
        Assert.False(h.PipelineEditor.IsPipelineValid);
    }

    // ═══ Duplicate ═══
    [Fact]
    public void DuplicateSelected_WithSelection_CreatesCopy()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        h.PipelineEditor.DuplicateSelectedCommand.Execute(null);
        Assert.Equal(2, h.PipelineEditor.Nodes.Count);
        Assert.Contains("(copy)", h.PipelineEditor.Nodes[1].DisplayName);
    }

    [Fact]
    public void DuplicateSelected_WithoutSelection_DoesNothing()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.DuplicateSelectedCommand.Execute(null));
        Assert.Null(exception);
        Assert.Empty(h.PipelineEditor.Nodes);
    }

    [Fact]
    public void DuplicateSelected_OffsetPosition()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[0]);
        h.PipelineEditor.DuplicateSelectedCommand.Execute(null);
        Assert.NotEqual(h.PipelineEditor.Nodes[0].CanvasX, h.PipelineEditor.Nodes[1].CanvasX);
    }

    // ═══ Zoom ═══
    [Fact]
    public void ZoomIn_IncreasesScale()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.ZoomInCommand.Execute(null);
        Assert.True(h.PipelineEditor.Scale > 1.0);
    }

    [Fact]
    public void ZoomOut_DecreasesScale()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.Scale = 2.0;
        h.PipelineEditor.ZoomOutCommand.Execute(null);
        Assert.True(h.PipelineEditor.Scale < 2.0);
    }

    [Fact]
    public void ZoomLevel_Clamped_Max5()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++) h.PipelineEditor.ZoomInCommand.Execute(null);
        Assert.True(h.PipelineEditor.Scale <= 5.0);
    }

    [Fact]
    public void ZoomLevel_Clamped_Min0_1()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 20; i++) h.PipelineEditor.ZoomOutCommand.Execute(null);
        Assert.True(h.PipelineEditor.Scale >= 0.1);
    }

    [Fact]
    public void ResetZoom_ResetsTo1()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.ZoomInCommand.Execute(null);
        h.PipelineEditor.ZoomInCommand.Execute(null);
        h.PipelineEditor.ResetZoomCommand.Execute(null);
        Assert.Equal(1.0, h.PipelineEditor.Scale);
        Assert.Equal(0, h.PipelineEditor.OffsetX);
        Assert.Equal(0, h.PipelineEditor.OffsetY);
    }

    // ═══ FitAll ═══
    [Fact]
    public void FitAll_EmptyNodes_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.PipelineEditor.FitAllCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void FitAll_WithNodes_SetsOffset()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.FitAllCommand.Execute(null);
        Assert.True(h.PipelineEditor.OffsetX != 0 || h.PipelineEditor.OffsetY != 0);
    }

    // ═══ Node dragging ═══
    [Fact]
    public void NodeMouseDown_SetsDragging()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.OnNodeMouseDown(h.PipelineEditor.Nodes[0], 100, 100);
        Assert.True(h.PipelineEditor.Nodes[0].IsDragging);
        Assert.True(h.PipelineEditor.IsDraggingNode);
    }

    [Fact]
    public void NodeMouseMove_MovesNode()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        double initX = h.PipelineEditor.Nodes[0].CanvasX;
        h.PipelineEditor.OnNodeMouseDown(h.PipelineEditor.Nodes[0], 0, 0);
        h.PipelineEditor.OnNodeMouseMove(50, 50);
        Assert.True(h.PipelineEditor.Nodes[0].CanvasX > initX);
    }

    [Fact]
    public void NodeMouseUp_StopsDragging()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.OnNodeMouseDown(h.PipelineEditor.Nodes[0], 100, 100);
        h.PipelineEditor.OnNodeMouseUp();
        Assert.False(h.PipelineEditor.Nodes[0].IsDragging);
        Assert.False(h.PipelineEditor.IsDraggingNode);
    }

    // ═══ Port dragging ═══
    [Fact]
    public void PortDragStart_SetsDrawingConnection()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.OnPortDragStart(h.PipelineEditor.Nodes[0].OutputPorts[0], 50, 50);
        Assert.True(h.PipelineEditor.IsDrawingConnection);
        Assert.NotNull(h.PipelineEditor.DraggingPort);
    }

    [Fact]
    public void PortDrag_UpdatesConnectionLine()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.OnPortDragStart(h.PipelineEditor.Nodes[0].OutputPorts[0], 50, 50);
        h.PipelineEditor.OnPortDrag(200, 300);
        Assert.Equal(200, h.PipelineEditor.ConnectionLineX2);
        Assert.Equal(300, h.PipelineEditor.ConnectionLineY2);
    }

    // ═══ Multi-node chains ═══
    [Fact]
    public async Task ThreeNodeChain_AllConnected_Valid()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("a", "A"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("b", "B"));
        h.PipelineEditor.AddNodeCommand.Execute(TestPlugin("c", "C"));

        // Connect a → b
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[1]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[0].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);

        // Connect b → c
        h.PipelineEditor.SelectNodeCommand.Execute(h.PipelineEditor.Nodes[2]);
        h.PipelineEditor.DraggingPort = h.PipelineEditor.Nodes[1].OutputPorts[0];
        h.PipelineEditor.ConnectPortsCommand.Execute(null);

        await h.PipelineEditor.ValidatePipelineCommand.ExecuteAsync(null);
        Assert.True(h.PipelineEditor.IsPipelineValid);
        Assert.Equal(2, h.PipelineEditor.Edges.Count);
    }

    [Fact]
    public void TenNodes_AddSequentially_AllPresent()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 10; i++)
            h.PipelineEditor.AddNodeCommand.Execute(TestPlugin($"p{i}", $"Plugin {i}"));
        Assert.Equal(10, h.PipelineEditor.Nodes.Count);
    }
}
