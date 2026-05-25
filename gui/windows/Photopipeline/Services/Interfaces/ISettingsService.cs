using Photopipeline.Models;

namespace Photopipeline.Services;

public interface ISettingsService
{
    AppSettings Current { get; }
    Task LoadAsync(CancellationToken ct = default);
    Task SaveAsync(AppSettings settings, CancellationToken ct = default);
    Task ResetAsync(CancellationToken ct = default);
    event EventHandler<AppSettings>? SettingsChanged;
}
