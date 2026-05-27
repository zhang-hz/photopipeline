using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class BatchUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public BatchUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> BatchUiTestCases =>
        TestCaseCatalog.GetByCategory("batch")
            .Where(t => !t.SkipUiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(BatchUiTestCases))]
    public async Task BatchProcessViaUi(TestCaseDefinition tc)
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

        var images = tc.InputImages ?? new[] { tc.InputImage };

        foreach (var imageName in images)
        {
            var outputPath = await driver.RunFullWorkflowAsync(
                TestDataCatalog.Instance.GetPath(imageName),
                tc.Pipeline!.Nodes.Select(n => n.PluginId).ToArray(),
                outputFormat: tc.OutputFormat);

            Assert.True(File.Exists(outputPath), $"Batch UI output missing: {outputPath}");
            var fi = new FileInfo(outputPath);
            Assert.True(fi.Length > 0, $"Batch UI output empty: {outputPath}");
            _output.WriteLine($"  ✓ {imageName} → {fi.Length} bytes");
        }

        _output.WriteLine($"PASS UI: {tc.Name}");
    }
}
