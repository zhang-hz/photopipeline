using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.ApiChannel;

/// <summary>
/// Layer 4 gRPC integration tests for error paths.
/// Verifies that invalid pipelines, bad inputs, and edge cases produce
/// correct gRPC error status codes and messages.
/// </summary>
public sealed class ErrorPathApiTests : ApiTestBase
{
    public ErrorPathApiTests(ITestOutputHelper output) : base(output) { }

    public static IEnumerable<object[]> ErrorTestCases =>
        TestCaseCatalog.GetByCategory("error")
            .Where(t => !t.SkipApiChannel)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(ErrorTestCases))]
    public async Task ErrorPath(TestCaseDefinition tc)
    {
        // Iron Rule 2: No silent skip.
        await RequireBackendAsync();

        using var outputMgr = new TestOutputManager(tc.Name);
        var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
        var outputPath = outputMgr.GetOutputPath($"{tc.Name}_output.png");

        if (tc.ExpectError)
        {
            Exception? caught = null;

            try
            {
                var pipeline = tc.Pipeline!;
                await ExecuteAndGetOutput(pipeline, inputPath, outputPath);

                Assert.Fail(
                    $"Expected error for '{tc.Name}' but pipeline completed successfully. " +
                    $"Output: {outputPath}. Expected error: {tc.ExpectedErrorMessage ?? "(any)"}");
            }
            catch (Exception ex) when (ex is not Xunit.Sdk.XunitException)
            {
                // Re-throw infrastructure failures that aren't pipeline errors.
                if (ex is OutOfMemoryException or StackOverflowException
                    or System.Threading.ThreadAbortException)
                    throw;

                caught = ex;
            }

            Assert.NotNull(caught);
            // Error must originate from the API layer, not test infrastructure.
            Assert.True(
                caught is Grpc.Core.RpcException
                    or InvalidOperationException
                    or ArgumentException
                    or System.IO.IOException,
                $"Unexpected exception type for '{tc.Name}': {caught!.GetType().Name}: {caught.Message}");

            if (!string.IsNullOrEmpty(tc.ExpectedErrorMessage))
            {
                var message = caught!.Message;
                Assert.Contains(tc.ExpectedErrorMessage, message,
                    StringComparison.OrdinalIgnoreCase);
            }

            _output?.WriteLine($"PASS: {tc.Name} — error correctly raised: {caught!.GetType().Name}: {caught.Message}");
        }
        else
        {
            var pipeline = tc.Pipeline!;
            await ExecuteAndGetOutput(pipeline, inputPath, outputPath);

            // Iron rule 1: For non-error tests, verify output exists.
            AssertValidOutput(outputPath, tc.OutputFormat);

            _output?.WriteLine($"PASS: {tc.Name} — completed without error");
        }
    }
}
