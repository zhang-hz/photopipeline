using Photopipeline.Models;
using Photopipeline.Tests.TestInfrastructure;
using SkiaSharp;

namespace Photopipeline.Tests.SystemTests;

public sealed class BatchOutputVerificationTests : IDisposable
{
    private readonly TestPipelineService _pipeline;
    private readonly string _tempDir;

    public BatchOutputVerificationTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"pp_batch_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
        _pipeline = new TestPipelineService(_tempDir);
        TestImageFactory.GenerateFullTestSet();
    }

    public void Dispose()
    {
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
    }

    [Fact]
    public async Task SingleImage_BatchProcess_ProducesOutput()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "batch1" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });

        var result = await _pipeline.ExecutePipelineAsync(pipeline, input);
        Assert.True(result);
        Assert.True(File.Exists(_pipeline.LastOutputPath));
    }

    [Fact]
    public async Task ThreeImages_BatchProcess_AllSucceed()
    {
        var images = new[]
        {
            TestImageFactory.GetPath("solid_rgb_256.png"),
            TestImageFactory.GetPath("gradient_256.png"),
            TestImageFactory.GetPath("checkerboard_256.png"),
        };

        var pipeline = new PipelineModel { Name = "batch3" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "grayscale", DisplayName = "Gray" });

        var results = new List<bool>();
        foreach (var img in images)
            results.Add(await _pipeline.ExecutePipelineAsync(pipeline, img));

        Assert.All(results, Assert.True);
        Assert.Equal(3, _pipeline.ExecutionCount);
    }

    [Fact]
    public async Task FiveImages_DifferentPipelines_AllSucceed()
    {
        var images = TestImageFactory.GetByMinSize(256, 256).Take(5).ToList();
        Assert.NotEmpty(images);

        var pipelines = new[]
        {
            new PipelineModel { Name = "p1" },
            new PipelineModel { Name = "p2", Nodes = { new PipelineNode { PluginId = "grayscale", DisplayName = "G" } } },
            new PipelineModel { Name = "p3", Nodes = { new PipelineNode { PluginId = "invert", DisplayName = "I" } } },
            new PipelineModel { Name = "p4", Nodes = { new PipelineNode { PluginId = "brightness", DisplayName = "B", Parameters = new Dictionary<string, object> { ["value"] = 0.2 } } } },
            new PipelineModel { Name = "p5", Nodes = { new PipelineNode { PluginId = "blur", DisplayName = "Blur", Parameters = new Dictionary<string, object> { ["sigma"] = 1.0f } } } },
        };

        var results = new List<bool>();
        for (int i = 0; i < images.Count && i < pipelines.Length; i++)
            results.Add(await _pipeline.ExecutePipelineAsync(pipelines[i], images[i]));

        Assert.All(results, Assert.True);
    }

    [Fact]
    public async Task MixedFormatInput_AllProcessSuccessfully()
    {
        var pngInput = TestImageFactory.GetPath("solid_rgb_256.png");
        var jpegInput = TestImageFactory.GetPath("solid_rgb_256.jpg");

        var pipeline = new PipelineModel { Name = "mixed" };

        var r1 = await _pipeline.ExecutePipelineAsync(pipeline, pngInput);
        var r2 = await _pipeline.ExecutePipelineAsync(pipeline, jpegInput);

        Assert.True(r1);
        Assert.True(r2);
    }

    [Fact]
    public async Task BatchWithFailure_OneMissingFile_OtherSucceeds()
    {
        var pipeline = new PipelineModel { Name = "partial-fail" };

        var r1 = await _pipeline.ExecutePipelineAsync(pipeline, "/nonexistent/a.png");
        var r2 = await _pipeline.ExecutePipelineAsync(pipeline, TestImageFactory.GetPath("solid_rgb_256.png"));

        Assert.False(r1);
        Assert.True(r2);
        Assert.Equal(2, _pipeline.ExecutionCount);
    }

    [Fact]
    public async Task BatchWithFailure_AllMissingFiles_AllFail()
    {
        var pipeline = new PipelineModel { Name = "all-fail" };

        var r1 = await _pipeline.ExecutePipelineAsync(pipeline, "/nonexistent/a.png");
        var r2 = await _pipeline.ExecutePipelineAsync(pipeline, "/nonexistent/b.png");

        Assert.False(r1);
        Assert.False(r2);
    }

    [Fact]
    public async Task OutputFiles_UniqueNames_NoCollision()
    {
        var input = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "collision" };

        var paths = new HashSet<string>();
        for (int i = 0; i < 5; i++)
        {
            await _pipeline.ExecutePipelineAsync(pipeline, input);
            Assert.True(paths.Add(_pipeline.LastOutputPath),
                $"Duplicate output path: {_pipeline.LastOutputPath}");
        }
    }

    [Fact]
    public async Task BatchProcessing_OutputFilesDecodable()
    {
        var images = new[]
        {
            TestImageFactory.GetPath("solid_rgb_256.png"),
            TestImageFactory.GetPath("gradient_256.png"),
            TestImageFactory.GetPath("color_bars_256.png"),
        };

        var pipeline = new PipelineModel { Name = "batch-decode" };
        pipeline.Nodes.Add(new PipelineNode { PluginId = "invert", DisplayName = "Inv" });

        var outputPaths = new List<string>();
        foreach (var img in images)
        {
            await _pipeline.ExecutePipelineAsync(pipeline, img);
            outputPaths.Add(_pipeline.LastOutputPath);
        }

        foreach (var path in outputPaths)
        {
            using var decoded = SKBitmap.Decode(path);
            Assert.NotNull(decoded);
            Assert.True(decoded!.Width > 0);
            Assert.True(decoded.Height > 0);
        }
    }

    [Fact]
    public async Task TenImages_SequentialProcessing_AllOutputFilesExist()
    {
        var basePath = TestImageFactory.GetPath("solid_rgb_256.png");
        var pipeline = new PipelineModel { Name = "batch10" };

        for (int i = 0; i < 10; i++)
        {
            var result = await _pipeline.ExecutePipelineAsync(pipeline, basePath);
            Assert.True(result);
            Assert.True(File.Exists(_pipeline.LastOutputPath));
        }
    }
}
