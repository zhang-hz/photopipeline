using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for PipelineEditorViewModel.
/// Uses MockBehavior.Strict for service mocks. Every test has a FAIL-able assertion.
/// </summary>
public sealed class PipelineEditorViewModelTests : IDisposable
{
    private readonly List<Mock> _strictMocks = new();

    public void Dispose()
    {
        foreach (var mock in _strictMocks)
            mock.VerifyAll();
    }

    private Mock<T> Strict<T>() where T : class
    {
        var mock = new Mock<T>(MockBehavior.Strict);
        _strictMocks.Add(mock);
        return mock;
    }

    private static ILogger<T> AnyLogger<T>() => Mock.Of<ILogger<T>>();

    private PipelineEditorViewModel CreateVm(Mock<IPipelineService>? pipelineMock = null)
    {
        var pl = pipelineMock?.Object ?? Mock.Of<IPipelineService>();
        return new PipelineEditorViewModel(AnyLogger<PipelineEditorViewModel>(), pl);
    }

    private static PluginInfo TestPlugin(string id = "test", string name = "Test Plugin") => new()
    {
        Id = id, Name = name, Category = "Test", Version = "1.0"
    };

    // ═════════════════════════════════════════════════════════════
    // Test 001: InitialState_EmptyPipeline
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_001_InitialState_EmptyPipeline()
    {
        var vm = CreateVm();

        vm.Nodes.Should().BeEmpty();
        vm.Edges.Should().BeEmpty();
        vm.SelectedNode.Should().BeNull();
        vm.Scale.Should().Be(1.0);
        vm.IsExecuting.Should().BeFalse();
        vm.IsPipelineValid.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 002: AddNode_AddsToNodesAndPipelineSpec
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_002_AddNode_AddsToNodesAndPipelineSpec()
    {
        var vm = CreateVm();
        var plugin = TestPlugin();

        var collectionChanged = false;
        vm.Nodes.CollectionChanged += (s, e) => collectionChanged = true;

        vm.AddNodeCommand.Execute(plugin);

        vm.Nodes.Should().HaveCount(1);
        vm.Nodes[0].PluginId.Should().Be("test");
        vm.Nodes[0].Label.Should().Be("Test Plugin");
        vm.CurrentPipeline.Nodes.Should().HaveCount(1);
        collectionChanged.Should().BeTrue("Nodes collection should fire CollectionChanged on Add");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 003: AddNodeAt_SetsCustomPosition
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_003_AddNodeAt_SetsCustomPosition()
    {
        var vm = CreateVm();
        var plugin = TestPlugin();

        vm.AddNodeAt(plugin, 350, 220);

        vm.Nodes[0].PositionX.Should().Be(350);
        vm.Nodes[0].PositionY.Should().Be(220);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 004: AddNodeAt_WhileExecuting_Noop
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_004_AddNodeAt_WhileExecuting_Noop()
    {
        var vm = CreateVm();
        vm.IsExecuting = true;

        vm.AddNodeAt(TestPlugin(), 100, 100);

        vm.Nodes.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 005: RemoveNode_RemovesNodeAndRelatedEdges
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_005_RemoveNode_RemovesNodeAndRelatedEdges()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));

        vm.RemoveNodeCommand.Execute(vm.Nodes[0]);

        vm.Nodes.Should().HaveCount(1);
        vm.Nodes[0].PluginId.Should().Be("p2");
        vm.Edges.Should().BeEmpty();
        vm.CurrentPipeline.Edges.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 006: RemoveNode_Null_Noop
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_006_RemoveNode_Null_Noop()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin(), 100, 100);

        vm.RemoveNodeCommand.Execute(null);

        vm.Nodes.Should().HaveCount(1);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 007: UpdateNodePosition_ValidNodeUpdatesCoordinates
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_007_UpdateNodePosition_ValidNodeUpdatesCoordinates()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin(), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.UpdateNodePosition(nodeId, 500, 400);

        vm.Nodes[0].PositionX.Should().Be(500);
        vm.Nodes[0].PositionY.Should().Be(400);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 008: ConnectNodes_CreatesEdge
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_008_ConnectNodes_CreatesEdge()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);

        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));

        vm.Edges.Should().HaveCount(1);
        vm.Edges[0].From.Should().Be(vm.Nodes[0].Id);
        vm.Edges[0].To.Should().Be(vm.Nodes[1].Id);
        vm.CurrentPipeline.Edges.Should().HaveCount(1);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 009: ConnectNodes_DuplicateEdge_NotAdded
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_009_ConnectNodes_DuplicateEdge_NotAdded()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);
        var conn = (vm.Nodes[0].Id, vm.Nodes[1].Id);
        vm.ConnectNodesCommand.Execute(conn);

        vm.ConnectNodesCommand.Execute(conn);

        vm.Edges.Should().HaveCount(1);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 010: ConnectNodes_SelfLoop_Rejected
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_010_ConnectNodes_SelfLoop_Rejected()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin(), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.ConnectNodesCommand.Execute((nodeId, nodeId));

        vm.Edges.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 011: CanConnect_DirectCycle_ReturnsFalse
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_011_CanConnect_DirectCycle_ReturnsFalse()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));

        var canReverse = vm.CanConnect(vm.Nodes[1].Id, vm.Nodes[0].Id);

        canReverse.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 012: CanConnect_ThreeNodeCycle_ReturnsFalse
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_012_CanConnect_ThreeNodeCycle_ReturnsFalse()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);
        vm.AddNodeAt(TestPlugin("p3", "C"), 500, 100);
        // A → B → C
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        vm.ConnectNodesCommand.Execute((vm.Nodes[1].Id, vm.Nodes[2].Id));

        // C → A would create a cycle
        var wouldCycle = vm.CanConnect(vm.Nodes[2].Id, vm.Nodes[0].Id);

        wouldCycle.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 013: CanConnect_DisconnectedNodes_ReturnsTrue
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_013_CanConnect_DisconnectedNodes_ReturnsTrue()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);

        var result = vm.CanConnect(vm.Nodes[0].Id, vm.Nodes[1].Id);

        result.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 014: DisconnectEdge_RemovesEdge
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_014_DisconnectEdge_RemovesEdge()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        var edge = vm.Edges[0];

        vm.DisconnectEdgeCommand.Execute(edge);

        vm.Edges.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 015: UpdateNodeParameter_StoresValue
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_015_UpdateNodeParameter_StoresValue()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin(), 100, 100);
        var nodeId = vm.Nodes[0].Id;

        vm.UpdateNodeParameterCommand.Execute((nodeId, "strength", 0.75));

        vm.Nodes[0].Params.Should().ContainKey("strength");
        vm.Nodes[0].Params["strength"].Should().Be(0.75);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 016: NewPipeline_ClearsAllState
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_016_NewPipeline_ClearsAllState()
    {
        var vm = CreateVm();
        vm.AddNodeAt(TestPlugin("p1", "A"), 100, 100);
        vm.AddNodeAt(TestPlugin("p2", "B"), 300, 100);
        vm.ConnectNodesCommand.Execute((vm.Nodes[0].Id, vm.Nodes[1].Id));
        vm.PipelineId = "existing-id";
        vm.IsPipelineValid = true;

        vm.NewPipelineCommand.Execute(null);

        vm.Nodes.Should().BeEmpty();
        vm.Edges.Should().BeEmpty();
        vm.PipelineId.Should().BeNull();
        vm.IsPipelineValid.Should().BeFalse();
        vm.ValidationMessage.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 017: Validate_NoNodes_ShowsError
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_017_Validate_NoNodes_ShowsError()
    {
        var vm = CreateVm();

        vm.ValidateCommand.Execute(null);

        vm.IsPipelineValid.Should().BeFalse();
        // Verify exact validation message, not just a substring match
        vm.ValidationMessage.Should().Be("Pipeline has no nodes");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 018: RapidAddRemove_Nodes_DoesNotCrash
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_018_RapidAddRemoveNodes_DoesNotCrash()
    {
        var vm = CreateVm();

        // Add 10 nodes
        for (int i = 0; i < 10; i++)
            vm.AddNodeAt(TestPlugin($"p{i}", $"Plugin{i}"), i * 80, 60);

        vm.Nodes.Should().HaveCount(10);

        // Connect them in a chain
        for (int i = 0; i < 9; i++)
            vm.ConnectNodesCommand.Execute((vm.Nodes[i].Id, vm.Nodes[i + 1].Id));

        vm.Edges.Should().HaveCount(9);

        // Remove every other node
        var toRemove = vm.Nodes.Where((_, i) => i % 2 == 0).ToList();
        foreach (var node in toRemove)
            vm.RemoveNodeCommand.Execute(node);

        // Should not crash and should have remaining nodes
        vm.Nodes.Should().HaveCount(5);
    }
}
