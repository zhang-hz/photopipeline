using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class ContentApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public ContentApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> ContentTestCases =>
        TestCaseCatalog.GetByCategory("content")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(ContentTestCases))]
    public async Task ContentValidation(TestCaseDefinition tc)
    {
        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping content API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.tif");

        await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);

        Assert.True(File.Exists(outputPath), $"Output file not found: {outputPath}");
        var fileInfo = new FileInfo(outputPath);
        Assert.True(fileInfo.Length > 0, $"Content output is empty: {outputPath}");

        _output.WriteLine($"PASS: {tc.Name} ({fileInfo.Length} bytes)");
    }
}
