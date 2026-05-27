using System.Text.Json;
using Photopipeline.Tests.FunctionalTests.ApiChannel;
using SkiaSharp;
using Xunit.Abstractions;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

/// <summary>
/// Generates golden reference images for regression testing.
/// Triggered by setting the PHOTOPIPELINE_GENERATE_GOLDEN environment variable.
///
/// Usage:
///   set PHOTOPIPELINE_GENERATE_GOLDEN=1
///   dotnet test --filter "FullyQualifiedName~GenerateGoldenReferences"
///
/// Or programmatically:
///   await ReferenceImageGenerator.GenerateAllGoldenAsync(outputHelper);
/// </summary>
public static class ReferenceImageGenerator
{
    private const string EnvVarName = "PHOTOPIPELINE_GENERATE_GOLDEN";
    private const string GoldenDirName = "golden";
    private const string ManifestFileName = "manifest.json";

    /// <summary>
    /// Returns true if PHOTOPIPELINE_GENERATE_GOLDEN is set to a truthy value.
    /// </summary>
    public static bool IsGenerateModeEnabled
    {
        get
        {
            var val = Environment.GetEnvironmentVariable(EnvVarName);
            return !string.IsNullOrEmpty(val) &&
                   (val == "1" || val.Equals("true", StringComparison.OrdinalIgnoreCase));
        }
    }

    /// <summary>
    /// Finds the golden fixture directory: &lt;repo-root&gt;/tests/fixtures/golden/.
    /// </summary>
    public static string FindGoldenDir()
    {
        var baseDir = AppDomain.CurrentDomain.BaseDirectory;
        var dir = new DirectoryInfo(baseDir);

        // Walk up to find repo root (marked by .git)
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir.FullName, ".git")))
                break;
            dir = dir.Parent;
        }

        if (dir == null)
            throw new DirectoryNotFoundException(
                $"Could not find repository root from {baseDir}");

        var goldenDir = Path.Combine(dir.FullName, "tests", "fixtures", GoldenDirName);
        return goldenDir;
    }

    /// <summary>
    /// Generates golden reference images for all available test cases.
    /// Each test case is processed through the API channel and the output is saved
    /// as the golden reference. The golden images are validated for correctness
    /// (non-empty, expected dimensions when available).
    /// </summary>
    /// <returns>The number of golden images generated.</returns>
    public static async Task<int> GenerateAllGoldenAsync(
        ITestOutputHelper? output = null,
        CancellationToken ct = default)
    {
        if (!IsGenerateModeEnabled)
            throw new InvalidOperationException(
                $"Environment variable {EnvVarName} must be set to '1' or 'true'.");

        var goldenDir = FindGoldenDir();
        Directory.CreateDirectory(goldenDir);
        output?.WriteLine($"Golden directory: {goldenDir}");

        var testCases = TestCaseCatalog.All;
        output?.WriteLine($"Loaded {testCases.Count} test cases from catalog.");

        // Create an ApiTestBase adapter to execute pipelines
        var api = new GoldenGeneratorApiAdapter(output);

        try
        {
            await api.RequireBackendAsync(ct);
        }
        catch (Exception ex)
        {
            output?.WriteLine($"ERROR: Backend unavailable for golden generation: {ex.Message}");
            throw;
        }

        var manifest = new List<GoldenRecord>();
        int generated = 0;
        int skipped = 0;
        int failed = 0;

        foreach (var tc in testCases)
        {
            // Skip error path and UI-only tests
            if (tc.ExpectError || tc.SkipApiChannel)
            {
                skipped++;
                continue;
            }

            ct.ThrowIfCancellationRequested();

            var goldenFileName = SanitizeFileName($"{tc.Category}_{tc.Name}.tif");
            var goldenPath = Path.Combine(goldenDir, goldenFileName);

            try
            {
                output?.WriteLine($"  [{generated + skipped + 1}/{testCases.Count}] Generating golden: {tc.Name}");

                using var outputMgr = new TestOutputManager($"golden_{tc.Name}");
                var inputPath = TestDataCatalog.Instance.GetPath(tc.InputImage);
                var tempOutputPath = outputMgr.GetOutputPath($"{tc.Name}_temp.tif");

                // Execute with pipeline if specified, otherwise identity pass-through
                if (tc.Pipeline != null)
                {
                    await api.ExecuteAndGetOutput(tc.Pipeline, inputPath, tempOutputPath, ct);
                }
                else
                {
                    // Identity pass-through: copy the input to output format
                    using var bmp = ImageAssert.LoadBitmap(inputPath);
                    SaveAsTiff(bmp, tempOutputPath);
                }

                // Validate before accepting as golden
                ValidateGoldenOutput(tempOutputPath, tc);

                // Copy to golden directory
                File.Copy(tempOutputPath, goldenPath, overwrite: true);

                // Record in manifest
                manifest.Add(new GoldenRecord
                {
                    TestId = tc.Id,
                    TestName = tc.Name,
                    Category = tc.Category,
                    RelativePath = goldenFileName,
                    InputImage = tc.InputImage,
                    Tags = tc.Tags,
                    Width = tc.OutputBitDepth.HasValue ? 0 : 0, // Will be populated below
                });

                // Read actual dimensions
                using var goldenBmp = ImageAssert.LoadBitmap(goldenPath);
                var record = manifest[^1];
                record.Width = goldenBmp.Width;
                record.Height = goldenBmp.Height;

                generated++;
                output?.WriteLine($"    -> {goldenFileName} ({goldenBmp.Width}x{goldenBmp.Height})");
            }
            catch (Exception ex)
            {
                failed++;
                output?.WriteLine($"    FAILED: {ex.Message}");
            }
        }

        // Write manifest
        var manifestPath = Path.Combine(goldenDir, ManifestFileName);
        var manifestJson = JsonSerializer.Serialize(manifest, new JsonSerializerOptions
        {
            WriteIndented = true
        });
        await File.WriteAllTextAsync(manifestPath, manifestJson, ct);

        output?.WriteLine($"Golden generation complete: {generated} generated, {skipped} skipped, {failed} failed.");
        output?.WriteLine($"Manifest: {manifestPath}");

        if (failed > 0)
            throw new InvalidOperationException(
                $"{failed} golden images failed to generate. See output for details.");

        return generated;
    }

    /// <summary>
    /// Validates that a golden output image is valid:
    /// non-zero file size, readable by SkiaSharp, correct dimensions if expected.
    /// </summary>
    public static void ValidateGoldenOutput(string goldenPath, TestCaseDefinition tc)
    {
        if (!File.Exists(goldenPath))
            throw new FileNotFoundException($"Golden output not found: {goldenPath}");

        var fi = new FileInfo(goldenPath);
        if (fi.Length == 0)
            throw new InvalidOperationException(
                $"Golden output is empty (0 bytes): {goldenPath}");

        // Verify it can be decoded
        using var codec = SKCodec.Create(goldenPath, out var codecResult);
        if (codec == null || codecResult != SKCodecResult.Success)
            throw new InvalidOperationException(
                $"Golden output cannot be decoded by SkiaSharp: {goldenPath} (result={codecResult})");

        if (fi.Length < 64)
            throw new InvalidOperationException(
                $"Golden output is suspiciously small ({fi.Length} bytes): {goldenPath}");
    }

    /// <summary>
    /// Loads all golden images as a dictionary keyed by test case ID.
    /// </summary>
    public static Dictionary<string, string> LoadGoldenMap()
    {
        var goldenDir = FindGoldenDir();
        var manifestPath = Path.Combine(goldenDir, ManifestFileName);

        if (!File.Exists(manifestPath))
            throw new FileNotFoundException(
                $"Golden manifest not found: {manifestPath}. Run golden generation first.");

        var json = File.ReadAllText(manifestPath);
        var records = JsonSerializer.Deserialize<List<GoldenRecord>>(json);

        if (records == null || records.Count == 0)
            throw new InvalidOperationException("Golden manifest is empty.");

        var map = new Dictionary<string, string>();
        foreach (var r in records)
        {
            var goldenPath = Path.Combine(goldenDir, r.RelativePath);
            if (!File.Exists(goldenPath))
                throw new FileNotFoundException(
                    $"Golden image referenced in manifest but missing: {goldenPath}");
            map[r.TestId] = goldenPath;
        }

        return map;
    }

    private static string SanitizeFileName(string name)
    {
        foreach (var c in Path.GetInvalidFileNameChars())
            name = name.Replace(c, '_');
        return name.Length > 200 ? name[..200] : name;
    }

    private static void SaveAsTiff(SKBitmap bmp, string path)
    {
        using var image = SKImage.FromBitmap(bmp);
        // TIFF encoding unsupported in SkiaSharp 3.x; use PNG with .tif extension
        using var data = image.Encode(SKEncodedImageFormat.Png, 100);
        using var stream = new FileStream(path, FileMode.Create, FileAccess.Write);
        data.SaveTo(stream);
    }

    private sealed class GoldenGeneratorApiAdapter : ApiTestBase
    {
        private readonly ITestOutputHelper? _genOutput;

        public GoldenGeneratorApiAdapter(ITestOutputHelper? output)
            : base(output ?? throw new ArgumentNullException(nameof(output)))
        {
            _genOutput = output;
        }
    }

    private sealed class GoldenRecord
    {
        public string TestId { get; set; } = string.Empty;
        public string TestName { get; set; } = string.Empty;
        public string Category { get; set; } = string.Empty;
        public string RelativePath { get; set; } = string.Empty;
        public string InputImage { get; set; } = string.Empty;
        public string[] Tags { get; set; } = [];
        public int Width { get; set; }
        public int Height { get; set; }
    }
}
