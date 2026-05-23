using System.Collections.ObjectModel;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class PluginPanelViewModelTests
{
    [Fact]
    public void PluginPanel_Creation_EmptyState()
    {
        var vm = new PluginPanelViewModel();

        vm.SelectedPlugin.Should().BeNull();
        vm.ParameterControls.Should().BeEmpty();
        vm.SearchText.Should().BeEmpty();
        vm.FilteredPlugins.Should().BeEmpty();
        vm.SelectedCategory.Should().Be("All");
        vm.Categories.Should().BeEmpty();
    }

    [Fact]
    public void LoadPlugins_PopulatesCategories()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Plugin A", Category = "Color" },
            new() { Id = "b", Name = "Plugin B", Category = "Color" },
            new() { Id = "c", Name = "Plugin C", Category = "Metadata" },
        };

        vm.LoadPlugins(plugins);

        vm.Categories.Should().Contain("All");
        vm.Categories.Should().Contain("Color");
        vm.Categories.Should().Contain("Metadata");
        vm.Categories.Should().HaveCount(3);
    }

    [Fact]
    public void LoadPlugins_FilteredPluginsShowsAllByDefault()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Plugin A", Category = "Color" },
            new() { Id = "b", Name = "Plugin B", Category = "Tonal" },
        };

        vm.LoadPlugins(plugins);

        vm.FilteredPlugins.Should().HaveCount(2);
    }

    [Fact]
    public void SearchText_FiltersByName()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Exposure", Category = "Tonal" },
            new() { Id = "b", Name = "White Balance", Category = "Color" },
            new() { Id = "c", Name = "Sharpen", Category = "Detail" },
        };
        vm.LoadPlugins(plugins);

        vm.SearchText = "White";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Name.Should().Be("White Balance");
    }

    [Fact]
    public void SearchText_FiltersByDescription()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Noise Reducer", Description = "AI-based denoising", Category = "Noise" },
            new() { Id = "b", Name = "WB", Description = "White balance adjustment", Category = "Color" },
        };
        vm.LoadPlugins(plugins);

        vm.SearchText = "denoising";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Name.Should().Be("Noise Reducer");
    }

    [Fact]
    public void SearchText_CaseInsensitive()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "EXPOSURE", Category = "Tonal" },
        };
        vm.LoadPlugins(plugins);

        vm.SearchText = "exposure";

        vm.FilteredPlugins.Should().HaveCount(1);
    }

    [Fact]
    public void SelectedCategory_FiltersPlugins()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Demosaic", Category = "Raw Processing" },
            new() { Id = "b", Name = "Exposure", Category = "Tonal" },
            new() { Id = "c", Name = "Denoise", Category = "Tonal" },
        };
        vm.LoadPlugins(plugins);

        vm.SelectedCategory = "Tonal";

        vm.FilteredPlugins.Should().HaveCount(2);
        vm.FilteredPlugins.All(p => p.Category == "Tonal").Should().BeTrue();
    }

    [Fact]
    public void SelectPlugin_BuildsParameterControls()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test",
            Name = "Test Plugin",
            Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "threshold", DisplayName = "Threshold",
                    ParameterType = ParameterType.Float, DefaultValue = 0.5
                },
                new ParameterSchema
                {
                    Name = "enabled", DisplayName = "Enabled",
                    ParameterType = ParameterType.Boolean, DefaultValue = true
                }
            }
        };

        vm.SelectPlugin(plugin);

        vm.SelectedPlugin.Should().Be(plugin);
        vm.ParameterControls.Should().HaveCount(2);
    }

    [Fact]
    public void SelectPlugin_ParameterControlInheritsSchemaDefaultValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test",
            Name = "Test",
            Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "ev", DisplayName = "Exposure",
                    ParameterType = ParameterType.Float, DefaultValue = 1.5
                }
            }
        };

        vm.SelectPlugin(plugin);

        vm.ParameterControls[0].Value.Should().Be(1.5);
        vm.ParameterControls[0].Schema.Name.Should().Be("ev");
    }

    [Fact]
    public void StringParameter_HasStringValueProperty()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "title", DisplayName = "Title",
                    ParameterType = ParameterType.String, DefaultValue = "Hello"
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.StringValue = "New Text";
        control.Value.Should().Be("New Text");
    }

    [Fact]
    public void IntegerParameter_UsesNumericValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "border", DisplayName = "Border",
                    ParameterType = ParameterType.Integer,
                    DefaultValue = 3, MinValue = 0, MaxValue = 8,
                    Step = 1
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.NumericValue = 5;
        control.Value.Should().Be(5.0);

        control.Schema.MinValue.Should().Be(0);
        control.Schema.MaxValue.Should().Be(8);
        control.Schema.Step.Should().Be(1.0);
    }

    [Fact]
    public void FloatParameter_UsesNumericValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "ev", DisplayName = "Exposure",
                    ParameterType = ParameterType.Float,
                    DefaultValue = 0.0, MinValue = -5.0, MaxValue = 5.0,
                    Step = 0.01
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.NumericValue = 2.5;
        control.Value.Should().Be(2.5);
    }

    [Fact]
    public void BooleanParameter_UsesBoolValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "auto_wb", DisplayName = "Auto WB",
                    ParameterType = ParameterType.Boolean, DefaultValue = false
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.BoolValue = true;
        control.Value.Should().Be(true);
    }

    [Fact]
    public void EnumParameter_UsesSelectedEnumIndex()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "algorithm", DisplayName = "Algorithm",
                    ParameterType = ParameterType.Enum,
                    EnumValues = new ObservableCollection<object> { "AMaZE", "LMMSE", "VNG4" },
                    DefaultValue = "LMMSE"
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.SelectedEnumIndex = 2;
        control.Value.Should().Be("VNG4");
    }

    [Fact]
    public void EnumParameter_InvalidIndex_DoesNotChangeValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "algorithm", DisplayName = "Algorithm",
                    ParameterType = ParameterType.Enum,
                    EnumValues = new ObservableCollection<object> { "AMaZE", "LMMSE" },
                    DefaultValue = "AMaZE"
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.SelectedEnumIndex = -1;
        control.Value.Should().Be("AMaZE");

        control.SelectedEnumIndex = 5;
        control.Value.Should().Be("AMaZE");
    }

    [Fact]
    public void FilePathParameter_UsesStringValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "preset_path", DisplayName = "Preset File",
                    ParameterType = ParameterType.FilePath, DefaultValue = ""
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.StringValue = @"C:\Presets\my_preset.json";
        control.Value.Should().Be(@"C:\Presets\my_preset.json");
    }

    [Fact]
    public void DirectoryPathParameter_UsesStringValue()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "output_dir", DisplayName = "Output Directory",
                    ParameterType = ParameterType.DirectoryPath, DefaultValue = ""
                }
            }
        };

        vm.SelectPlugin(plugin);

        var control = vm.ParameterControls[0];
        control.StringValue = @"C:\Output";
        control.Value.Should().Be(@"C:\Output");
    }

    [Fact]
    public void ResetParameters_RevertsToDefaults()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "strength", DisplayName = "Strength",
                    ParameterType = ParameterType.Float, DefaultValue = 0.5
                }
            }
        };
        vm.SelectPlugin(plugin);

        vm.ParameterControls[0].NumericValue = 0.9;

        vm.ResetParametersCommand.Execute(null);

        vm.ParameterControls[0].Value.Should().Be(0.5);
    }

    [Fact]
    public void GetCurrentParameterValues_ReturnsDictionary()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema
                {
                    Name = "strength", DisplayName = "Strength",
                    ParameterType = ParameterType.Float, DefaultValue = 0.5
                },
                new ParameterSchema
                {
                    Name = "enabled", DisplayName = "Enabled",
                    ParameterType = ParameterType.Boolean, DefaultValue = true
                }
            }
        };
        vm.SelectPlugin(plugin);

        var values = vm.GetCurrentParameterValues();

        values.Should().ContainKey("strength");
        values.Should().ContainKey("enabled");
    }

    [Fact]
    public void SelectPlugin_ReplacesPreviousControls()
    {
        var vm = new PluginPanelViewModel();
        var plugin1 = new PluginInfo
        {
            Id = "a", Name = "A", Category = "Test",
            ParameterSchemas = { new() { Name = "p1", ParameterType = ParameterType.String } }
        };
        var plugin2 = new PluginInfo
        {
            Id = "b", Name = "B", Category = "Test",
            ParameterSchemas =
            {
                new() { Name = "x", ParameterType = ParameterType.Float },
                new() { Name = "y", ParameterType = ParameterType.Float }
            }
        };

        vm.SelectPlugin(plugin1);
        vm.SelectPlugin(plugin2);

        vm.ParameterControls.Should().HaveCount(2);
        vm.ParameterControls[0].Schema.Name.Should().Be("x");
        vm.ParameterControls[1].Schema.Name.Should().Be("y");
    }

    [Fact]
    public void ParameterControl_IsValid_DefaultsTrue()
    {
        var control = new ParameterControlViewModel();

        control.IsValid.Should().BeTrue();
        control.ValidationMessage.Should().BeEmpty();
    }

    [Fact]
    public void ParameterControl_NonMatchingParameterType_StringValueDoesNotUpdateValue()
    {
        var control = new ParameterControlViewModel
        {
            Schema = new ParameterSchema
            {
                Name = "strength",
                ParameterType = ParameterType.Float,
                DefaultValue = 0.5
            },
            Value = 0.5
        };

        control.StringValue = "not a number";

        control.Value.Should().Be(0.5);
    }

    [Fact]
    public void ApplyParameters_Command_ExecutesWithoutException()
    {
        var vm = new PluginPanelViewModel();
        var plugin = new PluginInfo
        {
            Id = "test", Name = "Test", Category = "Test",
            ParameterSchemas =
            {
                new ParameterSchema { Name = "param1", ParameterType = ParameterType.String }
            }
        };
        vm.SelectPlugin(plugin);

        vm.ApplyParametersCommand.Execute(null);
    }

    [Fact]
    public void SelectedCategory_All_ShowsAllPlugins()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "A", Category = "X" },
            new() { Id = "b", Name = "B", Category = "Y" },
        };
        vm.LoadPlugins(plugins);

        vm.SelectedCategory = "X";
        vm.FilteredPlugins.Should().HaveCount(1);

        vm.SelectedCategory = "All";
        vm.FilteredPlugins.Should().HaveCount(2);
    }

    [Fact]
    public void SearchTextAndCategory_CombinedFilter()
    {
        var vm = new PluginPanelViewModel();
        var plugins = new ObservableCollection<PluginInfo>
        {
            new() { Id = "a", Name = "Demosaic", Category = "Raw Processing" },
            new() { Id = "b", Name = "Denoise", Category = "Noise Reduction" },
        };
        vm.LoadPlugins(plugins);

        vm.SelectedCategory = "Raw Processing";
        vm.SearchText = "Demosaic";

        vm.FilteredPlugins.Should().HaveCount(1);
        vm.FilteredPlugins[0].Name.Should().Be("Demosaic");
    }
}
