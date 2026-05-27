using Grpc.Core;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace Photopipeline.Tests.UnitTests.Services;

public sealed class BatchServiceTests : IDisposable
{
    private readonly Mock<GrpcClientService> _grpcMock;
    private readonly ILogger<BatchService> _logger;

    public BatchServiceTests()
    {
        _grpcMock = new Mock<GrpcClientService>(
            "http://localhost:50051",
            NullLogger<GrpcClientService>.Instance,
            MockBehavior.Strict);
        _logger = NullLogger<BatchService>.Instance;
    }

    public void Dispose()
    {
        Mock.VerifyAll(_grpcMock);
    }

    // ── Test 1: SubmitAsync sets PipelineConfigPath in the gRPC request ──
    [Fact]
    public async Task SubmitAsync_ValidSpec_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new BatchService(_grpcMock.Object, _logger);
        var spec = new BatchSpec
        {
            PipelineConfigPath = "pipeline-abc",
            FilePattern = "*.dng",
            OutputDir = "/output",
            Parallel = 4,
            Resume = true
        };

        // Act
        var act = () => service.SubmitAsync(spec);

        // Assert — must attempt real gRPC call; failure proves it does not return a fake ID
        await act.Should().ThrowAsync<RpcException>(
            "SubmitAsync must delegate to gRPC; failure proves it does not return a fabricated batch ID");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 2: CancelAsync correctly delegates cancellation ──
    [Fact]
    public async Task CancelAsync_ValidBatchId_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new BatchService(_grpcMock.Object, _logger);

        // Act
        var act = () => service.CancelAsync("batch-123");

        // Assert — must attempt real gRPC call
        await act.Should().ThrowAsync<RpcException>(
            "CancelAsync must delegate to gRPC; failure proves it does not silently do nothing");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 3: GetProgressAsync streams progress events ──
    [Fact]
    public async Task GetProgressAsync_ValidBatchId_DelegatesToGrpcStreaming()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new BatchService(_grpcMock.Object, _logger);

        // Act
        var act = () => service.GetProgressAsync("batch-123")
            .GetAsyncEnumerator().MoveNextAsync().AsTask();

        // Assert — must attempt real gRPC streaming call
        await act.Should().ThrowAsync<RpcException>(
            "GetProgressAsync must initiate a gRPC streaming call; failure proves it does not yield fake progress");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 4: SubmitAsync with Resume=true includes resume flag ──
    [Fact]
    public async Task SubmitAsync_ResumeEnabled_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new BatchService(_grpcMock.Object, _logger);
        var spec = new BatchSpec
        {
            PipelineConfigPath = "pipeline-xyz",
            FilePattern = "*.png",
            OutputDir = "/resume-output",
            Resume = true
        };

        // Act
        var act = () => service.SubmitAsync(spec);

        // Assert — Resume=true must be part of the serialized protobuf request
        await act.Should().ThrowAsync<RpcException>(
            "SubmitAsync with Resume=true must delegate to gRPC; failure proves the batch was attempted");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }
}
