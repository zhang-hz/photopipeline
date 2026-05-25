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
                services.AddSingleton<DispatcherHelper>();

                // gRPC channel
                services.AddSingleton<GrpcClientService>(sp =>
                {
                    var config = sp.GetRequiredService<IConfiguration>();
                    var port = config.GetValue<int>("Server:Port", 50051);
                    return new GrpcClientService($"http://localhost:{port}");
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

            ApplyTheme(settings.Current.Theme);

            await _host.StartAsync();

            var mainWindow = _host.Services.GetRequiredService<MainWindow>();
            mainWindow.Show();
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

    protected override async void OnExit(ExitEventArgs e)
    {
        base.OnExit(e);

        try
        {
            var settings = _host.Services.GetRequiredService<ISettingsService>();
            var mainWindow = _host.Services.GetRequiredService<MainWindow>();
            settings.Current.WindowWidth = mainWindow.Width;
            settings.Current.WindowHeight = mainWindow.Height;
            settings.Current.WindowLeft = mainWindow.Left;
            settings.Current.WindowTop = mainWindow.Top;
            settings.Current.IsMaximized = mainWindow.WindowState == WindowState.Maximized;
            await settings.SaveAsync(settings.Current);
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

    public static void ApplyTheme(string theme)
    {
        var appTheme = theme.ToLowerInvariant() switch
        {
            "light" => Wpf.Ui.Appearance.ApplicationTheme.Light,
            _ => Wpf.Ui.Appearance.ApplicationTheme.Dark,
        };
        Wpf.Ui.Appearance.ApplicationThemeManager.Apply(appTheme);
    }
}
