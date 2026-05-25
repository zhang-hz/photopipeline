namespace Photopipeline.Models;

public sealed class PipelineSpec
{
    public string Name { get; set; } = string.Empty;
    public List<PipelineNode> Nodes { get; set; } = new();
    public List<PipelineEdge> Edges { get; set; } = new();
    public Dictionary<string, object> Params { get; set; } = new();
}

public sealed class PipelineNode
{
    public string Id { get; set; } = Guid.NewGuid().ToString();
    public string PluginId { get; set; } = string.Empty;
    public string Label { get; set; } = string.Empty;
    public bool Enabled { get; set; } = true;
    public double PositionX { get; set; }
    public double PositionY { get; set; }
    public Dictionary<string, object> Params { get; set; } = new();
}

public sealed class PipelineEdge
{
    public string From { get; set; } = string.Empty;
    public string To { get; set; } = string.Empty;
}

public sealed class PipelineId
{
    public string Id { get; set; } = string.Empty;
}

public sealed class ValidationResult
{
    public bool Valid { get; set; }
    public List<ValidationIssue> Issues { get; set; } = new();
}

public sealed class ValidationIssue
{
    public ValidationSeverity Severity { get; set; }
    public string Param { get; set; } = string.Empty;
    public string Message { get; set; } = string.Empty;
}

public enum ValidationSeverity
{
    Info,
    Warning,
    Error
}

public sealed class ExecuteProgress
{
    public ExecuteStage Stage { get; set; }
    public string NodeId { get; set; } = string.Empty;
    public string NodeLabel { get; set; } = string.Empty;
    public float Fraction { get; set; }
    public string Message { get; set; } = string.Empty;
    public long ElapsedMs { get; set; }
}

public enum ExecuteStage
{
    Loading,
    Decoding,
    Processing,
    Encoding,
    Done,
    Error
}

public sealed class NodeSchema
{
    public string PluginId { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public string Version { get; set; } = string.Empty;
    public string Category { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public Dictionary<string, object> ParameterSchema { get; set; } = new();
    public Dictionary<string, object> GuiSchema { get; set; } = new();
}
