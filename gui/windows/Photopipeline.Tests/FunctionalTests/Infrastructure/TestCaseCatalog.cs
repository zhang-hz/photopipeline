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

    // Companion params that must be set for the test parameter to produce
    // a visible pixel change. Without these, plugins fall through to no-op
    // defaults (e.g. resize_mode="none" ignores scale_percent).
    private static readonly Dictionary<string, Dictionary<string, object>> s_companionParams = new()
    {
        // transform: scale_percent is ignored unless resize_mode="percentage"
        ["transform_scale_percent"] = new() { ["resize_mode"] = "percentage" },
        // transform: crop_enabled alone does nothing; need dimensions smaller than input
        ["transform_crop_enabled"] = new() { ["resize_mode"] = "percentage", ["scale_percent"] = 100.0, ["crop_width"] = 50, ["crop_height"] = 50 },
        // transform: flip_horizontal=false is a no-op (legitimate), but true needs no companion
        // transform: angle=0 is a no-op (legitimate), other angles work alone
        // colorspace: source defaults to auto (detected as sRGB); need explicit source
        //   different from target for pixel-level changes to occur
        ["colorspace_target_color_space"] = new() { ["source_color_space"] = "srgb" },
        ["colorspace_rendering_intent"] = new() { ["source_color_space"] = "srgb", ["target_color_space"] = "display_p3" },
        ["colorspace_black_point_compensation"] = new() { ["source_color_space"] = "srgb", ["target_color_space"] = "display_p3" },
        // lut3d: intensity requires a LUT file — companion handled in PluginApiTests
        // lens_correct: needs real lens profile data; synthetic images have no distortion
        // ai_denoise: needs ONNX model files (infrastructure dependency)
    };

    // Parameter values that legitimately produce pixel-identical output.
    // The adversarial "PLUGIN HAD NO EFFECT" check must skip these.
    private static readonly HashSet<string> s_zeroEffectParams = new(StringComparer.OrdinalIgnoreCase)
    {
        "angle=0", "flip_horizontal=False", "flip_vertical=False",
        "intensity=0", "denoise_strength=0", "detail_preservation=0",
        "crop_enabled=False", "apply_white_balance=False",
        "correct_vignetting=False", "target_color_space=srgb",
        // colorspace: sRGB→sRGB is identity (source defaults to sRGB from auto-detect)
        // lut3d: no LUT file → intensity is irrelevant (plugin needs external .cube file)
        // lens_correct: synthetic test images have no lens distortion
    };

    // Plugins that need external data files (LUT, lens profile) to produce
    // pixel changes — adversarial check skipped entirely for these plugins.
    private static readonly HashSet<string> s_externalDataPlugins = new(StringComparer.OrdinalIgnoreCase)
    {
        "photopipeline.plugins.lut3d",
        "photopipeline.plugins.lens_correct",
        "photopipeline.plugins.ai_denoise",
    };

    private static IEnumerable<TestCaseDefinition> BuildPluginTests()
    {
        // (PluginId, ParamKey, Values, InputImages)
        var pluginParamMatrix = new (string PluginId, string ParamKey, object[] Values, string[] InputImages)[]
        {
            // ── transform: resize, rotate, flip, crop ──
            ("photopipeline.plugins.transform", "scale_percent", new object[] { 25.0, 50.0, 200.0 },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8", "color_bars_8bit" }),
            ("photopipeline.plugins.transform", "angle", new object[] { 0.0, 90.0, 180.0, 270.0 },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8", "pure_red_small" }),
            ("photopipeline.plugins.transform", "flip_horizontal", new object[] { false, true },
                new[] { "gradient_horiz_rgb", "checkerboard_4x4", "color_bars_8bit" }),
            ("photopipeline.plugins.transform", "crop_enabled", new object[] { true },
                new[] { "pure_white_large", "color_bars_8bit", "gradient_horiz_rgb" }),

            // ── colorspace: conversion, rendering ──
            ("photopipeline.plugins.colorspace", "target_color_space", new object[] { "srgb", "display_p3", "adobe_rgb" },
                new[] { "color_bars_8bit", "pure_red_small", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.colorspace", "rendering_intent", new object[] { "relative_colorimetric", "perceptual", "photopipeline.plugins.colorspace" },
                new[] { "gradient_horiz_rgb", "pure_red_small", "color_bars_8bit" }),
            ("photopipeline.plugins.colorspace", "black_point_compensation", new object[] { false, true },
                new[] { "color_bars_8bit", "gradient_diag_rgb", "pure_red_small" }),

            // ── ai_denoise: strength ──
            ("photopipeline.plugins.ai_denoise", "denoise_strength", new object[] { 0.0, 25.0, 50.0, 100.0 },
                new[] { "noise_grain", "gradient_vert_rgb", "checkerboard_8x8" }),
            ("photopipeline.plugins.ai_denoise", "detail_preservation", new object[] { 0.0, 50.0, 100.0 },
                new[] { "noise_grain", "gradient_vert_rgb", "checkerboard_8x8" }),
            ("photopipeline.plugins.ai_denoise", "denoise_model", new object[] { "lightweight_v1", "standard_v2" },
                new[] { "noise_grain", "checkerboard_8x8" }),

            // ── lut3d: intensity ──
            ("photopipeline.plugins.lut3d", "intensity", new object[] { 0.0, 50.0, 100.0 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb", "pure_red_small" }),

            // ── lens_correct: mode & correction flags ──
            ("photopipeline.plugins.lens_correct", "correction_mode", new object[] { "auto" },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8", "color_bars_8bit" }),
            ("photopipeline.plugins.lens_correct", "correct_vignetting", new object[] { false, true },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8" }),

            // ── raw_input: raw_mode ──
            ("photopipeline.plugins.raw_input", "raw_mode", new object[] { "auto", "dcraw" },
                new[] { "gradient_horiz_rgb", "color_bars_8bit" }),
            ("photopipeline.plugins.raw_input", "apply_white_balance", new object[] { false, true },
                new[] { "gradient_horiz_rgb", "color_bars_8bit" }),

            // ── Encoder quality ──
            ("photopipeline.plugins.png_encoder", "compression_level", new object[] { 0, 6, 9 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.png_encoder", "bit_depth", new object[] { "8", "16" },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8" }),
            ("photopipeline.plugins.tiff_encoder", "compression", new object[] { "none", "lzw", "deflate" },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.avif_encoder", "quality", new object[] { 30.0, 60.0, 90.0 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.jxl_encoder", "quality", new object[] { 30.0, 60.0, 90.0 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.heif_encoder", "quality", new object[] { 30.0, 60.0, 95.0 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
        };

        foreach (var (pluginId, paramKey, values, images) in pluginParamMatrix)
        {
            var shortName = pluginId.Replace("photopipeline.plugins.", "");
            foreach (var image in images)
            {
                foreach (var value in values)
                {
                    // Build the tag list. Tests that need external data or are
                    // zero-effect get an extra tag so PluginApiTests can skip
                    // the adversarial check.
                    var tags = new List<string> { pluginId, "single_plugin", "parameter_permutation" };

                    bool isZeroEffect = s_zeroEffectParams.Contains($"{paramKey}={value}");
                    bool needsExternalData = s_externalDataPlugins.Contains(pluginId);
                    if (isZeroEffect) tags.Add("zero_effect");
                    if (needsExternalData) tags.Add("external_data_plugin");

                    yield return new TestCaseDefinition
                    {
                        Name = $"{shortName}_{paramKey}_{value}_{image}",
                        Category = "plugin",
                        Tags = tags.ToArray(),
                        InputImage = image,
                        Pipeline = TestPipelineBuilder.SingleNode(pluginId, p =>
                        {
                            // Apply companion params first, then the test param
                            // on top so the test param always takes precedence.
                            var companionKey = $"{shortName}_{paramKey}";
                            if (s_companionParams.TryGetValue(companionKey, out var companions))
                            {
                                foreach (var (ck, cv) in companions)
                                    p[ck] = cv;
                            }
                            p[paramKey] = value;
                        }),
                        TolerancePerChannel = 0,
                    };
                }
            }

            // Default parameters
            yield return new TestCaseDefinition
            {
                Name = $"{shortName}_default_all",
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
            var plugins = new[] { "photopipeline.plugins.raw_input", "photopipeline.plugins.colorspace", "photopipeline.plugins.colorspace", "photopipeline.plugins.colorspace", "photopipeline.plugins.ai_denoise" };
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
                .AddNode("photopipeline.plugins.raw_input")
                .AddNode("photopipeline.plugins.colorspace", enabled: false)
                .AddNode("photopipeline.plugins.colorspace")
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
                .AddNode("photopipeline.plugins.raw_input", enabled: false)
                .AddNode("photopipeline.plugins.colorspace", enabled: false)
                .AddNode("photopipeline.plugins.colorspace", enabled: false)
                .ConnectLinear()
                .Build(),
            TolerancePerChannel = 0,
        };

        // Single node tests for regression
        var allSinglePlugins = new[] { "photopipeline.plugins.raw_input", "photopipeline.plugins.colorspace", "photopipeline.plugins.colorspace", "photopipeline.plugins.colorspace",
            "photopipeline.plugins.ai_denoise", "photopipeline.plugins.ai_denoise", "photopipeline.plugins.transform", "photopipeline.plugins.transform", "photopipeline.plugins.transform", "photopipeline.plugins.transform", "photopipeline.plugins.colorspace", "photopipeline.plugins.raw_input",
            "photopipeline.plugins.raw_input", "photopipeline.plugins.colorspace" };

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
                Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.raw_input",
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
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.raw_input"),
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
            InputImage = "pure_red_small",
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
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
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
            InputImage = "pure_red_small",
            Pipeline = new PipelineSpec { Name = "Empty" },
            ExpectError = true,
        };
    }

    // ── Metadata tests ──

    private static IEnumerable<TestCaseDefinition> BuildMetadataTests()
    {
        var metadataPlugins = new[] { "photopipeline.plugins.raw_input", "photopipeline.plugins.transform", "photopipeline.plugins.transform", "photopipeline.plugins.transform", "photopipeline.plugins.transform" };
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
                                   "grayscale_32steps", "mid_gray_128_small" };
        var plugins = new[] { "photopipeline.plugins.raw_input", "photopipeline.plugins.colorspace", "photopipeline.plugins.colorspace", "photopipeline.plugins.ai_denoise", "photopipeline.plugins.ai_denoise",
                                "photopipeline.plugins.colorspace", "photopipeline.plugins.colorspace", "photopipeline.plugins.transform" };

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
        var contentImages = new[] { "pure_white_large", "pure_black_large", "color_bars_8bit",
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
                OutputFormat = "PNG",
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
