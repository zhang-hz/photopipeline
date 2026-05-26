using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

public sealed class ErrorPathApiTests : ApiTestBase
{
    private readonly ITestOutputHelper _output;

    public ErrorPathApiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> ErrorTestCases =>
        TestCaseCatalog.GetByCategory("error")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(ErrorTestCases))]
    public async Task ErrorPath(TestCaseDefinition tc)
    {
        try { await EnsureConnectedAsync(); }
        catch
        {
            _output.WriteLine("Backend not available — skipping error path API test");
            return;
        }

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.tif");

        if (tc.ExpectError)
        {
            await Assert.ThrowsAnyAsync<Exception>(async () =>
            {
                await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);
            });
            _output.WriteLine($"PASS: {tc.Name} — error correctly raised");
        }
        else
        {
            await ExecuteAndGetOutput(tc.Pipeline!, inputPath, outputPath);
            Assert.True(File.Exists(outputPath));
            _output.WriteLine($"PASS: {tc.Name} — completed without error");
        }
    }
}
