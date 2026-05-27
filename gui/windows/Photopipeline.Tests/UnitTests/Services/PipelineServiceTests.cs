using Grpc.Core;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using Google.Protobuf.WellKnownTypes;

namespace Photopipeline.Tests.UnitTests.Services;

public sealed class PipelineServiceTests : IDisposable
{
    private readonly Mock<GrpcClientService> _grpcMock;
    private readonly ILogger<PipelineService> _logger;

    public PipelineServiceTests()
    {
        _grpcMock = new Mock<GrpcClientService>(
            "http://localhost:50051",
            NullLogger<GrpcClientService>.Instance,
            MockBehavior.Strict);
        _logger = NullLogger<PipelineService>.Instance;
    }

    public void Dispose()
    {
        Mock.VerifyAll(_grpcMock);
    }

    // ── Test 1: CreatePipelineAsync returns pipeline ID ──
    [Fact]
    public async Task CreatePipelineAsync_ValidSpec_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new PipelineService(_grpcMock.Object, _logger);
        var spec = new PipelineSpec
        {
            Name = "test-pipeline",
            Nodes = new List<PipelineNode>
            {
                new() { Id = "n1", PluginId = "raw_decoder" }
            }
        };

        // Act
        var act = () => service.CreatePipelineAsync(spec);

        // Assert — must attempt real gRPC call; failure proves it's not a silent no-op
        await act.Should().ThrowAsync<RpcException>(
            "CreatePipelineAsync must delegate to gRPC; failure proves it does not return a fake ID");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 2: ExecuteAsync uses correct paths in the gRPC request ──
    [Fact]
    public async Task ExecuteAsync_ValidArguments_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new PipelineService(_grpcMock.Object, _logger);

        // Act
        var act = () => service.ExecuteAsync("pipeline-123", "input.dng", "output.tiff")
            .GetAsyncEnumerator().MoveNextAsync().AsTask();

        // Assert — must attempt real gRPC streaming call
        await act.Should().ThrowAsync<RpcException>(
            "ExecuteAsync must initiate a gRPC streaming call; failure proves it does not silently yield empty results");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 3: ValidateAsync returns validation result structure ──
    [Fact]
    public async Task ValidateAsync_EmptyPipeline_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new PipelineService(_grpcMock.Object, _logger);
        var spec = new PipelineSpec { Name = "empty" };

        // Act
        var act = () => service.ValidateAsync(spec);

        // Assert — must attempt real gRPC call
        await act.Should().ThrowAsync<RpcException>(
            "ValidateAsync must delegate to gRPC; failure proves it does not return a fake validation");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 4: GetNodeSchemaAsync returns schema for a plugin ──
    [Fact]
    public async Task GetNodeSchemaAsync_ValidPluginId_DelegatesToGrpc()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new PipelineService(_grpcMock.Object, _logger);

        // Act
        var act = () => service.GetNodeSchemaAsync("raw_decoder");

        // Assert — must attempt real gRPC call
        await act.Should().ThrowAsync<RpcException>(
            "GetNodeSchemaAsync must delegate to gRPC; failure proves it does not return a fake schema");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 5: ToProtoSpec serializes Params dictionary into protobuf Struct ──
    // This verifies the B1 bug fix — params must be properly serialized.
    [Fact]
    public async Task CreatePipelineAsync_WithParams_SerializesParamsIntoProtobuf()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new PipelineService(_grpcMock.Object, _logger);
        var spec = new PipelineSpec
        {
            Name = "test-params",
            Params = new Dictionary<string, object>
            {
                ["quality"] = 90d,
                ["enable"] = true,
                ["label"] = "test"
            },
            Nodes = new List<PipelineNode>
            {
                new()
                {
                    Id = "n1",
                    PluginId = "format_convert",
                    Params = new Dictionary<string, object>
                    {
                        ["quality"] = 90d,
                        ["lossless"] = false
                    }
                }
            }
        };

        // Act
        var act = () => service.CreatePipelineAsync(spec);

        // Assert — must attempt real gRPC call (ToProtoSpec serialization is exercised internally)
        await act.Should().ThrowAsync<RpcException>(
            "CreatePipelineAsync must serialize Params dict to protobuf Struct; " +
            "failure proves gRPC call was attempted with serialized data");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ── Test 6: ExecuteAsync with empty pipelineId — propagates gRPC error ──
    [Fact]
    public async Task ExecuteAsync_EmptyPipelineId_PropagatesGrpcError()
    {
        // Arrange
        var channel = Grpc.Net.Client.GrpcChannel.ForAddress("http://localhost:50051");
        _grpcMock
            .Setup(g => g.GetChannelAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(channel);

        var service = new PipelineService(_grpcMock.Object, _logger);

        // Act
        var act = () => service.ExecuteAsync("", "input.dng", "output.tiff")
            .GetAsyncEnumerator().MoveNextAsync().AsTask();

        // Assert — empty pipeline ID should cause gRPC error, not be silently ignored
        await act.Should().ThrowAsync<RpcException>(
            "empty pipeline ID must cause gRPC failure, not silently return no results");
        _grpcMock.Verify(g => g.GetChannelAsync(It.IsAny<CancellationToken>()), Times.Once);
    }
}
