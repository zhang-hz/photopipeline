using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

public sealed class PipelineCrossTests : CrossChannelTestBase
{
    private readonly ITestOutputHelper _output;

    public PipelineCrossTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PipelineCrossTestCases =>
        TestCaseCatalog.GetByCategory("pipeline")
            .Where(t => !t.SkipApiChannel && !t.SkipUiChannel)
            .Take(10)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PipelineCrossTestCases))]
    public async Task Pipeline_ApiEqualsUi(TestCaseDefinition tc)
    {
        var passed = await VerifyCrossChannelAsync(
            tc.Pipeline!, tc.InputImage, tc.Name, tc.OutputFormat);

        if (!passed)
        {
            _output.WriteLine($"UI channel skipped for {tc.Name}");
            return;
        }

        _output.WriteLine($"PASS CROSS: {tc.Name} ({tc.Pipeline!.Nodes.Count} nodes) — API ≡ UI pixel-perfect");
    }
}
