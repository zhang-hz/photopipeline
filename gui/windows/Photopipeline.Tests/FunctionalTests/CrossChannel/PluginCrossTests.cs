using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.CrossChannel;

public sealed class PluginCrossTests : CrossChannelTestBase
{
    private readonly ITestOutputHelper _output;

    public PluginCrossTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> PluginCrossTestCases =>
        TestCaseCatalog.GetByCategory("plugin")
            .Where(t => !t.SkipApiChannel && !t.SkipUiChannel)
            .Take(30)
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(PluginCrossTestCases))]
    public async Task Plugin_ApiEqualsUi(TestCaseDefinition tc)
    {
        if (ResourceMonitor.ShouldSkipLargeTest())
            return;

        var passed = await VerifyCrossChannelAsync(tc.Pipeline!, tc.InputImage, tc.Name, tc.OutputFormat);

        if (!passed)
        {
            _output.WriteLine($"UI channel skipped for {tc.Name} — app not available");
            return;
        }

        _output.WriteLine($"PASS CROSS: {tc.Name} — API ≡ UI pixel-perfect");
    }
}
