using Photopipeline.Models;
using Photopipeline.ViewModels;
using Wpf.Ui.Controls;

namespace Photopipeline.Views;

public partial class SettingsDialog : FluentWindow
{
    private readonly SettingsViewModel _vm;
    private bool _saved;
    private readonly AppSettings _snapshot;

    public SettingsDialog(SettingsViewModel viewModel)
    {
        InitializeComponent();
        _vm = viewModel;
        DataContext = viewModel;

        _snapshot = SnapshotCurrent();

        SaveBtn.Click += async (_, _) =>
        {
            try
            {
                await _vm.SaveCommand.ExecuteAsync(null);
                App.ApplyTheme(_vm.Theme);
                _saved = true;
                Close();
            }
            catch (Exception ex)
            {
                System.Windows.MessageBox.Show($"Failed to save settings: {ex.Message}",
                    "Save Error", System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Warning);
            }
        };

        CancelBtn.Click += (_, _) => Close();

        ResetBtn.Click += async (_, _) =>
        {
            var result = System.Windows.MessageBox.Show(
                "Reset all settings to defaults?",
                "Confirm Reset",
                System.Windows.MessageBoxButton.YesNo,
                System.Windows.MessageBoxImage.Question);
            if (result == System.Windows.MessageBoxResult.Yes)
                await _vm.ResetCommand.ExecuteAsync(null);
        };
    }

    protected override void OnClosed(EventArgs e)
    {
        base.OnClosed(e);
        if (!_saved)
        {
            _vm.LoadFrom(_snapshot);
        }
    }

    private AppSettings SnapshotCurrent()
    {
        return new AppSettings
        {
            Theme = _vm.Theme,
            ServerPath = _vm.ServerPath,
            ServerPort = _vm.ServerPort,
            AutoStartServer = _vm.AutoStartServer,
            DefaultOutputFormat = _vm.DefaultOutputFormat,
            DefaultOutputDirectory = _vm.DefaultOutputDirectory,
            JpegQuality = _vm.JpegQuality,
            EmbedMetadata = _vm.EmbedMetadata,
            ThumbnailSize = _vm.ThumbnailSize,
            MaxRecentFiles = _vm.MaxRecentFiles
        };
    }
}
