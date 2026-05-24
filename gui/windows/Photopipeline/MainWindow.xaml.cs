using Microsoft.Extensions.DependencyInjection;
using System.Diagnostics;
using System.Windows;
using Photopipeline.ViewModels;

namespace Photopipeline;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        // Set DataContext from DI
        try
        {
            DataContext = App.Services.GetRequiredService<MainViewModel>();
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"[MainWindow] DI failed: {ex}");
        }
    }
}
