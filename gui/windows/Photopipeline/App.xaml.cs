using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Photopipeline.Helpers;
using Photopipeline.Services;
using Photopipeline.ViewModels;
using Serilog;
using System.IO;
using System.Windows;

namespace Photopipeline;

public partial class App : Application
{
    private readonly IHost _host;
    private MainWindow? _mainWindow;

    public static new App Current => (App)Application.Current;
    public static IServiceProvider Services => Current._host.Services;

    public App()
    {
        _host = Host.CreateDefaultBuilder()
            .ConfigureAppConfiguration((ctx, config) =>
            {
                var appData = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                    "Photopipeline");
                Directory.CreateDirectory(appData);
                config.SetBasePath(appData);
                config.AddJsonFile("appsettings.json", optional: true, reloadOnChange: true);
                config.AddEnvironmentVariables("PHOTOPIPELINE_");
            })
            .UseSerilog((ctx, loggerConfig) =>
            {
                loggerConfig
                    .MinimumLevel.Information()
                    .WriteTo.Console()
                    .WriteTo.File(
                        Path.Combine(
                            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                            "Photopipeline", "logs", "photopipeline-.log"),
                        rollingInterval: RollingInterval.Day,
                        retainedFileCountLimit: 7);
            })
            .ConfigureServices((ctx, services) =>
            {
                // Infrastructure
                services.AddSingleton<DispatcherHelper>(_ => new DispatcherHelper(Application.Current.Dispatcher));

                // gRPC channel
                services.AddSingleton<GrpcClientService>(sp =>
                {
                    var config = sp.GetRequiredService<IConfiguration>();
                    var port = config.GetValue<int>("Server:Port", 50051);
                    var logger = sp.GetRequiredService<ILogger<GrpcClientService>>();
                    return new GrpcClientService($"http://localhost:{port}", logger);
                });

                // Services
                services.AddSingleton<IPipelineService>(sp =>
                {
                    var grpc = sp.GetRequiredService<GrpcClientService>();
                    var logger = sp.GetRequiredService<ILogger<PipelineService>>();
                    return new PipelineService(grpc, logger);
                });
                services.AddSingleton<IImageService>(sp =>
                {
                    var grpc = sp.GetRequiredService<GrpcClientService>();
                    var logger = sp.GetRequiredService<ILogger<ImageService>>();
                    return new ImageService(grpc, logger);
                });
                services.AddSingleton<IBatchService>(sp =>
                {
                    var grpc = sp.GetRequiredService<GrpcClientService>();
                    var logger = sp.GetRequiredService<ILogger<BatchService>>();
                    return new BatchService(grpc, logger);
                });
                services.AddSingleton<IDialogService, WindowsDialogService>();
                services.AddSingleton<IPluginService, PluginService>();
                services.AddSingleton<ISettingsService, SettingsService>();
                services.AddSingleton<IBackendService>(sp =>
                {
                    var logger = sp.GetRequiredService<ILogger<BackendService>>();
                    var config = sp.GetRequiredService<IConfiguration>();
                    var serverPath = config.GetValue<string>("Server:Path", "photopipeline-server.exe")
                        ?? "photopipeline-server.exe";
                    var port = config.GetValue<int>("Server:Port", 50051);
                    return new BackendService(logger, serverPath, port);
                });

                // Hosted service for backend lifecycle
                services.AddHostedService(sp => sp.GetRequiredService<IBackendService>() as BackendService
                    ?? throw new InvalidOperationException("IBackendService must be BackendService"));

                // ViewModels
                services.AddSingleton<MainViewModel>();
                services.AddTransient<FilmstripViewModel>();
                services.AddTransient<PreviewViewModel>();
                services.AddTransient<PipelineEditorViewModel>();
                services.AddTransient<PluginBrowserViewModel>();
                services.AddTransient<BatchViewModel>();
                services.AddTransient<SettingsViewModel>();

                // Views
                services.AddTransient<MainWindow>();
                services.AddTransient<Views.SettingsDialog>();
            })
            .Build();
    }

    protected override async void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        try
        {
            var settings = _host.Services.GetRequiredService<ISettingsService>();
            await settings.LoadAsync();

            try { ApplyTheme(settings.Current.Theme); }
            catch (Exception ex)
            {
                var logger = _host.Services.GetRequiredService<ILogger<App>>();
                logger.LogWarning(ex, "Theme application failed; continuing with defaults");
            }

            _mainWindow = _host.Services.GetRequiredService<MainWindow>();

            if (!double.IsNaN(settings.Current.WindowLeft) && !double.IsNaN(settings.Current.WindowTop))
            {
                _mainWindow.Left = settings.Current.WindowLeft;
                _mainWindow.Top = settings.Current.WindowTop;
            }
            if (settings.Current.WindowWidth > 0 && settings.Current.WindowHeight > 0)
            {
                _mainWindow.Width = settings.Current.WindowWidth;
                _mainWindow.Height = settings.Current.WindowHeight;
            }
            if (settings.Current.IsMaximized)
                _mainWindow.WindowState = WindowState.Maximized;

            _mainWindow.Show();

#pragma warning disable CS4014
            _ = StartHostWithErrorLogging();
#pragma warning restore CS4014
        }
        catch (Exception ex)
        {
            var logger = _host.Services.GetRequiredService<ILogger<App>>();
            logger.LogCritical(ex, "Application startup failed");
            MessageBox.Show($"Startup failed: {ex.Message}", "Photopipeline Error",
                MessageBoxButton.OK, MessageBoxImage.Error);
            Shutdown(1);
        }
    }

#pragma warning disable CS4014
    private async Task StartHostWithErrorLogging()
    {
        try
        {
            await _host.StartAsync();
        }
        catch (Exception ex)
        {
            var logger = _host.Services.GetRequiredService<ILogger<App>>();
            logger.LogCritical(ex, "Host startup failed (backend may be unavailable)");
            Dispatcher.InvokeAsync(() =>
            {
                MessageBox.Show(
                    $"Backend failed to start: {ex.Message}\n\nThe application will continue but backend features will be unavailable.",
                    "Photopipeline - Backend Error",
                    MessageBoxButton.OK, MessageBoxImage.Warning);
            });
        }
    }
#pragma warning restore CS4014

    protected override async void OnExit(ExitEventArgs e)
    {
        base.OnExit(e);

        try
        {
            var settings = _host.Services.GetRequiredService<ISettingsService>();

            if (_mainWindow is not null)
            {
                settings.Current.WindowWidth = SafeDouble(_mainWindow.Width, 1440);
                settings.Current.WindowHeight = SafeDouble(_mainWindow.Height, 900);
                settings.Current.WindowLeft = SafeDouble(_mainWindow.Left, 0);
                settings.Current.WindowTop = SafeDouble(_mainWindow.Top, 0);
                settings.Current.IsMaximized = _mainWindow.WindowState == WindowState.Maximized;
                await settings.SaveAsync(settings.Current);
            }
        }
        catch (Exception ex)
        {
            var logger = _host.Services.GetRequiredService<ILogger<App>>();
            logger.LogWarning(ex, "Failed to persist window state on exit");
        }

        using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        await _host.StopAsync(cts.Token);
        _host.Dispose();
    }

    private static double SafeDouble(double value, double fallback)
        => double.IsNaN(value) || double.IsInfinity(value) ? fallback : value;

    public static void ApplyTheme(string theme)
    {
        var appTheme = theme.ToLowerInvariant() switch
        {
            "light" => Wpf.Ui.Appearance.ApplicationTheme.Light,
            _ => Wpf.Ui.Appearance.ApplicationTheme.Dark,
        };
        Wpf.Ui.Appearance.ApplicationThemeManager.Apply(appTheme,
            Wpf.Ui.Controls.WindowBackdropType.Mica);
    }
}
