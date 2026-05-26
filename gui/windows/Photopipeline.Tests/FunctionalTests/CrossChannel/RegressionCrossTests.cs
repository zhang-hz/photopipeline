using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

public sealed class RegressionCrossTests : CrossChannelTestBase
{
    private readonly ITestOutputHelper _output;

    public RegressionCrossTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> RegressionCrossTestCases =>
        TestCaseCatalog.GetByCategory("regression")
            .Where(t => !t.SkipApiChannel && !t.SkipUiChannel)
            .Take(10)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(RegressionCrossTestCases))]
    public async Task Regression_ApiEqualsUi(TestCaseDefinition tc)
    {
        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        var passed = await VerifyCrossChannelAsync(
            tc.Pipeline!, tc.InputImage, tc.Name, tc.OutputFormat);

        if (!passed)
        {
            _output.WriteLine($"UI channel skipped for {tc.Name}");
            return;
        }

        _output.WriteLine($"PASS CROSS: {tc.Name} — API ≡ UI pixel-perfect");
    }
}
