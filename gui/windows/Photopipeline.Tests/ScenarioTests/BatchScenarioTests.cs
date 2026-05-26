using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class BatchScenarioTests
{
    private static BatchViewModel Create(Mock<IBatchService>? batchServiceMock = null)
    {
        var logger = Mock.Of<ILogger<BatchViewModel>>();
        var batchService = batchServiceMock?.Object ?? Mock.Of<IBatchService>();
        return new BatchViewModel(logger, batchService, null!);
    }

    [Fact]
    public void BuildQueue_ConfigureStart()
    {
        var vm = Create();
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "a.dng" });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "b.dng" });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "c.dng" });
        vm.OutputDirectory = @"C:\Output";
        vm.SelectedOutputFormat = "JPEG";
        vm.JpegQuality = 85;

        vm.BatchQueue.Should().HaveCount(3);
        vm.OutputDirectory.Should().Be(@"C:\Output");
        vm.SelectedOutputFormat.Should().Be("JPEG");
    }

    [Fact]
    public void Start_EmptyQueue_Rejected()
    {
        var vm = Create();
        vm.OutputDirectory = @"C:\Output";

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.ErrorMessage.Should().Contain("No items");
    }

    [Fact]
    public void Start_NoOutputDir_Rejected()
    {
        var vm = Create();
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "a.dng" });

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.ErrorMessage.Should().Contain("output directory");
    }

    [Fact]
    public void PauseResume_Lifecycle()
    {
        var vm = Create();

        vm.PauseBatchCommand.Execute(null);
        vm.IsPaused.Should().BeTrue();

        vm.StopBatchCommand.Execute(null);
        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
    }

    [Fact]
    public void ClearCompleted_LeavesPendingItems()
    {
        var vm = Create();
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "done.tif", Status = ImageStatus.Overridden });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "failed.tif", Status = ImageStatus.Error });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "pending.tif", Status = ImageStatus.None });

        vm.ClearCompletedCommand.Execute(null);

        vm.BatchQueue.Should().HaveCount(1);
        vm.BatchQueue[0].FileName.Should().Be("pending.tif");
    }

    [Fact]
    public void QueueNoDuplicates_WhenAddingSameImage()
    {
        var vm = Create();
        var img = new ImageEntry { FileName = "photo.dng" };
        vm.AddToQueueCommand.Execute(img);
        vm.AddToQueueCommand.Execute(img);
        vm.AddToQueueCommand.Execute(img);

        vm.BatchQueue.Should().HaveCount(1);
    }
}
