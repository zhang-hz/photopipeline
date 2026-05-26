using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class PipelineUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public PipelineUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PipelineUiTestCases =>
        TestCaseCatalog.GetByCategory("pipeline")
            .Where(t => !t.SkipUiChannel)
            .Take(12)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PipelineUiTestCases))]
    public async Task PipelineTopologyViaUi(TestCaseDefinition tc)
    {
        if (!File.Exists(AppPath))
        {
            _output.WriteLine($"App not found at {AppPath} — skipping UI test");
            return;
        }

        using var driver = new UiTestDriver();
        var outputPath = await driver.RunFullWorkflowAsync(
            TestDataCatalog.Instance.GetPath(tc.InputImage),
            tc.Pipeline!,
            outputFormat: tc.OutputFormat);

        Assert.True(File.Exists(outputPath), $"Pipeline UI output not found: {outputPath}");
        var fi = new FileInfo(outputPath);
        Assert.True(fi.Length > 0, $"Pipeline UI output empty: {outputPath}");

        ImageAssert.IsValidFormat(outputPath, tc.OutputFormat, expectedBitDepth: (int?)tc.OutputBitDepth);

        _output.WriteLine($"PASS UI: {tc.Name} — {tc.Pipeline!.Nodes.Count} nodes, {tc.Pipeline.Edges.Count} edges");
    }
}
