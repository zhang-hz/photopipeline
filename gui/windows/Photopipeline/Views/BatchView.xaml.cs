using System.Windows;
using System.Windows.Controls;
using Photopipeline.Models;
using Photopipeline.ViewModels;

namespace Photopipeline.Views;

public partial class BatchView : UserControl
{
    public BatchView() => InitializeComponent();

    private void OnItemDrop(object sender, DragEventArgs e)
    {
        if (DataContext is not BatchViewModel vm) return;

        if (e.Data.GetData(typeof(ImageEntry)) is ImageEntry entry)
        {
            vm.AddToQueueCommand.Execute(entry);
        }
        else if (e.Data.GetData(typeof(IEnumerable<ImageEntry>)) is IEnumerable<ImageEntry> entries)
        {
            foreach (var item in entries)
                vm.AddToQueueCommand.Execute(item);
        }
    }
}
