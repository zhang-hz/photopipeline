namespace Photopipeline.Tests.UnitTests.Models;

public sealed class PipelineModelTests
{
    [Fact]
    public void PipelineModel_Creation_HasDefaultValues()
    {
        var pipeline = new PipelineModel();

        pipeline.Id.Should().NotBeNullOrEmpty();
        pipeline.Name.Should().Be("New Pipeline");
        pipeline.Description.Should().BeEmpty();
        pipeline.Nodes.Should().BeEmpty();
        pipeline.Edges.Should().BeEmpty();
        pipeline.IsValid.Should().BeFalse();
        pipeline.IsExecuting.Should().BeFalse();
        pipeline.ValidationError.Should().BeEmpty();
    }

    [Fact]
    public void PipelineNode_Creation_HasDefaultPorts()
    {
        var node = new PipelineNode();

        node.Id.Should().NotBeNullOrEmpty();
        node.PluginId.Should().BeEmpty();
        node.DisplayName.Should().BeEmpty();
        node.InputPorts.Should().HaveCount(1);
        node.InputPorts[0].Id.Should().Be("in");
        node.InputPorts[0].Name.Should().Be("Input");
        node.InputPorts[0].Direction.Should().Be(PortDirection.Input);
        node.OutputPorts.Should().HaveCount(1);
        node.OutputPorts[0].Id.Should().Be("out");
        node.OutputPorts[0].Name.Should().Be("Output");
        node.OutputPorts[0].Direction.Should().Be(PortDirection.Output);
    }

    [Fact]
    public void PipelineNode_Ports_HaveCorrectParentNodeId()
    {
        var node = new PipelineNode();

        node.InputPorts[0].ParentNodeId.Should().Be(node.Id);
        node.OutputPorts[0].ParentNodeId.Should().Be(node.Id);
    }

    [Fact]
    public void PipelineNode_OnIdChanged_UpdatesPortParentIds()
    {
        var node = new PipelineNode();
        var oldId = node.Id;

        node.Id = "new-test-id";

        node.InputPorts[0].ParentNodeId.Should().Be("new-test-id");
        node.OutputPorts[0].ParentNodeId.Should().Be("new-test-id");
    }

    [Fact]
    public void PipelineModel_AddNodes_WithCorrectPositions()
    {
        var pipeline = new PipelineModel();
        var node1 = new PipelineNode { PluginId = "plugin-a", DisplayName = "Node A", CanvasX = 100, CanvasY = 200 };
        var node2 = new PipelineNode { PluginId = "plugin-b", DisplayName = "Node B", CanvasX = 300, CanvasY = 400 };

        pipeline.Nodes.Add(node1);
        pipeline.Nodes.Add(node2);

        pipeline.Nodes.Should().HaveCount(2);
        pipeline.Nodes[0].CanvasX.Should().Be(100);
        pipeline.Nodes[0].CanvasY.Should().Be(200);
        pipeline.Nodes[1].CanvasX.Should().Be(300);
        pipeline.Nodes[1].CanvasY.Should().Be(400);
    }

    [Fact]
    public void PipelineModel_AddEdge_ConnectsTwoNodes()
    {
        var pipeline = new PipelineModel();
        var node1 = new PipelineNode();
        var node2 = new PipelineNode();
        pipeline.Nodes.Add(node1);
        pipeline.Nodes.Add(node2);

        var edge = new PipelineEdge
        {
            SourceNodeId = node1.Id,
            SourcePortId = "out",
            TargetNodeId = node2.Id,
            TargetPortId = "in"
        };
        pipeline.Edges.Add(edge);

        pipeline.Edges.Should().HaveCount(1);
        pipeline.Edges[0].SourceNodeId.Should().Be(node1.Id);
        pipeline.Edges[0].TargetNodeId.Should().Be(node2.Id);
    }

    [Fact]
    public void PipelineEdge_ConnectsCorrectPortDirections()
    {
        var node1 = new PipelineNode();
        var node2 = new PipelineNode();

        var edge = new PipelineEdge
        {
            SourceNodeId = node1.Id,
            SourcePortId = "out",
            TargetNodeId = node2.Id,
            TargetPortId = "in"
        };

        edge.SourcePortId.Should().Be("out");
        edge.TargetPortId.Should().Be("in");
        edge.Id.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void PipelineModel_ClearingEdges_RemovesAllEdges()
    {
        var pipeline = new PipelineModel();
        var n1 = new PipelineNode();
        var n2 = new PipelineNode();
        pipeline.Nodes.Add(n1);
        pipeline.Nodes.Add(n2);
        pipeline.Edges.Add(new PipelineEdge { SourceNodeId = n1.Id, TargetNodeId = n2.Id });

        pipeline.Edges.Clear();

        pipeline.Edges.Should().BeEmpty();
        pipeline.Nodes.Should().HaveCount(2);
    }

    [Fact]
    public void PipelineNode_DefaultSize_Is160x80()
    {
        var node = new PipelineNode();

        node.Width.Should().Be(160);
        node.Height.Should().Be(80);
    }

    [Fact]
    public void PipelineNode_IsSelected_TracksSelectionState()
    {
        var node = new PipelineNode();

        node.IsSelected.Should().BeFalse();

        node.IsSelected = true;
        node.IsSelected.Should().BeTrue();
    }

    [Fact]
    public void PipelineNode_Parameters_InitializesEmpty()
    {
        var node = new PipelineNode();

        node.Parameters.Should().NotBeNull();
        node.Parameters.Should().BeEmpty();
    }

    [Fact]
    public void PipelineNode_Parameters_StoresKeyValuePairs()
    {
        var node = new PipelineNode();

        node.Parameters["threshold"] = 0.5;
        node.Parameters["enabled"] = true;
        node.Parameters["name"] = "test";

        node.Parameters.Should().HaveCount(3);
        node.Parameters["threshold"].Should().Be(0.5);
        node.Parameters["enabled"].Should().Be(true);
        node.Parameters["name"].Should().Be("test");
    }

    [Fact]
    public void PipelineModel_Validate_TopologicalOrder_DirectedAcyclicGraph()
    {
        var pipeline = new PipelineModel();
        var a = new PipelineNode { DisplayName = "A" };
        var b = new PipelineNode { DisplayName = "B" };
        var c = new PipelineNode { DisplayName = "C" };
        pipeline.Nodes.Add(a);
        pipeline.Nodes.Add(b);
        pipeline.Nodes.Add(c);

        pipeline.Edges.Add(new PipelineEdge { SourceNodeId = a.Id, SourcePortId = "out", TargetNodeId = b.Id, TargetPortId = "in" });
        pipeline.Edges.Add(new PipelineEdge { SourceNodeId = b.Id, SourcePortId = "out", TargetNodeId = c.Id, TargetPortId = "in" });

        var orderedIds = new List<string>();
        foreach (var edge in pipeline.Edges)
        {
            orderedIds.Add(edge.SourceNodeId);
            orderedIds.Add(edge.TargetNodeId);
        }

        orderedIds.Distinct().Should().HaveCount(3);
        orderedIds.Should().ContainInOrder(a.Id, b.Id, b.Id, c.Id);
    }

    [Fact]
    public void PipelineModel_CycleDetection_AddingBackEdge()
    {
        var pipeline = new PipelineModel();
        var a = new PipelineNode { DisplayName = "A" };
        var b = new PipelineNode { DisplayName = "B" };
        pipeline.Nodes.Add(a);
        pipeline.Nodes.Add(b);

        pipeline.Edges.Add(new PipelineEdge { SourceNodeId = a.Id, TargetNodeId = b.Id });

        var backEdge = new PipelineEdge { SourceNodeId = b.Id, TargetNodeId = a.Id };

        var existingForward = pipeline.Edges.Any(e =>
            e.SourceNodeId == backEdge.SourceNodeId && e.TargetNodeId == backEdge.TargetNodeId);
        existingForward.Should().BeFalse();

        pipeline.Edges.Add(backEdge);
        pipeline.Edges.Should().HaveCount(2);
    }

    [Fact]
    public void PipelineModel_NodesAndEdges_ImmuneToEmptyPipeline()
    {
        var pipeline = new PipelineModel();

        pipeline.Nodes.Should().BeEmpty();
        pipeline.Edges.Should().BeEmpty();
        pipeline.IsValid.Should().BeFalse();
    }

    [Fact]
    public void Port_RelativePosition_Defaults()
    {
        var port = new Port { Id = "test", Name = "Test", Direction = PortDirection.Input };

        port.RelativeX.Should().Be(0);
        port.RelativeY.Should().Be(0);
        port.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void Port_IsConnected_TracksConnectionState()
    {
        var port = new Port();

        port.IsConnected = true;
        port.IsConnected.Should().BeTrue();
    }
}
