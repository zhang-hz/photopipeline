namespace Photopipeline.Services;

public interface IBackendService
{
    bool IsRunning { get; }
    bool IsHealthy { get; }
    Task StartAsync(CancellationToken ct = default);
    Task StopAsync();
    event EventHandler<bool>? HealthChanged;
}
