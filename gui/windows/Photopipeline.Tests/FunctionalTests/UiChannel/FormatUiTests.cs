using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class FormatUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public FormatUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> FormatUiTestCases =>
        TestCaseCatalog.GetByCategory("format")
            .Where(t => !t.SkipUiChannel)
            .Take(15)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(FormatUiTestCases))]
    public async Task FormatConversionViaUi(TestCaseDefinition tc)
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

        Assert.True(File.Exists(outputPath), $"UI output not found: {outputPath}");
        var fi = new FileInfo(outputPath);
        Assert.True(fi.Length > 0, $"UI format output empty: {outputPath}");

        // Validate format header
        if (tc.OutputFormat != null)
            ImageAssert.IsValidFormat(outputPath, tc.OutputFormat, expectedBitDepth: (int?)tc.OutputBitDepth);

        _output.WriteLine($"PASS UI: {tc.Name} ({fi.Length} bytes, {tc.OutputFormat})");
    }
}
