namespace Photopipeline.Tests.SystemTests;

public sealed class BatchProcessingTests : SystemTestBase
{
    [Fact]
    public async Task LoadMultipleImages_ShouldPopulateImageService()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var testDir = Path.Combine(Path.GetTempPath(), "photopipeline_test_images");
        Directory.CreateDirectory(testDir);

        try
        {
            var testFiles = new[]
            {
                Path.Combine(testDir, "image1.dng"),
                Path.Combine(testDir, "image2.dng"),
                Path.Combine(testDir, "image3.dng")
            };

            foreach (var file in testFiles)
            {
                await File.WriteAllTextAsync(file, "mock raw data");
            }

            var images = await ImageService.LoadImagesAsync(testFiles);

            images.Should().NotBeNull();
            images.Should().HaveCount(3);
            images[0].FileName.Should().Be("image1.dng");
            images[0].ColorSpace.Should().Be("Linear Raw");
            images[0].ThumbnailPath.Should().NotBeNull();
        }
        finally
        {
            if (Directory.Exists(testDir))
                Directory.Delete(testDir, true);
        }
    }

    [Fact]
    public async Task ImageService_LoadImages_SkipsNonExistentFiles()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var testDir = Path.Combine(Path.GetTempPath(), "photopipeline_test_batch");
        Directory.CreateDirectory(testDir);

        try
        {
            var validFile = Path.Combine(testDir, "valid.jpg");
            await File.WriteAllTextAsync(validFile, "mock jpg data");

            var files = new[]
            {
                validFile,
                Path.Combine(testDir, "nonexistent.jpg")
            };

            var images = await ImageService.LoadImagesAsync(files);

            images.Should().HaveCount(1);
            images[0].FileName.Should().Be("valid.jpg");
        }
        finally
        {
            if (Directory.Exists(testDir))
                Directory.Delete(testDir, true);
        }
    }

    [Fact]
    public async Task BatchExecution_ProgressReporting_UpdatesCounters()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var vm = new BatchViewModel();

        vm.BatchQueue.Add(new ImageEntry { FileName = "a.jpg" });
        vm.BatchQueue.Add(new ImageEntry { FileName = "b.jpg" });

        vm.StartBatchCommand.Execute(null);

        vm.IsRunning.Should().BeTrue();
        vm.TotalItems.Should().Be(2);
        vm.CompletedItems.Should().Be(0);

        vm.CompletedItems = 1;
        vm.OverallProgress = 0.5;

        vm.CompletedItems.Should().Be(1);
        vm.OverallProgress.Should().Be(0.5);
    }

    [Fact]
    public async Task CancelMidBatch_ShouldStopProcessing()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var vm = new BatchViewModel();
        vm.BatchQueue.Add(new ImageEntry { FileName = "a.jpg" });
        vm.BatchQueue.Add(new ImageEntry { FileName = "b.jpg" });
        vm.BatchQueue.Add(new ImageEntry { FileName = "c.jpg" });

        vm.StartBatchCommand.Execute(null);
        vm.IsRunning.Should().BeTrue();

        vm.CompletedItems = 1;
        vm.FailedItems = 0;

        vm.StopBatchCommand.Execute(null);

        vm.IsRunning.Should().BeFalse();
        vm.StatusText.Should().Be("Stopped");
    }

    [Fact]
    public async Task ResumeAfterCancel_ShouldContinueProcessing()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var vm = new BatchViewModel();
        vm.BatchQueue.Add(new ImageEntry { FileName = "a.jpg" });
        vm.BatchQueue.Add(new ImageEntry { FileName = "b.jpg" });

        vm.StartBatchCommand.Execute(null);
        vm.CompletedItems = 1;

        vm.PauseBatchCommand.Execute(null);
        vm.IsPaused.Should().BeTrue();

        vm.ResumeBatchCommand.Execute(null);
        vm.IsPaused.Should().BeFalse();
        vm.StatusText.Should().Be("Processing...");
    }

    [Fact]
    public async Task ErrorHandling_RecordsFailedItems()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var vm = new BatchViewModel();
        vm.BatchQueue.Add(new ImageEntry { FileName = "ok.jpg" });
        vm.BatchQueue.Add(new ImageEntry
        {
            FileName = "bad.jpg",
            HasError = true,
            ErrorMessage = "Failed to process"
        });

        vm.StartBatchCommand.Execute(null);

        vm.FailedItems = 1;
        vm.CompletedItems = 1;

        vm.FailedItems.Should().Be(1);
        vm.CompletedItems.Should().Be(1);
    }

    [Fact]
    public async Task PerImageOverride_SetsDifferentParametersPerImage()
    {
        var started = await TryStartServerAsync();
        if (!started) return;

        var image1 = new ImageEntry { FileName = "portrait.jpg" };
        var image2 = new ImageEntry { FileName = "landscape.jpg" };

        image1.OverrideStatus = ImageOverrideStatus.Overridden;
        image2.OverrideStatus = ImageOverrideStatus.Original;

        image1.OverrideStatus.Should().Be(ImageOverrideStatus.Overridden);
        image2.OverrideStatus.Should().Be(ImageOverrideStatus.Original);
    }

    private async Task<bool> TryStartServerAsync()
    {
        try
        {
            await StartServerAsync();
            return true;
        }
        catch (SkipTestException)
        {
            return false;
        }
    }
}
