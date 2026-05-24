namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class MainViewModelTests
{
    [Fact]
    public void MainViewModel_InitialState_NoImagesLoaded()
    {
        var vm = new MainViewModel();

        vm.Images.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.IsProcessing.Should().BeFalse();
        vm.StatusMessage.Should().Be("Ready");
    }

    [Fact]
    public void MainViewModel_InitialState_DefaultPipelineExists()
    {
        var vm = new MainViewModel();

        vm.CurrentPipeline.Should().NotBeNull();
        vm.CurrentPipeline.Name.Should().Be("Default Pipeline");
    }

    [Fact]
    public void MainViewModel_InitialState_ZoomLevelIsOne()
    {
        var vm = new MainViewModel();

        vm.ZoomLevel.Should().Be(1.0);
    }

    [Fact]
    public void MainViewModel_InitialState_SplitPositionIsHalf()
    {
        var vm = new MainViewModel();

        vm.SplitPosition.Should().Be(0.5);
    }

    [Fact]
    public void MainViewModel_RemoveImage_WithSelectedImage_RemovesFromCollection()
    {
        var vm = new MainViewModel();
        var image = new ImageEntry { FileName = "test.jpg" };
        vm.Images.Add(image);
        vm.SelectedImage = image;

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
    }

    [Fact]
    public void MainViewModel_RemoveImage_NoSelectedImage_NoException()
    {
        var vm = new MainViewModel();

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().BeEmpty();
    }

    [Fact]
    public void MainViewModel_ClearImages_RemovesAllAndResetsSelection()
    {
        var vm = new MainViewModel();
        vm.Images.Add(new ImageEntry { FileName = "a.jpg" });
        vm.Images.Add(new ImageEntry { FileName = "b.jpg" });
        vm.SelectedImage = vm.Images[0];
        vm.BeforeImage = vm.Images[0];
        vm.AfterImage = vm.Images[1];

        vm.ClearImagesCommand.Execute(null);

        vm.Images.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.BeforeImage.Should().BeNull();
        vm.AfterImage.Should().BeNull();
    }

    [Fact]
    public void MainViewModel_RunPipeline_UpdatesStatus()
    {
        var vm = new MainViewModel();
        vm.SelectedImage = new ImageEntry { FilePath = "test.jpg" };

        vm.RunPipelineCommand.Execute(null);

        vm.StatusMessage.Should().Be("Pipeline completed");
    }

    [Fact]
    public void MainViewModel_StopExecution_UpdatesStatus()
    {
        var vm = new MainViewModel();
        vm.RunPipelineCommand.Execute(null);

        vm.StopExecutionCommand.Execute(null);

        vm.CurrentPipeline.IsExecuting.Should().BeFalse();
        vm.StatusMessage.Should().Be("Stopped");
    }

    [Fact]
    public void MainViewModel_ZoomIn_IncreasesZoomLevel()
    {
        var vm = new MainViewModel();

        vm.ZoomInCommand.Execute(null);

        vm.ZoomLevel.Should().BeApproximately(1.25, 0.001);
    }

    [Fact]
    public void MainViewModel_ZoomOut_DecreasesZoomLevel()
    {
        var vm = new MainViewModel { ZoomLevel = 2.0 };

        vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().BeApproximately(1.6, 0.001);
    }

    [Fact]
    public void MainViewModel_ZoomIn_AtMaximum_DoesNotExceed()
    {
        var vm = new MainViewModel { ZoomLevel = 7.5 };

        vm.ZoomInCommand.Execute(null);

        vm.ZoomLevel.Should().Be(8.0);
    }

    [Fact]
    public void MainViewModel_ZoomOut_AtMinimum_DoesNotGoBelow()
    {
        var vm = new MainViewModel { ZoomLevel = 0.15 };

        vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().BeApproximately(0.12, 0.01);

        vm.ZoomOutCommand.Execute(null);

        vm.ZoomLevel.Should().Be(0.1);
    }

    [Fact]
    public void MainViewModel_ResetZoom_ReturnsToDefault()
    {
        var vm = new MainViewModel { ZoomLevel = 4.0 };

        vm.ResetZoomCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
    }

    [Fact]
    public void MainViewModel_FitToWindow_SetsZoomToOne()
    {
        var vm = new MainViewModel { ZoomLevel = 3.5 };

        vm.FitToWindowCommand.Execute(null);

        vm.ZoomLevel.Should().Be(1.0);
    }

    [Fact]
    public void MainViewModel_SplitPosition_PropertyChangedNotification()
    {
        var vm = new MainViewModel();
        var eventRaised = false;

        vm.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(MainViewModel.SplitPosition))
                eventRaised = true;
        };

        vm.SplitPosition = 0.75;

        eventRaised.Should().BeTrue();
    }

    [Fact]
    public void MainViewModel_SelectedImageChanged_TriggersPropertyChange()
    {
        var vm = new MainViewModel();
        var eventRaised = false;

        vm.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(MainViewModel.SelectedImage))
                eventRaised = true;
        };

        vm.SelectedImage = new ImageEntry { FileName = "selected.jpg" };

        eventRaised.Should().BeTrue();
    }

    [Fact]
    public void MainViewModel_NewPipeline_ReplacesCurrentPipeline()
    {
        var vm = new MainViewModel();
        var oldPipeline = vm.CurrentPipeline;

        vm.NewPipelineCommand.Execute(null);

        vm.CurrentPipeline.Should().NotBeSameAs(oldPipeline);
        vm.CurrentPipeline.Name.Should().Be("New Pipeline");
        vm.StatusMessage.Should().Be("Created new pipeline");
    }

    [Fact]
    public void MainViewModel_SubViewModels_AreNotNull()
    {
        var vm = new MainViewModel();

        vm.PipelineEditor.Should().NotBeNull();
        vm.PluginPanel.Should().NotBeNull();
        vm.Batch.Should().NotBeNull();
    }

    [Fact]
    public void MainViewModel_Log_AppendsMessageAndSetsStatus()
    {
        var vm = new MainViewModel();

        vm.Log("Test log message");

        vm.LogMessages.Should().ContainSingle(msg => msg.EndsWith("Test log message"));
        vm.StatusMessage.Should().Be("Test log message");
    }

    [Fact]
    public void MainViewModel_Log_MultipleMessages_AccumulateInOrder()
    {
        var vm = new MainViewModel();

        vm.Log("First");
        vm.Log("Second");

        vm.LogMessages.Should().HaveCount(2);
        vm.LogMessages[0].Should().EndWith("First");
        vm.LogMessages[1].Should().EndWith("Second");
    }

    [Fact]
    public void MainViewModel_AddImage_SetsStatusMessage()
    {
        var vm = new MainViewModel();

        vm.AddImageCommand.Execute(null);

        vm.StatusMessage.Should().Be("Opening file picker...");
    }

    [Fact]
    public void MainViewModel_SendToPhotoshop_SetsStatusMessage()
    {
        var vm = new MainViewModel();

        vm.SendToPhotoshopCommand.Execute(null);

        vm.StatusMessage.Should().Be("Sending current image to Photoshop...");
    }

    [Fact]
    public void MainViewModel_RemoveImage_SelectsNextAvailable()
    {
        var vm = new MainViewModel();
        var img1 = new ImageEntry { FileName = "a.jpg" };
        var img2 = new ImageEntry { FileName = "b.jpg" };
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.SelectedImage = img1;

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().ContainSingle();
        vm.SelectedImage.Should().Be(img2);
    }

    [Fact]
    public void MainViewModel_LoadPipeline_SetsStatusMessage()
    {
        var vm = new MainViewModel();

        vm.LoadPipelineCommand.Execute(null);

        vm.StatusMessage.Should().Be("Loading pipeline...");
    }

    [Fact]
    public void MainViewModel_SavePipeline_SetsStatusMessage()
    {
        var vm = new MainViewModel();

        vm.SavePipelineCommand.Execute(null);

        vm.StatusMessage.Should().Contain("Saved pipeline:");
    }

    [Fact]
    public void MainViewModel_ExportImage_SetsStatusMessage()
    {
        var vm = new MainViewModel();
        vm.SelectedImage = new ImageEntry { FilePath = "test.jpg" };

        vm.ExportImageCommand.Execute(null);

        vm.StatusMessage.Should().Be("Export complete");
    }
}
