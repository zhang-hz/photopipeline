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
            .Take(6)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(BatchUiTestCases))]
    public async Task BatchProcessViaUi(TestCaseDefinition tc)
    {
        if (!File.Exists(AppPath))
        {
            _output.WriteLine($"App not found at {AppPath} — skipping UI test");
            return;
        }

        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        using var driver = new UiTestDriver();

        var images = tc.InputImages ?? new[] { tc.InputImage };

        foreach (var imageName in images)
        {
            var outputPath = await driver.RunFullWorkflowAsync(
                TestDataCatalog.Instance.GetPath(imageName),
                tc.Pipeline!,
                outputFormat: tc.OutputFormat);

            Assert.True(File.Exists(outputPath), $"Batch UI output missing: {outputPath}");
            var fi = new FileInfo(outputPath);
            Assert.True(fi.Length > 0, $"Batch UI output empty: {outputPath}");
            _output.WriteLine($"  ✓ {imageName} → {fi.Length} bytes");
        }

        _output.WriteLine($"PASS UI: {tc.Name}");
    }
}
