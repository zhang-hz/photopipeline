using Grpc.Net.Client;
using Microsoft.Extensions.Logging;
using System;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;

namespace Photopipeline.Services;

public sealed class GrpcClientService : IDisposable
{
    private GrpcChannel? _channel;
    private readonly string _serverAddress;
    private readonly ILogger<GrpcClientService> _logger;
    private readonly SemaphoreSlim _lock = new(1, 1);
    private bool _isConnected;
    private int _reconnectAttempts;

    public bool IsConnected => _isConnected;

    public GrpcClientService(string serverAddress = "http://localhost:50051",
        ILogger<GrpcClientService>? logger = null)
    {
        _serverAddress = serverAddress;
        _logger = logger ?? Microsoft.Extensions.Logging.Abstractions.NullLogger<GrpcClientService>.Instance;
    }

    public async Task ConnectAsync(CancellationToken ct = default)
    {
        await _lock.WaitAsync(ct);
        try
        {
            if (_isConnected) return;

            _channel?.Dispose();
            _channel = GrpcChannel.ForAddress(_serverAddress, new GrpcChannelOptions
            {
                MaxReceiveMessageSize = 256 * 1024 * 1024,
                MaxSendMessageSize = 256 * 1024 * 1024,
                HttpHandler = new SocketsHttpHandler
                {
                    EnableMultipleHttp2Connections = true,
                    KeepAlivePingDelay = TimeSpan.FromSeconds(30),
                    KeepAlivePingTimeout = TimeSpan.FromSeconds(10),
                }
            });

            _isConnected = true;
            _reconnectAttempts = 0;
            _logger.LogInformation("gRPC connected to {Address}", _serverAddress);
        }
        catch (Exception ex)
        {
            _isConnected = false;
            _logger.LogWarning(ex, "gRPC connection failed to {Address}", _serverAddress);
            throw;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<GrpcChannel> GetChannelAsync(CancellationToken ct = default)
    {
        if (!_isConnected || _channel is null)
            await ConnectAsync(ct);
        return _channel!;
    }

    public async Task ReconnectAsync(CancellationToken ct = default)
    {
        _isConnected = false;
        _reconnectAttempts++;
        var delay = Math.Min(_reconnectAttempts * 1000, 15000);
        _logger.LogWarning("gRPC reconnecting (attempt {Attempt}, delay {Delay}ms)", _reconnectAttempts, delay);
        try { await Task.Delay(delay, ct); }
        catch (OperationCanceledException) { return; }
        try { await ConnectAsync(ct); }
        catch (Exception ex) { _logger.LogWarning(ex, "gRPC reconnect attempt {Attempt} failed", _reconnectAttempts); }
    }

    /// <summary>
    /// Invoke a gRPC call with automatic retry-on-reconnect for transient failures.
    /// NOTE: Currently most services bypass this and call <see cref="GetChannelAsync"/> directly.
    /// This method is retained for future centralized retry logic.
    /// </summary>
    public async Task<TResponse> CallAsync<TResponse>(
        Func<GrpcChannel, Task<TResponse>> call,
        CancellationToken ct = default)
    {
        try
        {
            var channel = await GetChannelAsync(ct);
            return await call(channel);
        }
        catch (Grpc.Core.RpcException ex) when (
            ex.StatusCode == Grpc.Core.StatusCode.Unavailable ||
            ex.StatusCode == Grpc.Core.StatusCode.DeadlineExceeded ||
            ex.StatusCode == Grpc.Core.StatusCode.Aborted)
        {
            _logger.LogWarning(ex, "gRPC call failed ({Status}), retrying after reconnect", ex.StatusCode);
            await ReconnectAsync(ct);
            var channel = await GetChannelAsync(ct);
            return await call(channel);
        }
    }

    public void Dispose()
    {
        _isConnected = false;
        _channel?.Dispose();
        _lock.Dispose();
        _logger.LogDebug("gRPC client disposed");
    }
}
