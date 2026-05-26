namespace Photopipeline.Services;

public interface IDialogService
{
    string[]? ShowOpenFileDialog(string title, string filter, bool multiselect = true);
    string? ShowSaveFileDialog(string title, string filter, string defaultExt = ".tif");
    string? ShowOpenFolderDialog(string title);
}
