namespace Photopipeline.UIAutomationTests;

public sealed class StartupSmokeTests
{
    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void Application_Launches_AndShowsMainWindow() { }

    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void MainWindow_HasCorrectTitle() { }

    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void ThreePanelLayout_RendersOnStartup() { }

    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void BackendStatusIndicator_ShowsOnStart() { }

    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void StatusBar_ShowsReadyState() { }
}
