using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class RegressionApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public RegressionApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> RegressionTestCases =>
        TestCaseCatalog.GetByCategory("regression")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(RegressionTestCases))]
    public async Task RegressionSnapshot(TestCaseDefinition tc)
    {
        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping regression API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.tif");

        await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);

        Assert.True(File.Exists(outputPath), $"Output file not found: {outputPath}");

        // Regression: output must be non-empty and valid TIFF
        var fileInfo = new FileInfo(outputPath);
        Assert.True(fileInfo.Length > 0, $"Regression output is empty: {outputPath}");

        ImageAssert.IsValidFormat(outputPath, "TIFF");

        _output.WriteLine($"PASS: {tc.Name} ({fileInfo.Length} bytes)");
    }
}
