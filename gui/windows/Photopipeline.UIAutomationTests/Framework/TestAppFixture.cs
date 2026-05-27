using System.Diagnostics;
using Photopipeline.Tests.FunctionalTests.UiChannel;
using Xunit;
using Xunit.Abstractions;

namespace Photopipeline.UIAutomationTests.Framework;

/// <summary>
/// Collection definition for FlaUI-based GUI tests.
/// All tests in this collection share a common app instance lifecycle.
/// </summary>
[CollectionDefinition("FlaUITests")]
public sealed class FlaUITestCollection : ICollectionFixture<TestAppFixture> { }

/// <summary>
/// Manages the test application process, output directories, and test data paths.
/// Provides factory methods for creating UiTestDriver instances.
/// </summary>
public sealed class TestAppFixture : IDisposable
{
    public string AppPath { get; }
    public string TestDataDir { get; }
    public string OutputDir { get; }
    public string ScreenshotDir { get; }
    public string EvidenceDir { get; }
    public string? ResolvedRuntimeDir { get; }

    public TestAppFixture()
    {
        // Resolve the Photopipeline.exe binary path.
        // Priority: publish output > Debug build > Release build
        AppPath = ResolveAppPath();

        if (!File.Exists(AppPath))
            Assert.Fail($"Photopipeline.exe not found at '{AppPath}'. " +
                        "Build the project first: dotnet build -c Release");

        ResolvedRuntimeDir = Path.GetDirectoryName(AppPath);

        // Resolve test data directory
        TestDataDir = ResolveTestDataDir();

        // Create output directories
        OutputDir = Path.Combine(Path.GetTempPath(), "photopipeline_tests", "output",
            DateTime.Now.ToString("yyyyMMdd_HHmmss"));
        Directory.CreateDirectory(OutputDir);

        ScreenshotDir = Path.Combine(Path.GetTempPath(), "photopipeline_tests", "screenshots");
        Directory.CreateDirectory(ScreenshotDir);

        EvidenceDir = Path.Combine(OutputDir, "evidence");
        Directory.CreateDirectory(EvidenceDir);
    }

    /// <summary>
    /// Creates a new UiTestDriver instance bound to this fixture's app path and test data.
    /// Does NOT launch the app — the driver manages its own lifecycle.
    /// </summary>
    public UiTestDriver CreateDriver(ITestOutputHelper output)
    {
        return new UiTestDriver(AppPath, TestDataDir, OutputDir, output);
    }

    public void Dispose()
    {
        // Cleanup any leftover output files older than 7 days
        try
        {
            var cleanupBase = Path.Combine(Path.GetTempPath(), "photopipeline_tests", "output");
            if (Directory.Exists(cleanupBase))
            {
                foreach (var dir in Directory.GetDirectories(cleanupBase))
                {
                    try
                    {
                        var dirInfo = new DirectoryInfo(dir);
                        if (dirInfo.CreationTime < DateTime.Now.AddDays(-7))
                            Directory.Delete(dir, recursive: true);
                    }
                    catch
                    {
                        // Best-effort cleanup
                    }
                }
            }
        }
        catch
        {
            // Best-effort cleanup
        }
    }

    // ── Path resolution ──

    private static string ResolveAppPath()
    {
        var baseDir = AppDomain.CurrentDomain.BaseDirectory;

        // Build configurations to try, in priority order
        var candidates = new List<string>();

        // Candidate 0: Staging publish output (highest priority)
        candidates.Add(Path.Combine(baseDir, "..", "..", "..", "..",
            "Photopipeline", "bin", "publish", "staging", "Photopipeline.exe"));

        // Candidate 1: Published self-contained app
        foreach (var rid in new[] { "win-x64", "win-arm64" })
        foreach (var cfg in new[] { "Release", "Debug" })
        {
            candidates.Add(Path.Combine(baseDir, "..", "..", "..", "..",
                "Photopipeline", "bin", rid, cfg, "net9.0-windows", "publish", "Photopipeline.exe"));
            candidates.Add(Path.Combine(baseDir, "..", "..", "..", "..",
                "Photopipeline", "bin", "x64", cfg, "net9.0-windows", "publish", "Photopipeline.exe"));
        }

        // Candidate 2: Standard build output (x64)
        foreach (var cfg in new[] { "Release", "Debug" })
        {
            candidates.Add(Path.Combine(baseDir, "..", "..", "..", "..",
                "Photopipeline", "bin", "x64", cfg, "net9.0-windows", "Photopipeline.exe"));
        }

        // Candidate 3: Standard build output (win-x64 RID)
        foreach (var cfg in new[] { "Release", "Debug" })
        {
            candidates.Add(Path.Combine(baseDir, "..", "..", "..", "..",
                "Photopipeline", "bin", "win-x64", cfg, "net9.0-windows", "Photopipeline.exe"));
        }

        // Candidate 4: Side-by-side publish output
        candidates.Add(Path.Combine(baseDir, "Photopipeline.exe"));

        foreach (var candidate in candidates)
        {
            var full = Path.GetFullPath(candidate);
            if (File.Exists(full))
                return full;
        }

        // Return the most likely path for a clear error message
        return Path.GetFullPath(candidates[0]);
    }

    private static string ResolveTestDataDir()
    {
        var baseDir = AppDomain.CurrentDomain.BaseDirectory;

        // Try the TestData directory in the Tests project
        var candidates = new[]
        {
            Path.Combine(baseDir, "..", "..", "..", "..",
                "Photopipeline.Tests", "TestData", "input"),
            Path.Combine(baseDir, "TestData", "input"),
            Path.Combine(baseDir, "..", "..", "..", "TestData", "input"),
        };

        foreach (var candidate in candidates)
        {
            var full = Path.GetFullPath(candidate);
            if (Directory.Exists(full))
                return full;
        }

        // Fallback: create the directory
        var fallback = Path.GetFullPath(Path.Combine(baseDir, "..", "..", "..", "..",
            "Photopipeline.Tests", "TestData", "input"));
        Directory.CreateDirectory(fallback);
        return fallback;
    }
}
