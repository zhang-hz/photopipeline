using Photopipeline.ViewModels;
using Wpf.Ui.Appearance;
using Wpf.Ui.Controls;

namespace Photopipeline;

public partial class MainWindow : FluentWindow
{
    private readonly MainViewModel _viewModel;

    public MainWindow(MainViewModel viewModel)
    {
        _viewModel = viewModel;
        DataContext = viewModel;
        InitializeComponent();
        Closing += (_, _) => _viewModel.Shutdown();
    }

    private void OnThemeToggleClick(object sender, System.Windows.RoutedEventArgs e)
    {
        var current = ApplicationThemeManager.GetAppTheme();
        var next = current == ApplicationTheme.Dark
            ? ApplicationTheme.Light
            : ApplicationTheme.Dark;
        ApplicationThemeManager.Apply(next, WindowBackdropType.Mica);
    }
}
