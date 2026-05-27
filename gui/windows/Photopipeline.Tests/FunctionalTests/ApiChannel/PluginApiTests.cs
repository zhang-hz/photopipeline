using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for single-plugin pipelines.
/// Each test exercises a plugin through the full gRPC stack and verifies
/// the output with pixel-level assertions.
/// </summary>
public sealed class PluginApiTests : ApiTestBase
{
    public PluginApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> PluginTestCases =>
        TestCaseCatalog.GetByCategory("plugin")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PluginTestCases))]
    public async Task ExecutePluginPipeline(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip — backend unavailable kills the test.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var pipeline = tc.Pipeline!;

        // Encoder plugins produce native-format encoded bytes, so the output extension
        // must match the encoder's output format for SkiaSharp to decode correctly.
        string outputExt = pipeline.Nodes.FirstOrDefault()?.PluginId switch
        {
            "photopipeline.plugins.png_encoder" => "png",
            "photopipeline.plugins.tiff_encoder" => "tif",
            "photopipeline.plugins.avif_encoder" => "avif",
            "photopipeline.plugins.jxl_encoder" => "jxl",
            "photopipeline.plugins.heif_encoder" => "heif",
            _ => "png",
        };
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.{outputExt}");

        // Regression and default-param tests: verify determinism (iron rule 5).
        if (tc.Tags.Contains("regression") || tc.Tags.Contains("default_params"))
        {
            await ExecuteTwiceAndAssertDeterministic(pipeline, inputPath, outputPath, outputMgr);
        }
        else
        {
            await ExecuteAndGetOutput(pipeline, inputPath, outputPath);
        }

        // Iron rule 1: Every test must have at least one assertion that CAN fail.
        AssertValidOutput(outputPath, tc.OutputFormat);

        // Adversarial check: for parameter permutation tests, verify output differs
        // from input (proves the plugin actually executed).
        //
        // Skip conditions:
        // 1. Format/encoder plugins — their job is to encode, not modify pixels
        // 2. raw_input — input-only plugin, passes pixels through unchanged
        // 3. external_data_plugin tag — plugins that need external files (LUT, lens profile, ONNX model)
        // 4. zero_effect tag — parameter values that legitimately produce no pixel change (angle=0, intensity=0, etc.)
        // 5. Dimensions differ — resize/crop changes size, proving the plugin ran
        bool isFormatPlugin = pipeline.Nodes.FirstOrDefault()?.PluginId?.Contains("_encoder") == true
            || pipeline.Nodes.FirstOrDefault()?.PluginId == "photopipeline.plugins.raw_input";
        bool skipAdversarial = isFormatPlugin
            || tc.Tags.Contains("zero_effect")
            || tc.Tags.Contains("external_data_plugin");

        if (tc.Tags.Contains("parameter_permutation") && !skipAdversarial)
        {
            try
            {
                using var inputBmp = ImageAssert.LoadBitmap(inputPath);
                using var outputBmp = ImageAssert.LoadBitmap(outputPath);

                if (inputBmp.Width == outputBmp.Width && inputBmp.Height == outputBmp.Height)
                {
                    ImageAssert.PixelsEqual(outputPath, inputPath, tolerancePerChannel: 0);
                    Assert.Fail(
                        $"PLUGIN HAD NO EFFECT: Output is pixel-identical to input. " +
                        $"Test case '{tc.Name}' — plugin likely did not execute. " +
                        $"Parameters: {string.Join(", ", pipeline.Nodes.FirstOrDefault()?.Params.Select(kv => $"{kv.Key}={kv.Value}") ?? Array.Empty<string>())}");
                }
                // Size difference proves plugin ran (e.g. resize, crop)
            }
            catch (Xunit.Sdk.XunitException ex) when (ex.Message.StartsWith("Pixel mismatch"))
            {
                // Expected: plugin changed pixel values. This is the success path.
            }
        }

        _output?.WriteLine($"PASS: {tc.Name}");
    }
}
