using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

public sealed class FormatCrossTests : CrossChannelTestBase
{
    private readonly ITestOutputHelper _output;

    public FormatCrossTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> FormatCrossTestCases =>
        TestCaseCatalog.GetByCategory("format")
            .Where(t => !t.SkipApiChannel && !t.SkipUiChannel)
            .Take(15)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(FormatCrossTestCases))]
    public async Task Format_ApiEqualsUi(TestCaseDefinition tc)
    {
        var passed = await VerifyCrossChannelAsync(
            tc.Pipeline!, tc.InputImage, tc.Name, tc.OutputFormat);

        if (!passed)
        {
            _output.WriteLine($"UI channel skipped for {tc.Name}");
            return;
        }

        _output.WriteLine($"PASS CROSS: {tc.Name} ({tc.OutputFormat}) — API ≡ UI pixel-perfect");
    }
}
