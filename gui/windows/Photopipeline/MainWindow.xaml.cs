using Photopipeline.ViewModels;
using System.Windows;
using System.Windows.Controls;

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

    private void OnThemeComboBoxChanged(object sender, SelectionChangedEventArgs e)
    {
        if (sender is ComboBox cb && cb.SelectedItem is string theme)
            App.ApplyTheme(theme);
    }
}
