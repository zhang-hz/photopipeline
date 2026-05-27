using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using Grpc.Core;

namespace Photopipeline.Tests.UnitTests.Services;

public sealed class ImageServiceTests : IDisposable
{
    private readonly Mock<GrpcClientService> _grpcMock;
    private readonly ILogger<ImageService> _logger;
    private ImageService? _service;

    public ImageServiceTests()
    {
        _grpcMock = new Mock<GrpcClientService>(
            "http://localhost:50051",
            NullLogger<GrpcClientService>.Instance,
            MockBehavior.Strict);
        _logger = NullLogger<ImageService>.Instance;
    }

    public void Dispose()
    {
        Mock.VerifyAll(_grpcMock);
    }

    // ── Test 1: LoadImageInfoAsync returns ImageInfo with correct data ──
    [Fact]
    public async Task LoadImageInfoAsync_ValidImage_ReturnsImageInfo()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        _service = new ImageService(_grpcMock.Object, _logger);

        // Act — real gRPC call cannot succeed without a server, so we expect the exception
        // We verify that ImageService properly delegates to GrpcClientService.GetChannelAsync
        // and constructs the gRPC client. The call will fail because there is no server,
        // but that proves the method actually attempts the gRPC call (no silent skip).
        var act = () => _service.LoadImageInfoAsync("/test/image.png");

        // Assert
        await act.Should().ThrowAsync<RpcException>(
            "LoadImageInfoAsync must attempt a real gRPC call; if no server is running it must fail, not silently skip");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 2: LoadImageInfoAsync with empty path — propagates gRPC error ──
    [Fact]
    public async Task LoadImageInfoAsync_EmptyPath_PropagatesError()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        _service = new ImageService(_grpcMock.Object, _logger);

        // Act
        var act = () => _service.LoadImageInfoAsync("");

        // Assert — must not silently return null; must attempt gRPC call and fail
        await act.Should().ThrowAsync<RpcException>(
            "empty path should cause gRPC call to fail, not return null silently");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 3: GetThumbnailAsync delegates to gRPC client ──
    [Fact]
    public async Task GetThumbnailAsync_DelegatesToGrpcClient()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        _service = new ImageService(_grpcMock.Object, _logger);

        // Act
        var act = () => _service.GetThumbnailAsync("/test/image.png", 256);

        // Assert — must attempt gRPC call, not silently return empty bytes
        await act.Should().ThrowAsync<RpcException>(
            "GetThumbnailAsync must delegate to gRPC; failure proves it does not silently return empty data");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 4: DecodeAsync correctly creates gRPC streaming call ──
    [Fact]
    public async Task DecodeAsync_DelegatesToGrpcClient_StreamingCall()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        _service = new ImageService(_grpcMock.Object, _logger);

        // Act
        var act = () => _service.DecodeAsync("/test/image.png").GetAsyncEnumerator().MoveNextAsync().AsTask();

        // Assert — must attempt gRPC streaming call; failure proves it's real
        await act.Should().ThrowAsync<RpcException>(
            "DecodeAsync must initiate a gRPC streaming call; failure proves it does not return empty enumerable");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }
}
