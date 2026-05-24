using Photopipeline.Models;
using Photopipeline.Services;
using SkiaSharp;
using System.Collections.ObjectModel;

namespace Photopipeline.Tests.TestInfrastructure;

/// <summary>
/// Test-oriented pipeline service that performs actual image processing using SkiaSharp.
/// Supports: identity, grayscale, invert, brightness, resize operations.
/// This enables true L3 end-to-end tests without a running gRPC server.
/// </summary>
public sealed class TestPipelineService : IPipelineService
{
    private readonly string _outputDir;
    private readonly List<PluginInfo> _plugins;
    private int _executionCount;

    public int ExecutionCount => _executionCount;
    public string LastOutputPath { get; private set; } = string.Empty;

    public TestPipelineService(string? outputDir = null)
    {
        _outputDir = outputDir ?? Path.Combine(Path.GetTempPath(), "photopipeline_tests");
        Directory.CreateDirectory(_outputDir);
        _plugins = CreateTestPlugins();
    }

    private static List<PluginInfo> CreateTestPlugins()
    {
        return new List<PluginInfo>
        {
            new()
            {
                Id = "identity", Name = "Identity", Category = "Basic",
                Description = "Pass-through (no-op)",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema>()
            },
            new()
            {
                Id = "grayscale", Name = "Grayscale", Category = "Color",
                Description = "Convert to grayscale",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema>
                {
                    new() { Name = "method", ParameterType = ParameterType.Enum,
                        EnumValues = new ObservableCollection<object> { "luminance", "average", "lightness" },
                        DefaultValue = "luminance" }
                }
            },
            new()
            {
                Id = "invert", Name = "Invert", Category = "Color",
                Description = "Invert colors",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema>()
            },
            new()
            {
                Id = "brightness", Name = "Brightness", Category = "Tonal",
                Description = "Adjust brightness",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema>
                {
                    new() { Name = "value", ParameterType = ParameterType.Float,
                        DefaultValue = 0.0 }
                }
            },
            new()
            {
                Id = "resize", Name = "Resize", Category = "Transform",
                Description = "Resize image",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema>
                {
                    new() { Name = "width", ParameterType = ParameterType.Integer,
                        DefaultValue = 256 },
                    new() { Name = "height", ParameterType = ParameterType.Integer,
                        DefaultValue = 256 }
                }
            },
            new()
            {
                Id = "blur", Name = "Blur", Category = "Filter",
                Description = "Gaussian blur",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas = new ObservableCollection<ParameterSchema>
                {
                    new() { Name = "sigma", ParameterType = ParameterType.Float,
                        DefaultValue = 1.0 }
                }
            },
        };
    }

    public Task<PipelineModel> CreatePipelineAsync(string name, string description = "", CancellationToken ct = default)
    {
        return Task.FromResult(new PipelineModel
        {
            Id = Guid.NewGuid().ToString(),
            Name = name,
            Description = description
        });
    }

    public Task<bool> ValidatePipelineAsync(PipelineModel pipeline, CancellationToken ct = default)
    {
        pipeline.IsValid = true;
        pipeline.ValidationError = string.Empty;
        return Task.FromResult(true);
    }

    public async Task<bool> ExecutePipelineAsync(PipelineModel pipeline, string imagePath, CancellationToken ct = default)
    {
        _executionCount++;
        if (!File.Exists(imagePath)) return false;

        using var input = SKBitmap.Decode(imagePath);
        if (input is null) return false;

        using var output = ProcessNodes(pipeline, input);

        LastOutputPath = Path.Combine(_outputDir,
            $"output_{_executionCount}_{Path.GetFileName(imagePath)}");
        Directory.CreateDirectory(Path.GetDirectoryName(LastOutputPath)!);

        using var img = SKImage.FromBitmap(output);
        using var data = img.Encode(SKEncodedImageFormat.Png, 100);
        using var fs = File.OpenWrite(LastOutputPath);
        data.SaveTo(fs);

        return true;
    }

    private SKBitmap ProcessNodes(PipelineModel pipeline, SKBitmap input)
    {
        var current = input.Copy();
        if (pipeline.Nodes.Count == 0) return current;

        foreach (var node in pipeline.Nodes)
        {
            using var prev = current;
            current = ApplyPlugin(prev, node);
        }
        return current;
    }

    private SKBitmap ApplyPlugin(SKBitmap input, PipelineNode node)
    {
        var result = new SKBitmap(input.Width, input.Height);
        switch (node.PluginId)
        {
            case "identity":
                return input.Copy();

            case "grayscale":
                Grayscale(input, result);
                return result;

            case "invert":
                Invert(input, result);
                return result;

            case "brightness":
                Brightness(input, result,
                    Convert.ToSingle(node.Parameters.GetValueOrDefault("value", 0.0)));
                return result;

            case "resize":
                var w = Convert.ToInt32(node.Parameters.GetValueOrDefault("width", 256));
                var h = Convert.ToInt32(node.Parameters.GetValueOrDefault("height", 256));
                var resized = input.Resize(new SKSizeI(w, h), SKFilterQuality.Medium);
                return resized;

            case "blur":
                var sigma = Convert.ToSingle(node.Parameters.GetValueOrDefault("sigma", 1.0f));
                var blurred = new SKBitmap(input.Width, input.Height);
                using (var surface = SKSurface.Create(new SKImageInfo(input.Width, input.Height)))
                {
                    var canvas = surface.Canvas;
                    using var paint = new SKPaint
                    {
                        ImageFilter = SKImageFilter.CreateBlur(sigma, sigma)
                    };
                    canvas.DrawBitmap(input, 0, 0, paint);
                    canvas.Flush();
                    using var snap = surface.Snapshot();
                    return SKBitmap.FromImage(snap) ?? input.Copy();
                }

            default:
                return input.Copy();
        }
    }

    private static void Grayscale(SKBitmap src, SKBitmap dst)
    {
        for (int y = 0; y < src.Height; y++)
        for (int x = 0; x < src.Width; x++)
        {
            var c = src.GetPixel(x, y);
            byte v = (byte)(0.299 * c.Red + 0.587 * c.Green + 0.114 * c.Blue);
            dst.SetPixel(x, y, new SKColor(v, v, v, c.Alpha));
        }
    }

    private static void Invert(SKBitmap src, SKBitmap dst)
    {
        for (int y = 0; y < src.Height; y++)
        for (int x = 0; x < src.Width; x++)
        {
            var c = src.GetPixel(x, y);
            dst.SetPixel(x, y, new SKColor(
                (byte)(255 - c.Red), (byte)(255 - c.Green),
                (byte)(255 - c.Blue), c.Alpha));
        }
    }

    private static void Brightness(SKBitmap src, SKBitmap dst, float value)
    {
        int delta = (int)(value * 255);
        for (int y = 0; y < src.Height; y++)
        for (int x = 0; x < src.Width; x++)
        {
            var c = src.GetPixel(x, y);
            dst.SetPixel(x, y, new SKColor(
                Clamp(c.Red + delta), Clamp(c.Green + delta),
                Clamp(c.Blue + delta), c.Alpha));
        }
    }

    private static byte Clamp(int v) => (byte)Math.Max(0, Math.Min(255, v));

    public Task<ObservableCollection<PluginInfo>> GetAvailablePluginsAsync(CancellationToken ct = default)
        => Task.FromResult(new ObservableCollection<PluginInfo>(_plugins));

    public Task UpdateNodeParametersAsync(string nodeId, Dictionary<string, object> parameters, CancellationToken ct = default)
        => Task.CompletedTask;
}
