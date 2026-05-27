using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for batch processing.
/// Verifies all output files and progress callbacks for multi-image batches.
/// </summary>
public sealed class BatchApiTests : ApiTestBase
{
    public BatchApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> BatchTestCases =>
        TestCaseCatalog.GetByCategory("batch")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(BatchTestCases))]
    public async Task BatchProcess(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip. ResourceMonitor.ShouldSkipLargeTest removed —
        // if resources are insufficient the test must FAIL, not silently pass.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var completedCount = 0;
        var failedCount = 0;

        if (tc.InputImages is { Length: > 0 })
        {
            foreach (var imageName in tc.InputImages)
            {
                var inputPath = TestDataCatalog.Instance.GetPath(imageName);
                var outputPath = outputMgr.GetOutputPath($"{tc.Name}_{imageName}_output.png");

                try
                {
                    var pipeline = tc.Pipeline!;
                    await ExecuteAndGetOutput(pipeline, inputPath, outputPath);

                    // Iron rule 1: Verify each output file.
                    AssertValidOutput(outputPath, tc.OutputFormat);
                    completedCount++;
                    _output?.WriteLine($"  OK: {imageName}");
                }
                catch (Exception ex)
                {
                    failedCount++;
                    _output?.WriteLine($"  FAIL: {imageName} — {ex.Message}");
                    // For batch tests: individual failures should not prevent
                    // verifying other outputs. But the test as a whole should report
                    // failures.
                }
            }

            // Iron rule 1: All images must complete successfully.
            Assert.Equal(tc.InputImages.Length, completedCount);
            Assert.Equal(0, failedCount);
        }
        else
        {
            // Single-image batch
            var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
            var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.png");

            var pipeline = tc.Pipeline!;
            await ExecuteAndGetOutput(pipeline, inputPath, outputPath);

            AssertValidOutput(outputPath, tc.OutputFormat);
            completedCount = 1;
        }

        _output?.WriteLine($"PASS: {tc.Name} — {completedCount} completed, {failedCount} failed");
    }
}
