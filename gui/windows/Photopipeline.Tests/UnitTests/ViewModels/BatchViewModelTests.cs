namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class BatchViewModelTests
{
    [Fact]
    public void BatchViewModel_Creation_Defaults()
    {
        var vm = new BatchViewModel();

        vm.BatchQueue.Should().BeEmpty();
        vm.TotalItems.Should().Be(0);
        vm.CompletedItems.Should().Be(0);
        vm.FailedItems.Should().Be(0);
        vm.OverallProgress.Should().Be(0);
        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
        vm.StatusText.Should().Be("Idle");
        vm.ElapsedTime.Should().Be("00:00:00");
        vm.EstimatedRemaining.Should().Be("--:--:--");
        vm.SelectedOutputFormat.Should().Be("TIFF");
        vm.JpegQuality.Should().Be(95);
        vm.EmbedMetadata.Should().BeTrue();
    }

    [Fact]
    public void OutputFormats_ContainsExpectedFormats()
    {
        var vm = new BatchViewModel();

        vm.OutputFormats.Should().Contain(new[] { "TIFF", "JPEG", "PNG", "WebP", "HEIF" });
    }

    [Fact]
    public void AddToQueue_AddsImage()
    {
        var vm = new BatchViewModel();
        var image = new ImageEntry { FileName = "test.jpg" };

        vm.AddToQueueCommand.Execute(image);

        vm.BatchQueue.Should().HaveCount(1);
        vm.TotalItems.Should().Be(1);
    }

    [Fact]
    public void AddToQueue_DoesNotDuplicate()
    {
        var vm = new BatchViewModel();
        var image = new ImageEntry { FileName = "test.jpg" };

        vm.AddToQueueCommand.Execute(image);
        vm.AddToQueueCommand.Execute(image);

        vm.BatchQueue.Should().HaveCount(1);
        vm.TotalItems.Should().Be(1);
    }

    [Fact]
    public void AddToQueue_MultipleImages()
    {
        var vm = new BatchViewModel();
        var img1 = new ImageEntry { FileName = "a.jpg" };
        var img2 = new ImageEntry { FileName = "b.jpg" };
        var img3 = new ImageEntry { FileName = "c.jpg" };

        vm.AddToQueueCommand.Execute(img1);
        vm.AddToQueueCommand.Execute(img2);
        vm.AddToQueueCommand.Execute(img3);

        vm.BatchQueue.Should().HaveCount(3);
        vm.TotalItems.Should().Be(3);
    }

    [Fact]
    public void RemoveFromQueue_RemovesImage()
    {
        var vm = new BatchViewModel();
        var img1 = new ImageEntry { FileName = "a.jpg" };
        var img2 = new ImageEntry { FileName = "b.jpg" };
        vm.AddToQueueCommand.Execute(img1);
        vm.AddToQueueCommand.Execute(img2);

        vm.RemoveFromQueueCommand.Execute(img1);

        vm.BatchQueue.Should().HaveCount(1);
        vm.TotalItems.Should().Be(1);
        vm.BatchQueue[0].Should().Be(img2);
    }

    [Fact]
    public void RemoveFromQueue_UpdatesTotalItems()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);

        vm.RemoveFromQueueCommand.Execute(img);

        vm.BatchQueue.Should().BeEmpty();
        vm.TotalItems.Should().Be(0);
    }

    [Fact]
    public void StartBatch_SetsRunningState()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeTrue();
        vm.IsPaused.Should().BeFalse();
        vm.CompletedItems.Should().Be(0);
        vm.FailedItems.Should().Be(0);
        vm.OverallProgress.Should().Be(0);
        vm.StatusText.Should().Be("Processing...");
    }

    [Fact]
    public void StartBatch_EmptyQueue_DoesNotStart()
    {
        var vm = new BatchViewModel();

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.StatusText.Should().Be("Idle");
    }

    [Fact]
    public void PauseBatch_SetsPausedState()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);
        vm.StartBatchCommand.Execute(null);

        vm.PauseBatchCommand.Execute(null);

        vm.IsRunning.Should().BeTrue();
        vm.IsPaused.Should().BeTrue();
        vm.StatusText.Should().Be("Paused");
    }

    [Fact]
    public void ResumeBatch_ClearsPausedState()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);
        vm.StartBatchCommand.Execute(null);
        vm.PauseBatchCommand.Execute(null);

        vm.ResumeBatchCommand.Execute(null);

        vm.IsPaused.Should().BeFalse();
        vm.StatusText.Should().Be("Processing...");
    }

    [Fact]
    public void StopBatch_SetsStoppedState()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);
        vm.StartBatchCommand.Execute(null);

        vm.StopBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
        vm.StatusText.Should().Be("Stopped");
    }

    [Fact]
    public void ClearCompleted_RemovesFullyProcessedItems()
    {
        var vm = new BatchViewModel();
        var img1 = new ImageEntry { FileName = "a.jpg", ProcessingProgress = 1.0 };
        var img2 = new ImageEntry { FileName = "b.jpg", ProcessingProgress = 0.5 };
        var img3 = new ImageEntry { FileName = "c.jpg", ProcessingProgress = 1.0 };
        vm.AddToQueueCommand.Execute(img1);
        vm.AddToQueueCommand.Execute(img2);
        vm.AddToQueueCommand.Execute(img3);

        vm.ClearCompletedCommand.Execute(null);

        vm.BatchQueue.Should().HaveCount(1);
        vm.BatchQueue[0].Should().Be(img2);
        vm.TotalItems.Should().Be(1);
    }

    [Fact]
    public void ExportFormat_Selection_UpdatesOutputFormat()
    {
        var vm = new BatchViewModel();

        vm.SelectedOutputFormat = "JPEG";

        vm.SelectedOutputFormat.Should().Be("JPEG");
    }

    [Fact]
    public void JpegQuality_Range_AcceptsValidValues()
    {
        var vm = new BatchViewModel();

        vm.JpegQuality = 100;
        vm.JpegQuality.Should().Be(100);

        vm.JpegQuality = 1;
        vm.JpegQuality.Should().Be(1);

        vm.JpegQuality = 85;
        vm.JpegQuality.Should().Be(85);
    }

    [Fact]
    public void EmbedMetadata_TogglesCorrectly()
    {
        var vm = new BatchViewModel();

        vm.EmbedMetadata.Should().BeTrue();

        vm.EmbedMetadata = false;
        vm.EmbedMetadata.Should().BeFalse();

        vm.EmbedMetadata = true;
        vm.EmbedMetadata.Should().BeTrue();
    }

    [Fact]
    public void OutputDirectory_CanBeSet()
    {
        var vm = new BatchViewModel();

        vm.OutputDirectory = @"C:\Exports\Processed";

        vm.OutputDirectory.Should().Be(@"C:\Exports\Processed");
    }

    [Fact]
    public void Progress_UpdatesDuringProcessing()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);
        vm.StartBatchCommand.Execute(null);

        vm.OverallProgress = 0.5;
        vm.CompletedItems = 3;
        vm.FailedItems = 1;

        vm.OverallProgress.Should().Be(0.5);
        vm.CompletedItems.Should().Be(3);
        vm.FailedItems.Should().Be(1);
    }

    [Fact]
    public void ElapsedTime_UpdatesWhileRunning()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);
        vm.StartBatchCommand.Execute(null);

        vm.ElapsedTime = "00:01:23";

        vm.ElapsedTime.Should().Be("00:01:23");
    }

    [Fact]
    public void EstimatedRemaining_ShowsDashWhenIdle()
    {
        var vm = new BatchViewModel();

        vm.EstimatedRemaining.Should().Be("--:--:--");
    }

    [Fact]
    public void StopBatch_ElapsedTime_Retained()
    {
        var vm = new BatchViewModel();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);
        vm.StartBatchCommand.Execute(null);

        vm.StopBatchCommand.Execute(null);

        vm.ElapsedTime.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void ErrorHandling_FailedItemsIncremented()
    {
        var vm = new BatchViewModel();
        var img1 = new ImageEntry { FileName = "ok.jpg" };
        var img2 = new ImageEntry { FileName = "fail.jpg", HasError = true, ErrorMessage = "Decode error" };
        vm.AddToQueueCommand.Execute(img1);
        vm.AddToQueueCommand.Execute(img2);
        vm.StartBatchCommand.Execute(null);

        vm.FailedItems = 1;
        vm.CompletedItems = 1;
        vm.TotalItems = 2;

        vm.FailedItems.Should().Be(1);
        vm.CompletedItems.Should().Be(1);
    }

    [Fact]
    public void RemoveFromQueue_ImageNotInQueue_Noop()
    {
        var vm = new BatchViewModel();
        var image = new ImageEntry { FileName = "not_in_queue.jpg" };

        vm.RemoveFromQueueCommand.Execute(image);

        vm.TotalItems.Should().Be(0);
    }
}
