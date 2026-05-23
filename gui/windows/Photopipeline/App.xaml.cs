using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Photopipeline.ViewModels;
using System;

namespace Photopipeline;

public partial class App : Application
{
    private Window? _mainWindow;

    public App()
    {
        this.InitializeComponent();
    }

    protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        _mainWindow = new MainWindow();
        _mainWindow.Activate();
    }
}
