using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Photopipeline.ViewModels;
using System;
using WinRT.Interop;

namespace Photopipeline;

public sealed partial class MainWindow : Window
{
    private readonly MainViewModel _viewModel;

    public MainWindow()
    {
        this.InitializeComponent();

        _viewModel = new MainViewModel();
        RootGrid.DataContext = _viewModel;

        var hWnd = WindowNative.GetWindowHandle(this);
        var windowId = Win32Interop.GetWindowIdFromWindow(hWnd);
        var appWindow = AppWindow.GetFromWindowId(windowId);
        appWindow.Title = "Photopipeline";
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
