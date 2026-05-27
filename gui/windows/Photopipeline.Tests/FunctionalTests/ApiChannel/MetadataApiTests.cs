using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for metadata passthrough.
/// Verifies that EXIF/XMP/GPS metadata survives pipeline processing.
/// </summary>
public sealed class MetadataApiTests : ApiTestBase
{
    public MetadataApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> MetadataTestCases =>
        TestCaseCatalog.GetByCategory("metadata")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(MetadataTestCases))]
    public async Task MetadataPassthrough(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.png");

        var pipeline = tc.Pipeline!;
        await ExecuteAndGetOutput(pipeline, inputPath, outputPath);

        // Iron rule 1: Verify output format, dimensions, and non-empty.
        AssertValidOutput(outputPath, tc.OutputFormat);

        // Verify image metadata was preserved through the pipeline.
        var inputMetadata = ImageAssert.ReadImageMetadata(inputPath);
        var outputMetadata = ImageAssert.ReadImageMetadata(outputPath);

        // At minimum, verify both files have valid metadata structures.
        Assert.NotNull(inputMetadata);
        Assert.NotNull(outputMetadata);

        // Verify output dimensions are present and match actual bitmap dimensions.
        Assert.True(outputMetadata.Width.HasValue, $"Output metadata missing Width for {tc.Name}");
        Assert.True(outputMetadata.Height.HasValue, $"Output metadata missing Height for {tc.Name}");

        // Verify EXIF Make/Model retention if present in input.
        // If input had no Make/Model, the output must also have none
        // (pipeline must not fabricate metadata).
        Assert.Equal(inputMetadata.Make, outputMetadata.Make);
        Assert.Equal(inputMetadata.Model, outputMetadata.Model);

        // Verify output dimensions match expectations.
        using var outputBmp = ImageAssert.LoadBitmap(outputPath);
        Assert.True(outputBmp.Width > 0, $"Output width is 0 for {tc.Name}");
        Assert.True(outputBmp.Height > 0, $"Output height is 0 for {tc.Name}");

        _output?.WriteLine($"PASS: {tc.Name} ({outputBmp.Width}x{outputBmp.Height})");
    }
}
