using System.Collections.Specialized;
using System.Windows.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class FilmstripView : UserControl
{
    public FilmstripView() => InitializeComponent();

    /// <summary>
    /// Synchronizes the ListBox multi-selection with the VM's SelectedImages collection.
    /// Uses batch remove-then-add for efficiency and guards against duplicate/cross-thread updates.
    /// </summary>
    private void OnImageListSelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (DataContext is not FilmstripViewModel vm) return;

        try
        {
            // Batch removal
            foreach (var item in e.RemovedItems)
            {
                if (item is ImageEntry entry)
                    vm.SelectedImages.Remove(entry);
            }

            // Batch addition (avoid duplicates caused by re-selection)
            foreach (var item in e.AddedItems)
            {
                if (item is ImageEntry entry && !vm.SelectedImages.Contains(entry))
                    vm.SelectedImages.Add(entry);
            }
        }
        catch (InvalidOperationException)
        {
            // Collection may have been modified concurrently; best-effort sync
        }
    }
}
