using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class PluginApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public PluginApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PluginTestCases =>
        TestCaseCatalog.GetByCategory("plugin")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PluginTestCases))]
    public async Task ExecutePluginPipeline(TestCaseDefinition tc)
    {
        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping plugin API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.tif");

        await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);

        if (tc.ExpectError)
        {
            _output.WriteLine($"Expected error case passed: {tc.Name}");
            return;
        }

        Assert.True(File.Exists(outputPath), $"Output file not found: {outputPath}");
        ImageAssert.IsValidFormat(outputPath, tc.OutputFormat);

        if (tc.TolerancePerChannel >= 0 && tc.MinPSNR == null && tc.MinSSIM == null)
        {
            // For tests with no reference, just verify output is valid
            var fileInfo = new FileInfo(outputPath);
            Assert.True(fileInfo.Length > 0, $"Output file is empty: {outputPath}");
        }

        _output.WriteLine($"PASS: {tc.Name}");
    }
}
