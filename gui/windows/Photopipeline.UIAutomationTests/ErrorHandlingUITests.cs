namespace Photopipeline.UIAutomationTests;

public sealed class ErrorHandlingUITests : UIAutomationTestBase
{
    [Fact]
    public void ProcessAll_WithoutSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByName("Process All");
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void Export_WithoutSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ExportButton")
                  ?? FindByNameOrNull("Export");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void Validate_EmptyCanvas_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Validate");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void RemoveImage_NoSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Remove");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void StartBatch_EmptyQueue_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Start");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void StopBatch_NotRunning_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Stop");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void PauseBatch_NotRunning_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Pause");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void ApplyParameters_NoSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ApplyButton");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void ResetParameters_NoSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ResetButton");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void DeleteNode_NoSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Delete");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void DuplicateNode_NoSelection_NoCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Duplicate");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void RapidButtonClicks_NoCrash()
    {
        StartDriverOrThrow();
        var addBtn = FindByNameOrNull("Add");
        var processBtn = FindByNameOrNull("Process All");
        if (addBtn is null || processBtn is null) return;

        var exception = Record.Exception(() =>
        {
            for (int i = 0; i < 5; i++)
            {
                addBtn.Click();
                Thread.Sleep(100);
                processBtn.Click();
                Thread.Sleep(100);
            }
        });
        Assert.Null(exception);
    }
}
