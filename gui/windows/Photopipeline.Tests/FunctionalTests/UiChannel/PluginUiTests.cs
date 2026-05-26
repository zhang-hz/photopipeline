using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class PluginUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public PluginUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PluginUiTestCases =>
        TestCaseCatalog.GetByCategory("plugin")
            .Where(t => !t.SkipUiChannel)
            .Take(20) // Limit UI tests to keep runtime manageable
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PluginUiTestCases))]
    public async Task PluginViaUi(TestCaseDefinition tc)
    {
        if (!File.Exists(AppPath))
        {
            _output.WriteLine($"App not found at {AppPath} — skipping UI test");
            return;
        }

        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        using var driver = new UiTestDriver();
        var outputPath = await driver.RunFullWorkflowAsync(
            TestDataCatalog.Instance.GetPath(tc.InputImage),
            tc.Pipeline!,
            outputFormat: tc.OutputFormat);

        Assert.True(File.Exists(outputPath), $"UI output not found: {outputPath}");
        var fi = new FileInfo(outputPath);
        Assert.True(fi.Length > 0, $"UI output empty: {outputPath}");

        _output.WriteLine($"PASS UI: {tc.Name} ({fi.Length} bytes)");
    }
}
