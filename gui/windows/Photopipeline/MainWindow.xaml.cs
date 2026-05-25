using Photopipeline.ViewModels;
using System.Windows;

namespace Photopipeline;

public partial class MainWindow : Window
{
    private readonly MainViewModel _viewModel;

    public MainWindow(MainViewModel viewModel)
    {
        _viewModel = viewModel;
        DataContext = viewModel;
        InitializeComponent();
        Closing += (_, _) => _viewModel.Shutdown();
    }
}
