using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

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
        Assert.True(fi.Length > 0, $"Metadata UI output empty: {outputPath}");

        _output.WriteLine($"PASS UI: {tc.Name} ({fi.Length} bytes)");
    }
}
