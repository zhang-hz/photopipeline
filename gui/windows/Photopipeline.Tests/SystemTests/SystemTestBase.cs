using System.Diagnostics;

namespace Photopipeline.Tests.SystemTests;

public abstract class SystemTestBase : IDisposable
{
    protected Process? ServerProcess;
    protected GrpcClientService Client;
    protected PipelineService PipelineService;
    protected ImageService ImageService;
    private bool _disposed;

    private const string ServerAddress = "http://localhost:50051";
    private const string DefaultServerPath = "/usr/local/bin/photopipeline-server";

    protected SystemTestBase()
    {
        Client = new GrpcClientService(ServerAddress);
        PipelineService = new PipelineService(Client);
        ImageService = new ImageService(Client);
    }

    protected async Task StartServerAsync()
    {
        if (!File.Exists(GetServerPath()))
        {
            throw new SkipTestException("Server binary not found. Skipping system test.");
        }

        ServerProcess = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = GetServerPath(),
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            }
        };

        ServerProcess.Start();

        await Task.Delay(2000);

        try
        {
            await Client.ConnectAsync(new CancellationTokenSource(5000).Token);
        }
        catch
        {
            StopServer();
            throw new SkipTestException("Could not connect to server. Skipping system test.");
        }
    }

    protected void StopServer()
    {
        if (ServerProcess is { HasExited: false })
        {
            ServerProcess.Kill(entireProcessTree: true);
            ServerProcess.WaitForExit(3000);
            ServerProcess.Dispose();
            ServerProcess = null;
        }
    }

    protected virtual string GetServerPath()
    {
        return Environment.GetEnvironmentVariable("PHOTOPIPELINE_SERVER_PATH")
               ?? DefaultServerPath;
    }

    protected static string GetTestDataPath(string relativePath)
    {
        var env = Environment.GetEnvironmentVariable("PHOTOPIPELINE_TEST_DATA");
        if (!string.IsNullOrEmpty(env))
            return Path.Combine(env, relativePath);

        var candidates = new[]
        {
            Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "SystemTests", "TestData", relativePath),
            Path.Combine(AppContext.BaseDirectory, "TestData", relativePath),
            Path.Combine(Directory.GetCurrentDirectory(), "TestData", relativePath)
        };

        foreach (var path in candidates)
        {
            if (File.Exists(path))
                return path;
        }

        return candidates[0];
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        StopServer();
        Client.Dispose();
    }
}

public sealed class SkipTestException : Exception
{
    public SkipTestException(string message) : base(message) { }
}
