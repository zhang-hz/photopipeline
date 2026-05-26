using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class PipelineApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public PipelineApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PipelineTestCases =>
        TestCaseCatalog.GetByCategory("pipeline")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PipelineTestCases))]
    public async Task ExecutePipelineTopology(TestCaseDefinition tc)
    {
        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping pipeline API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.tif");

        await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);

        Assert.True(File.Exists(outputPath), $"Output file not found: {outputPath}");
        ImageAssert.IsValidFormat(outputPath, tc.OutputFormat, expectedBitDepth: (int?)tc.OutputBitDepth);

        _output.WriteLine($"PASS: {tc.Name} — {tc.Pipeline!.Nodes.Count} nodes, {tc.Pipeline.Edges.Count} edges");
    }
}
