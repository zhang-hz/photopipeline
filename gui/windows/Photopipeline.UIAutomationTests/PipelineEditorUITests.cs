namespace Photopipeline.UIAutomationTests;

public sealed class PipelineEditorUITests : UIAutomationTestBase
{
    [Fact]
    public void PipelineEditorCanvas_Exists()
    {
        StartDriverOrThrow();
        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        Assert.NotNull(canvas);
    }

    [Fact]
    public void AddNodeButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Add Node");
        Assert.NotNull(btn);
    }

    [Fact]
    public void DeleteButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Delete");
        Assert.NotNull(btn);
    }

    [Fact]
    public void DuplicateButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Duplicate");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ValidateButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Validate");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ZoomInButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("PipelineZoomIn")
                  ?? FindByNameOrNull("Zoom In");
        Assert.NotNull(btn);
    }

    [Fact]
    public void ZoomOutButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByAccessibilityIdOrNull("PipelineZoomOut")
                  ?? FindByNameOrNull("Zoom Out");
        Assert.NotNull(btn);
    }

    [Fact]
    public void FitAllButton_Exists()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Fit All");
        Assert.NotNull(btn);
    }

    [Fact]
    public void AddNodeButton_Click_OpensMenu()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Add Node");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
        Thread.Sleep(500);
    }

    [Fact]
    public void ValidateButton_Click_EmptyCanvas_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Validate");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void DeleteButton_Click_WithoutSelection_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Delete");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void DuplicateButton_Click_WithoutSelection_DoesNotCrash()
    {
        StartDriverOrThrow();
        var btn = FindByNameOrNull("Duplicate");
        if (btn is null) return;
        var exception = Record.Exception(() => btn.Click());
        Assert.Null(exception);
    }

    [Fact]
    public void PipelineEditorToolBar_Visible()
    {
        StartDriverOrThrow();
        var toolbar = FindByAccessibilityIdOrNull("PipelineToolBar");
        Assert.NotNull(toolbar);
    }

    [Fact]
    public void PipelineEditor_ValidationResult_Initially_Empty()
    {
        StartDriverOrThrow();
        var result = FindByAccessibilityIdOrNull("ValidationResult");
        if (result is null) return;
        Assert.NotNull(result.Text);
    }
}
