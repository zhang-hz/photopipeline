using Microsoft.Extensions.Logging;
using Photopipeline.Models;
using System.IO;
using System.Text.Json;

namespace Photopipeline.Services;

public sealed class SettingsService : ISettingsService
{
    private readonly ILogger<SettingsService> _logger;
    private readonly string _settingsPath;
    private readonly JsonSerializerOptions _jsonOptions;
    private AppSettings _current = new();

    public AppSettings Current => _current;
    public event EventHandler<AppSettings>? SettingsChanged;

    public SettingsService(ILogger<SettingsService> logger)
    {
        _logger = logger;
        _settingsPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "Photopipeline", "appsettings.json");
        _jsonOptions = new JsonSerializerOptions { WriteIndented = true };
    }

    public async Task LoadAsync(CancellationToken ct = default)
    {
        try
        {
            if (File.Exists(_settingsPath))
            {
                var json = await File.ReadAllTextAsync(_settingsPath, ct);
                var settings = JsonSerializer.Deserialize<AppSettings>(json);
                if (settings is not null)
                {
                    _current = settings;
                    _logger.LogInformation("Settings loaded from {Path}", _settingsPath);
                    return;
                }
            }
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to load settings; using defaults");
        }

        _current = new AppSettings();
    }

    public async Task SaveAsync(AppSettings settings, CancellationToken ct = default)
    {
        _current = settings;
        try
        {
            var dir = Path.GetDirectoryName(_settingsPath)!;
            Directory.CreateDirectory(dir);
            var json = JsonSerializer.Serialize(settings, _jsonOptions);
            await File.WriteAllTextAsync(_settingsPath, json, ct);
            _logger.LogInformation("Settings saved");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to save settings");
        }

        SettingsChanged?.Invoke(this, _current);
    }

    public async Task ResetAsync(CancellationToken ct = default)
    {
        await SaveAsync(new AppSettings(), ct);
    }
}
