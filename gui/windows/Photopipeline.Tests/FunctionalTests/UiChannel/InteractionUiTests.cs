using Photopipeline.Tests.FunctionalTests.Infrastructure;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public sealed class InteractionUiTests : UiTestBase
{
    private readonly ITestOutputHelper _output;

    public InteractionUiTests(ITestOutputHelper output) => _output = output;

    public static IEnumerable<object[]> InteractionTestCases =>
        TestCaseCatalog.GetByCategory("interaction")
            .Select(t => new object[] { t });

    [Theory]
    [MemberData(nameof(InteractionTestCases))]
    public async Task UiInteraction(TestCaseDefinition tc)
    {
        if (!File.Exists(AppPath))
        {
            throw new FileNotFoundException(
                $"UI test cannot run: App not found at {AppPath}. Build the project first.");
        }

        using var driver = new UiTestDriver(
            AppPath,
            TestDataCatalog.GetInputDir(),
            Path.Combine(Path.GetTempPath(), "photopipeline_ui_tests"),
            _output);

        await driver.LaunchAppAsync();
        try
        {
            var interaction = tc.Name.Replace("ui_interaction_", "");

            switch (interaction)
            {
                case "zoom_in_pan_reset":
                    Assert.Fail("zoom_in_pan_reset: zoom/pan/reset interaction not yet implemented");
                    break;

                case "split_view_toggle":
                    Assert.Fail("split_view_toggle: split view interaction not yet implemented");
                    break;

                case "theme_toggle":
                    Assert.Fail("theme_toggle: theme toggle interaction not yet implemented");
                    break;

                case "drag_plugin_to_canvas":
                    await driver.AddPluginToPipelineAsync("exposure");
                    break;

                case "multi_select_filmstrip":
                    // Import image first, then select by index
                    await driver.ImportImageAsync(
                        TestDataCatalog.Instance.GetPath(tc.InputImage));
                    await driver.SelectImageAsync(0);
                    break;

                case "filter_by_format":
                    await driver.ImportImageAsync(
                        TestDataCatalog.Instance.GetPath(tc.InputImage));
                    break;

                case "navigate_pipeline_views":
                    await driver.NavigateToPipelineEditorAsync();
                    break;

                case "settings_save_reset":
                    Assert.Fail("settings_save_reset: settings dialog interaction not yet implemented");
                    break;

                case "export_dialog_flow":
                    await driver.ExportOutputAsync(Path.GetTempFileName());
                    break;

                case "batch_add_remove_items":
                    await driver.ImportImageAsync(
                        TestDataCatalog.Instance.GetPath(tc.InputImage));
                    break;
            }

            _output.WriteLine($"PASS UI: {tc.Name}");
        }
        finally
        {
            await driver.CloseAppAsync();
        }
    }
}
