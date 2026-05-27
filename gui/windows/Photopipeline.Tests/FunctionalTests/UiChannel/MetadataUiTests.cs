using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class MetadataUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public MetadataUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> MetadataUiTestCases =>
        TestCaseCatalog.GetByCategory("metadata")
            .Where(t => !t.SkipUiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(MetadataUiTestCases))]
    public async Task MetadataPassthroughViaUi(TestCaseDefinition tc)
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

        Assert.True(File.Exists(outputPath), $"UI output not found: {outputPath}");
        var fi = new FileInfo(outputPath);
        Assert.True(fi.Length > 0, $"Metadata UI output empty: {outputPath}");

        _output.WriteLine($"PASS UI: {tc.Name} ({fi.Length} bytes)");
    }
}
