using System.Text.Json;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

/// <summary>
/// JSON-deserializable representation of a shared test case definition.
/// Mirrors <see cref="TestCaseDefinition"/> but in a flat, serialization-friendly shape.
/// </summary>
public sealed record SharedTestCase
{
    public string Id { get; init; } = string.Empty;
    public string Name { get; init; } = string.Empty;
    public string Category { get; init; } = string.Empty;
    public string[] Tags { get; init; } = [];

    public string InputImage { get; init; } = string.Empty;
    public string[]? InputImages { get; init; }

    public SharedPipelineNode[]? Nodes { get; init; }
    public SharedPipelineEdge[]? Edges { get; init; }

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

public sealed record SharedPipelineNode
{
    public string PluginId { get; init; } = string.Empty;
    public string? Label { get; init; }
    public bool Enabled { get; init; } = true;
    public double X { get; init; }
    public double Y { get; init; }
    public Dictionary<string, object>? Params { get; init; }
}

public sealed record SharedPipelineEdge
{
    public int FromIndex { get; init; }
    public int ToIndex { get; init; }
}

/// <summary>
/// Loads shared test case definitions from <c>shared/test_cases/</c> JSON files
/// and converts them into <see cref="TestCaseDefinition"/> objects compatible with
/// the existing test infrastructure.
/// </summary>
public static class SharedTestCaseLoader
{
    private const string SharedTestCasesDirName = "test_cases";
    private const string RepoMarkerFile = ".git";
    private const string RepoMarkerDir = ".git";

    private static readonly JsonSerializerOptions s_jsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        AllowTrailingCommas = true,
        ReadCommentHandling = JsonCommentHandling.Skip,
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
    };

    // ── JSON-matching DTOs for deserialization ──

    /// <summary>Root JSON file wrapper: { "cases": [...] }</summary>
    private sealed class JsonTestCaseFile
    {
        public JsonTestCaseDto[]? Cases { get; set; }
    }

    /// <summary>Matches the per-case JSON object as defined in schema.json.</summary>
    private sealed class JsonTestCaseDto
    {
        public string Id { get; set; } = "";
        public int Layer { get; set; }
        public string Name { get; set; } = "";
        public string Category { get; set; } = "";
        public string? Description { get; set; }
        public bool Skip { get; set; }
        public string? SkipReason { get; set; }
        public string[]? Tags { get; set; }
        /// <summary>input_images maps to InputImages (string[]), not InputImage (singular)</summary>
        public string[]? InputImages { get; set; }
        /// <summary>pipeline_spec: { nodes, edges, overrides }</summary>
        public JsonPipelineSpecDto? PipelineSpec { get; set; }
        /// <summary>assertions: { tolerance_per_channel, min_psnr, ... }</summary>
        public JsonAssertionsDto? Assertions { get; set; }
    }

    private sealed class JsonPipelineSpecDto
    {
        public JsonNodeDto[]? Nodes { get; set; }
        public JsonEdgeDto[]? Edges { get; set; }
    }

    private sealed class JsonNodeDto
    {
        public string Id { get; set; } = "";
        public string Plugin { get; set; } = "";
        public string? Label { get; set; }
        public bool Enabled { get; set; } = true;
        public Dictionary<string, object>? Params { get; set; }
    }

    private sealed class JsonEdgeDto
    {
        public string From { get; set; } = "";
        public string To { get; set; } = "";
    }

    private sealed class JsonAssertionsDto
    {
        public int TolerancePerChannel { get; set; }
        public string? AssertionType { get; set; }
        public string? ExpectedFormat { get; set; }
        public int? ExpectedWidth { get; set; }
        public int? ExpectedHeight { get; set; }
        public int? ExpectedBitDepth { get; set; }
        public double? MinPsnr { get; set; }
        public double? MinSsim { get; set; }
        public double? MaxDeltaE { get; set; }
        public double? MaxMae { get; set; }
        public bool? CheckMetadata { get; set; }
        public bool? ExpectError { get; set; }
        public string? ExpectedErrorMessage { get; set; }
        public string? GoldenReference { get; set; }
    }

    // ── Public API ──

    /// <summary>
    /// Loads all shared test case definitions from the repository's shared/test_cases/ directory.
    /// </summary>
    public static IReadOnlyList<(SharedTestCase Shared, TestCaseDefinition Definition)> LoadAll()
    {
        var dir = FindSharedTestCasesDir();
        if (!Directory.Exists(dir))
            throw new DirectoryNotFoundException(
                $"Shared test cases directory not found: {dir}. " +
                "Create shared/test_cases/ in the repository root with JSON test case files.");

        var jsonFiles = Directory.GetFiles(dir, "*.json", SearchOption.AllDirectories);
        if (jsonFiles.Length == 0)
            throw new InvalidOperationException(
                $"No .json test case files found in {dir}.");

        var results = new List<(SharedTestCase, TestCaseDefinition)>();
        var errors = new List<string>();

        foreach (var file in jsonFiles)
        {
            try
            {
                var json = File.ReadAllText(file);

                // Step 1: Deserialize the file wrapper { cases: [...] }
                var fileDto = JsonSerializer.Deserialize<JsonTestCaseFile>(json, s_jsonOptions);
                if (fileDto?.Cases is not { Length: > 0 })
                {
                    // Fallback: try direct deserialization of a single test case (no "cases" wrapper)
                    var single = JsonSerializer.Deserialize<JsonTestCaseDto>(json, s_jsonOptions);
                    if (single != null)
                    {
                        var shared = ConvertDtoToShared(single);
                        results.Add((shared, ConvertToTestCaseDefinition(shared)));
                    }
                    continue;
                }

                // Step 2: Convert each DTO to SharedTestCase then to TestCaseDefinition
                foreach (var dto in fileDto.Cases)
                {
                    var shared = ConvertDtoToShared(dto);
                    var definition = ConvertToTestCaseDefinition(shared);
                    results.Add((shared, definition));
                }
            }
            catch (JsonException ex)
            {
                // Deserialization errors must be thrown, not swallowed
                throw new InvalidOperationException(
                    $"Failed to deserialize shared test case '{file}': {ex.Message}", ex);
            }
        }

        if (results.Count == 0 && errors.Count > 0)
            throw new InvalidOperationException(
                $"All shared test cases failed to load:\n{string.Join("\n", errors)}");

        return results;
    }

    /// <summary>
    /// Loads only the <see cref="TestCaseDefinition"/> objects, discarding the raw shared data.
    /// </summary>
    public static IReadOnlyList<TestCaseDefinition> LoadDefinitions()
        => LoadAll().Select(r => r.Definition).ToList();

    /// <summary>
    /// Walks up from the test assembly's directory to find the repository root
    /// (identified by the presence of a .git directory), then returns
    /// &lt;repo-root&gt;/shared/test_cases/.
    /// </summary>
    public static string FindSharedTestCasesDir()
    {
        var baseDir = AppDomain.CurrentDomain.BaseDirectory;

        // Walk up to find repo root
        var dir = new DirectoryInfo(baseDir);
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir.FullName, RepoMarkerDir)) ||
                File.Exists(Path.Combine(dir.FullName, RepoMarkerFile)))
            {
                return Path.Combine(dir.FullName, "shared", SharedTestCasesDirName);
            }
            dir = dir.Parent;
        }

        throw new DirectoryNotFoundException(
            $"Could not find repository root (no {RepoMarkerDir} found) " +
            $"when searching upward from {baseDir}");
    }

    // ── DTO → SharedTestCase conversion ──

    /// <summary>
    /// Converts a JSON-matched <see cref="JsonTestCaseDto"/> to a flat <see cref="SharedTestCase"/>.
    /// Handles: input_images→InputImages, pipeline_spec→Nodes/Edges, assertions→flat props.
    /// </summary>
    private static SharedTestCase ConvertDtoToShared(JsonTestCaseDto dto)
    {
        // input_images array → InputImages (plural); derive singular InputImage from first element
        var inputImages = dto.InputImages;
        var inputImage = inputImages is { Length: > 0 } ? inputImages[0] : "";

        // pipeline_spec.nodes → flat SharedPipelineNode[]
        SharedPipelineNode[]? nodes = null;
        SharedPipelineEdge[]? edges = null;

        if (dto.PipelineSpec?.Nodes is { Length: > 0 } jsonNodes)
        {
            nodes = jsonNodes.Select(n => new SharedPipelineNode
            {
                PluginId = n.Plugin,
                Label = n.Label,
                Enabled = n.Enabled,
                Params = n.Params,
            }).ToArray();

            if (dto.PipelineSpec.Edges is { Length: > 0 } jsonEdges)
            {
                // Edges reference node IDs by string ("from"/"to"), convert to index-based
                var nodeIdToIndex = new Dictionary<string, int>();
                for (int i = 0; i < jsonNodes.Length; i++)
                    nodeIdToIndex[jsonNodes[i].Id] = i;

                edges = jsonEdges.Select(e => new SharedPipelineEdge
                {
                    FromIndex = nodeIdToIndex.TryGetValue(e.From, out var fi) ? fi : -1,
                    ToIndex = nodeIdToIndex.TryGetValue(e.To, out var ti) ? ti : -1,
                }).Where(e => e.FromIndex >= 0 && e.ToIndex >= 0).ToArray();
            }
        }

        // assertions → flat properties
        var assertions = dto.Assertions;
        int tolerancePerChannel = assertions?.TolerancePerChannel ?? 0;
        double? minPsnr = assertions?.MinPsnr;
        double? minSsim = assertions?.MinSsim;
        double? maxDeltaE = assertions?.MaxDeltaE;
        bool expectError = assertions?.ExpectError ?? false;
        string? expectedErrorMessage = assertions?.ExpectedErrorMessage;

        // Output format from assertions.expected_format, falling back to TIFF
        var outputFormat = assertions?.ExpectedFormat ?? "TIFF";
        var outputBitDepth = assertions?.ExpectedBitDepth is > 0 ? (uint?)assertions.ExpectedBitDepth.Value : null;

        return new SharedTestCase
        {
            Id = dto.Id,
            Name = dto.Name,
            Category = dto.Category,
            Tags = dto.Tags ?? [],
            InputImage = inputImage,
            InputImages = inputImages,
            Nodes = nodes,
            Edges = edges,
            OutputFormat = outputFormat,
            OutputBitDepth = outputBitDepth,
            TolerancePerChannel = tolerancePerChannel,
            MinPSNR = minPsnr,
            MinSSIM = minSsim,
            MaxDeltaE = maxDeltaE,
            ExpectError = expectError,
            ExpectedErrorMessage = expectedErrorMessage,
        };
    }

    /// <summary>
    /// Converts a <see cref="SharedTestCase"/> to a <see cref="TestCaseDefinition"/>,
    /// building the PipelineSpec from the Nodes/Edges arrays if present.
    /// </summary>
    public static TestCaseDefinition ConvertToTestCaseDefinition(SharedTestCase shared)
    {
        PipelineSpec? pipeline = null;

        if (shared.Nodes is { Length: > 0 })
        {
            var builder = new TestPipelineBuilder();

            foreach (var node in shared.Nodes)
            {
                builder.AddNode(node.PluginId, node.Label, node.Enabled,
                    node.X, node.Y, node.Params != null ? p =>
                    {
                        foreach (var kv in node.Params)
                            p[kv.Key] = kv.Value;
                    } : null);
            }

            if (shared.Edges is { Length: > 0 })
            {
                foreach (var edge in shared.Edges)
                {
                    builder.Connect(edge.FromIndex, edge.ToIndex);
                }
            }
            else
            {
                // Default: linear chain
                builder.ConnectLinear();
            }

            pipeline = builder.Build();
        }

        return new TestCaseDefinition
        {
            Id = shared.Id,
            Name = shared.Name,
            Category = shared.Category,
            Tags = shared.Tags,
            InputImage = shared.InputImage,
            InputImages = shared.InputImages,
            Pipeline = pipeline,
            OutputFormat = shared.OutputFormat,
            OutputBitDepth = shared.OutputBitDepth,
            OutputLossless = shared.OutputLossless,
            TolerancePerChannel = shared.TolerancePerChannel,
            MinPSNR = shared.MinPSNR,
            MinSSIM = shared.MinSSIM,
            MaxDeltaE = shared.MaxDeltaE,
            ExpectError = shared.ExpectError,
            ExpectedErrorMessage = shared.ExpectedErrorMessage,
            SkipUiChannel = shared.SkipUiChannel,
            SkipApiChannel = shared.SkipApiChannel,
            IsSerialOnly = shared.IsSerialOnly,
        };
    }
}
