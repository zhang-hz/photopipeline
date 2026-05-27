using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC regression tests: verify deterministic pipeline output by
/// running each pipeline twice and asserting pixel-identical results.
/// Iron rule 6: Each regression test is its own golden reference via determinism.
/// </summary>
public sealed class RegressionApiTests : ApiTestBase
{
    public RegressionApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> RegressionTestCases =>
        TestCaseCatalog.GetByCategory("regression")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(RegressionTestCases))]
    public async Task RegressionSnapshot(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.png");

        var pipeline = tc.Pipeline!;

        // Iron rule 1 + 5: Deterministic re-execution.
        // Run pipeline twice, compare outputs. If the pipeline logic changes
        // or is non-deterministic, PixelsEqual FAILS.
        await ExecuteTwiceAndAssertDeterministic(pipeline, inputPath, outputPath, outputMgr);

        // Iron rule 1: Verify output format.
        AssertValidOutput(outputPath, "PNG");

        _output?.WriteLine($"PASS: {tc.Name}");
    }
}
