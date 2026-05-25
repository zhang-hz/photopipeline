namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public sealed record TestCaseDefinition
{
    public string Id { get; init; } = string.Empty;
    public string Name { get; init; } = string.Empty;
    public string Category { get; init; } = string.Empty;
    public string[] Tags { get; init; } = [];

    public string InputImage { get; init; } = string.Empty;
    public string[]? InputImages { get; init; }

    public PipelineSpec? Pipeline { get; init; }

    public string OutputFormat { get; init; } = "TIFF";
    public uint? OutputBitDepth { get; init; }
    public bool? OutputLossless { get; init; }

    public int TolerancePerChannel { get; init; }
    public double? MinPSNR { get; init; }
    public double? MinSSIM { get; init; }
    public double? MaxDeltaE { get; init; }

    public bool ExpectError { get; init; }
    public string? ExpectedErrorMessage { get; init; }

    public bool SkipUiChannel { get; init; }
    public bool SkipApiChannel { get; init; }
    public bool IsSerialOnly { get; init; }
}
