using System.Text.Json;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public sealed class TestDataCatalog
{
    private static readonly string InputDir = Path.Combine(
        AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..",
        "FunctionalTests", "TestData", "input");

    private static TestDataCatalog? _instance;
    private readonly Dictionary<string, TestImageRecord> _records = new();

    public static TestDataCatalog Instance => _instance ??= new TestDataCatalog();

    private TestDataCatalog()
    {
        var manifestPath = Path.Combine(InputDir, "manifest.json");
        if (!File.Exists(manifestPath)) return;

        var json = File.ReadAllText(manifestPath);
        var records = JsonSerializer.Deserialize<List<TestImageRecord>>(json);
        if (records != null)
        {
            foreach (var r in records)
                _records[r.Id] = r;
        }
    }

    public IReadOnlyDictionary<string, TestImageRecord> Records => _records;

    public string GetPath(string id)
    {
        if (_records.TryGetValue(id, out var record))
            return Path.Combine(InputDir, record.RelativePath);
        return Path.Combine(InputDir, id);
    }

    public static string GetInputDir() => InputDir;

    public IEnumerable<TestImageRecord> GetByCategory(string category)
        => _records.Values.Where(r => r.Category == category);

    public IEnumerable<TestImageRecord> GetByTag(string tag)
        => _records.Values.Where(r => r.Tags.Contains(tag));
}

public sealed class TestImageRecord
{
    public string Id { get; set; } = string.Empty;
    public string RelativePath { get; set; } = string.Empty;
    public string Category { get; set; } = string.Empty;
    public string Format { get; set; } = "PNG";
    public int Width { get; set; }
    public int Height { get; set; }
    public int BitDepth { get; set; } = 8;
    public string ColorSpace { get; set; } = "sRGB";
    public bool HasAlpha { get; set; }
    public string[] Tags { get; set; } = [];
    public string Description { get; set; } = string.Empty;
}
