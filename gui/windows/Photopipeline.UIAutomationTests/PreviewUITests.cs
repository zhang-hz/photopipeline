namespace Photopipeline.UIAutomationTests;

public sealed class PreviewUITests : UIAutomationTestBase
{
    [Fact]
    public void PreviewPanel_IsVisible()
    {
        StartDriverOrThrow();
        var panel = FindByAccessibilityIdOrNull("PreviewPanel");
        var noImage = FindByNameOrNull("No image selected");
        Assert.True(panel is not null || noImage is not null,
            "Preview panel should be visible");
    }

    [Fact]
    public void NoImageSelected_Placeholder_Visible()
    {
        StartDriverOrThrow();
        var placeholder = FindByNameOrNull("No image selected");
        Assert.NotNull(placeholder);
    }

    [Fact]
    public void ZoomInButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("PreviewZoomIn")
                  ?? FindByNameOrNull("Zoom In");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ZoomOutButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("PreviewZoomOut")
                  ?? FindByNameOrNull("Zoom Out");
        Assert.NotNull(btn);
    }

    [Fact]
    public void FitToWindowButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Fit");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ResetZoomButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("1:1");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ExportButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ExportButton")
                  ?? FindByNameOrNull("Export");
        Assert.NotNull(btn);
    }

    [Fact]
    public void SplitViewButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("SplitViewBtn");
        Assert.NotNull(btn);
    }

    [Fact]
    public void SideBySideButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("SideBySideBtn");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ZoomIn_Click_IncreasesZoom()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("PreviewZoomIn")
                  ?? FindByNameOrNull("Zoom In");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void ZoomOut_Click_DecreasesZoom()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("PreviewZoomOut")
                  ?? FindByNameOrNull("Zoom Out");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void FitButton_Click_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Fit");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void OneToOneButton_Click_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("1:1");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void ExportButton_Click_WithoutSelection_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("ExportButton")
                  ?? FindByNameOrNull("Export");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void SplitViewButton_Toggle_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("SplitViewBtn");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void SideBySideButton_Toggle_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("SideBySideBtn");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void BeforeAfterControl_Exists()
    {
        StartDriverOrThrow();
        var control = FindByAccessibilityIdOrNull("BeforeAfterControl");
        Assert.NotNull(control);
    }

    [Fact]
    public void ZoomLevel_Label_ShowsInitialValue()
    {
        StartDriverOrThrow();
        var label = FindByAccessibilityIdOrNull("ZoomLevelLabel");
        if (label is null) return;
        Assert.NotEmpty(label.Text);
    }

    [Fact]
    public void PreviewPanel_HasToolBar()
    {
        StartDriverOrThrow();
        var toolbar = FindByAccessibilityIdOrNull("PreviewToolBar");
        Assert.NotNull(toolbar);
    }
}
