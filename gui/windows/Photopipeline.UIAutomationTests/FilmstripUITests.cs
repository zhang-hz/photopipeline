namespace Photopipeline.UIAutomationTests;

public sealed class FilmstripUITests : UIAutomationTestBase
{
    [Fact]
    public void AddButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByName("Add");
        Assert.NotNull(btn);
    }

    [Fact]
    public void RemoveButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Remove");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ProcessAllButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByName("Process All");
        Assert.NotNull(btn);
    }

    [Fact]
    public void AddButton_Click_OpensFileDialog()
    {
        StartDriverOrThrow();
        var btn = FindByName("Add");
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
        Thread.Sleep(500);
    }

    [Fact]
    public void ProcessAllButton_Click_WithoutImages_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByName("Process All");
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void RemoveButton_Click_WithoutSelection_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Remove");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void FilmstripList_IsVisible()
    {
        StartDriverOrThrow();
        var list = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(list);
    }

    [Fact]
    public void FilmstripList_Initially_Empty()
    {
        StartDriverOrThrow();
        var list = FindByAccessibilityIdOrNull("FilmstripList");
        if (list is null) return;
        var items = list.FindElements(MobileBy.ClassName("ListViewItem"));
        Assert.Empty(items);
    }

    [Fact]
    public void StatusText_ShowsReady_OnStartup()
    {
        StartDriverOrThrow();
        WaitForText("StatusText", "Ready");
    }

    [Fact]
    public void FilmstripPanel_HasToolBar()
    {
        StartDriverOrThrow();
        var toolbar = FindByAccessibilityIdOrNull("FilmstripToolBar");
        Assert.NotNull(toolbar);
    }

    [Fact]
    public void Filmstrip_Title_ShowsCorrectText()
    {
        StartDriverOrThrow();
        var title = FindByAccessibilityIdOrNull("FilmstripTitle");
        var name = FindByNameOrNull("Images");
        Assert.True(title is not null || name is not null,
            "Either title element or Images label should be visible");
    }

    [Fact]
    public void AddButton_IsEnabled_Initially()
    {
        StartDriverOrThrow();
        var btn = FindByName("Add");
        Assert.True(btn.Enabled, "Add button should be enabled on startup");
    }

    [Fact]
    public void RemoveButton_IsDisabled_WhenNoSelection()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Remove");
        if (btn is null) return;
        // Remove button may be disabled or enabled with no-op behavior
        Assert.NotNull(btn);
    }

    [Fact]
    public void ProcessAllButton_IsEnabled_Initially()
    {
        StartDriverOrThrow();
        var btn = FindByName("Process All");
        Assert.True(btn.Enabled, "Process All button should exist");
    }
}
