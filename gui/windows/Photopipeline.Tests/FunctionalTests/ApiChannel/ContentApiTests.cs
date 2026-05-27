using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for content identity/passthrough.
/// Verifies that decode-then-re-encode pipelines preserve pixel data.
/// </summary>
public sealed class ContentApiTests : ApiTestBase
{
    public ContentApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> ContentTestCases =>
        TestCaseCatalog.GetByCategory("content")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(ContentTestCases))]
    public async Task ContentValidation(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var ext = tc.OutputFormat.ToLowerInvariant();
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.{ext}");

        // Content identity tests may have no Pipeline — use passthrough.
        await ExecuteOrIdentity(tc.Pipeline, inputPath, outputPath);

        // Iron rule 1: Verify output format, dimensions, and non-empty.
        AssertValidOutput(outputPath, tc.OutputFormat);

        // For identity content tests (TolerancePerChannel=0): output must match input.
        if (tc.TolerancePerChannel == 0)
        {
            ImageAssert.PixelsEqual(outputPath, inputPath, tc.TolerancePerChannel);
            _output?.WriteLine($"PASS: {tc.Name} — pixel-identical to input (identity verified)");
        }
        else
        {
            // For content with tolerance, use PSNR with test-case-specified threshold.
            var psnrThreshold = tc.MinPSNR ?? 30.0;
            ImageAssert.PSNRAbove(outputPath, inputPath, minPSNR_dB: psnrThreshold);
            _output?.WriteLine($"PASS: {tc.Name} — PSNR above {psnrThreshold}dB");
        }
    }
}
