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
        // transform: resize_mode=absolute/fit need width+height; percentage needs scale_percent
        ["transform_resize_mode"] = new() { ["scale_percent"] = 50.0, ["width"] = 128, ["height"] = 128 },
        // colorspace: source_color_space test needs explicit target different from source
        ["colorspace_source_color_space"] = new() { ["target_color_space"] = "linear" },
        // colorspace: rendering_intent absolute_colorimetric needs source/target for pixel change
        // (reuses existing companion key "colorspace_rendering_intent" already defined above)
        // exif_rw: write_tags needs a JPEG input (handled via InputImage in matrix)
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
        // transform: 360deg rotation is identity, -90 is not
        "angle=360",
        // colorspace: source=linear → target=linear is identity (companion sets target=linear)
        "source_color_space=linear",
        // exif_rw: write_tags=false is a no-op
        "write_tags=False",
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
            ("photopipeline.plugins.colorspace", "rendering_intent", new object[] { "relative_colorimetric", "perceptual", "saturation" },
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

            // ── Encoder-specific parameters ──
            ("photopipeline.plugins.avif_encoder", "speed", new object[] { 0, 5, 10 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.jxl_encoder", "effort", new object[] { 1, 5, 9 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.jxl_encoder", "distance", new object[] { 0.0, 1.5, 3.0 },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.heif_encoder", "encoder_type", new object[] { "hevc", "avc" },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),
            ("photopipeline.plugins.png_encoder", "alpha_channel", new object[] { true, false },
                new[] { "alpha_solid_transparent", "checkerboard_8x8" }),

            // ── transform: resize mode ──
            ("photopipeline.plugins.transform", "resize_mode", new object[] { "percentage", "absolute", "fit" },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8" }),

            // ── transform: additional rotation variants (-90, 360) ──
            ("photopipeline.plugins.transform", "angle", new object[] { -90.0, 360.0 },
                new[] { "gradient_horiz_rgb", "checkerboard_8x8" }),

            // ── colorspace: source_color_space variants ──
            ("photopipeline.plugins.colorspace", "source_color_space", new object[] { "linear", "srgb", "adobe_rgb", "display_p3" },
                new[] { "color_bars_8bit", "gradient_horiz_rgb", "pure_red_small" }),

            // ── colorspace: rendering_intent absolute_colorimetric ──
            ("photopipeline.plugins.colorspace", "rendering_intent", new object[] { "absolute_colorimetric" },
                new[] { "color_bars_8bit", "gradient_horiz_rgb" }),

            // ── exif_rw: write_tags ──
            ("photopipeline.plugins.exif_rw", "write_tags", new object[] { true, false },
                new[] { "format_jpeg", "color_bars_8bit" }),
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

        // ── WEBP lossless mode tests ──
        var webpLosslessInputs = new[] { "format_png", "format_tiff", "format_bmp" };
        foreach (var input in webpLosslessInputs)
        {
            foreach (var bd in new uint?[] { 8, null })
            {
                yield return new TestCaseDefinition
                {
                    Name = $"{input}_to_WEBP_lossless_{bd ?? 0}bit",
                    Category = "format",
                    Tags = new[] { "format_conversion", "webp", "lossless", bd == 8 ? "8bit" : "auto" },
                    InputImage = input,
                    OutputFormat = "WEBP",
                    OutputBitDepth = bd,
                    OutputLossless = true,
                    MinSSIM = 0.999,
                };
            }
        }

        // ── AVIF, HEIF, JXL output formats (next-gen encoders) ──
        var nextGenOutputs = new[] { "AVIF", "HEIF", "JXL" };
        foreach (var input in inputFormats)
        {
            foreach (var output in nextGenOutputs)
            {
                yield return new TestCaseDefinition
                {
                    Name = $"{input}_to_{output.ToLowerInvariant()}",
                    Category = "format",
                    Tags = new[] { "format_conversion", output.ToLowerInvariant(), "next_gen_format", "8bit" },
                    InputImage = input,
                    OutputFormat = output,
                    OutputBitDepth = 8,
                    MinSSIM = 0.90,
                };
            }
        }

        // ── Alpha channel format conversions ──
        var alphaInputs = new[] { "alpha_solid_transparent", "alpha_gradient_transparent", "alpha_full_opaque" };
        var alphaOutputs = new[] { "PNG", "TIFF", "WEBP" };
        foreach (var input in alphaInputs)
        {
            foreach (var output in alphaOutputs)
            {
                yield return new TestCaseDefinition
                {
                    Name = $"alpha_{input}_to_{output.ToLowerInvariant()}",
                    Category = "format",
                    Tags = new[] { "format_conversion", "alpha_channel", output.ToLowerInvariant() },
                    InputImage = input,
                    OutputFormat = output,
                    OutputBitDepth = 8,
                    MinSSIM = 0.95,
                };
            }
        }

        // ── Grayscale format conversions ──
        var grayInputs = new[] { "gradient_horiz_gray", "gradient_vert_gray", "grayscale_256steps", "grayscale_32steps" };
        foreach (var input in grayInputs)
        {
            foreach (var output in new[] { "PNG", "TIFF", "JPEG", "WEBP" })
            {
                bool isLossless = output is "TIFF" or "PNG";
                yield return new TestCaseDefinition
                {
                    Name = $"grayscale_{input}_to_{output.ToLowerInvariant()}",
                    Category = "format",
                    Tags = new[] { "format_conversion", "grayscale", output.ToLowerInvariant() },
                    InputImage = input,
                    OutputFormat = output,
                    OutputBitDepth = 8,
                    OutputLossless = isLossless,
                    MinSSIM = output is "JPEG" or "WEBP" ? 0.95 : 0.999,
                };
            }
        }
    }

    // ── Pipeline topology tests ──

    private static IEnumerable<TestCaseDefinition> BuildPipelineTopologyTests()
    {
        // Linear chains: 2, 3, 4, 5 nodes (each node uses a distinct plugin)
        var linearPlugins = new[] {
            "photopipeline.plugins.raw_input",
            "photopipeline.plugins.colorspace",
            "photopipeline.plugins.transform",
            "photopipeline.plugins.ai_denoise",
            "photopipeline.plugins.lut3d"
        };
        foreach (int length in new[] { 2, 3, 4, 5 })
        {
            yield return new TestCaseDefinition
            {
                Name = $"linear_{length}nodes",
                Category = "pipeline",
                Tags = new[] { "topology", "linear", $"nodes_{length}" },
                InputImage = "color_bars_8bit",
                Pipeline = TestPipelineBuilder.Linear(linearPlugins.Take(length).ToArray()),
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
                .AddNode("photopipeline.plugins.transform", enabled: false)
                .ConnectLinear()
                .Build(),
            TolerancePerChannel = 0,
        };

        // Single node tests (deduplicated — one entry per unique plugin)
        var allSinglePlugins = new[] {
            "photopipeline.plugins.raw_input",
            "photopipeline.plugins.colorspace",
            "photopipeline.plugins.ai_denoise",
            "photopipeline.plugins.transform",
            "photopipeline.plugins.lut3d",
            "photopipeline.plugins.lens_correct",
            "photopipeline.plugins.exif_rw",
            "photopipeline.plugins.png_encoder",
            "photopipeline.plugins.tiff_encoder",
        };

        foreach (var plugin in allSinglePlugins)
        {
            yield return new TestCaseDefinition
            {
                Name = $"single_{plugin.Replace("photopipeline.plugins.", "")}",
                Category = "pipeline",
                Tags = new[] { "topology", "single_node", "regression", plugin },
                InputImage = "gradient_horiz_rgb",
                Pipeline = TestPipelineBuilder.SingleNode(plugin),
                TolerancePerChannel = 0,
            };
        }

        // ── Fork topology: A→B, A→C (one source, two targets) ──
        yield return new TestCaseDefinition
        {
            Name = "fork_topology",
            Category = "pipeline",
            Tags = new[] { "topology", "fork", "branching" },
            InputImage = "color_bars_8bit",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.raw_input")      // A
                .AddNode("photopipeline.plugins.colorspace")     // B
                .AddNode("photopipeline.plugins.transform")      // C
                .Connect(0, 1)  // A→B
                .Connect(0, 2)  // A→C
                .Build(),
            TolerancePerChannel = 0,
        };

        // ── Diamond topology: A→B, A→C, B→D, C→D ──
        yield return new TestCaseDefinition
        {
            Name = "diamond_topology",
            Category = "pipeline",
            Tags = new[] { "topology", "diamond", "branching" },
            InputImage = "gradient_horiz_rgb",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.raw_input")      // A
                .AddNode("photopipeline.plugins.colorspace")     // B
                .AddNode("photopipeline.plugins.transform")      // C
                .AddNode("photopipeline.plugins.colorspace")     // D
                .Connect(0, 1)  // A→B
                .Connect(0, 2)  // A→C
                .Connect(1, 3)  // B→D
                .Connect(2, 3)  // C→D
                .Build(),
            TolerancePerChannel = 0,
        };

        // ── Branch with disabled node in one branch ──
        yield return new TestCaseDefinition
        {
            Name = "branch_disabled_one_leg",
            Category = "pipeline",
            Tags = new[] { "topology", "fork", "disabled_node", "branching" },
            InputImage = "checkerboard_8x8",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.raw_input")                  // A
                .AddNode("photopipeline.plugins.colorspace", enabled: false) // B (disabled)
                .AddNode("photopipeline.plugins.transform")                  // C (active)
                .Connect(0, 1)  // A→B (disabled leg)
                .Connect(0, 2)  // A→C (active leg)
                .Build(),
            TolerancePerChannel = 0,
        };

        // ── Single node full pipeline for each encoder type ──
        var encoderPlugins = new[]
        {
            "photopipeline.plugins.png_encoder",
            "photopipeline.plugins.jpeg_encoder",
            "photopipeline.plugins.tiff_encoder",
            "photopipeline.plugins.webp_encoder",
            "photopipeline.plugins.avif_encoder",
            "photopipeline.plugins.jxl_encoder",
            "photopipeline.plugins.heif_encoder",
            "photopipeline.plugins.bmp_encoder",
        };
        foreach (var encoder in encoderPlugins)
        {
            yield return new TestCaseDefinition
            {
                Name = $"single_encoder_{encoder.Replace("photopipeline.plugins.", "")}",
                Category = "pipeline",
                Tags = new[] { "topology", "single_node", "encoder", encoder },
                InputImage = "gradient_horiz_rgb",
                Pipeline = TestPipelineBuilder.SingleNode(encoder),
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

        // ── Batch pause/resume lifecycle ──
        yield return new TestCaseDefinition
        {
            Name = "batch_pause_resume",
            Category = "batch",
            Tags = new[] { "batch", "pause_resume", "lifecycle", "is_serial_only" },
            InputImages = new[] { "color_bars_8bit", "gradient_horiz_rgb", "checkerboard_8x8", "pure_red_small", "noise_grain" },
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.colorspace",
                p => { p["target_color_space"] = "display_p3"; p["source_color_space"] = "srgb"; }),
            IsSerialOnly = true,
            SkipUiChannel = true,
        };

        // ── Batch cancel lifecycle ──
        yield return new TestCaseDefinition
        {
            Name = "batch_cancel",
            Category = "batch",
            Tags = new[] { "batch", "cancel", "lifecycle", "is_serial_only" },
            InputImages = Enumerable.Range(0, 8).Select(_ => "pure_white_large").ToArray(),
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.ai_denoise",
                p => p["denoise_strength"] = 100.0),
            IsSerialOnly = true,
            SkipUiChannel = true,
        };

        // ── Batch partial failure (mixed valid + invalid files) ──
        yield return new TestCaseDefinition
        {
            Name = "batch_partial_failure",
            Category = "batch",
            Tags = new[] { "batch", "partial_failure", "mixed_input", "is_serial_only" },
            InputImages = new[] { "color_bars_8bit", "nonexistent_file_xyz", "gradient_horiz_rgb", "pure_red_small" },
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            IsSerialOnly = true,
            SkipUiChannel = true,
        };
    }

    // ── Error path tests ──

    private static IEnumerable<TestCaseDefinition> BuildErrorPathTests()
    {
        // 1. Invalid plugin ID
        yield return new TestCaseDefinition
        {
            Name = "error_invalid_plugin",
            Category = "error", Tags = new[] { "error_path", "invalid_plugin" },
            InputImage = "pure_red_small",
            Pipeline = new TestPipelineBuilder().AddNode("nonexistent_plugin_xyz").Build(),
            ExpectError = true, SkipUiChannel = true,
        };

        // 2. Invalid parameter value (negative dimensions)
        yield return new TestCaseDefinition
        {
            Name = "error_invalid_param_value",
            Category = "error", Tags = new[] { "error_path", "invalid_param" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["width"] = -1; p["height"] = -1; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 3. Empty pipeline (no nodes)
        yield return new TestCaseDefinition
        {
            Name = "error_empty_pipeline",
            Category = "error", Tags = new[] { "error_path", "empty_pipeline" },
            InputImage = "pure_red_small",
            Pipeline = new PipelineSpec { Name = "Empty" },
            ExpectError = true, SkipUiChannel = true,
        };

        // 4. Input file does not exist
        yield return new TestCaseDefinition
        {
            Name = "error_missing_input_file",
            Category = "error", Tags = new[] { "error_path", "missing_file" },
            InputImage = "nonexistent_image_file_xyz",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            ExpectError = true, SkipUiChannel = true,
        };

        // 5. Parameter type mismatch (string where int expected)
        yield return new TestCaseDefinition
        {
            Name = "error_param_type_mismatch",
            Category = "error", Tags = new[] { "error_path", "type_mismatch" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["scale_percent"] = "not_a_number"; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 6. Parameter value out of range (above max)
        yield return new TestCaseDefinition
        {
            Name = "error_param_above_maximum",
            Category = "error", Tags = new[] { "error_path", "out_of_range" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["scale_percent"] = 999999; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 7. All nodes disabled
        yield return new TestCaseDefinition
        {
            Name = "error_all_nodes_disabled",
            Category = "error", Tags = new[] { "error_path", "disabled" },
            InputImage = "pure_red_small",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.transform", null, enabled: false)
                .AddNode("photopipeline.plugins.colorspace", null, enabled: false)
                .Connect(0, 1)
                .Build(),
            ExpectError = true, SkipUiChannel = true,
        };

        // 8. Self-loop edge
        yield return new TestCaseDefinition
        {
            Name = "error_self_loop",
            Category = "error", Tags = new[] { "error_path", "cycle" },
            InputImage = "pure_red_small",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.transform")
                .Connect(0, 0)
                .Build(),
            ExpectError = true, SkipUiChannel = true,
        };

        // 9. Edge to non-existent node
        yield return new TestCaseDefinition
        {
            Name = "error_edge_to_nonexistent",
            Category = "error", Tags = new[] { "error_path", "invalid_edge" },
            InputImage = "pure_red_small",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.transform")
                .Connect(0, 5)
                .Build(),
            ExpectError = true, SkipUiChannel = true,
        };

        // 10. Duplicate node IDs
        yield return new TestCaseDefinition
        {
            Name = "error_duplicate_nodes",
            Category = "error", Tags = new[] { "error_path", "duplicate" },
            InputImage = "pure_red_small",
            Pipeline = new PipelineSpec
            {
                Name = "DuplicateNodes",
                Nodes = new List<PipelineNode>
                {
                    new() { Id = "n1", PluginId = "photopipeline.plugins.transform" },
                    new() { Id = "n1", PluginId = "photopipeline.plugins.colorspace" },
                }
            },
            ExpectError = true, SkipUiChannel = true,
        };

        // 11. Unconnected sub-graph
        yield return new TestCaseDefinition
        {
            Name = "error_disconnected_graph",
            Category = "error", Tags = new[] { "error_path", "disconnected" },
            InputImage = "pure_red_small",
            Pipeline = new TestPipelineBuilder()
                .AddNode("photopipeline.plugins.transform")
                .AddNode("photopipeline.plugins.colorspace")
                .Build(),
            ExpectError = true, SkipUiChannel = true,
        };

        // 12. Zero-size parameter (scale_percent=0)
        yield return new TestCaseDefinition
        {
            Name = "error_zero_scale",
            Category = "error", Tags = new[] { "error_path", "zero_value" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["scale_percent"] = 0; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 13. Unknown parameter key (different from invalid plugin)
        yield return new TestCaseDefinition
        {
            Name = "error_unknown_param_key",
            Category = "error", Tags = new[] { "error_path", "unknown_param" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["this_param_does_not_exist"] = 42; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 14. Null parameter value
        yield return new TestCaseDefinition
        {
            Name = "error_null_param",
            Category = "error", Tags = new[] { "error_path", "null_param" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["scale_percent"] = null!; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 15. Invalid output format
        yield return new TestCaseDefinition
        {
            Name = "error_invalid_output_format",
            Category = "error", Tags = new[] { "error_path", "invalid_format" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            OutputFormat = "INVALID_FORMAT_XYZ",
            ExpectError = true, SkipUiChannel = true,
        };

        // 16. Corrupted input file
        yield return new TestCaseDefinition
        {
            Name = "error_corrupted_input",
            Category = "error", Tags = new[] { "error_path", "corrupted_file" },
            InputImage = "corrupted_file_ref",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            ExpectError = true, SkipUiChannel = true,
        };

        // 17. Disk full simulation (very large output dimensions)
        yield return new TestCaseDefinition
        {
            Name = "error_disk_full_simulation",
            Category = "error", Tags = new[] { "error_path", "disk_full", "resource_exhaustion" },
            InputImage = "pure_white_large",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform",
                p => { p["scale_percent"] = 10000; }),
            OutputFormat = "TIFF",
            ExpectError = true, SkipUiChannel = true,
        };

        // 18. Permission denied (read-only path)
        yield return new TestCaseDefinition
        {
            Name = "error_permission_denied",
            Category = "error", Tags = new[] { "error_path", "permission" },
            InputImage = "pure_red_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            ExpectError = true, SkipUiChannel = true,
            ExpectedErrorMessage = "permission",
        };

        // 19. Concurrent pipeline conflict
        yield return new TestCaseDefinition
        {
            Name = "error_concurrent_conflict",
            Category = "error", Tags = new[] { "error_path", "concurrent", "is_serial_only" },
            InputImages = new[] { "pure_white_large", "pure_black_large", "color_bars_8bit" },
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.ai_denoise",
                p => p["denoise_strength"] = 100.0),
            ExpectError = true, SkipUiChannel = true, IsSerialOnly = true,
        };

        // 20. Pipeline timeout simulation
        yield return new TestCaseDefinition
        {
            Name = "error_timeout",
            Category = "error", Tags = new[] { "error_path", "timeout" },
            InputImage = "zone_plate_32hz",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.ai_denoise",
                p => { p["denoise_strength"] = 100.0; p["detail_preservation"] = 100.0; }),
            ExpectError = true, SkipUiChannel = true,
        };

        // 21. Invalid file path (special characters)
        yield return new TestCaseDefinition
        {
            Name = "error_invalid_path_chars",
            Category = "error", Tags = new[] { "error_path", "invalid_path" },
            InputImage = "path/with/special/!@#$%^&()/test.png",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            ExpectError = true, SkipUiChannel = true,
        };

        // 22. Unsupported format input
        yield return new TestCaseDefinition
        {
            Name = "error_unsupported_format",
            Category = "error", Tags = new[] { "error_path", "unsupported_format" },
            InputImage = "unsupported_test_data.xyz",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.transform"),
            ExpectError = true, SkipUiChannel = true,
        };
    }

    // ── Metadata tests ──

    private static IEnumerable<TestCaseDefinition> BuildMetadataTests()
    {
        var metadataPlugins = new[]
        {
            "photopipeline.plugins.raw_input",
            "photopipeline.plugins.colorspace",
            "photopipeline.plugins.transform",
            "photopipeline.plugins.exif_rw",
            "photopipeline.plugins.gps_set",
            "photopipeline.plugins.time_shift",
        };
        foreach (var plugin in metadataPlugins)
        {
            yield return new TestCaseDefinition
            {
                Name = $"metadata_passthrough_{plugin.Replace("photopipeline.plugins.", "")}",
                Category = "metadata",
                Tags = new[] { "metadata", "passthrough", plugin },
                InputImage = "format_jpeg",
                Pipeline = TestPipelineBuilder.SingleNode(plugin),
                TolerancePerChannel = 0,
                // Metadata tests must verify EXIF fields survive pipeline processing
            };
        }
    }

    // ── Regression snapshot tests ──

    private static IEnumerable<TestCaseDefinition> BuildRegressionTests()
    {
        var inputImages = new[] { "gradient_horiz_rgb", "checkerboard_8x8", "color_bars_8bit",
                                   "grayscale_32steps", "mid_gray_128_small", "noise_grain" };
        // Deduplicated plugin list including lut3d, lens_correct, and encoders
        var plugins = new[] {
            "photopipeline.plugins.raw_input",
            "photopipeline.plugins.colorspace",
            "photopipeline.plugins.ai_denoise",
            "photopipeline.plugins.transform",
            "photopipeline.plugins.lut3d",
            "photopipeline.plugins.lens_correct",
            "photopipeline.plugins.png_encoder",
            "photopipeline.plugins.tiff_encoder",
        };

        foreach (var image in inputImages)
        {
            foreach (var plugin in plugins)
            {
                var shortName = plugin.Replace("photopipeline.plugins.", "");
                // Use quality metrics for lossy/output-heavy plugins
                bool isEncoder = plugin.Contains("_encoder");
                double? minSSIM = isEncoder ? 0.92 : null;
                double? minPSNR = isEncoder ? 30.0 : null;
                double? maxDeltaE = plugin.Contains("colorspace") ? 8.0 : null;

                yield return new TestCaseDefinition
                {
                    Name = $"regression_{shortName}_{image}",
                    Category = "regression",
                    Tags = new[] { "regression", "golden", plugin },
                    InputImage = image,
                    Pipeline = TestPipelineBuilder.SingleNode(plugin),
                    TolerancePerChannel = 0,
                    MinSSIM = minSSIM,
                    MinPSNR = minPSNR,
                    MaxDeltaE = maxDeltaE,
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

        // ── Boundary/edge-case content tests ──
        var boundaryImages = new[] { "boundary_1x1", "boundary_1x100", "boundary_2x2", "boundary_3x3" };
        foreach (var image in boundaryImages)
        {
            yield return new TestCaseDefinition
            {
                Name = $"content_boundary_{image}",
                Category = "content",
                Tags = new[] { "content", "boundary", "edge_case", "passthrough" },
                InputImage = image,
                OutputFormat = "PNG",
                TolerancePerChannel = 0,
            };
        }

        // ── Content assertion tests (verify specific pixel properties) ──
        yield return new TestCaseDefinition
        {
            Name = "content_assert_pure_white_is_white",
            Category = "content",
            Tags = new[] { "content", "assertion", "histogram" },
            InputImage = "pure_white_small",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.colorspace",
                p => { p["target_color_space"] = "srgb"; p["source_color_space"] = "srgb"; }),
            OutputFormat = "PNG",
            TolerancePerChannel = 0,
            MinSSIM = 0.999,
            // Reference against itself — output should be near-identical to input
            // since sRGB→sRGB is identity
        };

        yield return new TestCaseDefinition
        {
            Name = "content_assert_gradient_has_range",
            Category = "content",
            Tags = new[] { "content", "assertion", "histogram" },
            InputImage = "gradient_horiz_rgb",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.colorspace",
                p => { p["target_color_space"] = "srgb"; p["source_color_space"] = "srgb"; }),
            OutputFormat = "PNG",
            TolerancePerChannel = 0,
            MinSSIM = 0.999,
        };

        yield return new TestCaseDefinition
        {
            Name = "content_assert_checkerboard_equal_pixels",
            Category = "content",
            Tags = new[] { "content", "assertion", "pixel_count" },
            InputImage = "checkerboard_8x8",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.colorspace",
                p => { p["target_color_space"] = "srgb"; p["source_color_space"] = "srgb"; }),
            OutputFormat = "PNG",
            TolerancePerChannel = 0,
            MinSSIM = 0.999,
        };

        // DeltaE content quality test
        yield return new TestCaseDefinition
        {
            Name = "content_deltae_srgb_identity",
            Category = "content",
            Tags = new[] { "content", "deltae", "colorspace" },
            InputImage = "color_bars_8bit",
            Pipeline = TestPipelineBuilder.SingleNode("photopipeline.plugins.colorspace",
                p => { p["target_color_space"] = "srgb"; p["source_color_space"] = "srgb"; }),
            OutputFormat = "PNG",
            MaxDeltaE = 3.0,
        };
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
