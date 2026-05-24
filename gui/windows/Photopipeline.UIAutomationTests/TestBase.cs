using OpenQA.Selenium.Appium.Service;
using OpenQA.Selenium.Appium.Windows;
using System.Diagnostics;

namespace Photopipeline.UIAutomationTests;

public abstract class UIAutomationTestBase : IDisposable
{
    protected WindowsDriver? Driver;
    protected Process? ServerProcess;
    protected Process? GuiProcess;
    protected AppiumLocalService? AppiumService;

    private bool _disposed;
    private static bool _driverAvailable = true;

    protected UIAutomationTestBase()
    {
    }

    protected bool TryStartDriver()
    {
        if (!_driverAvailable)
            return false;

        try
        {
            StartServer();
        }
        catch
        {
            _driverAvailable = false;
            return false;
        }

        try
        {
            StartGui();
        }
        catch
        {
            StopServer();
            _driverAvailable = false;
            return false;
        }

        try
        {
            var appiumOptions = new AppiumOptions();
            appiumOptions.PlatformName = "Windows";
            appiumOptions.AddAdditionalAppiumOption("deviceName", "WindowsPC");
            appiumOptions.AddAdditionalAppiumOption("app", GetAppId());

            Driver = new WindowsDriver(
                new Uri("http://127.0.0.1:4723"),
                appiumOptions);

            Driver.Manage().Window.Maximize();
            Driver.Manage().Timeouts().ImplicitWait = TimeSpan.FromSeconds(10);

            return true;
        }
        catch
        {
            StopGui();
            StopServer();
            _driverAvailable = false;
            return false;
        }
    }

    protected void StartDriverOrThrow()
    {
        if (!TryStartDriver())
            throw new SkipTestException("UI automation environment not available");
    }

    protected AppiumElement WaitForElement(By by, int timeoutMs = 10000)
    {
        Assert.NotNull(Driver);
        var wait = new OpenQA.Selenium.Support.UI.WebDriverWait(
            Driver, TimeSpan.FromMilliseconds(timeoutMs));
        wait.PollingInterval = TimeSpan.FromMilliseconds(200);
        return (AppiumElement)wait.Until(d => d.FindElement(by));
    }

    protected AppiumElement? WaitForElementOrNull(By by, int timeoutMs = 5000)
    {
        try
        {
            return WaitForElement(by, timeoutMs);
        }
        catch
        {
            return null;
        }
    }

    protected void WaitForText(string automationId, string expected, int timeoutMs = 10000)
    {
        var element = WaitForElement(MobileBy.AccessibilityId(automationId), timeoutMs);
        var end = DateTime.Now.AddMilliseconds(timeoutMs);
        while (DateTime.Now < end)
        {
            if (element.Text.Contains(expected, StringComparison.OrdinalIgnoreCase)) return;
            Thread.Sleep(200);
        }
        throw new TimeoutException(
            $"Text '{expected}' not found in element '{automationId}'. Actual: '{element.Text}'");
    }

    protected int WaitForElementCount(By by, int minCount, int timeoutMs = 10000)
    {
        Assert.NotNull(Driver);
        var end = DateTime.Now.AddMilliseconds(timeoutMs);
        while (DateTime.Now < end)
        {
            var count = Driver!.FindElements(by).Count;
            if (count >= minCount) return count;
            Thread.Sleep(200);
        }
        return 0;
    }

    protected virtual string GetAppId()
    {
        var envAppId = Environment.GetEnvironmentVariable("PHOTOPIPELINE_APP_ID");
        if (!string.IsNullOrEmpty(envAppId))
            return envAppId;

        var candidates = new[]
        {
            Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "Photopipeline", "bin", "x64", "Debug", "net8.0-windows", "Photopipeline.exe"),
            Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "Photopipeline", "bin", "x64", "Release", "net8.0-windows", "Photopipeline.exe"),
            Path.Combine(Directory.GetCurrentDirectory(), "publish", "Photopipeline.exe"),
            Path.Combine(Directory.GetCurrentDirectory(), "Photopipeline.exe")
        };

        foreach (var candidate in candidates)
        {
            if (File.Exists(candidate))
                return candidate;
        }

        return candidates[0];
    }

    protected virtual string GetWinAppDriverPath()
    {
        var envPath = Environment.GetEnvironmentVariable("WINAPPDRIVER_PATH");
        if (!string.IsNullOrEmpty(envPath))
            return envPath;

        var candidates = new[]
        {
            @"C:\Program Files (x86)\Windows Application Driver\WinAppDriver.exe",
            @"C:\Program Files\Windows Application Driver\WinAppDriver.exe"
        };

        foreach (var candidate in candidates)
        {
            if (File.Exists(candidate))
                return candidate;
        }

        return candidates[0];
    }

    protected virtual string GetGuiExecutablePath()
    {
        var envPath = Environment.GetEnvironmentVariable("PHOTOPIPELINE_GUI_PATH");
        if (!string.IsNullOrEmpty(envPath))
            return envPath;

        var candidates = new[]
        {
            Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "Photopipeline", "bin", "x64", "Debug", "net8.0-windows", "Photopipeline.exe"),
            Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "Photopipeline", "bin", "x64", "Release", "net8.0-windows", "Photopipeline.exe"),
            Path.Combine(Directory.GetCurrentDirectory(), "publish", "Photopipeline.exe"),
            Path.Combine(Directory.GetCurrentDirectory(), "Photopipeline.exe")
        };

        foreach (var candidate in candidates)
        {
            if (File.Exists(candidate))
                return candidate;
        }

        return candidates[0];
    }

    protected void StartServer()
    {
        var driverPath = GetWinAppDriverPath();
        if (!File.Exists(driverPath))
            throw new SkipTestException("WinAppDriver not found. Skipping UI automation test.");

        ServerProcess = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = driverPath,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            }
        };

        ServerProcess.Start();
        Thread.Sleep(3000);
    }

    protected void StartGui()
    {
        var guiPath = GetGuiExecutablePath();
        if (!File.Exists(guiPath))
            throw new SkipTestException($"GUI executable not found at {guiPath}. Skipping UI automation test.");

        GuiProcess = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = guiPath,
                UseShellExecute = true,
                CreateNoWindow = false
            }
        };

        GuiProcess.Start();
        Thread.Sleep(5000);
    }

    protected void StopGui()
    {
        if (GuiProcess is { HasExited: false })
        {
            GuiProcess.CloseMainWindow();
            GuiProcess.WaitForExit(5000);
            if (!GuiProcess.HasExited)
            {
                GuiProcess.Kill(entireProcessTree: true);
                GuiProcess.WaitForExit(3000);
            }
            GuiProcess.Dispose();
            GuiProcess = null;
        }
    }

    protected void StopServer()
    {
        if (ServerProcess is { HasExited: false })
        {
            ServerProcess.Kill(entireProcessTree: true);
            ServerProcess.WaitForExit(3000);
            ServerProcess.Dispose();
            ServerProcess = null;
        }
    }

    protected AppiumElement FindByAccessibilityId(string id)
    {
        Assert.NotNull(Driver);
        return Driver!.FindElement(MobileBy.AccessibilityId(id));
    }

    protected AppiumElement FindByName(string name)
    {
        Assert.NotNull(Driver);
        return Driver!.FindElement(MobileBy.Name(name));
    }

    protected AppiumElement? FindByAccessibilityIdOrNull(string id)
    {
        Assert.NotNull(Driver);
        try
        {
            return Driver!.FindElement(MobileBy.AccessibilityId(id));
        }
        catch
        {
            return null;
        }
    }

    protected AppiumElement? FindByNameOrNull(string name)
    {
        Assert.NotNull(Driver);
        try
        {
            return Driver!.FindElement(MobileBy.Name(name));
        }
        catch
        {
            return null;
        }
    }

    protected void WaitForElementToDisappear(By by, int timeoutMs = 10000)
    {
        Assert.NotNull(Driver);
        var endTime = DateTime.Now.AddMilliseconds(timeoutMs);
        while (DateTime.Now < endTime)
        {
            try
            {
                if (!Driver!.FindElement(by).Displayed)
                    return;
            }
            catch
            {
                return;
            }
            Thread.Sleep(200);
        }
        throw new TimeoutException($"Element {by} still visible after {timeoutMs}ms");
    }

    protected void Click(By by)
    {
        Assert.NotNull(Driver);
        var element = Driver!.FindElement(by);
        element.Click();
    }

    protected void SendKeys(By by, string text)
    {
        Assert.NotNull(Driver);
        var element = Driver!.FindElement(by);
        element.Clear();
        element.SendKeys(text);
    }

    protected string GetText(By by)
    {
        Assert.NotNull(Driver);
        return Driver!.FindElement(by).Text;
    }

    protected void TakeScreenshot(string testName)
    {
        if (Driver is null) return;
        try
        {
            var screenshot = Driver.GetScreenshot();
            var screenshotDir = Path.Combine(Directory.GetCurrentDirectory(), "Screenshots");
            Directory.CreateDirectory(screenshotDir);
            var filename = Path.Combine(screenshotDir, $"{testName}_{DateTime.Now:yyyyMMdd_HHmmss}.png");
            screenshot.SaveAsFile(filename);
        }
        catch
        {
        }
    }

    protected void RestartApplication()
    {
        StopGui();
        Thread.Sleep(1000);
        StartGui();
    }

    protected void SwitchToDarkTheme()
    {
        try
        {
            FindByName("Edit").Click();
            Thread.Sleep(300);
        }
        catch
        {
        }
    }

    protected void SwitchToLightTheme()
    {
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        try
        {
            Driver?.Quit();
            Driver?.Dispose();
        }
        catch { }

        Driver = null;

        StopGui();
        StopServer();

        AppiumService?.Dispose();
    }
}

public sealed class SkipTestException : Exception
{
    public SkipTestException(string message) : base(message) { }
}
