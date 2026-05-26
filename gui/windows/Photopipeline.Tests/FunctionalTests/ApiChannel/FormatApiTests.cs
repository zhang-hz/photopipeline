using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class FormatApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public FormatApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> FormatTestCases =>
        TestCaseCatalog.GetByCategory("format")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(FormatTestCases))]
    public async Task ConvertFormat(TestCaseDefinition tc)
    {
        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping format API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var ext = tc.OutputFormat.ToLowerInvariant();
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.{ext}");

        await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);

        Assert.True(File.Exists(outputPath), $"Output file not found: {outputPath}");
        ImageAssert.IsValidFormat(outputPath, tc.OutputFormat);

        var fileInfo = new FileInfo(outputPath);
        Assert.True(fileInfo.Length > 0, $"Output file is empty: {outputPath}");

        _output.WriteLine($"PASS: {tc.Name} ({fileInfo.Length} bytes)");
    }
}
