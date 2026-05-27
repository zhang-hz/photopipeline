using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for BatchViewModel.
/// Uses MockBehavior.Strict for service mocks. Every test has a FAIL-able assertion.
/// </summary>
public sealed class BatchViewModelTests : IDisposable
{
    private readonly List<Mock> _strictMocks = new();

    public void Dispose()
    {
        foreach (var mock in _strictMocks)
            mock.VerifyAll();
    }

    private Mock<T> Strict<T>() where T : class
    {
        var mock = new Mock<T>(MockBehavior.Strict);
        _strictMocks.Add(mock);
        return mock;
    }

    private static ILogger<T> AnyLogger<T>() => Mock.Of<ILogger<T>>();

    private BatchViewModel CreateVm(
        Mock<IBatchService>? batchMock = null,
        Mock<IDialogService>? dialogMock = null)
    {
        var bat = batchMock?.Object ?? Mock.Of<IBatchService>();
        var dlg = dialogMock?.Object ?? Mock.Of<IDialogService>();
        return new BatchViewModel(AnyLogger<BatchViewModel>(), bat, dlg);
    }

    private static ImageEntry TestImage(string name = "test.jpg") => new()
    {
        FilePath = $"C:\\photos\\{name}",
        FileName = name,
        Format = "JPEG",
        Width = 1920,
        Height = 1080
    };

    // ═════════════════════════════════════════════════════════════
    // Test 001: InitialState_Defaults
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_001_InitialState_Defaults()
    {
        var vm = CreateVm();

        vm.BatchQueue.Should().BeEmpty();
        vm.TotalItems.Should().Be(0);
        vm.CompletedItems.Should().Be(0);
        vm.FailedItems.Should().Be(0);
        vm.OverallProgress.Should().Be(0);
        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
        vm.SelectedOutputFormat.Should().Be("TIFF");
        vm.JpegQuality.Should().Be(95);
        vm.EmbedMetadata.Should().BeTrue();
        vm.PipelineConfigPath.Should().BeNull();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 002: AddToQueue_AddsSingleImage
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_002_AddToQueue_AddsSingleImage()
    {
        var vm = CreateVm();
        var img = TestImage("photo.jpg");

        vm.AddToQueueCommand.Execute(img);

        vm.BatchQueue.Should().HaveCount(1);
        vm.BatchQueue[0].Should().Be(img);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 003: AddToQueue_Null_Noop
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_003_AddToQueue_Null_Noop()
    {
        var vm = CreateVm();

        vm.AddToQueueCommand.Execute(null);

        vm.BatchQueue.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 004: AddToQueue_Duplicate_NotAdded
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_004_AddToQueue_Duplicate_NotAdded()
    {
        var vm = CreateVm();
        var img = TestImage("photo.jpg");
        vm.AddToQueueCommand.Execute(img);

        vm.AddToQueueCommand.Execute(img);

        vm.BatchQueue.Should().HaveCount(1);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 005: RemoveFromQueue_RemovesImage
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_005_RemoveFromQueue_RemovesImage()
    {
        var vm = CreateVm();
        var img = TestImage("photo.jpg");
        vm.AddToQueueCommand.Execute(img);

        vm.RemoveFromQueueCommand.Execute(img);

        vm.BatchQueue.Should().BeEmpty();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 006: RemoveFromQueue_Null_Noop
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_006_RemoveFromQueue_Null_Noop()
    {
        var vm = CreateVm();
        vm.AddToQueueCommand.Execute(TestImage());

        vm.RemoveFromQueueCommand.Execute(null);

        vm.BatchQueue.Should().HaveCount(1);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 007: StartBatch_EmptyQueue_ShowsError
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_007_StartBatch_EmptyQueue_ShowsError()
    {
        var vm = CreateVm();

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.ErrorMessage.Should().NotBeNull();
        vm.ErrorMessage.Should().Contain("No items");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 008: StartBatch_NoOutputDirectory_ShowsError
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_008_StartBatch_NoOutputDirectory_ShowsError()
    {
        var vm = CreateVm();
        vm.AddToQueueCommand.Execute(TestImage());
        vm.OutputDirectory = ""; // Ensure empty

        vm.StartBatchCommand.Execute(null);

        vm.ErrorMessage.Should().NotBeNull();
        vm.ErrorMessage.Should().Contain("output directory");
        vm.IsRunning.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 009: StartBatch_NoPipelineConfigPath_ShowsError
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_009_StartBatch_NoPipelineConfigPath_ShowsError()
    {
        var vm = CreateVm();
        vm.AddToQueueCommand.Execute(TestImage());
        vm.OutputDirectory = @"C:\Export";
        vm.PipelineConfigPath = null;

        vm.StartBatchCommand.Execute(null);

        vm.ErrorMessage.Should().NotBeNull();
        vm.ErrorMessage.Should().Contain("pipeline");
        vm.IsRunning.Should().BeFalse();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 010: StartBatch_SimulatesProgressThroughMultipleUpdates
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_010_StartBatch_SimulatesProgressThroughMultipleUpdates()
    {
        var batchMock = Strict<IBatchService>();
        batchMock.Setup(b => b.SubmitAsync(It.IsAny<BatchSpec>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync("batch-id-001");
        // Simulate a multi-step progress stream: 25% → 50% → 75% → Done
        batchMock.Setup(b => b.GetProgressAsync("batch-id-001", It.IsAny<CancellationToken>()))
            .Returns(MultiStepBatchProgress());

        var vm = CreateVm(batchMock);
        vm.AddToQueueCommand.Execute(TestImage("a.jpg"));
        vm.AddToQueueCommand.Execute(TestImage("b.jpg"));
        vm.AddToQueueCommand.Execute(TestImage("c.jpg"));
        vm.AddToQueueCommand.Execute(TestImage("d.jpg"));
        vm.OutputDirectory = @"C:\Export";
        vm.PipelineConfigPath = "pipeline-cfg-path";

        var isRunningChanges = new List<bool>();
        vm.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(vm.IsRunning))
                isRunningChanges.Add(vm.IsRunning);
        };

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.OverallProgress.Should().Be(100);
        vm.CompletedItems.Should().Be(4);
        vm.FailedItems.Should().Be(0);
        isRunningChanges.Should().Contain(true, "IsRunning should have transitioned to true during execution");
        batchMock.Verify(b => b.SubmitAsync(
            It.Is<BatchSpec>(s => s.PipelineConfigPath == "pipeline-cfg-path" &&
                                   s.OutputDir == @"C:\Export"),
            It.IsAny<CancellationToken>()), Times.Once);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 011: PauseBatch_SetsPausedState
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_011_PauseBatch_SetsPausedState()
    {
        var vm = CreateVm();

        vm.PauseBatchCommand.Execute(null);

        vm.IsPaused.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════
    // Test 012: StopBatch_ResetsRunningState
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_012_StopBatch_ResetsRunningState()
    {
        var vm = CreateVm();

        vm.StopBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.IsPaused.Should().BeFalse();
        vm.StatusMessage.Should().Be("Stopped");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 013: ClearCompleted_RemovesOnlyProcessedItems
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_013_ClearCompleted_RemovesOnlyProcessedItems()
    {
        var vm = CreateVm();
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "done.jpg", Status = ImageStatus.Overridden });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "pending.jpg", Status = ImageStatus.None });
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "failed.jpg", Status = ImageStatus.Error });

        vm.ClearCompletedCommand.Execute(null);

        vm.BatchQueue.Should().HaveCount(1);
        vm.BatchQueue[0].FileName.Should().Be("pending.jpg");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 014: PipelineConfigPath_CanBeSetFromExternal
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_014_PipelineConfigPath_CanBeSetFromExternal()
    {
        var vm = CreateVm();

        vm.PipelineConfigPath = "pid-ext-001";

        vm.PipelineConfigPath.Should().Be("pid-ext-001");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 015: OutputDirectory_AndAdditionalProperties
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_015_OutputDirectory_AndAdditionalProperties()
    {
        var vm = CreateVm();

        vm.OutputDirectory = @"C:\Export\Processed";
        vm.FileNameTemplate = "{name}_v2";
        vm.JpegQuality = 80;
        vm.EmbedMetadata = false;
        vm.ParallelCount = 4;

        vm.OutputDirectory.Should().Be(@"C:\Export\Processed");
        vm.FileNameTemplate.Should().Be("{name}_v2");
        vm.JpegQuality.Should().Be(80);
        vm.EmbedMetadata.Should().BeFalse();
        vm.ParallelCount.Should().Be(4);
    }

    // ── Helper: multi-step async batch progress stream ──

    private static async IAsyncEnumerable<BatchProgress> MultiStepBatchProgress()
    {
        await Task.Yield();
        yield return new BatchProgress
        {
            Status = BatchStatus.Running,
            TotalFiles = 4,
            CompletedFiles = 1,
            FailedFiles = 0,
            Fraction = 0.25f
        };
        yield return new BatchProgress
        {
            Status = BatchStatus.Running,
            TotalFiles = 4,
            CompletedFiles = 2,
            FailedFiles = 0,
            Fraction = 0.50f
        };
        yield return new BatchProgress
        {
            Status = BatchStatus.Running,
            TotalFiles = 4,
            CompletedFiles = 3,
            FailedFiles = 0,
            Fraction = 0.75f
        };
        yield return new BatchProgress
        {
            Status = BatchStatus.Done,
            TotalFiles = 4,
            CompletedFiles = 4,
            FailedFiles = 0,
            Fraction = 1.0f
        };
    }
}
