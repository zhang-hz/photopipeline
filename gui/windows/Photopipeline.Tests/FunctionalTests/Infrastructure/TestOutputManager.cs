namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public sealed class TestOutputManager : IDisposable
{
    private readonly string _runDir;
    private readonly List<string> _createdFiles = new();
    private static long _totalBytesWritten;

    public TestOutputManager(string? testName = null)
    {
        var runId = Environment.GetEnvironmentVariable("CI_RUN_ID")
                    ?? DateTime.Now.ToString("yyyyMMdd_HHmmss");
        _runDir = Path.Combine(Path.GetTempPath(), "photopipeline_tests", runId);

        if (!string.IsNullOrEmpty(testName))
            _runDir = Path.Combine(_runDir, SanitizePath(testName));

        Directory.CreateDirectory(_runDir);
    }

    public string RunDir => _runDir;

    public string GetOutputPath(string fileName)
    {
        var path = Path.Combine(_runDir, SanitizePath(fileName));
        _createdFiles.Add(path);
        return path;
    }

    public void TrackFile(string path)
    {
        if (File.Exists(path))
        {
            _createdFiles.Add(path);
            Interlocked.Add(ref _totalBytesWritten, new FileInfo(path).Length);
        }
    }

    public void Dispose()
    {
        foreach (var file in _createdFiles)
        {
            try { if (File.Exists(file)) File.Delete(file); }
            catch { /* best-effort cleanup */ }
        }

        try
        {
            if (Directory.Exists(_runDir) && !Directory.EnumerateFileSystemEntries(_runDir).Any())
                Directory.Delete(_runDir, recursive: true);
        }
        catch { /* best-effort cleanup */ }
    }

    public static long TotalBytesWritten => Interlocked.Read(ref _totalBytesWritten);

    private static string SanitizePath(string name)
    {
        foreach (var c in Path.GetInvalidFileNameChars())
            name = name.Replace(c, '_');
        return name.Length > 120 ? name[..120] : name;
    }
}
