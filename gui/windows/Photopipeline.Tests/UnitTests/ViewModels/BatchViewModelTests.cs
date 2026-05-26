using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class BatchViewModelTests
{
    private static BatchViewModel Create(Mock<IBatchService>? batchServiceMock = null)
    {
        var logger = Mock.Of<ILogger<BatchViewModel>>();
        var batchService = batchServiceMock?.Object ?? Mock.Of<IBatchService>();
        return new BatchViewModel(logger, batchService, null!);
    }

    [Fact]
    public void InitialState_Defaults()
    {
        var vm = Create();

        vm.BatchQueue.Should().BeEmpty();
        vm.TotalItems.Should().Be(0);
        vm.CompletedItems.Should().Be(0);
        vm.FailedItems.Should().Be(0);
        vm.OverallProgress.Should().Be(0);
        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
        vm.ElapsedTime.Should().Be("00:00:00");
        vm.EstimatedRemaining.Should().Be("--:--:--");
        vm.SelectedOutputFormat.Should().Be("TIFF");
        vm.JpegQuality.Should().Be(95);
        vm.EmbedMetadata.Should().BeTrue();
    }

    [Fact]
    public void OutputFormats_ContainsExpectedFormats()
    {
        var vm = Create();

        vm.OutputFormats.Should().Contain(new[] { "TIFF", "JPEG", "PNG", "WebP", "HEIF", "AVIF", "JPEG XL" });
    }

    [Fact]
    public void StartBatch_EmptyQueue_ShowsError()
    {
        var vm = Create();

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.ErrorMessage.Should().Contain("No items");
    }

    [Fact]
    public void StartBatch_NoOutputDir_ShowsError()
    {
        var vm = Create();
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "test.jpg" });

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.ErrorMessage.Should().Contain("output directory");
    }

    [Fact]
    public void AddToQueue_AddsImage()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "test.jpg" };

        vm.AddToQueueCommand.Execute(img);

        vm.BatchQueue.Should().HaveCount(1);
    }

    [Fact]
    public void AddToQueue_Null_Noop()
    {
        var vm = Create();

        vm.AddToQueueCommand.Execute(null);

        vm.BatchQueue.Should().BeEmpty();
    }

    [Fact]
    public void AddToQueue_NoDuplicates()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);

        vm.AddToQueueCommand.Execute(img);

        vm.BatchQueue.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveFromQueue_RemovesImage()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "test.jpg" };
        vm.AddToQueueCommand.Execute(img);

        vm.RemoveFromQueueCommand.Execute(img);

        vm.BatchQueue.Should().BeEmpty();
    }

    [Fact]
    public void RemoveFromQueue_NotInQueue_Noop()
    {
        var vm = Create();

        vm.RemoveFromQueueCommand.Execute(new ImageEntry { FileName = "unknown.jpg" });

        vm.BatchQueue.Should().BeEmpty();
    }

    [Fact]
    public void StopBatch_ResetsRunningState()
    {
        var vm = Create();

        vm.StopBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
    }

    [Fact]
    public void ClearCompleted_RemovesProcessedItems()
    {
        var vm = Create();
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "done.jpg", Status = ImageStatus.Overridden });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "pending.jpg", Status = ImageStatus.None });

        vm.ClearCompletedCommand.Execute(null);

        vm.BatchQueue.Should().HaveCount(1);
        vm.BatchQueue[0].FileName.Should().Be("pending.jpg");
    }

    [Fact]
    public void PauseBatch_SetsPausedState()
    {
        var vm = Create();

        vm.PauseBatchCommand.Execute(null);

        vm.IsPaused.Should().BeTrue();
    }

    [Fact]
    public void JpegQuality_AcceptsValidValues()
    {
        var vm = Create();

        vm.JpegQuality = 100;
        vm.JpegQuality.Should().Be(100);

        vm.JpegQuality = 1;
        vm.JpegQuality.Should().Be(1);
    }

    [Fact]
    public void OutputDirectory_CanBeSet()
    {
        var vm = Create();

        vm.OutputDirectory = @"C:\Export\Processed";

        vm.OutputDirectory.Should().Be(@"C:\Export\Processed");
    }
}
