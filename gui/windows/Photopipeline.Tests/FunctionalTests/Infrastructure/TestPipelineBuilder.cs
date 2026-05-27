namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public sealed class TestPipelineBuilder
{
    private readonly PipelineSpec _spec = new() { Name = "Test Pipeline" };

    public TestPipelineBuilder WithName(string name)
    {
        _spec.Name = name;
        return this;
    }

    public TestPipelineBuilder AddNode(string pluginId, string? label = null, bool enabled = true,
        double x = 0, double y = 0, Action<Dictionary<string, object>>? configureParams = null)
    {
        var node = new PipelineNode
        {
            Id = Guid.NewGuid().ToString(),
            PluginId = pluginId,
            Label = label ?? pluginId,
            Enabled = enabled,
            PositionX = x,
            PositionY = y
        };
        configureParams?.Invoke(node.Params);
        _spec.Nodes.Add(node);
        return this;
    }

    public TestPipelineBuilder Connect(int fromIndex, int toIndex)
    {
        if (fromIndex < 0 || toIndex < 0 || fromIndex >= _spec.Nodes.Count || toIndex >= _spec.Nodes.Count)
            throw new ArgumentOutOfRangeException(
                $"Invalid edge indices: from={fromIndex}, to={toIndex}, nodeCount={_spec.Nodes.Count}");
        _spec.Edges.Add(new PipelineEdge
        {
            From = _spec.Nodes[fromIndex].Id,
            To = _spec.Nodes[toIndex].Id
        });
        return this;
    }

    public TestPipelineBuilder ConnectLinear()
    {
        if (_spec.Edges.Count > 0)
            System.Diagnostics.Debug.WriteLine(
                $"TestPipelineBuilder: ConnectLinear clearing {_spec.Edges.Count} existing edge(s)");
        _spec.Edges.Clear();
        for (int i = 0; i < _spec.Nodes.Count - 1; i++)
            Connect(i, i + 1);
        return this;
    }

    public TestPipelineBuilder WithGlobalParam(string key, object value)
    {
        _spec.Params[key] = value;
        return this;
    }

    public TestPipelineBuilder ToggleNode(int index, bool enabled)
    {
        if (index >= 0 && index < _spec.Nodes.Count)
            _spec.Nodes[index].Enabled = enabled;
        return this;
    }

    public PipelineSpec Build() => _spec;

    public static PipelineSpec SingleNode(string pluginId, Action<Dictionary<string, object>>? configureParams = null)
        => new TestPipelineBuilder().AddNode(pluginId, configureParams: configureParams).Build();

    public static PipelineSpec Linear(params string[] pluginIds)
    {
        var builder = new TestPipelineBuilder();
        foreach (var id in pluginIds)
            builder.AddNode(id);
        return builder.ConnectLinear().Build();
    }
}
