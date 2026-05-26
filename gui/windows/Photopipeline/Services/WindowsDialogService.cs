namespace Photopipeline.Services;

public sealed class WindowsDialogService : IDialogService
{
    public string[]? ShowOpenFileDialog(string title, string filter, bool multiselect = true)
    {
        var dialog = new Microsoft.Win32.OpenFileDialog
        {
            Title = title,
            Filter = filter,
            Multiselect = multiselect
        };
        return dialog.ShowDialog() == true ? dialog.FileNames : null;
    }

    public string? ShowSaveFileDialog(string title, string filter, string defaultExt = ".tif")
    {
        var dialog = new Microsoft.Win32.SaveFileDialog
        {
            Title = title,
            Filter = filter,
            DefaultExt = defaultExt
        };
        return dialog.ShowDialog() == true ? dialog.FileName : null;
    }

    public string? ShowOpenFolderDialog(string title)
    {
        var dialog = new Microsoft.Win32.OpenFolderDialog
        {
            Title = title,
            Multiselect = false
        };
        return dialog.ShowDialog() == true ? dialog.FolderName : null;
    }
}
