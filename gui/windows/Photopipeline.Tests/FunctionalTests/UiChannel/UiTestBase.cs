using System.Diagnostics;
using System.Threading;

namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public abstract class UiTestBase : IDisposable
{
    public readonly string AppPath;
    protected Process? _appProcess;
    protected bool _isRunning;

    /// <summary>
    /// Exposes the underlying WPF process for diagnostics (exit code, stdout, stderr).
    /// </summary>
    public Process? AppProcess => _appProcess;

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

    public void StartApp(string? args = null)
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

        // Poll for app initialization instead of blind Thread.Sleep(5000).
        // Wait for the process to create a main window handle or until timeout.
        if (!WaitForAppReady(TimeSpan.FromSeconds(30)))
        {
            // Fallback: if polling times out, try a shorter sleep as last resort
            Thread.Sleep(2000);
        }
    }

    /// <summary>
    /// Polls the process until it is ready (has a main window handle, is responding).
    /// Returns true if the app became ready within the timeout.
    /// </summary>
    private bool WaitForAppReady(TimeSpan timeout)
    {
        if (_appProcess == null) return false;

        var deadline = DateTime.UtcNow + timeout;
        while (DateTime.UtcNow < deadline)
        {
            if (_appProcess.HasExited)
                throw new InvalidOperationException(
                    $"Application exited prematurely with code {_appProcess.ExitCode}");

            // The process has created its main window
            if (_appProcess.MainWindowHandle != IntPtr.Zero)
                return true;

            // Check if the process is responding
            try
            {
                if (_appProcess.Responding)
                    return true;
            }
            catch
            {
                // Process may not be fully initialized yet
            }

            Thread.Sleep(200);
        }

        return false;
    }

    public void StopApp()
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
