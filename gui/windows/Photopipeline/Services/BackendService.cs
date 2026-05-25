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

    public bool IsRunning => _isRunning;
    public bool IsHealthy => _isHealthy;
    public event EventHandler<bool>? HealthChanged;

    public BackendService(ILogger<BackendService> logger, string serverPath, int serverPort = 50051)
    {
        _logger = logger;
        _serverPath = serverPath;
        _serverPort = serverPort;
    }

    async Task IHostedService.StartAsync(CancellationToken ct)
    {
        if (!File.Exists(_serverPath))
        {
            _logger.LogWarning("Server executable not found at {Path}", _serverPath);
            return;
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

        _process.Start();
        _isRunning = true;
        _logger.LogInformation("Backend process started (PID {Pid})", _process.Id);

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

    async Task IHostedService.StopAsync(CancellationToken ct)
    {
        _healthCts?.Cancel();
        await StopAsync();
    }

    public Task StartAsync(CancellationToken ct = default)
        => ((IHostedService)this).StartAsync(ct);

    public async Task StopAsync()
    {
        if (_process is { HasExited: false })
        {
            _logger.LogInformation("Shutting down backend process...");
            _process.Kill(true);
            await Task.Run(() => _process.WaitForExit(5000));
            _process.Dispose();
            _process = null;
        }
        _isRunning = false;
        SetHealth(false);
    }

    private async Task<bool> CheckHealthAsync(CancellationToken ct)
    {
        try
        {
            using var channel = Grpc.Net.Client.GrpcChannel.ForAddress(
                $"http://localhost:{_serverPort}");
            await channel.ConnectAsync(ct);
            return true;
        }
        catch
        {
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
