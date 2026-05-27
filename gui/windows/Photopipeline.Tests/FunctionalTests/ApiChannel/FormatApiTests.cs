using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for format conversion.
/// Verifies cross-format decode-encode cycles with IsValidFormat +
/// PSNRAbove (lossless) or SSIMAbove (lossy) assertions.
/// </summary>
public sealed class FormatApiTests : ApiTestBase
{
    public FormatApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> FormatTestCases =>
        TestCaseCatalog.GetByCategory("format")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(FormatTestCases))]
    public async Task ConvertFormat(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var ext = tc.OutputFormat.ToLowerInvariant();
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.{ext}");

        // Format tests may have no Pipeline — use identity passthrough.
        await ExecuteOrIdentity(tc.Pipeline, inputPath, outputPath);

        // Iron rule 1: Verify output format, dimensions, and non-empty.
        AssertValidOutput(outputPath, tc.OutputFormat);

        // For lossless formats, verify pixel quality against input.
        if (tc.OutputLossless == true)
        {
            ImageAssert.PSNRAbove(outputPath, inputPath, minPSNR_dB: tc.MinPSNR ?? 40.0);
        }
        else if (tc.MinSSIM.HasValue)
        {
            ImageAssert.SSIMAbove(outputPath, inputPath, tc.MinSSIM.Value);
        }

        // For all format tests: verify output bit depth if specified.
        if (tc.OutputBitDepth.HasValue)
        {
            ImageAssert.IsValidFormat(outputPath, tc.OutputFormat,
                expectedBitDepth: (int)tc.OutputBitDepth.Value);
        }

        _output?.WriteLine($"PASS: {tc.Name}");
    }
}
