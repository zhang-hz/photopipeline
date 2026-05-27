using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class PipelineUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public PipelineUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PipelineUiTestCases =>
        TestCaseCatalog.GetByCategory("pipeline")
            .Where(t => !t.SkipUiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PipelineUiTestCases))]
    public async Task PipelineTopologyViaUi(TestCaseDefinition tc)
    {
        if (!File.Exists(AppPath))
        {
            throw new FileNotFoundException(
                $"UI test cannot run: App not found at {AppPath}. Build the project first.");
        }

        using var driver = new UiTestDriver(
            AppPath,
            TestDataCatalog.GetInputDir(),
            Path.Combine(Path.GetTempPath(), "photopipeline_ui_tests"),
            _output);
        var outputPath = await driver.RunFullWorkflowAsync(
            TestDataCatalog.Instance.GetPath(tc.InputImage),
            tc.Pipeline!.Nodes.Select(n => n.PluginId).ToArray(),
            outputFormat: tc.OutputFormat);

        Assert.True(File.Exists(outputPath), $"Pipeline UI output not found: {outputPath}");
        var fi = new FileInfo(outputPath);
        Assert.True(fi.Length > 0, $"Pipeline UI output empty: {outputPath}");

        ImageAssert.IsValidFormat(outputPath, tc.OutputFormat, expectedBitDepth: (int?)tc.OutputBitDepth);

        _output.WriteLine($"PASS UI: {tc.Name} — {tc.Pipeline!.Nodes.Count} nodes, {tc.Pipeline.Edges.Count} edges");
    }
}
