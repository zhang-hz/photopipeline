using System.Windows.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class FilmstripView : UserControl
{
    public FilmstripView() => InitializeComponent();

    private void OnImageListSelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (DataContext is not FilmstripViewModel vm) return;

        foreach (var item in e.RemovedItems)
        {
            if (item is ImageEntry entry)
                vm.SelectedImages.Remove(entry);
        }

        foreach (var item in e.AddedItems)
        {
            if (item is ImageEntry entry && !vm.SelectedImages.Contains(entry))
                vm.SelectedImages.Add(entry);
        }
    }
}
