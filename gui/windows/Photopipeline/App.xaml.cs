using System.Diagnostics;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline;

public partial class App : Application
{
    private Window? _mainWindow;
    private Process? _serverProcess;
    private const string ServerExe = "photopipeline-server.exe";
    private const string ServerUrl = "http://localhost:50051";

    public static IServiceProvider Services { get; private set; } = null!;

    public App()
    {
        this.InitializeComponent();
        Services = ConfigureServices();
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

    protected override async void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        StartServer();
        await WaitForServerAsync();

        _mainWindow = new MainWindow();
        _mainWindow.Activate();
    }
}
