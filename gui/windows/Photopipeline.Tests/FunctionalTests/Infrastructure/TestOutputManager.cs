namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public sealed class TestOutputManager : IDisposable
{
    private readonly string _runDir;
    private readonly List<string> _createdFiles = new();
    private readonly List<string> _createdSubdirs = new();
    private long _totalBytesWritten;
    private static long s_globalBytesWritten;
    private readonly string _instanceId = Guid.NewGuid().ToString("N")[..8];

    public TestOutputManager(string? testName = null)
    {
        var runId = Environment.GetEnvironmentVariable("CI_RUN_ID")
                    ?? $"{DateTime.Now:yyyyMMdd}_{DateTime.Now.Ticks}_{Guid.NewGuid():N}";
        _runDir = Path.Combine(Path.GetTempPath(), "photopipeline_tests", runId);

        if (!string.IsNullOrEmpty(testName))
            _runDir = Path.Combine(_runDir, SanitizePath(testName));

        Directory.CreateDirectory(_runDir);
        _createdSubdirs.Add(_runDir);
    }

    public string RunDir => _runDir;

    public string GetOutputPath(string fileName)
    {
        var path = Path.Combine(_runDir, SanitizePath(fileName));
        _createdFiles.Add(path);
        return path;
    }

    /// <summary>
    /// Creates a subdirectory under the run directory and returns its path.
    /// The subdirectory will be tracked for cleanup.
    /// </summary>
    public string CreateSubdirectory(string name)
    {
        var subPath = Path.Combine(_runDir, SanitizePath(name));
        Directory.CreateDirectory(subPath);
        _createdSubdirs.Add(subPath);
        return subPath;
    }

    public void TrackFile(string path)
    {
        if (File.Exists(path))
        {
            _createdFiles.Add(path);
            long size = new FileInfo(path).Length;
            Interlocked.Add(ref _totalBytesWritten, size);
            Interlocked.Add(ref s_globalBytesWritten, size);
        }
    }

    /// <summary>
    /// Tracks a directory for cleanup on Dispose.
    /// </summary>
    public void TrackDirectory(string path)
    {
        if (Directory.Exists(path))
            _createdSubdirs.Add(path);
    }

    public void Dispose()
    {
        // Clean up individual files first
        foreach (var file in _createdFiles)
        {
            try { if (File.Exists(file)) File.Delete(file); }
            catch { /* best-effort cleanup */ }
        }

        // Clean up tracked subdirectories (deepest first)
        foreach (var subdir in _createdSubdirs.OrderByDescending(d => d.Length))
        {
            try
            {
                if (Directory.Exists(subdir) && !Directory.EnumerateFileSystemEntries(subdir).Any())
                    Directory.Delete(subdir, recursive: true);
            }
            catch { /* best-effort cleanup */ }
        }

        // Clean up the run directory itself
        try
        {
            if (Directory.Exists(_runDir) && !Directory.EnumerateFileSystemEntries(_runDir).Any())
                Directory.Delete(_runDir, recursive: true);
        }
        catch { /* best-effort cleanup */ }
    }

    /// <summary>Bytes written by this instance.</summary>
    public long InstanceBytesWritten => Interlocked.Read(ref _totalBytesWritten);

    /// <summary>Bytes written across all instances.</summary>
    public static long TotalBytesWritten => Interlocked.Read(ref s_globalBytesWritten);

    private string SanitizePath(string name)
    {
        foreach (var c in Path.GetInvalidFileNameChars())
            name = name.Replace(c, '_');

        // Prevent truncation collisions: if the name is too long, keep the prefix
        // but append a short hash of the full name for uniqueness.
        const int maxLen = 120;
        if (name.Length > maxLen)
        {
            string hash = ComputeShortHash(name);
            int prefixLen = maxLen - hash.Length - 1; // -1 for separator
            name = name[..Math.Max(0, prefixLen)] + "_" + hash;
        }
        return name;
    }

    /// <summary>
    /// Compute a short (8-char) deterministic hash to prevent filename collisions
    /// when truncation would produce identical prefixes.
    /// </summary>
    private static string ComputeShortHash(string input)
    {
        uint hash = 2166136261;
        foreach (char c in input)
        {
            hash ^= c;
            hash *= 16777619;
        }
        return hash.ToString("X8");
    }
}
