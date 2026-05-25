namespace Photopipeline.Models;

public sealed class PluginInfo
{
    public string Id { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public string Version { get; set; } = string.Empty;
    public string Category { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public Dictionary<string, object> ParameterSchema { get; set; } = new();
    public string? Icon { get; set; }
    public string? Color { get; set; }
}

public sealed class BatchSpec
{
    public string PipelineConfigPath { get; set; } = string.Empty;
    public string FilePattern { get; set; } = string.Empty;
    public string OutputDir { get; set; } = string.Empty;
    public int Parallel { get; set; } = 1;
    public bool Resume { get; set; }
}

public sealed class BatchProgress
{
    public BatchStatus Status { get; set; }
    public int TotalFiles { get; set; }
    public int CompletedFiles { get; set; }
    public int FailedFiles { get; set; }
    public string CurrentFile { get; set; } = string.Empty;
    public float Fraction { get; set; }
    public string ProgressDetails { get; set; } = string.Empty;
}

public enum BatchStatus
{
    Pending,
    Running,
    Done,
    Canceled,
    Error
}
