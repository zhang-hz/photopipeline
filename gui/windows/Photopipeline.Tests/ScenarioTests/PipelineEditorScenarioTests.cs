using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class PipelineEditorScenarioTests
{
    private static PipelineEditorViewModel Create()
    {
        var logger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        return new PipelineEditorViewModel(logger, pipelineService);
    }

    private static PluginInfo P(string id, string name) => new() { Id = id, Name = name, Category = "Test" };

    [Fact]
    public void BuildLinearPipeline_ThreeNodes()
    {
        var vm = Create();
        vm.AddNodeAt(P("input", "Input"), 80, 100);
        vm.AddNodeAt(P("process", "Process"), 300, 100);
        vm.AddNodeAt(P("output", "Output"), 520, 100);

        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        vm.ConnectNodesCommand.Execute((vm.Nodes[1].Id, vm.Nodes[2].Id));

        vm.Nodes.Should().HaveCount(3);
        vm.Edges.Should().HaveCount(2);
        vm.CanConnect(vm.Nodes[2].Id, vm.Nodes[0].Id).Should().BeFalse(); // No cycles
    }

    [Fact]
    public void EditParameter_ThenValidate()
    {
        var vm = Create();
        vm.AddNodeAt(P("denoise", "Denoise"), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.UpdateNodeParameterCommand.Execute((nodeId, "strength", 0.9));
        vm.UpdateNodeParameterCommand.Execute((nodeId, "method", "AMaZE"));

        vm.Nodes[0].Params["strength"].Should().Be(0.9);
        vm.Nodes[0].Params["method"].Should().Be("AMaZE");
    }

    [Fact]
    public void RemoveMiddleNode_ReconnectsRemaining()
    {
        var vm = Create();
        vm.AddNodeAt(P("a", "A"), 80, 100);
        vm.AddNodeAt(P("b", "B"), 300, 100);
        vm.AddNodeAt(P("c", "C"), 520, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        vm.ConnectNodesCommand.Execute((vm.Nodes[1].Id, vm.Nodes[2].Id));

        vm.RemoveNodeCommand.Execute(vm.Nodes[1]);

        vm.Nodes.Should().HaveCount(2);
        vm.Edges.Should().BeEmpty();
    }

    [Fact]
    public void DragNode_UpdatesPosition()
    {
        var vm = Create();
        vm.AddNodeAt(P("test", "Test"), 100, 200);

        vm.UpdateNodePosition(vm.Nodes[0].Id, 450, 320);

        vm.Nodes[0].PositionX.Should().Be(450);
        vm.Nodes[0].PositionY.Should().Be(320);
    }

    [Fact]
    public void CycleDetection_ThreeNodeCycle_Rejected()
    {
        var vm = Create();
        vm.AddNodeAt(P("a", "A"), 80, 100);
        vm.AddNodeAt(P("b", "B"), 300, 100);
        vm.AddNodeAt(P("c", "C"), 520, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        vm.ConnectNodesCommand.Execute((vm.Nodes[1].Id, vm.Nodes[2].Id));

        var canClose = vm.CanConnect(vm.Nodes[2].Id, vm.Nodes[0].Id);

        canClose.Should().BeFalse();
    }

    [Fact]
    public void EmptyPipelineExecutes_ShowsError()
    {
        var vm = Create();

        vm.ExecuteCommand.Execute(null);

        vm.ErrorMessage.Should().Contain("least one node");
    }
}
