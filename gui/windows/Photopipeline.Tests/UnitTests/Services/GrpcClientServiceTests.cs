using Grpc.Core;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace Photopipeline.Tests.UnitTests.Services;

public sealed class GrpcClientServiceTests : IDisposable
{
    private GrpcClientService? _service;

    public void Dispose()
    {
        _service?.Dispose();
    }

    // ── Test 1: Constructor sets initial state correctly ──
    [Fact]
    public void Constructor_InitialState_IsNotConnected()
    {
        // Arrange & Act
        _service = new GrpcClientService("http://localhost:59999");

        // Assert
        _service.IsConnected.Should().BeFalse("service should not be connected before ConnectAsync is called");
    }

    // ── Test 2: ConnectAsync establishes a gRPC channel ──
    [Fact]
    public async Task ConnectAsync_EstablishesConnection_IsConnectedTrue()
    {
        // Arrange
        _service = new GrpcClientService("http://localhost:59999");

        // Act
        await _service.ConnectAsync();

        // Assert — GrpcChannel.ForAddress does not throw synchronously, so IsConnected becomes true
        _service.IsConnected.Should().BeTrue("ConnectAsync should create the channel and set IsConnected");
    }

    // ── Test 3: CallAsync with streaming-like Func — progress/result delivered ──
    [Fact]
    public async Task CallAsync_ValidCall_ReturnsResult()
    {
        // Arrange
        _service = new GrpcClientService("http://localhost:59999");
        await _service.ConnectAsync();

        static Task<string> Call(Grpc.Net.Client.GrpcChannel ch) => Task.FromResult("ok");

        // Act
        var result = await _service.CallAsync(Call);

        // Assert
        result.Should().Be("ok", "CallAsync should return the result from the passed function");
    }

    // ── Test 4: CallAsync retries on RpcException with Unavailable status ──
    [Fact]
    public async Task CallAsync_TransientFailure_RetriesAndSucceeds()
    {
        // Arrange
        _service = new GrpcClientService("http://localhost:59999");
        await _service.ConnectAsync();

        int callCount = 0;
        Task<string> CallWithRetry(Grpc.Net.Client.GrpcChannel ch)
        {
            callCount++;
            if (callCount == 1)
                throw new RpcException(new Status(StatusCode.Unavailable, "transient failure"));
            return Task.FromResult("retry-success");
        }

        // Act
        var result = await _service.CallAsync(CallWithRetry);

        // Assert
        result.Should().Be("retry-success", "CallAsync should retry on Unavailable and succeed on the second attempt");
        callCount.Should().Be(2, "the function should be called twice — once failing, once succeeding");
    }

    // ── Test 5: CallAsync rethrows non-retryable RpcException ──
    [Fact]
    public async Task CallAsync_NonRetryableError_PropagatesException()
    {
        // Arrange
        _service = new GrpcClientService("http://localhost:59999");
        await _service.ConnectAsync();

        static Task<string> CallThatFails(Grpc.Net.Client.GrpcChannel ch)
            => throw new RpcException(new Status(StatusCode.Internal, "internal error"));

        // Act
        var act = () => _service.CallAsync(CallThatFails);

        // Assert — Internal is NOT in the retry filter, so it must propagate
        await act.Should().ThrowAsync<RpcException>("non-retryable status codes should propagate to the caller")
            .Where(ex => ex.StatusCode == StatusCode.Internal);
    }

    // ── Test 6: Dispose cleans up resources and sets IsConnected to false ──
    [Fact]
    public async Task Dispose_AfterConnect_CleansUpAndSetsIsConnectedFalse()
    {
        // Arrange
        _service = new GrpcClientService("http://localhost:59999");
        await _service.ConnectAsync();
        _service.IsConnected.Should().BeTrue("precondition: connected");

        // Act
        _service.Dispose();

        // Assert
        _service.IsConnected.Should().BeFalse("Dispose should reset IsConnected");
    }
}
