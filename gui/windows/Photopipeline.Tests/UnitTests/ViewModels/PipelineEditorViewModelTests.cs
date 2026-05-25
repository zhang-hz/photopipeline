using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class PipelineEditorViewModelTests
{
    private static PipelineEditorViewModel Create(Mock<IPipelineService>? pipelineServiceMock = null)
    {
        var logger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = pipelineServiceMock?.Object ?? Mock.Of<IPipelineService>();
        return new PipelineEditorViewModel(logger, pipelineService);
    }

    private static PluginInfo CreatePlugin(string id = "test", string name = "Test Plugin")
    {
        return new PluginInfo { Id = id, Name = name, Category = "Test" };
    }

    [Fact]
    public void InitialState_EmptyPipeline()
    {
        var vm = Create();

        vm.Nodes.Should().BeEmpty();
        vm.Edges.Should().BeEmpty();
        vm.SelectedNode.Should().BeNull();
        vm.Scale.Should().Be(1.0);
        vm.IsExecuting.Should().BeFalse();
        vm.IsPipelineValid.Should().BeFalse();
        vm.CurrentPipeline.Name.Should().Be("New Pipeline");
    }

    [Fact]
    public void AddNode_AddsToNodesAndPipelineSpec()
    {
        var vm = Create();
        var plugin = CreatePlugin();

        vm.AddNodeCommand.Execute(plugin);

        vm.Nodes.Should().HaveCount(1);
        vm.Nodes[0].PluginId.Should().Be("test");
        vm.Nodes[0].Label.Should().Be("Test Plugin");
        vm.CurrentPipeline.Nodes.Should().HaveCount(1);
    }

    [Fact]
    public void AddNodeAt_SetsPosition()
    {
        var vm = Create();
        var plugin = CreatePlugin();

        vm.AddNodeAt(plugin, 350, 220);

        vm.Nodes[0].PositionX.Should().Be(350);
        vm.Nodes[0].PositionY.Should().Be(220);
    }

    [Fact]
    public void AddNodeAt_WhileExecuting_Noop()
    {
        var vm = Create();
        vm.IsExecuting = true;

        vm.AddNodeAt(CreatePlugin(), 100, 100);

        vm.Nodes.Should().BeEmpty();
    }

    [Fact]
    public void RemoveNode_RemovesNodeAndEdges()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(CreatePlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));

        vm.RemoveNodeCommand.Execute(vm.Nodes[0]);

        vm.Nodes.Should().HaveCount(1);
        vm.Edges.Should().BeEmpty();
    }

    [Fact]
    public void RemoveNode_Null_Noop()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin(), 100, 100);

        vm.RemoveNodeCommand.Execute(null);

        vm.Nodes.Should().HaveCount(1);
    }

    [Fact]
    public void UpdateNodePosition_UpdatesCoordinates()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin(), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.UpdateNodePosition(nodeId, 500, 400);

        vm.Nodes[0].PositionX.Should().Be(500);
        vm.Nodes[0].PositionY.Should().Be(400);
    }

    [Fact]
    public void UpdateNodePosition_UnknownId_Noop()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin(), 100, 100);

        vm.UpdateNodePosition("nonexistent", 500, 400);

        vm.Nodes[0].PositionX.Should().Be(100);
    }

    [Fact]
    public void ConnectNodes_CreatesEdge()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(CreatePlugin("p2", "B"), 300, 100);

        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));

        vm.Edges.Should().HaveCount(1);
        vm.Edges[0].From.Should().Be(vm.Nodes[0].Id);
        vm.Edges[0].To.Should().Be(vm.Nodes[1].Id);
    }

    [Fact]
    public void ConnectNodes_DuplicateEdge_NotAdded()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(CreatePlugin("p2", "B"), 300, 100);
        var conn = (vm.Nodes[0].Id, vm.Nodes[1].Id);
        vm.ConnectNodesCommand.Execute(conn);

        vm.ConnectNodesCommand.Execute(conn);

        vm.Edges.Should().HaveCount(1);
    }

    [Fact]
    public void ConnectNodes_SameNode_Rejected()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin(), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.ConnectNodesCommand.Execute((nodeId, nodeId));

        vm.Edges.Should().BeEmpty();
    }

    [Fact]
    public void CanConnect_DirectCycle_ReturnsFalse()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(CreatePlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));

        var canReverse = vm.CanConnect(vm.Nodes[1].Id, vm.Nodes[0].Id);

        canReverse.Should().BeFalse();
    }

    [Fact]
    public void CanConnect_SameNode_ReturnsFalse()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin(), 100, 100);

        var result = vm.CanConnect(vm.Nodes[0].Id, vm.Nodes[0].Id);

        result.Should().BeFalse();
    }

    [Fact]
    public void DisconnectEdge_RemovesEdge()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(CreatePlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        var edge = vm.Edges[0];

        vm.DisconnectEdgeCommand.Execute(edge);

        vm.Edges.Should().BeEmpty();
    }

    [Fact]
    public void UpdateNodeParameter_StoresValue()
    {
        var vm = Create();
        vm.AddNodeAt(CreatePlugin(), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.UpdateNodeParameterCommand.Execute((nodeId, "strength", 0.75));

        vm.Nodes[0].Params["strength"].Should().Be(0.75);
    }

    [Fact]
    public void ZoomCanvas_ClampsToRange()
    {
        var vm = Create();

        vm.ZoomCanvasCommand.Execute(-2.0);

        vm.Scale.Should().BeGreaterOrEqualTo(0.1);
    }

    [Fact]
    public void ResetCanvas_ReturnsToDefault()
    {
        var vm = Create();
        vm.Scale = 2.5;
        vm.OffsetX = 100;
        vm.OffsetY = 200;

        vm.ResetCanvasCommand.Execute(null);

        vm.Scale.Should().Be(1.0);
        vm.OffsetX.Should().Be(0);
        vm.OffsetY.Should().Be(0);
    }

    [Fact]
    public void Validate_NoNodes_ShowsError()
    {
        var vm = Create();

        vm.ValidateCommand.Execute(null);

        vm.IsPipelineValid.Should().BeFalse();
        vm.ValidationMessage.Should().Contain("no nodes");
    }
}
