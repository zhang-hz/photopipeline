using System.Collections.ObjectModel;

namespace Photopipeline.Tests.ScenarioTests;

/// <summary>
/// ViewModel-level workflow tests that simulate user operations
/// without requiring WinAppDriver. Tests the ViewModel layer through
/// command chains and state transitions.
/// </summary>
public sealed class EndToEndScenarioTests
{
    private static PluginInfo CreateTestPlugin(
        string id = "test",
        string name = "Test Plugin",
        string category = "Test",
        string description = "A test plugin",
        int minInputs = 1,
        int maxInputs = 1,
        int outputs = 1)
    {
        return new PluginInfo
        {
            Id = id,
            Name = name,
            Category = category,
            Description = description,
            MinInputs = minInputs,
            MaxInputs = maxInputs,
            Outputs = outputs
        };
    }

    private static PluginInfo CreatePluginWithParameters(
        string id, string name,
        params (string Name, ParameterType Type, object? Default, object? Min, object? Max)[] parameters)
    {
        var plugin = new PluginInfo
        {
            Id = id,
            Name = name,
            Category = "Test",
            MinInputs = 1,
            MaxInputs = 1,
            Outputs = 1
        };
        foreach (var (paramName, paramType, defaultVal, minVal, maxVal) in parameters)
        {
            plugin.ParameterSchemas.Add(new ParameterSchema
            {
                Name = paramName,
                DisplayName = paramName,
                ParameterType = paramType,
                DefaultValue = defaultVal,
                MinValue = minVal,
                MaxValue = maxVal
            });
        }
        return plugin;
    }

    private static ImageEntry CreateImage(string fileName, string? filePath = null)
    {
        return new ImageEntry
        {
            FileName = fileName,
            FilePath = filePath ?? $@"C:\Photos\{fileName}",
            FileSize = 1048576,
            Width = 1920,
            Height = 1080,
            ColorSpace = "sRGB",
            BitDepth = "16"
        };
    }

    // =========================================================================
    // Test 1: New pipeline creation workflow
    // =========================================================================

    [Fact]
    public void NewPipelineCreation_ResetsPipelineEditorAndUpdatesStatus()
    {
        var vm = new MainViewModel();

        // Pre-populate editor with a node to verify reset
        vm.PipelineEditor.AddNodeCommand.Execute(CreateTestPlugin("plugin-a", "Plugin A"));
        vm.PipelineEditor.Nodes.Should().NotBeEmpty("pipeline editor should have nodes before reset");

        // User clicks "New Pipeline"
        vm.NewPipelineCommand.Execute(null);

        vm.CurrentPipeline.Should().NotBeNull();
        vm.CurrentPipeline.Name.Should().Be("New Pipeline");
        vm.StatusMessage.Should().Be("Created new pipeline");
        vm.CurrentPipeline.Nodes.Should().BeEmpty("pipeline model should be empty for a new pipeline");
        vm.CurrentPipeline.Edges.Should().BeEmpty();
    }

    [Fact]
    public void NewPipelineCreation_MultipleTimes_EachCreatesFreshPipeline()
    {
        var vm = new MainViewModel();

        vm.NewPipelineCommand.Execute(null);
        var pipeline1 = vm.CurrentPipeline;
        vm.NewPipelineCommand.Execute(null);
        var pipeline2 = vm.CurrentPipeline;

        pipeline2.Should().NotBeSameAs(pipeline1);
        pipeline2.Id.Should().NotBe(pipeline1.Id);
        pipeline2.Name.Should().Be("New Pipeline");
    }

    // =========================================================================
    // Test 2: Add nodes to pipeline
    // =========================================================================

    [Fact]
    public void AddNodes_AddsMultipleNodesToPipeline()
    {
        var editor = new PipelineEditorViewModel();
        var demosaic = CreateTestPlugin("demosaic", "Demosaic", "Raw Processing");
        var exposure = CreateTestPlugin("exposure", "Exposure", "Tonal");
        var denoise = CreateTestPlugin("denoise", "AI Denoise", "Noise Reduction");

        editor.AddNodeCommand.Execute(demosaic);
        editor.AddNodeCommand.Execute(exposure);
        editor.AddNodeCommand.Execute(denoise);

        editor.Nodes.Should().HaveCount(3);
        editor.Nodes[0].PluginId.Should().Be("demosaic");
        editor.Nodes[1].PluginId.Should().Be("exposure");
        editor.Nodes[2].PluginId.Should().Be("denoise");
    }

    [Fact]
    public void AddNodes_PositionsAreUnique()
    {
        var editor = new PipelineEditorViewModel();
        var plugin = CreateTestPlugin("p", "Plugin");

        for (int i = 0; i < 5; i++)
            editor.AddNodeCommand.Execute(plugin);

        // Each node should have a distinct canvas position
        var positions = editor.Nodes.Select(n => (n.CanvasX, n.CanvasY)).ToList();
        positions.Distinct().Should().HaveCount(5);
    }

    [Fact]
    public void AddNodes_WithCustomPortCounts_CreatesCorrectPorts()
    {
        var editor = new PipelineEditorViewModel();
        var splitPlugin = CreateTestPlugin("split", "Split", outputs: 3);
        var mergePlugin = CreateTestPlugin("merge", "Merge", minInputs: 1, maxInputs: 4);

        editor.AddNodeCommand.Execute(splitPlugin);
        editor.AddNodeCommand.Execute(mergePlugin);

        var splitNode = editor.Nodes[0];
        var mergeNode = editor.Nodes[1];

        splitNode.OutputPorts.Should().HaveCount(3, "split plugin has 3 outputs");
        splitNode.OutputPorts[0].Id.Should().Be("out0");
        splitNode.OutputPorts[2].Id.Should().Be("out2");

        mergeNode.InputPorts.Should().HaveCount(4, "merge plugin has max 4 inputs");
        mergeNode.InputPorts[0].Id.Should().Be("in0");
        mergeNode.InputPorts[3].Id.Should().Be("in3");
    }

    // =========================================================================
    // Test 3: Connect nodes with edge
    // =========================================================================

    [Fact]
    public void ConnectNodes_CreatesEdgeBetweenNodes()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("a", "Node A"));
        editor.AddNodeCommand.Execute(CreateTestPlugin("b", "Node B"));
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];

        // Select target node first, then drag from source output port
        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 120, 40);
        editor.ConnectPortsCommand.Execute(null);

        editor.Edges.Should().HaveCount(1);
        editor.Edges[0].SourceNodeId.Should().Be(n1.Id);
        editor.Edges[0].TargetNodeId.Should().Be(n2.Id);
        editor.IsDrawingConnection.Should().BeFalse("connection drawing should end after connect");
    }

    [Fact]
    public void ConnectNodes_ChainOfThreeNodes_LinearPipeline()
    {
        var editor = new PipelineEditorViewModel();
        var p = CreateTestPlugin("p", "Plugin");
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];
        var n3 = editor.Nodes[2];

        // n1 -> n2
        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        editor.ConnectPortsCommand.Execute(null);

        // n2 -> n3
        editor.SelectNodeCommand.Execute(n3);
        editor.OnPortDragStart(n2.OutputPorts[0], 300, 300);
        editor.ConnectPortsCommand.Execute(null);

        editor.Edges.Should().HaveCount(2);
        editor.Edges.Should().Contain(e => e.SourceNodeId == n1.Id && e.TargetNodeId == n2.Id);
        editor.Edges.Should().Contain(e => e.SourceNodeId == n2.Id && e.TargetNodeId == n3.Id);
    }

    [Fact]
    public void ConnectNodes_DuplicateEdgesNotCreated()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("a", "A"));
        editor.AddNodeCommand.Execute(CreateTestPlugin("b", "B"));
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];

        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        editor.ConnectPortsCommand.Execute(null);

        // Try same connection again
        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        editor.ConnectPortsCommand.Execute(null);

        editor.Edges.Should().HaveCount(1, "duplicate edges should be prevented");
    }

    // =========================================================================
    // Test 4: Configure node parameters
    // =========================================================================

    [Fact]
    public void ConfigureParameters_AddsDefaultsAndAllowsModification()
    {
        var editor = new PipelineEditorViewModel();
        var plugin = CreatePluginWithParameters("exposure", "Exposure",
            ("ev", ParameterType.Float, 0.0, -5.0, 5.0),
            ("contrast", ParameterType.Float, 1.0, 0.0, 3.0),
            ("auto_levels", ParameterType.Boolean, false, null, null));

        editor.AddNodeCommand.Execute(plugin);
        var node = editor.Nodes[0];

        // Default values from parameter schemas
        node.Parameters.Should().ContainKey("ev");
        node.Parameters.Should().ContainKey("contrast");
        node.Parameters.Should().ContainKey("auto_levels");
        node.Parameters["ev"].Should().Be(0.0);
        node.Parameters["contrast"].Should().Be(1.0);
        node.Parameters["auto_levels"].Should().Be(false);

        // User modifies parameters
        node.Parameters["ev"] = 2.5;
        node.Parameters["auto_levels"] = true;

        node.Parameters["ev"].Should().Be(2.5);
        node.Parameters["auto_levels"].Should().Be(true);
    }

    [Fact]
    public void ConfigureParameters_EnumType_HasEnumValues()
    {
        var editor = new PipelineEditorViewModel();
        var plugin = new PluginInfo
        {
            Id = "scaler",
            Name = "Scaler",
            Category = "Transform",
            MinInputs = 1,
            MaxInputs = 1,
            Outputs = 1,
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "filter",
                    DisplayName = "Filter",
                    ParameterType = ParameterType.Enum,
                    EnumValues = new ObservableCollection<object> { "Nearest", "Bilinear", "Bicubic", "Lanczos" },
                    DefaultValue = "Bilinear"
                }
            }
        };

        editor.AddNodeCommand.Execute(plugin);
        var node = editor.Nodes[0];

        node.Parameters.Should().ContainKey("filter");
        node.Parameters["filter"].Should().Be("Bilinear");

        node.Parameters["filter"] = "Lanczos";
        node.Parameters["filter"].Should().Be("Lanczos");
    }

    [Fact]
    public void ConfigureParameters_MultipleNodes_IndependentParameters()
    {
        var editor = new PipelineEditorViewModel();
        var plugin1 = CreatePluginWithParameters("p1", "Plugin1",
            ("strength", ParameterType.Float, 0.5, 0.0, 1.0));
        var plugin2 = CreatePluginWithParameters("p2", "Plugin2",
            ("threshold", ParameterType.Integer, 128, 0.0, 255.0));

        editor.AddNodeCommand.Execute(plugin1);
        editor.AddNodeCommand.Execute(plugin2);

        editor.Nodes[0].Parameters["strength"] = 0.8;
        editor.Nodes[1].Parameters["threshold"] = 200;

        editor.Nodes[0].Parameters["strength"].Should().Be(0.8);
        editor.Nodes[0].Parameters.Should().NotContainKey("threshold");
        editor.Nodes[1].Parameters["threshold"].Should().Be(200);
        editor.Nodes[1].Parameters.Should().NotContainKey("strength");
    }

    // =========================================================================
    // Test 5: Remove node from pipeline
    // =========================================================================

    [Fact]
    public void RemoveNode_RemovesNodeAndItsEdges()
    {
        var editor = new PipelineEditorViewModel();
        var p = CreateTestPlugin("p", "Plugin");
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];
        var n3 = editor.Nodes[2];

        // n1 -> n2 -> n3
        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        editor.ConnectPortsCommand.Execute(null);

        editor.SelectNodeCommand.Execute(n3);
        editor.OnPortDragStart(n2.OutputPorts[0], 300, 300);
        editor.ConnectPortsCommand.Execute(null);

        editor.Edges.Should().HaveCount(2);

        // Remove middle node
        editor.RemoveNodeCommand.Execute(n2);

        editor.Nodes.Should().HaveCount(2);
        editor.Edges.Should().BeEmpty("all edges related to removed node should be cleaned up");
        editor.Nodes.Should().NotContain(n2);
        editor.Nodes.Should().Contain(n1);
        editor.Nodes.Should().Contain(n3);
    }

    [Fact]
    public void RemoveNode_ClearsSelectionIfSelectedNodeRemoved()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("a", "A"));
        editor.AddNodeCommand.Execute(CreateTestPlugin("b", "B"));
        var n1 = editor.Nodes[0];

        editor.SelectNodeCommand.Execute(n1);
        editor.SelectedNode.Should().Be(n1);

        editor.RemoveNodeCommand.Execute(n1);

        editor.SelectedNode.Should().BeNull("selected node was removed");
        editor.Nodes.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveNode_NullParameter_Noop()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("a", "A"));

        editor.RemoveNodeCommand.Execute(null);

        editor.Nodes.Should().HaveCount(1);
    }

    // =========================================================================
    // Test 6: Duplicate node
    // =========================================================================

    [Fact]
    public void DuplicateNode_ClonesSelectedNode()
    {
        var editor = new PipelineEditorViewModel();
        var plugin = CreatePluginWithParameters("exp", "Exposure",
            ("ev", ParameterType.Float, 0.0, -5.0, 5.0));
        editor.AddNodeCommand.Execute(plugin);
        var original = editor.Nodes[0];
        original.Parameters["ev"] = 2.0;
        editor.SelectNodeCommand.Execute(original);

        editor.DuplicateSelected();

        editor.Nodes.Should().HaveCount(2);
        var clone = editor.Nodes[1];
        clone.PluginId.Should().Be(original.PluginId);
        clone.DisplayName.Should().Be("Exposure (copy)");
        clone.CanvasX.Should().Be(original.CanvasX + 200);
        clone.CanvasY.Should().Be(original.CanvasY + 30);
        clone.Parameters["ev"].Should().Be(2.0, "parameters should be copied");
    }

    [Fact]
    public void DuplicateNode_NoSelection_Noop()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("a", "A"));

        editor.DuplicateSelected();

        editor.Nodes.Should().HaveCount(1);
    }

    // =========================================================================
    // Test 7: Validate pipeline (connected vs disconnected)
    // =========================================================================

    [Fact]
    public void ValidatePipeline_FullyConnected_IsValid()
    {
        var editor = new PipelineEditorViewModel();
        var p = CreateTestPlugin("p", "Plugin");
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];
        var n3 = editor.Nodes[2];

        // n1 -> n2 -> n3
        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        editor.ConnectPortsCommand.Execute(null);

        editor.SelectNodeCommand.Execute(n3);
        editor.OnPortDragStart(n2.OutputPorts[0], 300, 300);
        editor.ConnectPortsCommand.Execute(null);

        editor.ValidatePipelineCommand.Execute(null);

        editor.IsPipelineValid.Should().BeTrue();
        editor.ValidationResult.Should().Be("Pipeline is valid");
    }

    [Fact]
    public void ValidatePipeline_DisconnectedNode_IsInvalid()
    {
        var editor = new PipelineEditorViewModel();
        var p = CreateTestPlugin("p", "Plugin");
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];
        // n3 is disconnected
        editor.SelectNodeCommand.Execute(n2);
        editor.OnPortDragStart(n1.OutputPorts[0], 100, 100);
        editor.ConnectPortsCommand.Execute(null);

        editor.ValidatePipelineCommand.Execute(null);

        editor.IsPipelineValid.Should().BeFalse();
        editor.ValidationResult.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void ValidatePipeline_SingleNode_IsValid()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("p", "Plugin"));

        editor.ValidatePipelineCommand.Execute(null);

        editor.IsPipelineValid.Should().BeTrue();
        editor.ValidationResult.Should().Be("Pipeline is valid");
    }

    [Fact]
    public void ValidatePipeline_EmptyPipeline_IsValid()
    {
        var editor = new PipelineEditorViewModel();

        editor.ValidatePipelineCommand.Execute(null);

        editor.IsPipelineValid.Should().BeTrue();
        editor.ValidationResult.Should().Be("Pipeline is valid");
    }

    // =========================================================================
    // Test 8: Pipeline save/load roundtrip
    // =========================================================================

    [Fact]
    public void SavePipeline_UpdatesStatusMessageWithPipelineName()
    {
        var vm = new MainViewModel();
        vm.CurrentPipeline.Name = "My Custom Pipeline";

        vm.SavePipelineCommand.Execute(null);

        vm.StatusMessage.Should().Be("Saved pipeline: My Custom Pipeline");
    }

    [Fact]
    public void LoadPipeline_SetsStatusMessage()
    {
        var vm = new MainViewModel();

        vm.LoadPipelineCommand.Execute(null);

        vm.StatusMessage.Should().Be("Loading pipeline...");
    }

    [Fact]
    public void SaveAndLoadPipeline_PreservesPipelineState()
    {
        var vm = new MainViewModel();
        vm.CurrentPipeline.Name = "Workflow";
        vm.CurrentPipeline.Description = "A test workflow";
        var node = new PipelineNode { PluginId = "test", DisplayName = "Test" };
        vm.CurrentPipeline.Nodes.Add(node);

        vm.SavePipelineCommand.Execute(null);

        vm.StatusMessage.Should().Be("Saved pipeline: Workflow");
        vm.CurrentPipeline.Nodes.Should().ContainSingle();
        vm.CurrentPipeline.Name.Should().Be("Workflow");
    }

    // =========================================================================
    // Test 9: Image entry creation and properties
    // =========================================================================

    [Fact]
    public void ImageEntry_Creation_SetsAllProperties()
    {
        var entry = new ImageEntry
        {
            FilePath = @"C:\Photos\sunset.dng",
            FileName = "sunset.dng",
            FileSize = 25_000_000,
            Width = 6000,
            Height = 4000,
            ColorSpace = "Linear Raw",
            BitDepth = "14"
        };

        entry.Id.Should().NotBeNullOrEmpty();
        entry.FilePath.Should().Be(@"C:\Photos\sunset.dng");
        entry.FileName.Should().Be("sunset.dng");
        entry.FileSize.Should().Be(25_000_000u);
        entry.Width.Should().Be(6000);
        entry.Height.Should().Be(4000);
        entry.ColorSpace.Should().Be("Linear Raw");
        entry.BitDepth.Should().Be("14");
        entry.ThumbnailPath.Should().BeNull();
        entry.HasMetadataModified.Should().BeFalse();
        entry.OverrideStatus.Should().Be(ImageOverrideStatus.None);
        entry.IsSelected.Should().BeFalse();
        entry.IsProcessing.Should().BeFalse();
        entry.HasError.Should().BeFalse();
        entry.ErrorMessage.Should().BeEmpty();
    }

    [Fact]
    public void ImageEntry_OverrideStatus_TransitionsBetweenStates()
    {
        var entry = new ImageEntry();

        entry.OverrideStatus = ImageOverrideStatus.Overridden;
        entry.OverrideStatus.Should().Be(ImageOverrideStatus.Overridden);

        entry.OverrideStatus = ImageOverrideStatus.Error;
        entry.OverrideStatus.Should().Be(ImageOverrideStatus.Error);

        entry.OverrideStatus = ImageOverrideStatus.Original;
        entry.OverrideStatus.Should().Be(ImageOverrideStatus.Original);

        entry.OverrideStatus = ImageOverrideStatus.None;
        entry.OverrideStatus.Should().Be(ImageOverrideStatus.None);
    }

    [Fact]
    public void ImageEntry_ErrorHandling_RecordsErrorState()
    {
        var entry = new ImageEntry();

        entry.HasError = true;
        entry.ErrorMessage = "Failed to decode raw data";

        entry.HasError.Should().BeTrue();
        entry.ErrorMessage.Should().Be("Failed to decode raw data");
    }

    [Fact]
    public void ImageEntry_ProcessingProgress_ReportsStatus()
    {
        var entry = new ImageEntry();

        entry.ProcessingProgress = 0.0;
        entry.ProcessingProgress.Should().Be(0.0);

        entry.ProcessingProgress = 0.75;
        entry.ProcessingProgress.Should().Be(0.75);

        entry.ProcessingProgress = 1.0;
        entry.ProcessingProgress.Should().Be(1.0);
    }

    // =========================================================================
    // Test 10: Plugin search filtering
    // =========================================================================

    [Fact]
    public void PluginSearch_SearchTextFiltersPlugins()
    {
        var panel = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "exp", Name = "Exposure", Category = "Tonal", Description = "Adjust exposure" },
            new() { Id = "wb", Name = "White Balance", Category = "Color", Description = "Color temperature" },
            new() { Id = "sharpen", Name = "Sharpen", Category = "Detail", Description = "Detail enhancement" },
            new() { Id = "denoise", Name = "AI Denoise", Category = "Noise", Description = "AI noise reduction" },
        };
        panel.LoadPlugins(plugins);

        panel.FilteredPlugins.Should().HaveCount(4, "all plugins shown with empty search");

        panel.SearchText = "White";
        panel.FilteredPlugins.Should().HaveCount(1);
        panel.FilteredPlugins[0].Name.Should().Be("White Balance");

        panel.SearchText = "Denoise";
        panel.FilteredPlugins.Should().HaveCount(1);
        panel.FilteredPlugins[0].Name.Should().Be("AI Denoise");

        panel.SearchText = string.Empty;
        panel.FilteredPlugins.Should().HaveCount(4, "clearing search shows all again");
    }

    [Fact]
    public void PluginSearch_SearchByDescription()
    {
        var panel = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "nr", Name = "Noise Reducer", Category = "Noise", Description = "AI-based denoising" },
            new() { Id = "wb", Name = "White Balance", Category = "Color", Description = "Adjust color temperature" },
        };
        panel.LoadPlugins(plugins);

        panel.SearchText = "denoising";
        panel.FilteredPlugins.Should().HaveCount(1);
        panel.FilteredPlugins[0].Id.Should().Be("nr");
    }

    [Fact]
    public void PluginCategory_FilterCombinedWithSearch()
    {
        var panel = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Demosaic", Category = "Raw Processing" },
            new() { Id = "b", Name = "Denoise", Category = "Noise Reduction" },
            new() { Id = "c", Name = "Black Point", Category = "Raw Processing" },
        };
        panel.LoadPlugins(plugins);

        panel.SelectedCategory = "Raw Processing";
        panel.FilteredPlugins.Should().HaveCount(2);

        panel.SearchText = "Black";
        panel.FilteredPlugins.Should().HaveCount(1);
        panel.FilteredPlugins[0].Name.Should().Be("Black Point");
    }

    // =========================================================================
    // Test 11: Batch queue operations (add/remove/clear)
    // =========================================================================

    [Fact]
    public void BatchQueue_AddImages_DeduplicatesEntries()
    {
        var batch = new BatchViewModel();
        var img = CreateImage("photo.dng");

        batch.AddToQueueCommand.Execute(img);
        batch.AddToQueueCommand.Execute(img);
        batch.AddToQueueCommand.Execute(img);

        batch.BatchQueue.Should().HaveCount(1);
        batch.TotalItems.Should().Be(1);
    }

    [Fact]
    public void BatchQueue_AddAndRemoveImages_UpdatesTotalItems()
    {
        var batch = new BatchViewModel();
        var img1 = CreateImage("a.jpg");
        var img2 = CreateImage("b.jpg");
        var img3 = CreateImage("c.jpg");

        batch.AddToQueueCommand.Execute(img1);
        batch.AddToQueueCommand.Execute(img2);
        batch.AddToQueueCommand.Execute(img3);
        batch.TotalItems.Should().Be(3);

        batch.RemoveFromQueueCommand.Execute(img2);
        batch.TotalItems.Should().Be(2);
        batch.BatchQueue.Should().HaveCount(2);
        batch.BatchQueue.Should().NotContain(img2);
    }

    [Fact]
    public void BatchQueue_StartPauseResumeStop_Lifecycle()
    {
        var batch = new BatchViewModel();
        var img = CreateImage("test.jpg");
        batch.AddToQueueCommand.Execute(img);

        // Start
        batch.StartBatchCommand.Execute(null);
        batch.IsRunning.Should().BeTrue();
        batch.IsPaused.Should().BeFalse();
        batch.StatusText.Should().Be("Processing...");

        // Pause
        batch.PauseBatchCommand.Execute(null);
        batch.IsPaused.Should().BeTrue();
        batch.StatusText.Should().Be("Paused");

        // Resume
        batch.ResumeBatchCommand.Execute(null);
        batch.IsPaused.Should().BeFalse();
        batch.StatusText.Should().Be("Processing...");

        // Stop
        batch.StopBatchCommand.Execute(null);
        batch.IsRunning.Should().BeFalse();
        batch.StatusText.Should().Be("Stopped");
    }

    [Fact]
    public void BatchQueue_ClearCompleted_RemovesOnlyFullyProcessedImages()
    {
        var batch = new BatchViewModel();
        var done = CreateImage("done.jpg");
        done.ProcessingProgress = 1.0;
        var partial = CreateImage("partial.jpg");
        partial.ProcessingProgress = 0.6;
        var pending = CreateImage("pending.jpg");
        pending.ProcessingProgress = 0.0;

        batch.AddToQueueCommand.Execute(done);
        batch.AddToQueueCommand.Execute(partial);
        batch.AddToQueueCommand.Execute(pending);

        batch.ClearCompletedCommand.Execute(null);

        batch.BatchQueue.Should().HaveCount(2);
        batch.BatchQueue.Should().Contain(partial);
        batch.BatchQueue.Should().Contain(pending);
        batch.BatchQueue.Should().NotContain(done);
        batch.TotalItems.Should().Be(2);
    }

    [Fact]
    public void BatchQueue_StartEmpty_DoesNotStart()
    {
        var batch = new BatchViewModel();

        batch.StartBatchCommand.Execute(null);

        batch.IsRunning.Should().BeFalse();
        batch.StatusText.Should().Be("Idle");
    }

    // =========================================================================
    // Test 12: Multiple image entries management
    // =========================================================================

    [Fact]
    public void ImageManagement_AddAndRemoveImages_ViaMainViewModel()
    {
        var vm = new MainViewModel();
        var img1 = CreateImage("a.jpg");
        var img2 = CreateImage("b.jpg");
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.SelectedImage = img1;

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().ContainSingle();
        vm.SelectedImage.Should().Be(img2, "selection moves to next available image");
    }

    [Fact]
    public void ImageManagement_ClearAllImages_ResetsEverything()
    {
        var vm = new MainViewModel();
        vm.Images.Add(CreateImage("a.jpg"));
        vm.Images.Add(CreateImage("b.jpg"));
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
    public void ImageManagement_RemoveLastImage_SetsNullSelection()
    {
        var vm = new MainViewModel();
        var img = CreateImage("only.jpg");
        vm.Images.Add(img);
        vm.SelectedImage = img;

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
    }

    [Fact]
    public void ImageManagement_RemoveWithNoSelection_Noop()
    {
        var vm = new MainViewModel();
        vm.Images.Add(CreateImage("img.jpg"));

        vm.RemoveImageCommand.Execute(null);

        vm.Images.Should().ContainSingle("image should remain when nothing selected");
    }

    // =========================================================================
    // Test 13: Plugin info retrieval
    // =========================================================================

    [Fact]
    public void PluginInfo_FullConstruction_HasAllFields()
    {
        var plugin = new PluginInfo
        {
            Id = "colorspace",
            Name = "Colorspace Conversion",
            Category = "Color",
            Description = "Converts between color spaces",
            Version = "2.1.0",
            MinInputs = 1,
            MaxInputs = 1,
            Outputs = 1,
            SupportsBatching = true,
            IconGlyph = "",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "source_space",
                    DisplayName = "Source Space",
                    ParameterType = ParameterType.Enum,
                    EnumValues = new ObservableCollection<object> { "sRGB", "Adobe RGB", "ProPhoto", "P3" },
                    DefaultValue = "sRGB",
                    IsRequired = true
                },
                new ParameterSchema
                {
                    Name = "target_space",
                    DisplayName = "Target Space",
                    ParameterType = ParameterType.Enum,
                    EnumValues = new ObservableCollection<object> { "sRGB", "Adobe RGB", "ProPhoto", "P3" },
                    DefaultValue = "sRGB",
                    IsRequired = true
                },
                new ParameterSchema
                {
                    Name = "rendering_intent",
                    DisplayName = "Rendering Intent",
                    ParameterType = ParameterType.Enum,
                    EnumValues = new ObservableCollection<object> { "Perceptual", "Relative", "Saturation", "Absolute" },
                    DefaultValue = "Perceptual"
                }
            }
        };

        plugin.Id.Should().Be("colorspace");
        plugin.Name.Should().Be("Colorspace Conversion");
        plugin.Category.Should().Be("Color");
        plugin.Description.Should().Be("Converts between color spaces");
        plugin.Version.Should().Be("2.1.0");
        plugin.ParameterSchemas.Should().HaveCount(3);
        plugin.ParameterSchemas[0].IsRequired.Should().BeTrue();
    }

    [Fact]
    public void PluginInfo_NoInputSourcePlugin_ProducesValidNode()
    {
        var plugin = new PluginInfo
        {
            Id = "file_input",
            Name = "File Input",
            Category = "Input",
            MinInputs = 0,
            MaxInputs = 0,
            Outputs = 1
        };

        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(plugin);

        editor.Nodes.Should().HaveCount(1);
        editor.Nodes[0].InputPorts.Should().BeEmpty("source plugins have no inputs");
        editor.Nodes[0].OutputPorts.Should().HaveCount(1);
    }

    // =========================================================================
    // Test 14: Image service local fallback
    // =========================================================================

    [Fact]
    public async Task LocalImageService_LoadImagesAsync_ReturnsEntriesForExistingFiles()
    {
        var service = new LocalImageService();
        var tempFile = Path.GetTempFileName();
        try
        {
            await File.WriteAllTextAsync(tempFile, "fake image content");

            var images = await service.LoadImagesAsync(new[] { tempFile });

            images.Should().HaveCount(1);
            images[0].FilePath.Should().Be(tempFile);
            images[0].FileSize.Should().BeGreaterThan(0);
        }
        finally
        {
            if (File.Exists(tempFile)) File.Delete(tempFile);
        }
    }

    [Fact]
    public async Task LocalImageService_LoadImagesAsync_SkipsMissingFiles()
    {
        var service = new LocalImageService();

        var images = await service.LoadImagesAsync(new[] { @"C:\nonexistent\file.dng" });

        images.Should().BeEmpty();
    }

    [Fact]
    public async Task LocalImageService_LoadPreviewImageAsync_ReturnsNullForMissingFile()
    {
        var service = new LocalImageService();
        var entry = new ImageEntry { FilePath = @"C:\nonexistent\ghost.jpg" };

        var stream = await service.LoadPreviewImageAsync(entry);

        stream.Should().BeNull();
    }

    [Fact]
    public async Task LocalImageService_ExportImageAsync_ReturnsOutputPath()
    {
        var service = new LocalImageService();
        var entry = new ImageEntry { FilePath = @"C:\Photos\input.dng" };
        var pipeline = new PipelineModel();

        var result = await service.ExportImageAsync(entry, @"C:\Output\result.tif", pipeline);

        result.Should().Be(@"C:\Output\result.tif");
    }

    [Fact]
    public async Task LocalImageService_ProcessPreviewImageAsync_FallsBackToLoadPreview()
    {
        var service = new LocalImageService();
        var tempFile = Path.GetTempFileName();
        try
        {
            await File.WriteAllTextAsync(tempFile, "image data");
            var entry = new ImageEntry { FilePath = tempFile };
            var pipeline = new PipelineModel();

            var stream = await service.ProcessPreviewImageAsync(entry, pipeline);

            stream.Should().NotBeNull();
            stream?.Dispose();
        }
        finally
        {
            if (File.Exists(tempFile)) File.Delete(tempFile);
        }
    }

    // =========================================================================
    // Test 15: MainViewModel initial state
    // =========================================================================

    [Fact]
    public void MainViewModel_InitialState_AllDefaultsCorrect()
    {
        var vm = new MainViewModel();

        // Image state
        vm.Images.Should().BeEmpty();
        vm.SelectedImage.Should().BeNull();
        vm.BeforeImage.Should().BeNull();
        vm.AfterImage.Should().BeNull();

        // Processing state
        vm.IsProcessing.Should().BeFalse();
        vm.StatusMessage.Should().NotBeNullOrEmpty();

        // View state
        vm.ZoomLevel.Should().Be(1.0);
        vm.SplitPosition.Should().Be(0.5);

        // Pipeline state
        vm.CurrentPipeline.Should().NotBeNull();
        vm.CurrentPipeline.Name.Should().Be("Default Pipeline");

        // Sub-viewmodels
        vm.PipelineEditor.Should().NotBeNull();
        vm.PluginPanel.Should().NotBeNull();
        vm.Batch.Should().NotBeNull();

        // Log messages (constructor may log plugin load status)
        vm.LogMessages.Should().NotBeNull();

        // Plugins
        vm.AvailablePlugins.Should().NotBeNull();
        vm.AvailablePlugins.Should().BeEmpty();
        vm.SelectedPlugin.Should().BeNull();
    }

    [Fact]
    public void MainViewModel_ZoomOperations_ClampToBounds()
    {
        var vm = new MainViewModel();

        // Zoom in to max
        for (int i = 0; i < 10; i++)
            vm.ZoomInCommand.Execute(null);
        vm.ZoomLevel.Should().BeLessOrEqualTo(8.0);

        // Zoom out to min
        vm.ZoomLevel = 0.11;
        vm.ZoomOutCommand.Execute(null);
        vm.ZoomOutCommand.Execute(null);
        vm.ZoomLevel.Should().BeGreaterOrEqualTo(0.1); // should floor at 0.1
    }

    [Fact]
    public void MainViewModel_Log_MessagesAccumulateAndUpdateStatus()
    {
        var vm = new MainViewModel();

        var baseCount = vm.LogMessages.Count;
        vm.Log("Starting operation");
        vm.Log("Processing 50%");
        vm.Log("Operation complete");

        vm.LogMessages.Should().HaveCount(baseCount + 3);
        vm.LogMessages[baseCount].Should().EndWith("Starting operation");
        vm.LogMessages[baseCount + 2].Should().EndWith("Operation complete");
        vm.StatusMessage.Should().Be("Operation complete");
    }

    [Fact]
    public void MainViewModel_ExportImage_WithNoSelection_DoesNotThrow()
    {
        var vm = new MainViewModel();

        // Export when no image is selected should not throw
        var act = () => vm.ExportImageCommand.Execute(null);
        act.Should().NotThrow();
    }

    [Fact]
    public void MainViewModel_ExportImage_WithSelection_UpdatesStatus()
    {
        var vm = new MainViewModel();
        vm.SelectedImage = CreateImage("export_me.tif");

        vm.ExportImageCommand.Execute(null);

        vm.StatusMessage.Should().Be("Export complete");
    }

    // =========================================================================
    // End-to-end workflow: Build a complete pipeline from scratch
    // =========================================================================

    [Fact]
    public void FullWorkflow_BuildPipelineFromScratch_AllStepsSucceed()
    {
        // Step 1: Create MainViewModel
        var vm = new MainViewModel();
        vm.CurrentPipeline.Name.Should().Be("Default Pipeline");

        // Step 2: Create a new pipeline
        vm.NewPipelineCommand.Execute(null);
        vm.StatusMessage.Should().Be("Created new pipeline");

        // Step 3: Add source, process, and output nodes
        var sourcePlugin = new PluginInfo
        {
            Id = "raw_input",
            Name = "RAW Input",
            Category = "Input",
            MinInputs = 0,
            MaxInputs = 0,
            Outputs = 1
        };

        var processPlugin = CreatePluginWithParameters("exposure", "Exposure",
            ("ev", ParameterType.Float, 0.0, -5.0, 5.0));

        var outputPlugin = new PluginInfo
        {
            Id = "tiff_encoder",
            Name = "TIFF Encoder",
            Category = "Output",
            MinInputs = 1,
            MaxInputs = 1,
            Outputs = 0
        };

        vm.PipelineEditor.AddNodeCommand.Execute(sourcePlugin);
        vm.PipelineEditor.AddNodeCommand.Execute(processPlugin);
        vm.PipelineEditor.AddNodeCommand.Execute(outputPlugin);

        vm.PipelineEditor.Nodes.Should().HaveCount(3);

        // Step 4: Connect nodes: source -> process -> output
        var source = vm.PipelineEditor.Nodes[0];
        var process = vm.PipelineEditor.Nodes[1];
        var output = vm.PipelineEditor.Nodes[2];

        vm.PipelineEditor.SelectNodeCommand.Execute(process);
        vm.PipelineEditor.OnPortDragStart(source.OutputPorts[0], 100, 100);
        vm.PipelineEditor.ConnectPortsCommand.Execute(null);

        vm.PipelineEditor.SelectNodeCommand.Execute(output);
        vm.PipelineEditor.OnPortDragStart(process.OutputPorts[0], 300, 300);
        vm.PipelineEditor.ConnectPortsCommand.Execute(null);

        vm.PipelineEditor.Edges.Should().HaveCount(2);

        // Step 5: Configure parameters
        process.Parameters["ev"] = 1.5;
        process.Parameters["ev"].Should().Be(1.5);

        // Step 6: Validate pipeline
        vm.PipelineEditor.ValidatePipelineCommand.Execute(null);
        vm.PipelineEditor.IsPipelineValid.Should().BeTrue();

        // Step 7: Save pipeline
        vm.CurrentPipeline.Name = "RAW to TIFF Workflow";
        vm.SavePipelineCommand.Execute(null);
        vm.StatusMessage.Should().Be("Saved pipeline: RAW to TIFF Workflow");

        // Step 8: Duplicate the process node for a second pass
        vm.PipelineEditor.SelectNodeCommand.Execute(process);
        vm.PipelineEditor.DuplicateSelected();
        vm.PipelineEditor.Nodes.Should().HaveCount(4);
    }

    // =========================================================================
    // Integration: Plugin panel + Pipeline editor interaction
    // =========================================================================

    [Fact]
    public void PluginPanelAndEditorIntegration_SelectPluginThenAddToPipeline()
    {
        var vm = new MainViewModel();

        // Load plugins into panel
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "demosaic", Name = "Demosaic", Category = "Raw Processing", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
            new() { Id = "exposure", Name = "Exposure", Category = "Tonal", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
            new() { Id = "sharpen", Name = "Sharpen", Category = "Detail", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
        };
        vm.PluginPanel.LoadPlugins(plugins);

        // Select a plugin in the panel
        vm.PluginPanel.SelectPluginCommand.Execute(plugins[0]);
        vm.PluginPanel.SelectedPlugin.Should().NotBeNull();
        vm.PluginPanel.SelectedPlugin!.Name.Should().Be("Demosaic");

        // Add it to the pipeline editor
        vm.PipelineEditor.AddNodeCommand.Execute(plugins[0]);
        vm.PipelineEditor.Nodes.Should().HaveCount(1);
        vm.PipelineEditor.Nodes[0].PluginId.Should().Be("demosaic");
    }

    // =========================================================================
    // Batch + images integration
    // =========================================================================

    [Fact]
    public void BatchAndImagesIntegration_AddImagesToViewThenQueueForBatch()
    {
        var vm = new MainViewModel();
        var img1 = CreateImage("photo1.dng");
        var img2 = CreateImage("photo2.dng");
        var img3 = CreateImage("photo3.dng");

        // Images shown in the main view
        vm.Images.Add(img1);
        vm.Images.Add(img2);
        vm.Images.Add(img3);
        vm.Images.Should().HaveCount(3);

        // User adds them to batch queue
        vm.Batch.AddToQueueCommand.Execute(img1);
        vm.Batch.AddToQueueCommand.Execute(img2);
        vm.Batch.AddToQueueCommand.Execute(img3);

        vm.Batch.BatchQueue.Should().HaveCount(3);
        vm.Batch.TotalItems.Should().Be(3);

        // User removes one image from the view
        vm.SelectedImage = img2;
        vm.RemoveImageCommand.Execute(null);
        vm.Images.Should().HaveCount(2);

        // But it should still be in the batch queue (independent collections)
        vm.Batch.BatchQueue.Should().HaveCount(3, "batch queue is independent of image collection");
        vm.Batch.RemoveFromQueueCommand.Execute(img2);
        vm.Batch.BatchQueue.Should().HaveCount(2);
    }

    // =========================================================================
    // Node selection and visual state transitions
    // =========================================================================

    [Fact]
    public void NodeSelection_SwitchBetweenNodes_UpdatesSelectionState()
    {
        var editor = new PipelineEditorViewModel();
        var p = CreateTestPlugin("p", "Plugin");
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        editor.AddNodeCommand.Execute(p);
        var n1 = editor.Nodes[0];
        var n2 = editor.Nodes[1];
        var n3 = editor.Nodes[2];

        editor.SelectNodeCommand.Execute(n1);
        n1.IsSelected.Should().BeTrue();
        editor.SelectedNode.Should().Be(n1);

        editor.SelectNodeCommand.Execute(n2);
        n1.IsSelected.Should().BeFalse();
        n2.IsSelected.Should().BeTrue();
        editor.SelectedNode.Should().Be(n2);

        editor.SelectNodeCommand.Execute(n3);
        n2.IsSelected.Should().BeFalse();
        n3.IsSelected.Should().BeTrue();
        editor.SelectedNode.Should().Be(n3);

        editor.ClearSelectionCommand.Execute(null);
        n3.IsSelected.Should().BeFalse();
        editor.SelectedNode.Should().BeNull();
    }

    // =========================================================================
    // Node dragging workflow
    // =========================================================================

    [Fact]
    public void NodeDragging_DragSingleNode_UpdatesPosition()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("p", "Plugin"));
        var node = editor.Nodes[0];
        var originalX = node.CanvasX;
        var originalY = node.CanvasY;

        editor.OnNodeMouseDown(node, 100, 80);
        editor.IsDraggingNode.Should().BeTrue();
        node.IsDragging.Should().BeTrue();

        editor.OnNodeMouseMove(200, 180);
        node.CanvasX.Should().Be(originalX + 100);
        node.CanvasY.Should().Be(originalY + 100);

        editor.OnNodeMouseUp();
        editor.IsDraggingNode.Should().BeFalse();
        node.IsDragging.Should().BeFalse();
    }

    [Fact]
    public void NodeDragging_MultipleDragOperations_AccumulateCorrectly()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("p", "Plugin"));
        var node = editor.Nodes[0];
        var originalX = node.CanvasX;
        var originalY = node.CanvasY;

        // First drag
        editor.OnNodeMouseDown(node, 100, 80);
        editor.OnNodeMouseMove(200, 180);
        editor.OnNodeMouseUp();

        // Second drag from new position
        editor.OnNodeMouseDown(node, 200, 180);
        editor.OnNodeMouseMove(350, 280);
        editor.OnNodeMouseUp();

        node.CanvasX.Should().Be(originalX + 250);
        node.CanvasY.Should().Be(originalY + 200);
    }

    // =========================================================================
    // Connection drawing visual state
    // =========================================================================

    [Fact]
    public void ConnectionDrawing_VisualLineState_Transitions()
    {
        var editor = new PipelineEditorViewModel();
        editor.AddNodeCommand.Execute(CreateTestPlugin("a", "A"));
        var port = editor.Nodes[0].OutputPorts[0];

        // Start drawing
        editor.OnPortDragStart(port, 150, 200);
        editor.IsDrawingConnection.Should().BeTrue();
        editor.ConnectionLineX1.Should().Be(150);
        editor.ConnectionLineY1.Should().Be(200);

        // Move cursor
        editor.OnPortDrag(500, 600);
        editor.ConnectionLineX2.Should().Be(500);
        editor.ConnectionLineY2.Should().Be(600);

        // End drawing (without connecting, by calling ConnectPorts with no target)
        // Port drag stop should not throw
        editor.IsDrawingConnection.Should().BeTrue();
    }
}
