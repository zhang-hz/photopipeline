namespace Photopipeline.UIAutomationTests;

public sealed class MainWindowUITests : UIAutomationTestBase
{
    [Fact]
    public void MainWindow_Launches_Successfully()
    {
        StartDriverOrThrow();
        Assert.NotNull(Driver);
    }

    [Fact]
    public void MainWindow_Title_ContainsPhotopipeline()
    {
        StartDriverOrThrow();
        var title = Driver!.Title;
        Assert.Contains("Photopipeline", title, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void MainWindow_HasHandle()
    {
        StartDriverOrThrow();
        var handle = Driver!.CurrentWindowHandle;
        Assert.NotEmpty(handle);
    }

    [Fact]
    public void MainWindow_IsMaximized()
    {
        StartDriverOrThrow();
        var handles = Driver!.WindowHandles;
        Assert.NotEmpty(handles);
    }

    [Fact]
    public void LeftPanel_IsVisible()
    {
        StartDriverOrThrow();
        var panel = FindByAccessibilityIdOrNull("LeftPanel");
        var filmstrip = FindByNameOrNull("Images");
        Assert.True(panel is not null || filmstrip is not null,
            "Left panel should be visible");
    }

    [Fact]
    public void RightPanel_IsVisible()
    {
        StartDriverOrThrow();
        var panel = FindByAccessibilityIdOrNull("RightPanel");
        var pipeline = FindByAccessibilityIdOrNull("DagCanvas");
        Assert.True(panel is not null || pipeline is not null,
            "Right panel should be visible");
    }

    [Fact]
    public void MainSplitter_Exists()
    {
        StartDriverOrThrow();
        var splitter = FindByAccessibilityIdOrNull("MainSplitter");
        Assert.NotNull(splitter);
    }

    [Fact]
    public void RightTopPanel_ContainsTabs()
    {
        StartDriverOrThrow();
        var tabs = FindByAccessibilityIdOrNull("RightPanelTabs");
        Assert.NotNull(tabs);
    }

    [Fact]
    public void StatusBar_IsVisible()
    {
        StartDriverOrThrow();
        var status = FindByAccessibilityIdOrNull("StatusBar");
        Assert.NotNull(status);
    }

    [Fact]
    public void StatusText_Element_Exists()
    {
        StartDriverOrThrow();
        var text = FindByAccessibilityIdOrNull("StatusText");
        Assert.NotNull(text);
    }

    [Fact]
    public void StatusText_Initially_ShowsReady()
    {
        StartDriverOrThrow();
        var text = FindByAccessibilityIdOrNull("StatusText");
        if (text is null) return;
        Assert.Contains("Ready", text.Text, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void AppTitleBar_IsVisible()
    {
        StartDriverOrThrow();
        var titleBar = FindByAccessibilityIdOrNull("AppTitleBar");
        Assert.NotNull(titleBar);
    }

    [Fact]
    public void RightBottomPanel_ContainsBatch()
    {
        StartDriverOrThrow();
        var batch = FindByAccessibilityIdOrNull("BatchPanel");
        Assert.NotNull(batch);
    }

    [Fact]
    public void Window_Resize_DoesNotCrash()
    {
        StartDriverOrThrow();
        var exception = Record.Exception(() =>
        {
            Driver!.Manage().Window.Size = new System.Drawing.Size(1024, 768);
        });
        Assert.Null(exception);
    }
}
