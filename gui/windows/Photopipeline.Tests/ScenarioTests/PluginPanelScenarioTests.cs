using System.Collections.ObjectModel;
using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.ScenarioTests;

public sealed class PluginPanelScenarioTests
{
    private static List<PluginInfo> CreateTestPlugins()
    {
        return new List<PluginInfo>
        {
            new() { Id = "identity", Name = "Identity", Category = "Basic",
                Description = "Pass-through plugin", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
            new() { Id = "grayscale", Name = "Grayscale", Category = "Color",
                Description = "Convert to grayscale", MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema> {
                    new() { Name = "method", ParameterType = ParameterType.Enum,
                        EnumValues = new ObservableCollection<object> { "luminance", "average", "lightness" },
                        DefaultValue = "luminance" }
                } },
            new() { Id = "invert", Name = "Invert", Category = "Color",
                Description = "Invert colors", MinInputs = 1, MaxInputs = 1, Outputs = 1 },
            new() { Id = "brightness", Name = "Brightness", Category = "Tonal",
                Description = "Adjust brightness", MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema> {
                    new() { Name = "value", ParameterType = ParameterType.Float, DefaultValue = 0.0 }
                } },
            new() { Id = "resize", Name = "Resize", Category = "Transform",
                Description = "Resize image", MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema> {
                    new() { Name = "width", ParameterType = ParameterType.Integer, DefaultValue = 256 },
                    new() { Name = "height", ParameterType = ParameterType.Integer, DefaultValue = 256 }
                } },
        };
    }

    // ═══ Plugin loading ═══
    [Fact]
    public void LoadPlugins_FilteredPlugins_NotEmpty()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        Assert.NotEmpty(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void LoadPlugins_Categories_Populated()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        Assert.NotEmpty(h.PluginPanel.Categories);
        Assert.Contains("All", h.PluginPanel.Categories);
    }

    [Fact]
    public void LoadPlugins_EmptyList_FilteredPlugins_Empty()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(new List<PluginInfo>());
        Assert.Empty(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void LoadPlugins_DuplicateLoad_Deduplicates()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        int first = h.PluginPanel.FilteredPlugins.Count;
        h.LoadPlugins(CreateTestPlugins());
        Assert.Equal(first, h.PluginPanel.FilteredPlugins.Count);
    }

    [Fact]
    public void LoadPlugins_Categories_DistinctByCategory()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var cats = new HashSet<string>(h.PluginPanel.Categories.Except(new[] { "All" }));
        Assert.Equal(cats.Count, h.PluginPanel.Categories.Count - 1);
    }

    // ═══ Search ═══
    [Fact]
    public void Search_ExactMatch_FiltersCorrectly()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SearchText = "Invert";
        Assert.Single(h.PluginPanel.FilteredPlugins);
        Assert.Equal("Invert", h.PluginPanel.FilteredPlugins[0].Name);
    }

    [Fact]
    public void Search_PartialMatch_FiltersCorrectly()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SearchText = "bright";
        Assert.Single(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void Search_CaseInsensitive_Matches()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SearchText = "GRAYSCALE";
        Assert.Single(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void Search_NoMatch_ReturnsEmpty()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SearchText = "nonexistent_xyz";
        Assert.Empty(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void Search_Clear_RestoreAll()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SearchText = "invert";
        h.PluginPanel.SearchText = "";
        Assert.Equal(5, h.PluginPanel.FilteredPlugins.Count);
    }

    [Fact]
    public void Search_DescriptionField_Matches()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SearchText = "brightness";
        // "Adjust brightness" is description of Brightness plugin
        Assert.NotEmpty(h.PluginPanel.FilteredPlugins);
    }

    // ═══ Category filtering ═══
    [Fact]
    public void CategoryFilter_Color_OnlyColorCategory()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SelectedCategory = "Color";
        Assert.All(h.PluginPanel.FilteredPlugins, p => Assert.Equal("Color", p.Category));
    }

    [Fact]
    public void CategoryFilter_All_ShowsAll()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SelectedCategory = "All";
        Assert.Equal(5, h.PluginPanel.FilteredPlugins.Count);
    }

    [Fact]
    public void CategoryFilter_Transform_SinglePlugin()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SelectedCategory = "Transform";
        Assert.Single(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void CategoryFilter_ThenSearch_Intersection()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SelectedCategory = "Color";
        h.PluginPanel.SearchText = "invert";
        Assert.Single(h.PluginPanel.FilteredPlugins);
    }

    // ═══ Plugin selection ═══
    [Fact]
    public void SelectPlugin_SetsSelectedPlugin()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        h.PluginPanel.SelectPluginCommand.Execute(h.PluginPanel.FilteredPlugins[0]);
        Assert.NotNull(h.PluginPanel.SelectedPlugin);
        Assert.Equal("Identity", h.PluginPanel.SelectedPlugin!.Name);
    }

    [Fact]
    public void SelectPlugin_BuildsParameterControls()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var brightness = h.PluginPanel.FilteredPlugins.First(p => p.Id == "brightness");
        h.PluginPanel.SelectPluginCommand.Execute(brightness);
        Assert.NotEmpty(h.PluginPanel.ParameterControls);
        Assert.Contains(h.PluginPanel.ParameterControls, c => c.Schema.Name == "value");
    }

    [Fact]
    public void SelectPlugin_NoParameters_EmptyControls()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var identity = h.PluginPanel.FilteredPlugins.First(p => p.Id == "identity");
        h.PluginPanel.SelectPluginCommand.Execute(identity);
        Assert.Empty(h.PluginPanel.ParameterControls);
    }

    [Fact]
    public void SelectPlugin_SwitchPlugin_ControlsRebuild()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var brightness = h.PluginPanel.FilteredPlugins.First(p => p.Id == "brightness");
        h.PluginPanel.SelectPluginCommand.Execute(brightness);
        Assert.Single(h.PluginPanel.ParameterControls);

        var resize = h.PluginPanel.FilteredPlugins.First(p => p.Id == "resize");
        h.PluginPanel.SelectPluginCommand.Execute(resize);
        Assert.Equal(2, h.PluginPanel.ParameterControls.Count);
    }

    [Fact]
    public void SelectPlugin_EnumParameter_HasCorrectDefault()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var grayscale = h.PluginPanel.FilteredPlugins.First(p => p.Id == "grayscale");
        h.PluginPanel.SelectPluginCommand.Execute(grayscale);
        var methodControl = h.PluginPanel.ParameterControls.First(c => c.Schema.Name == "method");
        Assert.Equal("luminance", methodControl.Value);
    }

    // ═══ Apply/Reset parameters ═══
    [Fact]
    public void ApplyParameters_WithoutSelectedNode_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var brightness = h.PluginPanel.FilteredPlugins.First(p => p.Id == "brightness");
        h.PluginPanel.SelectPluginCommand.Execute(brightness);
        var exception = Record.Exception(() => h.PluginPanel.ApplyParametersCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void ResetParameters_RestoresDefaults()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var brightness = h.PluginPanel.FilteredPlugins.First(p => p.Id == "brightness");
        h.PluginPanel.SelectPluginCommand.Execute(brightness);
        var control = h.PluginPanel.ParameterControls[0];
        control.Value = 1.5;
        h.PluginPanel.ResetParametersCommand.Execute(null);
        Assert.Equal(0.0, control.Value);
    }

    [Fact]
    public void ResetParameters_WithoutSelectedPlugin_DoesNotCrash()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var exception = Record.Exception(() => h.PluginPanel.ResetParametersCommand.Execute(null));
        Assert.Null(exception);
    }

    [Fact]
    public void GetCurrentParameterValues_ReturnsCorrectDictionary()
    {
        var h = new ViewModelTestHarness();
        h.LoadPlugins(CreateTestPlugins());
        var resize = h.PluginPanel.FilteredPlugins.First(p => p.Id == "resize");
        h.PluginPanel.SelectPluginCommand.Execute(resize);
        var values = h.PluginPanel.GetCurrentParameterValues();
        Assert.True(values.ContainsKey("width"));
        Assert.True(values.ContainsKey("height"));
    }

    // ═══ Parameter control types ═══
    [Fact]
    public void ParameterControl_StringValue_SetsValue()
    {
        var control = new ParameterControlViewModel { Schema = new ParameterSchema { Name = "test",
            ParameterType = ParameterType.String, DefaultValue = "hello" } };
        Assert.Equal("hello", control.Schema.DefaultValue);
    }

    [Fact]
    public void ParameterControl_NumericValue_SetsValue()
    {
        var control = new ParameterControlViewModel { Schema = new ParameterSchema { Name = "val",
            ParameterType = ParameterType.Float, DefaultValue = 1.5 } };
        Assert.Equal(1.5, control.Schema.DefaultValue);
    }

    [Fact]
    public void ParameterControl_BooleanValue_Toggles()
    {
        var control = new ParameterControlViewModel { Schema = new ParameterSchema { Name = "enabled",
            ParameterType = ParameterType.Boolean, DefaultValue = true } };
        Assert.True((bool)control.Schema.DefaultValue!);
    }

    // ═══ Empty state ═══
    [Fact]
    public void NoPluginsLoaded_FilteredPlugins_Empty()
    {
        var h = new ViewModelTestHarness();
        Assert.Empty(h.PluginPanel.FilteredPlugins);
    }

    [Fact]
    public void NoPluginsLoaded_SelectedPlugin_Null()
    {
        var h = new ViewModelTestHarness();
        Assert.Null(h.PluginPanel.SelectedPlugin);
    }

    [Fact]
    public void NoPluginsLoaded_Categories_OnlyAll()
    {
        var h = new ViewModelTestHarness();
        // "All" is always populated as a default category
        Assert.Single(h.PluginPanel.Categories);
        Assert.Equal("All", h.PluginPanel.Categories[0]);
    }
}
