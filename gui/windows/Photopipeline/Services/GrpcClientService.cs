using Grpc.Net.Client;
using System;
using System.Threading;
using System.Threading.Tasks;

namespace Photopipeline.Services;

public sealed class GrpcClientService : IDisposable
{
    private GrpcChannel? _channel;
    private readonly string _serverAddress;
    private readonly SemaphoreSlim _lock = new(1, 1);
    private bool _isConnected;
    private int _reconnectAttempts;

    public bool IsConnected => _isConnected;

    public GrpcClientService(string serverAddress = "http://localhost:50051")
    {
        _serverAddress = serverAddress;
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
        }
        catch
        {
            _isConnected = false;
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
        await Task.Delay(delay, ct);
        await ConnectAsync(ct);
    }

    public async Task<TResponse> CallAsync<TResponse>(
        Func<GrpcChannel, Task<TResponse>> call,
        CancellationToken ct = default)
    {
        try
        {
            var channel = await GetChannelAsync(ct);
            return await call(channel);
        }
        catch
        {
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
    }
}
