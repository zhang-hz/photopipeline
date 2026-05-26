using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class RegressionUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public RegressionUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> RegressionUiTestCases =>
        TestCaseCatalog.GetByCategory("regression")
            .Where(t => !t.SkipUiChannel)
            .Take(15)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(RegressionUiTestCases))]
    public async Task RegressionViaUi(TestCaseDefinition tc)
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

        Assert.True(File.Exists(outputPath), $"Regression UI output not found: {outputPath}");
        var fi = new FileInfo(outputPath);
        Assert.True(fi.Length > 0, $"Regression UI output empty: {outputPath}");

        ImageAssert.IsValidFormat(outputPath, tc.OutputFormat);

        _output.WriteLine($"PASS UI: {tc.Name} ({fi.Length} bytes)");
    }
}
