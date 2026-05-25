using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using System.Diagnostics;
using System.IO;

namespace Photopipeline.Services;

public sealed class BackendService : IBackendService, IHostedService
{
    private Process? _process;
    private readonly ILogger<BackendService> _logger;
    private readonly string _serverPath;
    private readonly int _serverPort;
    private bool _isRunning;
    private bool _isHealthy;
    private CancellationTokenSource? _healthCts;
    private Grpc.Net.Client.GrpcChannel? _healthChannel;

    public bool IsRunning => _isRunning;
    public bool IsHealthy => _isHealthy;
    public event EventHandler<bool>? HealthChanged;

    public BackendService(ILogger<BackendService> logger, string serverPath, int serverPort = 50051)
    {
        _logger = logger;
        _serverPath = serverPath;
        _serverPort = serverPort;
    }

    Task IHostedService.StartAsync(CancellationToken ct)
    {
        if (!File.Exists(_serverPath))
        {
            _logger.LogWarning("Server executable not found at {Path}", _serverPath);
            return Task.CompletedTask;
        }

        var psi = new ProcessStartInfo
        {
            FileName = _serverPath,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true,
        };

        _process = new Process { StartInfo = psi, EnableRaisingEvents = true };
        _process.Exited += (_, _) =>
        {
            _logger.LogWarning("Backend process exited");
            SetHealth(false);
        };

        try
        {
            _process.Start();
            _isRunning = true;
            _logger.LogInformation("Backend process started (PID {Pid})", _process.Id);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to start backend process at {Path}", _serverPath);
            return Task.CompletedTask;
        }

        _ = WaitForHealthyAndMonitor(ct);
        return Task.CompletedTask;
    }

    private async Task WaitForHealthyAndMonitor(CancellationToken ct)
    {
        try
        {
            for (int i = 0; i < 30 && !ct.IsCancellationRequested; i++)
            {
                if (await CheckHealthAsync(ct))
                {
                    SetHealth(true);
                    StartHealthMonitor();
                    return;
                }
                await Task.Delay(500, ct);
            }
            _logger.LogError("Backend did not become healthy within 15 seconds");
        }
        catch (OperationCanceledException) { }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Health check error during backend startup");
        }
    }

    async Task IHostedService.StopAsync(CancellationToken ct)
    {
        await StopAsync();
        _healthCts?.Cancel();
        _healthCts?.Dispose();
        _healthCts = null;
        _healthChannel?.Dispose();
        _healthChannel = null;
    }

    public Task StartAsync(CancellationToken ct = default)
        => ((IHostedService)this).StartAsync(ct);

    public async Task StopAsync()
    {
        if (_process is { HasExited: false } process)
        {
            process.EnableRaisingEvents = false;
            _logger.LogInformation("Shutting down backend process...");
            process.Kill(true);
            await Task.Run(() => process.WaitForExit(5000));
        }
        _process?.Dispose();
        _process = null;
        _isRunning = false;
        SetHealth(false);
    }

    private async Task<bool> CheckHealthAsync(CancellationToken ct)
    {
        try
        {
            _healthChannel ??= Grpc.Net.Client.GrpcChannel.ForAddress(
                $"http://localhost:{_serverPort}");
            await _healthChannel.ConnectAsync(ct);
            return true;
        }
        catch
        {
            // Dispose and recreate the channel on failure to refresh the connection
            _healthChannel?.Dispose();
            _healthChannel = null;
            return false;
        }
    }

    private void StartHealthMonitor()
    {
        _healthCts = new CancellationTokenSource();
        _ = MonitorAsync(_healthCts.Token);
    }

    private async Task MonitorAsync(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                await Task.Delay(5000, ct);
                bool healthy = await CheckHealthAsync(ct);
                if (healthy != _isHealthy)
                    SetHealth(healthy);
            }
            catch (OperationCanceledException) { return; }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Health monitor error");
            }
        }
    }

    private void SetHealth(bool healthy)
    {
        if (_isHealthy == healthy) return;
        _isHealthy = healthy;
        HealthChanged?.Invoke(this, healthy);
    }
}
