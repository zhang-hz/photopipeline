using System.Diagnostics;
using System.Threading;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public abstract class UiTestBase : IDisposable
{
    protected readonly string AppPath;
    protected Process? _appProcess;
    protected bool _isRunning;

    protected UiTestBase()
    {
        // Resolve from publish output or debug build
        var publishDir = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..", "..",
            "Photopipeline", "bin", "x64", "Release", "net9.0-windows", "publish");

        if (!Directory.Exists(publishDir))
        {
            publishDir = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..", "..",
                "Photopipeline", "bin", "x64", "Debug", "net9.0-windows");
        }

        AppPath = Path.Combine(publishDir, "Photopipeline.exe");
    }

    protected void StartApp(string? args = null)
    {
        if (!File.Exists(AppPath))
            throw new FileNotFoundException($"Application not found at: {AppPath}");

        var startInfo = new ProcessStartInfo
        {
            FileName = AppPath,
            Arguments = args ?? string.Empty,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = false
        };

        _appProcess = Process.Start(startInfo);
        _isRunning = true;

        // Wait for app to initialize
        Thread.Sleep(5000);
    }

    protected void StopApp()
    {
        if (_appProcess is { HasExited: false })
        {
            _appProcess.CloseMainWindow();
            if (!_appProcess.WaitForExit(10000))
            {
                try { _appProcess.Kill(); }
                catch { /* best-effort cleanup */ }
            }
        }
        _isRunning = false;
    }

    protected bool WaitForCondition(Func<bool> condition, int timeoutMs = 30000)
    {
        var sw = Stopwatch.StartNew();
        while (sw.ElapsedMilliseconds < timeoutMs)
        {
            if (condition()) return true;
            Thread.Sleep(200);
        }
        return false;
    }

    public virtual void Dispose()
    {
        StopApp();
        _appProcess?.Dispose();
    }
}
