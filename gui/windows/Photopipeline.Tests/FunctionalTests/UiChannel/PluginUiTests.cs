using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class PluginUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public PluginUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PluginUiTestCases =>
        TestCaseCatalog.GetByCategory("plugin")
            .Where(t => !t.SkipUiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PluginUiTestCases))]
    public async Task PluginViaUi(TestCaseDefinition tc)
    {
        if (!File.Exists(AppPath))
        {
            throw new FileNotFoundException(
                $"UI test cannot run: App not found at {AppPath}. Build the project first.");
        }

        if (ResourceMonitor.ShouldSkipLargeTest())
            throw new InvalidOperationException(
                "Insufficient resources to run UI test; refusing to silently skip per Iron Rule 2.");

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
        Assert.True(fi.Length > 0, $"UI output empty: {outputPath}");

        _output.WriteLine($"PASS UI: {tc.Name} ({fi.Length} bytes)");
    }
}
