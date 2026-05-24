using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class ThreadingScenarioTests
{
    // ═══ UI thread safety ═══
    [Fact]
    public void AllCommands_ExecuteWithoutException()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(new List<PluginInfo>
        {
            new() { Id = "test", Name = "Test", Category = "Basic",
                Description = "", MinInputs = 1, MaxInputs = 1, Outputs = 1 }
        });

        var exceptions = new List<Exception>();

        // Test all major commands on a harness
        try { h.Main.NewPipelineCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.ClearImagesCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.RemoveImageCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.StopExecutionCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.ZoomInCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.ZoomOutCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.ResetZoomCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Main.FitToWindowCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }

        Assert.Empty(exceptions);
    }

    [Fact]
    public void BatchCommands_ExecuteWithoutException()
    {
        var h = new ViewModelTestHarness();
        var img = new ImageEntry { FilePath = "/a.png", FileName = "a.png" };
        h.Batch.AddToQueueCommand.Execute(img);

        var exceptions = new List<Exception>();
        try { h.Batch.StartBatchCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Batch.PauseBatchCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Batch.ResumeBatchCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.Batch.StopBatchCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }

        Assert.Empty(exceptions);
    }

    [Fact]
    public void PipelineCommands_ExecuteWithoutException()
    {
        var h = new ViewModelTestHarness();
        var plugin = new PluginInfo { Id = "test", Name = "Test", Category = "Basic",
            Description = "", MinInputs = 1, MaxInputs = 1, Outputs = 1 };

        var exceptions = new List<Exception>();
        try { h.PipelineEditor.AddNodeCommand.Execute(plugin); } catch (Exception e) { exceptions.Add(e); }
        try { h.PipelineEditor.ZoomInCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.PipelineEditor.ZoomOutCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.PipelineEditor.ResetZoomCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.PipelineEditor.FitAllCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }
        try { h.PipelineEditor.DuplicateSelectedCommand.Execute(null); } catch (Exception e) { exceptions.Add(e); }

        Assert.Empty(exceptions);
    }

    // ═══ PropertyChanged ═══
    [Fact]
    public void PropertyChanged_Images_FiresOnAdd()
    {
        var h = new ViewModelTestHarness();
        int count = 0;
        h.Main.Images.CollectionChanged += (_, _) => count++;
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        Assert.True(count > 0);
    }

    [Fact]
    public void PropertyChanged_SelectedImage_Fires()
    {
        var h = new ViewModelTestHarness();
        h.AddTestImage(TestImageFactory.GetPath("solid_rgb_256.png"), "test.png");
        bool fired = false;
        h.Main.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Main.SelectedImage)) fired = true;
        };
        h.Main.SelectedImage = h.Main.Images[0];
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_ZoomLevel_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.Main.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Main.ZoomLevel)) fired = true;
        };
        h.Main.ZoomInCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_IsSplitView_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.Main.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Main.IsSplitView)) fired = true;
        };
        h.Main.IsSplitView = false;
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_IsSideBySide_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.Main.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Main.IsSideBySide)) fired = true;
        };
        h.Main.IsSideBySide = true;
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_StatusMessage_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.Main.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Main.StatusMessage)) fired = true;
        };
        h.Main.NewPipelineCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Batch_StatusText_Fires()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        bool fired = false;
        h.Batch.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Batch.StatusText)) fired = true;
        };
        h.Batch.StartBatchCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Batch_IsRunning_Fires()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        bool fired = false;
        h.Batch.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Batch.IsRunning)) fired = true;
        };
        h.Batch.StartBatchCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Batch_IsPaused_Fires()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        bool fired = false;
        h.Batch.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Batch.IsPaused)) fired = true;
        };
        h.Batch.PauseBatchCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Batch_OverallProgress_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.Batch.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Batch.OverallProgress)) fired = true;
        };
        h.Batch.OverallProgress = 50;
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Batch_TotalItems_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.Batch.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.Batch.TotalItems)) fired = true;
        };
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Pipeline_Scale_Fires()
    {
        var h = new ViewModelTestHarness();
        bool fired = false;
        h.PipelineEditor.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.PipelineEditor.Scale)) fired = true;
        };
        h.PipelineEditor.ZoomInCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void PropertyChanged_Pipeline_Offset_Fires()
    {
        var h = new ViewModelTestHarness();
        h.PipelineEditor.AddNodeCommand.Execute(new PluginInfo { Id = "a", Name = "A", Category = "T",
            Description = "", MinInputs = 1, MaxInputs = 1, Outputs = 1 });
        bool fired = false;
        h.PipelineEditor.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(h.PipelineEditor.OffsetX)) fired = true;
        };
        h.PipelineEditor.FitAllCommand.Execute(null);
        Assert.True(fired);
    }

    [Fact]
    public void MultiplePropertyChanged_OnSingleOperation()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });

        var changedProperties = new HashSet<string>();
        h.Batch.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName is not null) changedProperties.Add(e.PropertyName);
        };

        h.Batch.StartBatchCommand.Execute(null);

        Assert.Contains(nameof(h.Batch.IsRunning), changedProperties);
        Assert.Contains(nameof(h.Batch.StatusText), changedProperties);
    }

    // ═══ Timer lifecycle ═══
    [Fact]
    public void BatchTimer_StartStop_NoLeakedTimer()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });

        // Start and stop multiple times
        for (int i = 0; i < 5; i++)
        {
            h.Batch.StartBatchCommand.Execute(null);
            Thread.Sleep(50);
            h.Batch.StopBatchCommand.Execute(null);
        }

        // Should not crash
        Assert.False(h.Batch.IsRunning);
    }

    [Fact]
    public void ElapsedTime_UpdatesWhileRunning()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);

        Thread.Sleep(1200); // Wait for at least one timer tick (1s interval)

        var elapsed = h.Batch.ElapsedTime;
        Assert.NotEqual("00:00:00", elapsed);

        h.Batch.StopBatchCommand.Execute(null);
    }

    [Fact]
    public void BatchTimer_Pause_StopsUpdatingElapsed()
    {
        var h = new ViewModelTestHarness();
        h.Batch.AddToQueueCommand.Execute(new ImageEntry { FilePath = "/a.png", FileName = "a.png" });
        h.Batch.StartBatchCommand.Execute(null);
        Thread.Sleep(500);
        h.Batch.PauseBatchCommand.Execute(null);

        var elapsedAtPause = h.Batch.ElapsedTime;
        Thread.Sleep(1200);

        // Timer should be stopped, so elapsed shouldn't change significantly
        var elapsedAfterPause = h.Batch.ElapsedTime;
        Assert.Equal(elapsedAtPause, elapsedAfterPause);

        h.Batch.StopBatchCommand.Execute(null);
    }
}
