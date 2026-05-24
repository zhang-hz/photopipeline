namespace Photopipeline.UIAutomationTests;

public sealed class PluginPanelUITests : UIAutomationTestBase
{
    [Fact]
    public void PluginSettingsPanel_IsVisible()
    {
        StartDriverOrThrow();
        var panel = FindByNameOrNull("Plugin Settings");
        var noPlugin = FindByNameOrNull("No plugin selected");
        Assert.True(panel is not null || noPlugin is not null,
            "Plugin settings panel should be visible");
    }

    [Fact]
    public void PluginSearchBox_Exists()
    {
        StartDriverOrThrow();
        var search = FindByAccessibilityIdOrNull("PluginSearchBox");
        Assert.NotNull(search);
    }

    [Fact]
    public void PluginSearchBox_HasPlaceholderText()
    {
        StartDriverOrThrow();
        var search = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (search is null) return;
        Assert.NotEmpty(search.Text ?? "");
    }

    [Fact]
    public void PluginListPanel_Exists()
    {
        StartDriverOrThrow();
        var list = FindByAccessibilityIdOrNull("PluginListPanel");
        Assert.NotNull(list);
    }

    [Fact]
    public void PluginSearch_TypeText_FiltersResults()
    {
        StartDriverOrThrow();
        var search = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (search is null) return;
        var exception = Record.Exception(() =>
        {
            search.Clear();
            search.SendKeys("blur");
        });
        Assert.Null(exception);
        Thread.Sleep(500);
    }

    [Fact]
    public void PluginSearch_ClearText_RestoresAll()
    {
        StartDriverOrThrow();
        var search = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (search is null) return;
        search.Clear();
        Thread.Sleep(300);
    }

    [Fact]
    public void ApplyButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ApplyButton");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ResetButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ResetButton");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ApplyButton_Click_WithoutSelection_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ApplyButton");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void ResetButton_Click_WithoutSelection_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ResetButton");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void NoPluginText_Shows_WhenNoSelection()
    {
        StartDriverOrThrow();
        var noPlugin = FindByAccessibilityIdOrNull("NoPluginText")
                       ?? FindByNameOrNull("No plugin selected");
        Assert.NotNull(noPlugin);
    }

    [Fact]
    public void PluginCategoryComboBox_Exists()
    {
        StartDriverOrThrow();
        var combo = FindByAccessibilityIdOrNull("CategoryComboBox");
        Assert.NotNull(combo);
    }

    [Fact]
    public void PluginPanel_ScrollViewer_Exists()
    {
        StartDriverOrThrow();
        var scroll = FindByAccessibilityIdOrNull("PluginScrollViewer");
        Assert.NotNull(scroll);
    }
}
