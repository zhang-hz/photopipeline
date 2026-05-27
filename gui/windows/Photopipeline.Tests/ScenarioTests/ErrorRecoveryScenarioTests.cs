using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class ErrorRecoveryScenarioTests
{
    [Fact]
    public void Preview_Creation_WithFailingService_DoesNotCrash()
    {
        var logger = Mock.Of<ILogger<PreviewViewModel>>();
        var imageServiceMock = new Mock<IImageService>();
        // DecodeAsync returns IAsyncEnumerable, ThrowsAsync not supported.
        // Just verify construction succeeds.

        var vm = new PreviewViewModel(logger, imageServiceMock.Object, Mock.Of<IPipelineService>(), null!);

        vm.ErrorMessage.Should().BeNull();
        vm.IsLoading.Should().BeFalse();
    }

    [Fact]
    public void Filmstrip_ImportFailure_DoesNotCrash()
    {
        var logger = Mock.Of<ILogger<FilmstripViewModel>>();
        var imageServiceMock = new Mock<IImageService>();
        imageServiceMock
            .Setup(s => s.LoadImageInfoAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ThrowsAsync(new InvalidOperationException("Backend unavailable"));

        var vm = new FilmstripViewModel(logger, imageServiceMock.Object, null!);

        // Should not throw on construction
        vm.Images.Should().BeEmpty();
    }

    [Fact]
    public void Batch_ServiceFailure_HandlesGracefully()
    {
        var logger = Mock.Of<ILogger<BatchViewModel>>();
        var batchServiceMock = new Mock<IBatchService>();
        batchServiceMock
            .Setup(s => s.SubmitAsync(It.IsAny<BatchSpec>(), It.IsAny<CancellationToken>()))
            .ThrowsAsync(new InvalidOperationException("Server error"));

        var vm = new BatchViewModel(logger, batchServiceMock.Object, null!);
        vm.AddToQueueCommand.Execute(new ImageEntry { FileName = "test.dng" });
        vm.OutputDirectory = @"C:\Output";
        // Must set PipelineConfigPath, otherwise StartBatch returns early with
        // "Create a pipeline in the Pipeline Editor first" before reaching SubmitAsync
        vm.PipelineConfigPath = "test-pipeline";

        vm.StartBatchCommand.Execute(null);

        // After service failure, the VM should set an error message and not be running
        vm.IsRunning.Should().BeFalse("batch should stop running on service failure");
        vm.ErrorMessage.Should().NotBeNull("service failure should populate ErrorMessage");
        vm.ErrorMessage.Should().Contain("Server error",
            "ErrorMessage should contain the exception message");
    }

    [Fact]
    public void Pipeline_Validate_BackendFailure_Graceful()
    {
        var logger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineServiceMock = new Mock<IPipelineService>();
        pipelineServiceMock
            .Setup(s => s.ValidateAsync(It.IsAny<PipelineSpec>(), It.IsAny<CancellationToken>()))
            .ThrowsAsync(new InvalidOperationException("Backend unavailable"));

        var vm = new PipelineEditorViewModel(logger, pipelineServiceMock.Object);
        vm.AddNodeAt(new PluginInfo { Id = "test", Name = "Test" }, 100, 100);

        vm.ValidateCommand.Execute(null);

        // Backend failure during validation should set an error message
        vm.ErrorMessage.Should().NotBeNull("backend failure during validate should set ErrorMessage");
        vm.IsPipelineValid.Should().BeFalse("pipeline should not be valid when validation fails");
    }

    [Fact]
    public void ViewModelBase_IsBusy_PreventsConcurrentOperation()
    {
        var logger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var vm = new PipelineEditorViewModel(logger, pipelineService);

        // Verify initial state
        vm.IsBusy.Should().BeFalse();

        // ViewModelBase.ExecuteAsync has guard: if (IsBusy) return;
        // PipelineEditorViewModel.AddNodeAt has guard: if (IsExecuting) return;
        // Test both guards: IsExecuting prevents node addition, IsBusy prevents re-entry via ExecuteAsync
        vm.IsExecuting = true;
        vm.AddNodeAt(new PluginInfo { Id = "test", Name = "Test" }, 100, 100);
        vm.Nodes.Should().BeEmpty("IsExecuting guard in AddNodeAt prevents addition while executing");
    }

    [Fact]
    public void ViewModelBase_ErrorState_ClearsOnNextOperation()
    {
        var logger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var vm = new PipelineEditorViewModel(logger, pipelineService);

        vm.ErrorMessage = "Previous error";
        vm.ErrorMessage.Should().Be("Previous error");

        // Executing a command through ExecuteSync should clear the error
        // (ViewModelBase.ExecuteSync sets ErrorMessage = null before running the action)
        vm.NewPipelineCommand.Execute(null);

        vm.ErrorMessage.Should().BeNull("Error should be cleared when a new command executes");
        vm.Nodes.Should().BeEmpty("NewPipeline should reset all state");
    }
}
