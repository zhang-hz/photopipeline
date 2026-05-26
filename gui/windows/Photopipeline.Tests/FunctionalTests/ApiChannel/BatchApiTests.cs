using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class BatchApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public BatchApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> BatchTestCases =>
        TestCaseCatalog.GetByCategory("batch")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(BatchTestCases))]
    public async Task BatchProcess(TestCaseDefinition tc)
    {
        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping batch API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);

        if (tc.InputImages is { Length: > 0 })
        {
            foreach (var imageName in tc.InputImages)
            {
                var inputPath = TestDataCatalog.Instance.GetPath(imageName);
                var outputPath = outputMgr.GetOutputPath($"{tc.Name}_{imageName}_output.tif");

                if (tc.Pipeline != null)
                    await ExecuteAndGetOutput(tc.Pipeline, inputPath, outputPath);
                else
                    continue;

                Assert.True(File.Exists(outputPath), $"Batch output missing: {outputPath}");
                var fi = new FileInfo(outputPath);
                Assert.True(fi.Length > 0, $"Batch output empty: {outputPath}");
                _output.WriteLine($"  ✓ {imageName} → {fi.Length} bytes");
            }
        }
        else
        {
            var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
            var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.tif");

            if (tc.Pipeline != null)
                await ExecuteAndGetOutput(tc.Pipeline, inputPath, outputPath);

            Assert.True(File.Exists(outputPath), $"Batch output missing: {outputPath}");
        }

        _output.WriteLine($"PASS: {tc.Name}");
    }
}
