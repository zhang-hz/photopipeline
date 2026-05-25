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
            await _vm.SaveCommand.ExecuteAsync(null);
            _saved = true;
            Close();
        };

        CancelBtn.Click += (_, _) => Close();
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
