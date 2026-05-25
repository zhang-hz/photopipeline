namespace Photopipeline.Tests.UnitTests.Models;

public sealed class PipelineModelTests
{
    [Fact]
    public void PipelineSpec_Creation_HasDefaultValues()
    {
        var spec = new PipelineSpec();

        spec.Name.Should().BeEmpty();
        spec.Nodes.Should().BeEmpty();
        spec.Edges.Should().BeEmpty();
        spec.Params.Should().BeEmpty();
    }

    [Fact]
    public void PipelineNode_Creation_HasDefaultValues()
    {
        var node = new PipelineNode();

        node.Id.Should().NotBeNullOrEmpty();
        node.PluginId.Should().BeEmpty();
        node.Label.Should().BeEmpty();
        node.Enabled.Should().BeTrue();
        node.PositionX.Should().Be(0);
        node.PositionY.Should().Be(0);
        node.Params.Should().BeEmpty();
    }

    [Fact]
    public void PipelineNode_UniqueIdPerInstance()
    {
        var n1 = new PipelineNode();
        var n2 = new PipelineNode();

        n1.Id.Should().NotBe(n2.Id);
    }

    [Fact]
    public void PipelineEdge_Creation_HasDefaultValues()
    {
        var edge = new PipelineEdge();

        edge.From.Should().BeEmpty();
        edge.To.Should().BeEmpty();
    }

    [Fact]
    public void PipelineSpec_AddNodesAndEdges()
    {
        var spec = new PipelineSpec { Name = "Test Pipeline" };
        var n1 = new PipelineNode { PluginId = "p1", Label = "Node 1" };
        var n2 = new PipelineNode { PluginId = "p2", Label = "Node 2" };
        spec.Nodes.Add(n1);
        spec.Nodes.Add(n2);
        spec.Edges.Add(new PipelineEdge { From = n1.Id, To = n2.Id });

        spec.Nodes.Should().HaveCount(2);
        spec.Edges.Should().HaveCount(1);
        spec.Edges[0].From.Should().Be(n1.Id);
        spec.Edges[0].To.Should().Be(n2.Id);
    }

    [Fact]
    public void PipelineNode_Position_LayoutCoordinates()
    {
        var node = new PipelineNode { PositionX = 300, PositionY = 150 };

        node.PositionX.Should().Be(300);
        node.PositionY.Should().Be(150);
    }

    [Fact]
    public void PipelineNode_Params_StoresValues()
    {
        var node = new PipelineNode();
        node.Params["strength"] = 0.75;
        node.Params["enabled"] = true;

        node.Params.Should().HaveCount(2);
        node.Params["strength"].Should().Be(0.75);
        node.Params["enabled"].Should().Be(true);
    }

    [Fact]
    public void ValidationResult_Defaults()
    {
        var result = new ValidationResult();

        result.Valid.Should().BeFalse();
        result.Issues.Should().BeEmpty();
    }

    [Fact]
    public void ValidationIssue_SeverityLevels()
    {
        var issue = new ValidationIssue
        {
            Severity = ValidationSeverity.Error,
            Param = "exposure",
            Message = "Out of range"
        };

        issue.Severity.Should().Be(ValidationSeverity.Error);
        issue.Param.Should().Be("exposure");
        issue.Message.Should().Be("Out of range");
    }

    [Fact]
    public void ExecuteProgress_TracksStageProgress()
    {
        var progress = new ExecuteProgress
        {
            Stage = ExecuteStage.Processing,
            NodeId = "n1",
            NodeLabel = "Denoise",
            Fraction = 0.5f,
            Message = "Processing...",
            ElapsedMs = 1200
        };

        progress.Stage.Should().Be(ExecuteStage.Processing);
        progress.Fraction.Should().Be(0.5f);
        progress.ElapsedMs.Should().Be(1200);
    }

    [Fact]
    public void NodeSchema_MapsPluginMetadata()
    {
        var schema = new NodeSchema
        {
            PluginId = "denoise_v1",
            Name = "Denoise",
            Version = "1.2.0",
            Category = "Noise Reduction",
            Description = "AI-based denoising",
            ParameterSchema = new Dictionary<string, object> { ["strength"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.5 } },
            GuiSchema = new Dictionary<string, object> { ["group"] = "Advanced" }
        };

        schema.PluginId.Should().Be("denoise_v1");
        schema.ParameterSchema.Should().ContainKey("strength");
        schema.GuiSchema.Should().ContainKey("group");
    }
}
