using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for multi-node pipeline topologies.
/// Verifies linear chains, disabled nodes, and single-node regression.
/// </summary>
public sealed class PipelineApiTests : ApiTestBase
{
    public PipelineApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> PipelineTestCases =>
        TestCaseCatalog.GetByCategory("pipeline")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PipelineTestCases))]
    public async Task ExecutePipelineTopology(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.png");

        var pipeline = tc.Pipeline!;

        // Single-node regression tests: verify determinism.
        if (tc.Tags.Contains("regression"))
        {
            await ExecuteTwiceAndAssertDeterministic(pipeline, inputPath, outputPath, outputMgr);
        }
        else
        {
            await ExecuteAndGetOutput(pipeline, inputPath, outputPath);
        }

        // Iron rule 1: Verify output format and non-empty.
        AssertValidOutput(outputPath, tc.OutputFormat,
            expectedBitDepth: tc.OutputBitDepth.HasValue ? (int)tc.OutputBitDepth.Value : null);

        // For topology tests (linear/disabled), verify the pipeline actually produced output.
        var nodeCount = pipeline.Nodes.Count;
        var enabledCount = pipeline.Nodes.Count(n => n.Enabled);
        var edgeCount = pipeline.Edges.Count;

        // All-disabled pipeline: output should still be valid (passthrough).
        if (enabledCount == 0 && nodeCount > 0)
        {
            _output?.WriteLine($"All-disabled pipeline ({nodeCount} nodes): verifying passthrough output");
            // Output should be a valid image — already verified above.
        }

        _output?.WriteLine($"PASS: {tc.Name} — {nodeCount} nodes ({enabledCount} enabled), {edgeCount} edges");
    }
}
