using System.Diagnostics;
using System.IO;
using System.Windows;
using Microsoft.Extensions.DependencyInjection;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline;

public partial class App : Application
{
    private Process? _serverProcess;
    private const string ServerExe = "photopipeline-server.exe";

    public static IServiceProvider Services { get; private set; } = null!;

    private static readonly string TraceLogDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "Photopipeline", "logs");

    private static void WriteTrace(string msg)
    {
        try
        {
            Directory.CreateDirectory(TraceLogDir);
            File.AppendAllText(Path.Combine(TraceLogDir, "trace.log"),
                $"{DateTime.Now:HH:mm:ss.fff} [{Environment.CurrentManagedThreadId}] {msg}\n");
        }
        catch { }
    }

    static App()
    {
        WriteTrace("App static constructor");
    }

    public App()
    {
        WriteTrace("App instance constructor");
        Services = ConfigureServices();
        WriteTrace("App: DI configured");

        DispatcherUnhandledException += (_, e) =>
        {
            WriteTrace($"DUE: {e.Exception.GetType().Name}: {e.Exception.Message}\n{e.Exception.StackTrace}");
            LogException(e.Exception);
            e.Handled = true;
        };
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            if (e.ExceptionObject is Exception ex)
                LogException(ex);
        };
        TaskScheduler.UnobservedTaskException += (_, e) =>
        {
            LogException(e.Exception);
            e.SetObserved();
        };
        WriteTrace("App instance constructor: handlers registered - DONE");
    }

    private static void LogException(Exception ex)
    {
        try
        {
            Directory.CreateDirectory(TraceLogDir);
            var logFile = Path.Combine(TraceLogDir,
                $"crash_{DateTime.Now:yyyyMMdd_HHmmss}.log");
            File.WriteAllText(logFile, ex.ToString());
            Debug.WriteLine($"Unhandled exception logged to {logFile}: {ex}");
        }
        catch
        {
            Debug.WriteLine($"Unhandled exception (could not log): {ex}");
        }
    }

    private static IServiceProvider ConfigureServices()
    {
        var services = new ServiceCollection();

        services.AddSingleton<GrpcClientService>();
        services.AddSingleton<IPipelineService, PipelineService>();
        services.AddSingleton<IImageService, ImageService>();
        services.AddSingleton<MainViewModel>();
        services.AddTransient<PluginPanelViewModel>();
        services.AddTransient<PipelineEditorViewModel>();
        services.AddTransient<BatchViewModel>();

        return services.BuildServiceProvider();
    }

    protected override void OnStartup(StartupEventArgs e)
    {
        WriteTrace("OnStartup: begin");
        base.OnStartup(e);
        WriteTrace("OnStartup: base.OnStartup done, window should be loaded");
        _ = InitializeServerAsync();
        WriteTrace("OnStartup: end");
    }

    protected override void OnExit(ExitEventArgs e)
    {
        WriteTrace("OnExit: shutting down");
        try { _serverProcess?.Kill(); } catch { }
        base.OnExit(e);
    }

    private async Task InitializeServerAsync()
    {
        try
        {
            StartServer();
            await WaitForServerAsync();
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Server initialization failed: {ex}");
            try
            {
                var vm = Services.GetRequiredService<MainViewModel>();
                vm.StatusMessage = "Server unavailable - offline mode";
            }
            catch { }
        }
    }

    private void StartServer()
    {
        var baseDir = AppContext.BaseDirectory;
        var exePath = Path.Combine(baseDir, ServerExe);

        var customPath = Environment.GetEnvironmentVariable("PHOTOPIPELINE_SERVER_PATH");
        if (!string.IsNullOrEmpty(customPath))
            exePath = customPath;

        if (!File.Exists(exePath))
        {
            Debug.WriteLine($"Server not found: {exePath}");
            return;
        }

        try
        {
            _serverProcess = new Process
            {
                StartInfo = new ProcessStartInfo
                {
                    FileName = exePath,
                    UseShellExecute = false,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    CreateNoWindow = true,
                    WorkingDirectory = baseDir,
                },
                EnableRaisingEvents = true,
            };

            _serverProcess.Exited += (_, _) => Debug.WriteLine("Server process exited");
            _serverProcess.Start();
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to start server: {ex}");
        }
    }

    private async Task WaitForServerAsync(int timeoutMs = 10000)
    {
        var sw = Stopwatch.StartNew();
        while (sw.ElapsedMilliseconds < timeoutMs)
        {
            try
            {
                using var cts = new CancellationTokenSource(500);
                var svc = App.Services.GetRequiredService<GrpcClientService>();
                await svc.ConnectAsync(cts.Token);
                return;
            }
            catch
            {
                await Task.Delay(200);
            }
        }
    }
}
