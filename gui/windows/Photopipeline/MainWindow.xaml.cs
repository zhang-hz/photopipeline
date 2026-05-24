using System.Diagnostics;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Photopipeline.Services;
using Photopipeline.ViewModels;
using WinRT.Interop;

namespace Photopipeline;

public sealed partial class MainWindow : Window
{
    private readonly MainViewModel _viewModel;

    public MainWindow()
    {
        this.InitializeComponent();

        _viewModel = App.Services.GetRequiredService<MainViewModel>();
        RootGrid.DataContext = _viewModel;

        // Set title bar extension in code-behind (not supported in XAML for WinUI 3)
        this.ExtendsContentIntoTitleBar = true;
        this.Activated += OnWindowActivated;

        var hWnd = WindowNative.GetWindowHandle(this);
        var windowId = Win32Interop.GetWindowIdFromWindow(hWnd);
        var appWindow = AppWindow.GetFromWindowId(windowId);
        appWindow.Title = "Photopipeline";

        this.Closed += (_, _) =>
        {
            var procs = Process.GetProcessesByName("photopipeline-server");
            foreach (var p in procs)
            {
                try { p.Kill(); } catch { }
            }
        };
    }

    private void OnWindowActivated(object sender, WindowActivatedEventArgs args)
    {
        if (args.WindowActivationState == WindowActivationState.Deactivated)
        {
            AppTitleBar.Opacity = 0.6;
        }
        else
        {
            AppTitleBar.Opacity = 1.0;
        }
    }
}
