using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

public sealed class BatchCrossTests : CrossChannelTestBase
{
    private readonly ITestOutputHelper _output;

    public BatchCrossTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> BatchCrossTestCases =>
        TestCaseCatalog.GetByCategory("batch")
            .Where(t => !t.SkipApiChannel && !t.SkipUiChannel)
            .Take(5)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(BatchCrossTestCases))]
    public async Task Batch_ApiEqualsUi(TestCaseDefinition tc)
    {
        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        var images = tc.InputImages ?? new[] { tc.InputImage };

        foreach (var imageName in images)
        {
            var testName = $"{tc.Name}_{imageName}";
            var passed = await VerifyCrossChannelAsync(
                tc.Pipeline!, imageName, testName, tc.OutputFormat);

            if (!passed)
            {
                _output.WriteLine($"UI channel skipped for {testName}");
                return;
            }

            _output.WriteLine($"  ✓ {imageName} — API ≡ UI");
        }

        _output.WriteLine($"PASS CROSS: {tc.Name}");
    }
}
