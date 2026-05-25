using Photopipeline.Tests.FunctionalTests.Infrastructure;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public static class TestCaseCatalog
{
    private static List<TestCaseDefinition>? _all;

    public static IReadOnlyList<TestCaseDefinition> All => _all ??= BuildAll();

    public static IEnumerable<TestCaseDefinition> GetByCategory(string category) =>
        All.Where(t => t.Category == category);

    public static IEnumerable<TestCaseDefinition> GetByTag(string tag) =>
        All.Where(t => t.Tags.Contains(tag));

    private static List<TestCaseDefinition> BuildAll()
    {
        var cases = new List<TestCaseDefinition>();
        cases.AddRange(BuildPluginTests());
        cases.AddRange(BuildFormatTests());
        cases.AddRange(BuildPipelineTopologyTests());
        cases.AddRange(BuildBatchTests());
        cases.AddRange(BuildErrorPathTests());
        cases.AddRange(BuildMetadataTests());
        cases.AddRange(BuildRegressionTests());
        cases.AddRange(BuildContentTests());
        cases.AddRange(BuildInteractionTests());

        for (int i = 0; i < cases.Count; i++)
        {
            if (string.IsNullOrEmpty(cases[i].Id))
                cases[i] = cases[i] with { Id = $"{cases[i].Category}_{i:D4}" };
        }

        return cases;
    }

    // ── Plugin parameter permutation tests ──

    private static IEnumerable<TestCaseDefinition> BuildPluginTests()
    {
        var pluginParamMatrix = new (string PluginId, string ParamKey, object[] Values, string[] InputImages)[]
        {
            ("exposure", "ev", new object[] { -2.0, -1.0, 0.0, 1.0, 2.0 },
                new[] { "pure_white_small", "mid_gray_128_small", "color_bars_8bit" }),
            ("contrast", "amount", new object[] { 0.0, 0.5, 1.0, 1.5, 2.0 },
                new[] { "gradient_horiz_rgb", "color_bars_8bit", "solid_blue_small" }),
            ("saturation", "amount", new object[] { 0.0, 0.5, 1.0, 1.5, 2.0 },
                new[] { "color_bars_8bit", "pure_red_small", "gradient_diag_rgb" }),
            ("brightness", "amount", new object[] { -0.5, -0.1, 0.0, 0.1, 0.5 },
                new[] { "mid_gray_128_small", "dark_gray_32_small", "pure_white_small" }),
            ("white_balance", "temperature", new object[] { 2000, 4000, 6500, 10000 },
                new[] { "pure_white_small", "gradient_horiz_rgb", "color_bars_8bit" }),
            ("denoise", "strength", new object[] { 0.0, 0.25, 0.5, 0.75, 1.0 },
                new[] { "noise_grain", "gradient_vert_rgb", "checkerboard_2x2" }),
            ("sharpen", "amount", new object[] { 0.0, 0.5, 1.0, 1.5, 2.0 },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8", "natural_noise_marble" }),
            ("crop", "width", new object[] { 100, 200, 400 },
                new[] { "solid_large_white", "color_bars_8bit", "gradient_horiz_rgb" }),
            ("resize", "width", new object[] { 64, 128, 256, 512, 1024 },
                new[] { "gradient_horiz_rgb", "checkerboard_16x16", "color_bars_8bit" }),
            ("rotate", "degrees", new object[] { 0, 90, 180, 270, 45 },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8", "solid_red_small" }),
            ("flip", "mode", new object[] { "horizontal", "vertical", "both" },
                new[] { "gradient_horiz_rgb", "checkerboard_4x4", "color_bars_8bit" }),
            ("color_space", "target", new object[] { "sRGB", "AdobeRGB", "ProPhoto" },
                new[] { "color_bars_8bit", "pure_red_small", "gradient_diag_rgb" }),
            ("gamma", "value", new object[] { 0.5, 1.0, 2.2 },
                new[] { "mid_gray_128_small", "gradient_horiz_gray", "color_bars_8bit" }),
            ("auto_levels", "enabled", new object[] { false, true },
                new[] { "dark_gray_32_small", "gradient_vert_rgb", "pure_white_small" }),
        };

        foreach (var (pluginId, paramKey, values, images) in pluginParamMatrix)
        {
            foreach (var image in images)
            {
                foreach (var value in values)
                {
                    yield return new TestCaseDefinition
                    {
                        Name = $"{pluginId}_{paramKey}_{value}_{image}",
                        Category = "plugin",
                        Tags = new[] { pluginId, "single_plugin", "parameter_permutation" },
                        InputImage = image,
                        Pipeline = TestPipelineBuilder.SingleNode(pluginId,
                            p => p[paramKey] = value),
                        TolerancePerChannel = 0,
                    };
                }
            }

            // Default parameters — two images
            yield return new TestCaseDefinition
            {
                Name = $"{pluginId}_default_all",
                Category = "plugin",
                Tags = new[] { pluginId, "single_plugin", "default_params", "regression" },
                InputImage = images[0],
                Pipeline = TestPipelineBuilder.SingleNode(pluginId),
                TolerancePerChannel = 0,
            };
        }
    }

    // ── Format cross tests ──

    private static IEnumerable<TestCaseDefinition> BuildFormatTests()
    {
        var inputFormats = new[] { "format_png", "format_jpeg", "format_tiff", "format_webp", "format_bmp" };
        var outputFormats = new[] { "TIFF", "PNG", "JPEG", "WEBP", "BMP" };
        var bitDepths = new uint?[] { 8, 16, null };

        foreach (var input in inputFormats)
        {
            foreach (var output in outputFormats)
            {
                foreach (var bd in bitDepths)
                {
                    bool isLossless = output is "TIFF" or "PNG" or "BMP";
                    double? minSSIM = output == "JPEG" ? 0.95 : 0.999;

                    yield return new TestCaseDefinition
                    {
                        Name = $"{input}_to_{output}_{bd ?? 0}bit",
                        Category = "format",
                        Tags = new[] { "format_conversion", output.ToLowerInvariant(),
                                       bd == 8 ? "8bit" : bd == 16 ? "16bit" : "auto" },
                        InputImage = input,
                        OutputFormat = output,
                        OutputBitDepth = bd,
                        OutputLossless = isLossless,
                        MinSSIM = minSSIM,
                    };
                }
            }
        }

        // High bit-depth format tests
        foreach (var output in new[] { "TIFF", "PNG" })
        {
            yield return new TestCaseDefinition
            {
                Name = $"16bit_gradient_to_{output.ToLowerInvariant()}",
                Category = "format",
                Tags = new[] { "format_conversion", "16bit", output.ToLowerInvariant() },
                InputImage = "gradient_horiz_rgb_16bit",
                OutputFormat = output,
                OutputBitDepth = 16,
                OutputLossless = true,
                TolerancePerChannel = 0,
            };
        }
    }

    // ── Pipeline topology tests ──

    private static IEnumerable<TestCaseDefinition> BuildPipelineTopologyTests()
    {
        // Linear chains: 2, 3, 4, 5 nodes
        foreach (int length in new[] { 2, 3, 4, 5 })
        {
            var plugins = new[] { "exposure", "contrast", "saturation", "brightness", "sharpen" };
            yield return new TestCaseDefinition
            {
                Name = $"linear_{length}nodes",
                Category = "pipeline",
                Tags = new[] { "topology", "linear", $"nodes_{length}" },
                InputImage = "color_bars_8bit",
                Pipeline = TestPipelineBuilder.Linear(plugins.Take(length).ToArray()),
                TolerancePerChannel = 0,
            };
        }

        // Disabled node in middle of chain
        yield return new TestCaseDefinition
        {
            Name = "linear_disabled_mid",
            Category = "pipeline",
            Tags = new[] { "topology", "disabled_node", "linear" },
            InputImage = "gradient_horiz_rgb",
            Pipeline = new TestPipelineBuilder()
                .AddNode("exposure")
                .AddNode("contrast", enabled: false)
                .AddNode("saturation")
                .ConnectLinear()
                .Build(),
            TolerancePerChannel = 0,
        };

        // All nodes disabled
        yield return new TestCaseDefinition
        {
            Name = "all_disabled",
            Category = "pipeline",
            Tags = new[] { "topology", "all_disabled" },
            InputImage = "gradient_horiz_rgb",
            Pipeline = new TestPipelineBuilder()
                .AddNode("exposure", enabled: false)
                .AddNode("contrast", enabled: false)
                .AddNode("saturation", enabled: false)
                .ConnectLinear()
                .Build(),
            TolerancePerChannel = 0,
        };

        // Single node tests for regression
        var allSinglePlugins = new[] { "exposure", "contrast", "saturation", "brightness",
            "denoise", "sharpen", "crop", "resize", "rotate", "flip", "gamma", "auto_levels",
            "white_balance", "color_space" };

        foreach (var plugin in allSinglePlugins)
        {
            yield return new TestCaseDefinition
            {
                Name = $"single_{plugin}",
                Category = "pipeline",
                Tags = new[] { "topology", "single_node", "regression", plugin },
                InputImage = "gradient_horiz_rgb",
                Pipeline = TestPipelineBuilder.SingleNode(plugin),
                TolerancePerChannel = 0,
            };
        }
    }

    // ── Batch processing tests ──

    private static IEnumerable<TestCaseDefinition> BuildBatchTests()
    {
        for (int n = 1; n <= 5; n++)
        {
            yield return new TestCaseDefinition
            {
                Name = $"batch_{n}files",
                Category = "batch",
                Tags = new[] { "batch", $"files_{n}" },
                InputImages = Enumerable.Range(0, n).Select(_ => "color_bars_8bit").ToArray(),
                Pipeline = TestPipelineBuilder.SingleNode("exposure",
                    p => p["ev"] = 0.5),
                TolerancePerChannel = 0,
            };
        }

        yield return new TestCaseDefinition
        {
            Name = "batch_large_10files",
            Category = "batch",
            Tags = new[] { "batch", "files_10", "is_serial_only" },
            InputImages = Enumerable.Range(0, 10).Select(_ => "color_bars_8bit").ToArray(),
            Pipeline = TestPipelineBuilder.SingleNode("exposure"),
            IsSerialOnly = true,
        };
    }

    // ── Error path tests ──

    private static IEnumerable<TestCaseDefinition> BuildErrorPathTests()
    {
        // Invalid plugin ID
        yield return new TestCaseDefinition
        {
            Name = "error_invalid_plugin",
            Category = "error",
            Tags = new[] { "error_path", "invalid_plugin" },
            InputImage = "solid_red_small",
            Pipeline = new TestPipelineBuilder().AddNode("nonexistent_plugin_xyz").Build(),
            ExpectError = true,
            SkipUiChannel = true,
        };

        // Invalid parameter value
        yield return new TestCaseDefinition
        {
            Name = "error_invalid_param_value",
            Category = "error",
            Tags = new[] { "error_path", "invalid_param" },
            InputImage = "solid_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("crop",
                p => { p["width"] = -1; p["height"] = -1; }),
            ExpectError = true,
            SkipUiChannel = true,
        };

        // Malformed spec (no nodes)
        yield return new TestCaseDefinition
        {
            Name = "error_empty_pipeline",
            Category = "error",
            Tags = new[] { "error_path", "empty_pipeline" },
            InputImage = "solid_red_small",
            Pipeline = new PipelineSpec { Name = "Empty" },
            ExpectError = true,
        };
    }

    // ── Metadata tests ──

    private static IEnumerable<TestCaseDefinition> BuildMetadataTests()
    {
        var metadataPlugins = new[] { "exposure", "rotate", "resize", "crop", "flip" };
        foreach (var plugin in metadataPlugins)
        {
            yield return new TestCaseDefinition
            {
                Name = $"metadata_passthrough_{plugin}",
                Category = "metadata",
                Tags = new[] { "metadata", "passthrough", plugin },
                InputImage = "format_jpeg",
                Pipeline = TestPipelineBuilder.SingleNode(plugin),
                TolerancePerChannel = 0,
            };
        }
    }

    // ── Regression snapshot tests ──

    private static IEnumerable<TestCaseDefinition> BuildRegressionTests()
    {
        var inputImages = new[] { "gradient_horiz_rgb", "checkerboard_8x8", "color_bars_8bit",
                                   "grayscale_32steps", "solid_mid_gray_128_small" };
        var plugins = new[] { "exposure", "contrast", "saturation", "sharpen", "denoise",
                                "brightness", "gamma", "rotate" };

        foreach (var image in inputImages)
        {
            foreach (var plugin in plugins)
            {
                yield return new TestCaseDefinition
                {
                    Name = $"regression_{plugin}_{image}",
                    Category = "regression",
                    Tags = new[] { "regression", "golden", plugin },
                    InputImage = image,
                    Pipeline = TestPipelineBuilder.SingleNode(plugin),
                    TolerancePerChannel = 0,
                };
            }
        }
    }

    // ── Content verification tests ──

    private static IEnumerable<TestCaseDefinition> BuildContentTests()
    {
        var contentImages = new[] { "solid_white_large", "solid_black_large", "color_bars_8bit",
                                      "checkerboard_16x16", "grayscale_256steps", "zone_plate_16hz",
                                      "gradient_horiz_rgb", "gradient_radial_rgb" };

        foreach (var image in contentImages)
        {
            yield return new TestCaseDefinition
            {
                Name = $"content_identity_{image}",
                Category = "content",
                Tags = new[] { "content", "identity", "passthrough" },
                InputImage = image,
                OutputFormat = "TIFF",
                TolerancePerChannel = 0,
            };
        }
    }

    // ── UI interaction tests (UI only, no API equivalent) ──

    private static IEnumerable<TestCaseDefinition> BuildInteractionTests()
    {
        string[] interactions =
        {
            "zoom_in_pan_reset", "split_view_toggle", "drag_plugin_to_canvas",
            "multi_select_filmstrip", "filter_by_format", "navigate_pipeline_views",
            "theme_toggle", "settings_save_reset", "export_dialog_flow",
            "batch_add_remove_items"
        };

        foreach (var interaction in interactions)
        {
            yield return new TestCaseDefinition
            {
                Name = $"ui_interaction_{interaction}",
                Category = "interaction",
                Tags = new[] { "ui_only", "interaction", interaction },
                InputImage = "color_bars_8bit",
                SkipApiChannel = true,
            };
        }
    }
}
