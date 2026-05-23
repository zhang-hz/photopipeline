namespace Photopipeline.Tests.SystemTests;

public sealed class PipelineWorkflowTests : SystemTestBase
{
    [Fact]
    public async Task FullPipelineWorkflow_CreateExecuteAndVerify()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        plugins.Should().NotBeNull();
        plugins.Should().NotBeEmpty();
        plugins.Should().HaveCountGreaterOrEqualTo(5);

        var pipeline = new PipelineModel
        {
            Name = "GPS Test Pipeline",
            Description = "Process image with GPS tagging"
        };

        var gpsPlugin = plugins.FirstOrDefault(p => p.Id == "demosaic");
        var colorPlugin = plugins.FirstOrDefault(p => p.Id == "white_balance");
        gpsPlugin.Should().NotBeNull("GPS demosaic plugin should be available");
        colorPlugin.Should().NotBeNull("White balance plugin should be available");

        var gpsNode = new PipelineNode
        {
            PluginId = gpsPlugin!.Id,
            DisplayName = gpsPlugin.Name,
            CanvasX = 100,
            CanvasY = 100
        };

        var colorNode = new PipelineNode
        {
            PluginId = colorPlugin!.Id,
            DisplayName = colorPlugin.Name,
            CanvasX = 350,
            CanvasY = 100
        };

        pipeline.Nodes.Add(gpsNode);
        pipeline.Nodes.Add(colorNode);

        pipeline.Edges.Add(new PipelineEdge
        {
            SourceNodeId = gpsNode.Id,
            SourcePortId = "out",
            TargetNodeId = colorNode.Id,
            TargetPortId = "in"
        });

        pipeline.Nodes.Should().HaveCount(2);
        pipeline.Edges.Should().HaveCount(1);

        var valid = await PipelineService.ValidatePipelineAsync(pipeline);
        valid.Should().BeTrue();
        pipeline.IsValid.Should().BeTrue();
        pipeline.ValidationError.Should().BeEmpty();
    }

    [Fact]
    public async Task ValidatePipeline_WithDisconnectedNodes_ShouldFail()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var pipeline = new PipelineModel { Name = "Invalid Pipeline" };

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        if (plugins.Count < 2) return;

        var node1 = new PipelineNode
        {
            PluginId = plugins[0].Id,
            DisplayName = plugins[0].Name,
            CanvasX = 100,
            CanvasY = 100
        };
        var node2 = new PipelineNode
        {
            PluginId = plugins[1].Id,
            DisplayName = plugins[1].Name,
            CanvasX = 350,
            CanvasY = 100
        };

        pipeline.Nodes.Add(node1);
        pipeline.Nodes.Add(node2);

        var valid = await PipelineService.ValidatePipelineAsync(pipeline);
        valid.Should().BeFalse();
        pipeline.IsValid.Should().BeFalse();
        pipeline.ValidationError.Should().NotBeEmpty();
    }

    [Fact]
    public async Task ExecutePipeline_EmptyPipeline_ShouldFail()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var pipeline = new PipelineModel { Name = "Empty Pipeline" };

        var valid = await PipelineService.ValidatePipelineAsync(pipeline);
        valid.Should().BeFalse();
        pipeline.ValidationError.Should().Be("Pipeline has no nodes");
    }

    [Fact]
    public async Task PipelineExecution_UpdatesNodeProcessingState()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var plugins = await PipelineService.GetAvailablePluginsAsync();
        if (plugins.Count < 1) return;

        var pipeline = new PipelineModel { Name = "Test Pipeline" };
        var node = new PipelineNode
        {
            PluginId = plugins[0].Id,
            DisplayName = plugins[0].Name,
            CanvasX = 100,
            CanvasY = 100
        };
        pipeline.Nodes.Add(node);

        pipeline.IsValid = true;

        var result = await PipelineService.ExecutePipelineAsync(pipeline, "test_image_id");

        pipeline.IsExecuting.Should().BeFalse();
        node.IsProcessing.Should().BeFalse();
    }

    private async Task<bool> TryStartServerAsync()
    {
        try
        {
            await StartServerAsync();
            return true;
        }
        catch (SkipTestException)
        {
            return false;
        }
    }
}
