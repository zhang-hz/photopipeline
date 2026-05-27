namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public static class ResourceMonitor
{
    private const long MinDiskBytes = 1_000_000_000; // 1 GB
    private const long MaxMemoryBytes = 2_000_000_000; // 2 GB

    public static bool IsDiskOk()
    {
        try
        {
            var root = Path.GetPathRoot(Path.GetTempPath()) ?? "C:\\";
            var drive = new DriveInfo(root);
            return drive.AvailableFreeSpace >= MinDiskBytes;
        }
        catch
        {
            return false; // If monitoring fails, assume resources may be insufficient
        }
    }

    public static bool IsMemoryOk()
    {
        try
        {
            var workingSet = Environment.WorkingSet;
            return workingSet < MaxMemoryBytes;
        }
        catch
        {
            return false;
        }
    }

    public static bool ShouldSkipLargeTest()
    {
        if (!IsDiskOk())
        {
            Console.WriteLine($"[ResourceMonitor] DISK LOW - skipping large test");
            return true;
        }
        if (!IsMemoryOk())
        {
            Console.WriteLine($"[ResourceMonitor] MEMORY HIGH ({Environment.WorkingSet / 1024 / 1024}MB) - skipping large test");
            return true;
        }
        return false;
    }

    public static string GetStatus()
    {
        var ws = Environment.WorkingSet / 1024.0 / 1024.0;
        string diskStatus;
        try
        {
            var root = Path.GetPathRoot(Path.GetTempPath()) ?? "C:\\";
            var drive = new DriveInfo(root);
            diskStatus = $"{drive.AvailableFreeSpace / 1024.0 / 1024.0 / 1024.0:F1}GB free";
        }
        catch { diskStatus = "unknown"; }

        return $"Memory: {ws:F1}MB WS | Disk: {diskStatus}";
    }
}
