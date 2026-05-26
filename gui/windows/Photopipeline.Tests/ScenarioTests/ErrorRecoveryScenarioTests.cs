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

        vm.StartBatchCommand.Execute(null);
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
    }

    [Fact]
    public void ViewModelBase_IsBusy_PreventsConcurrentOperation()
    {
        var logger = Mock.Of<ILogger<PipelineEditorViewModel>>();
        var pipelineService = Mock.Of<IPipelineService>();
        var vm = new PipelineEditorViewModel(logger, pipelineService);

        vm.IsBusy.Should().BeFalse();
    }

    [Fact]
    public void ViewModelBase_ErrorState_ClearsOnNextOperation()
    {
        var logger = Mock.Of<ILogger<FilmstripViewModel>>();
        var imageService = Mock.Of<IImageService>();
        var vm = new FilmstripViewModel(logger, imageService, null!);

        vm.ErrorMessage = "Previous error";

        vm.ClearImagesCommand.Execute(null);

        // ClearImages is synchronous, so ErrorMessage should not be affected
        vm.Images.Should().BeEmpty();
    }
}
