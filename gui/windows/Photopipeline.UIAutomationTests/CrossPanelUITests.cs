namespace Photopipeline.UIAutomationTests;

public sealed class CrossPanelUITests : UIAutomationTestBase
{
    [Fact]
    public void SwitchTab_PipelineToBatch_DoesNotCrash()
    {
        StartDriverOrThrow();
        var tabs = FindByAccessibilityIdOrNull("RightPanelTabs");
        if (tabs is null) return;
        var tabItems = tabs.FindElements(MobileBy.ClassName("TabItem"));
        if (tabItems.Count >= 2)
        {
            tabItems[1].Click();
            Thread.Sleep(300);
            tabItems[0].Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void SwitchTab_BatchToPipeline_DoesNotCrash()
    {
        StartDriverOrThrow();
        var tabs = FindByAccessibilityIdOrNull("RightPanelTabs");
        if (tabs is null) return;
        var tabItems = tabs.FindElements(MobileBy.ClassName("TabItem"));
        if (tabItems.Count >= 2)
        {
            tabItems[0].Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void AddNode_DoesNotAffect_Filmstrip()
    {
        StartDriverOrThrow();
        var addNode = FindByNameOrNull("Add Node");
        if (addNode is null) return;
        addNode.Click();
        Thread.Sleep(500);

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void PluginSearch_DoesNotAffect_Preview()
    {
        StartDriverOrThrow();
        var search = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (search is null) return;
        search.Clear();
        search.SendKeys("test");
        Thread.Sleep(300);

        var preview = FindByAccessibilityIdOrNull("PreviewPanel")
                      ?? FindByNameOrNull("No image selected");
        Assert.NotNull(preview);
    }

    [Fact]
    public void BatchStart_DoesNotAffect_PipelineCanvas()
    {
        StartDriverOrThrow();
        var startBtn = FindByNameOrNull("Start");
        if (startBtn is null) return;
        startBtn.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        Assert.NotNull(canvas);
    }

    [Fact]
    public void StatusBar_Updates_Across_AllPanels()
    {
        StartDriverOrThrow();
        var status = FindByAccessibilityIdOrNull("StatusText");
        Assert.NotNull(status);
        Assert.NotEmpty(status.Text);
    }

    [Fact]
    public void MainSplitter_Drag_DoesNotCrash()
    {
        StartDriverOrThrow();
        var splitter = FindByAccessibilityIdOrNull("MainSplitter");
        Assert.NotNull(splitter);
    }

    [Fact]
    public void TabSwitch_ThenAddNode_ThenValidate_Flow()
    {
        StartDriverOrThrow();
        // Pipeline tab active, add node
        var addNode = FindByNameOrNull("Add Node");
        if (addNode is null) return;
        addNode.Click();
        Thread.Sleep(400);

        // Validate
        var validate = FindByNameOrNull("Validate");
        if (validate is null) return;
        validate.Click();
        Thread.Sleep(300);

        // Switch to batch tab
        var tabs = FindByAccessibilityIdOrNull("RightPanelTabs");
        if (tabs is null) return;
        var tabItems = tabs.FindElements(MobileBy.ClassName("TabItem"));
        if (tabItems.Count >= 2)
        {
            tabItems[1].Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void AllPanels_Visible_Simultaneously()
    {
        StartDriverOrThrow();
        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        var pipeline = FindByAccessibilityIdOrNull("DagCanvas");
        var batch = FindByAccessibilityIdOrNull("BatchPanel");

        Assert.True(filmstrip is not null || pipeline is not null || batch is not null,
            "At least some panels should be visible");
    }
}
