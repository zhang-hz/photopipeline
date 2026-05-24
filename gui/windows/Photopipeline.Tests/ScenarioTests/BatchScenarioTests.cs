using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class BatchScenarioTests
{
    // ═══ Queue management ═══
    [Fact]
    public void NewBatch_Queue_IsEmpty()
    {
        var h = new ViewModelTestHarness();
        Assert.Empty(h.Batch.BatchQueue);
    }

    [Fact]
    public void NewBatch_Status_IsIdle()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal("Idle", h.Batch.StatusText);
    }

    [Fact]
    public void NewBatch_Progress_IsZero()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal(0, h.Batch.OverallProgress);
    }

    [Fact]
    public void NewBatch_NotRunning()
    {
        var h = new ViewModelTestHarness();
        Assert.False(h.Batch.IsRunning);
        Assert.False(h.Batch.IsPaused);
    }

    [Fact]
    public void NewBatch_OutputFormats_HasDefaults()
    {
        var h = new ViewModelTestHarness();
        Assert.NotEmpty(h.Batch.OutputFormats);
        Assert.Contains("TIFF", h.Batch.OutputFormats);
        Assert.Contains("JPEG", h.Batch.OutputFormats);
        Assert.Contains("PNG", h.Batch.OutputFormats);
        Assert.Contains("WebP", h.Batch.OutputFormats);
        Assert.Contains("HEIF", h.Batch.OutputFormats);
    }

    [Fact]
    public void NewBatch_DefaultFormat_IsTIFF()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal("TIFF", h.Batch.SelectedOutputFormat);
    }

    [Fact]
    public void AddToQueue_AddsImage_IncreasesCount()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Batch.AddToQueueCommand.Execute(img);
        Assert.Single(h.Batch.BatchQueue);
        Assert.Equal(1, h.Batch.TotalItems);
    }

    [Fact]
    public void AddToQueue_MultipleImages_AllAdded()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 5; i++)
            h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = $"/img_{i}.png", FileName = $"img_{i}.png" });
        Assert.Equal(5, h.Batch.TotalItems);
    }

    [Fact]
    public void AddToQueue_Duplicate_NotAdded()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Batch.AddToQueueCommand.Execute(img);
        h.Batch.AddToQueueCommand.Execute(img);
        Assert.Single(h.Batch.BatchQueue);
        Assert.Equal(1, h.Batch.TotalItems);
    }

    [Fact]
    public void RemoveFromQueue_RemovesImage_DecreasesCount()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Batch.AddToQueueCommand.Execute(img);
        h.Batch.RemoveFromQueueCommand.Execute(img);
        Assert.Empty(h.Batch.BatchQueue);
        Assert.Equal(0, h.Batch.TotalItems);
    }

    [Fact]
    public void RemoveFromQueue_NonExistent_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        var exception = Record.Exception(() => h.Batch.RemoveFromQueueCommand.Execute(img));
        Assert.Null(exception);
    }

    [Fact]
    public void ClearCompleted_RemovesCompletedItems()
    {
        var h = new ViewModelTestHarness();
        var done = new ImageEntry { FilePath = "/done.png", FileName = "done.png", ProcessingProgress = 1.0 };
        var pending = new ImageEntry { FilePath = "/pending.png", FileName = "pending.png", ProcessingProgress = 0.0 };
        h.Batch.AddToQueueCommand.Execute(done);
        h.Batch.AddToQueueCommand.Execute(pending);
        h.Batch.ClearCompletedCommand.Execute(null);
        Assert.Single(h.Batch.BatchQueue);
        Assert.Equal("pending.png", h.Batch.BatchQueue[0].FileName);
    }

    // ═══ Batch lifecycle ═══
    [Fact]
    public void StartBatch_EmptyQueue_DoesNothing()
    {
        var h = new ViewModelTestHarness();
        h.Batch.StartBatchCommand.Execute(null);
        Assert.False(h.Batch.IsRunning);
    }

    [Fact]
    public void StartBatch_WithItems_SetsRunning()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Assert.True(h.Batch.IsRunning);
        Assert.False(h.Batch.IsPaused);
    }

    [Fact]
    public void StartBatch_WithItems_SetsTotalItems()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 3; i++)
            h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = $"/{i}.png", FileName = $"{i}.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Assert.Equal(3, h.Batch.TotalItems);
    }

    [Fact]
    public void StartBatch_ResetsCounters()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Assert.Equal(0, h.Batch.CompletedItems);
        Assert.Equal(0, h.Batch.FailedItems);
        Assert.Equal(0, h.Batch.OverallProgress);
    }

    [Fact]
    public void StartBatch_SetsStatusText()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Assert.Equal("Processing...", h.Batch.StatusText);
    }

    [Fact]
    public void PauseBatch_SetsPaused()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        h.Batch.PauseBatchCommand.Execute(null);
        Assert.True(h.Batch.IsPaused);
        Assert.Equal("Paused", h.Batch.StatusText);
        Assert.True(h.Batch.IsRunning);
    }

    [Fact]
    public void ResumeBatch_ClearsPaused()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        h.Batch.PauseBatchCommand.Execute(null);
        h.Batch.ResumeBatchCommand.Execute(null);
        Assert.False(h.Batch.IsPaused);
        Assert.Equal("Processing...", h.Batch.StatusText);
    }

    [Fact]
    public void StopBatch_ClearsRunning()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        h.Batch.StopBatchCommand.Execute(null);
        Assert.False(h.Batch.IsRunning);
        Assert.False(h.Batch.IsPaused);
        Assert.Equal("Stopped", h.Batch.StatusText);
    }

    [Fact]
    public void StopBatch_ElapsedTime_Set()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Thread.Sleep(1500);
        h.Batch.StopBatchCommand.Execute(null);
        Assert.NotEqual("00:00:00", h.Batch.ElapsedTime);
    }

    [Fact]
    public void PauseBatch_WithoutStart_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Batch.PauseBatchCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void StopBatch_WithoutStart_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        var exception = Record.Exception(() => h.Batch.StopBatchCommand.Execute(null));
        Assert.Null(exception);
    }

    // ═══ Output format ═══
    [Fact]
    public void OutputFormat_Default_IsTIFF()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal("TIFF", h.Batch.SelectedOutputFormat);
    }

    [Fact]
    public void OutputFormat_SwitchToJpeg_Updates()
    {
        var h = new ViewModelTestHarness();
        h.Batch.SelectedOutputFormat = "JPEG";
        Assert.Equal("JPEG", h.Batch.SelectedOutputFormat);
    }

    [Fact]
    public void OutputFormat_SwitchToPng_Updates()
    {
        var h = new ViewModelTestHarness();
        h.Batch.SelectedOutputFormat = "PNG";
        Assert.Equal("PNG", h.Batch.SelectedOutputFormat);
    }

    [Fact]
    public void OutputFormat_SwitchToWebP_Updates()
    {
        var h = new ViewModelTestHarness();
        h.Batch.SelectedOutputFormat = "WebP";
        Assert.Equal("WebP", h.Batch.SelectedOutputFormat);
    }

    // ═══ Settings ═══
    [Fact]
    public void JpegQuality_Default_Is95()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal(95, h.Batch.JpegQuality);
    }

    [Fact]
    public void JpegQuality_CanChange()
    {
        var h = new ViewModelTestHarness();
        h.Batch.JpegQuality = 75;
        Assert.Equal(75, h.Batch.JpegQuality);
    }

    [Fact]
    public void EmbedMetadata_Default_IsTrue()
    {
        var h = new ViewModelTestHarness();
        Assert.True(h.Batch.EmbedMetadata);
    }

    [Fact]
    public void OutputDirectory_Default_IsEmpty()
    {
        var h = new ViewModelTestHarness();
        Assert.Equal(string.Empty, h.Batch.OutputDirectory);
    }

    [Fact]
    public void OutputDirectory_CanSet()
    {
        var h = new ViewModelTestHarness();
        h.Batch.OutputDirectory = @"C:\output";
        Assert.Equal(@"C:\output", h.Batch.OutputDirectory);
    }

    // ═══ Multiple operations ═══
    [Fact]
    public void StartStopRestart_CountersReset()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        h.Batch.StopBatchCommand.Execute(null);
        // Add more and restart
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/b.png", FileName = "b.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Assert.Equal(2, h.Batch.TotalItems);
        Assert.True(h.Batch.IsRunning);
    }

    [Fact]
    public void PauseResume_MultipleTimes_DoesNotCorruptState()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);

        for (int i = 0; i < 3; i++)
        {
            h.Batch.PauseBatchCommand.Execute(null);
            Assert.True(h.Batch.IsPaused);
            h.Batch.ResumeBatchCommand.Execute(null);
            Assert.False(h.Batch.IsPaused);
        }
    }

    [Fact]
    public void ElapsedTime_FormattedCorrectly()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Thread.Sleep(1500);
        h.Batch.StopBatchCommand.Execute(null);
        // Elapsed should be in hh:mm:ss format and not 00:00:00
        Assert.Contains(":", h.Batch.ElapsedTime);
        Assert.NotEqual("00:00:00", h.Batch.ElapsedTime);
    }

    [Fact]
    public void Queue_Capacity_LargeBatchHandled()
    {
        var h = new ViewModelTestHarness();
        for (int i = 0; i < 100; i++)
            h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = $"/img_{i}.png", FileName = $"img_{i}.png" });
        Assert.Equal(100, h.Batch.TotalItems);
    }

    // ═══ Progress simulation ═══
    [Fact]
    public void Progress_CanSet()
    {
        var h = new ViewModelTestHarness();
        h.Batch.OverallProgress = 0.5;
        Assert.Equal(0.5, h.Batch.OverallProgress);
    }

    [Fact]
    public void Progress_CanSetTo100()
    {
        var h = new ViewModelTestHarness();
        h.Batch.OverallProgress = 100;
        Assert.Equal(100, h.Batch.OverallProgress);
    }

    [Fact]
    public void CompletedItems_CanIncrement()
    {
        var h = new ViewModelTestHarness();
        h.Batch.CompletedItems = 5;
        Assert.Equal(5, h.Batch.CompletedItems);
    }

    [Fact]
    public void FailedItems_CanSet()
    {
        var h = new ViewModelTestHarness();
        h.Batch.FailedItems = 2;
        Assert.Equal(2, h.Batch.FailedItems);
    }
}
