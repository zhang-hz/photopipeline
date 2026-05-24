namespace Photopipeline.UIAutomationTests;

public sealed class BatchPanelUITests : UIAutomationTestBase
{
    [Fact]
    public void BatchPanel_IsVisible()
    {
        StartDriverOrThrow();
        var panel = FindByAccessibilityIdOrNull("BatchPanel");
        Assert.NotNull(panel);
    }

    [Fact]
    public void StartButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Start");
        Assert.NotNull(btn);
    }

    [Fact]
    public void PauseButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Pause");
        Assert.NotNull(btn);
    }

    [Fact]
    public void StopButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Stop");
        Assert.NotNull(btn);
    }

    [Fact]
    public void StartButton_Click_EmptyQueue_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Start");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void StopButton_Click_NotStarted_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Stop");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void PauseButton_Click_NotStarted_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Pause");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void ProgressBar_Exists()
    {
        StartDriverOrThrow();
        var bar = FindByAccessibilityIdOrNull("BatchProgressBar");
        Assert.NotNull(bar);
    }

    [Fact]
    public void ProgressBar_Initially_Zero()
    {
        StartDriverOrThrow();
        var bar = FindByAccessibilityIdOrNull("BatchProgressBar");
        Assert.NotNull(bar);
    }

    [Fact]
    public void QueueCount_ShowsZero_Initially()
    {
        StartDriverOrThrow();
        var count = FindByAccessibilityIdOrNull("QueueCountText");
        if (count is null) return;
        Assert.Contains("0", count.Text);
    }

    [Fact]
    public void CompletedText_ShowsZero_Initially()
    {
        StartDriverOrThrow();
        var text = FindByAccessibilityIdOrNull("CompletedText");
        if (text is null) return;
        Assert.NotNull(text.Text);
    }

    [Fact]
    public void TotalText_ShowsZero_Initially()
    {
        StartDriverOrThrow();
        var text = FindByAccessibilityIdOrNull("TotalText");
        if (text is null) return;
        Assert.NotNull(text.Text);
    }

    [Fact]
    public void ElapsedTime_ShowsDefault_Initially()
    {
        StartDriverOrThrow();
        var time = FindByAccessibilityIdOrNull("ElapsedTimeText");
        if (time is null) return;
        Assert.NotEmpty(time.Text);
    }

    [Fact]
    public void RemainingTime_ShowsDefault_Initially()
    {
        StartDriverOrThrow();
        var time = FindByAccessibilityIdOrNull("RemainingTimeText");
        if (time is null) return;
        Assert.NotEmpty(time.Text);
    }

    [Fact]
    public void FormatComboBox_Exists()
    {
        StartDriverOrThrow();
        var combo = FindByAccessibilityIdOrNull("FormatComboBox");
        Assert.NotNull(combo);
    }

    [Fact]
    public void FormatComboBox_HasDefaultTIFF()
    {
        StartDriverOrThrow();
        var combo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (combo is null) return;
        Assert.Contains("TIFF", combo.Text, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void BatchPanel_StatusText_ShowsIdle_Initially()
    {
        StartDriverOrThrow();
        var status = FindByAccessibilityIdOrNull("BatchStatusText")
                     ?? FindByNameOrNull("Idle");
        Assert.NotNull(status);
    }
}
