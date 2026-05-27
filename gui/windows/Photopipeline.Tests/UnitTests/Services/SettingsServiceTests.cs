using Microsoft.Extensions.Logging.Abstractions;

namespace Photopipeline.Tests.UnitTests.Services;

public sealed class SettingsServiceTests : IDisposable
{
    private readonly string _tempDir;
    private readonly string _settingsPath;

    public SettingsServiceTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "PhotopipelineTests_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_tempDir);
        _settingsPath = Path.Combine(_tempDir, "appsettings.json");
    }

    public void Dispose()
    {
        try { Directory.Delete(_tempDir, recursive: true); } catch { /* best effort */ }
    }

    private SettingsService CreateService()
        => new(NullLogger<SettingsService>.Instance, _settingsPath);

    // ── Test 1: SaveAsync then LoadAsync round-trip ──
    [Fact]
    public async Task SaveAndLoad_RoundTrip_PreservesAllValues()
    {
        var service = CreateService();
        var settings = new AppSettings
        {
            Theme = "Light",
            ServerPort = 12345,
            AutoStartServer = false,
            ThumbnailSize = 512,
            MaxRecentFiles = 20,
            DefaultOutputFormat = "JPEG",
            JpegQuality = 80,
            EmbedMetadata = false,
            WindowWidth = 1920,
            WindowHeight = 1080
        };

        await service.SaveAsync(settings);

        var loaded = new SettingsService(NullLogger<SettingsService>.Instance, _settingsPath);
        await loaded.LoadAsync();

        loaded.Current.Theme.Should().Be("Light");
        loaded.Current.ServerPort.Should().Be(12345);
        loaded.Current.AutoStartServer.Should().BeFalse();
        loaded.Current.ThumbnailSize.Should().Be(512);
        loaded.Current.MaxRecentFiles.Should().Be(20);
        loaded.Current.DefaultOutputFormat.Should().Be("JPEG");
        loaded.Current.JpegQuality.Should().Be(80);
        loaded.Current.EmbedMetadata.Should().BeFalse();
        loaded.Current.WindowWidth.Should().Be(1920);
        loaded.Current.WindowHeight.Should().Be(1080);
    }

    // ── Test 2: ResetAsync restores default AppSettings ──
    [Fact]
    public async Task ResetAsync_RestoresDefaults()
    {
        var service = CreateService();
        // First save non-default settings
        await service.SaveAsync(new AppSettings
        {
            Theme = "Light",
            ServerPort = 99999,
            ThumbnailSize = 1024
        });

        await service.ResetAsync();

        service.Current.Theme.Should().Be("Dark");
        service.Current.ServerPort.Should().Be(50051);
        service.Current.AutoStartServer.Should().BeTrue();
        service.Current.ThumbnailSize.Should().Be(256);
        service.Current.DefaultOutputFormat.Should().Be("TIFF");
    }

    // ── Test 3: Loading from missing file returns defaults ──
    [Fact]
    public async Task LoadAsync_FileNotFound_ReturnsDefaults()
    {
        var service = CreateService();
        // _settingsPath does not exist, so LoadAsync must fall back to defaults

        await service.LoadAsync();

        service.Current.Should().NotBeNull();
        service.Current.Theme.Should().Be("Dark");
        service.Current.ServerPort.Should().Be(50051);
    }

    // ── Test 4: SaveAsync fires SettingsChanged event ──
    [Fact]
    public async Task SaveAsync_FiresSettingsChanged()
    {
        var service = CreateService();
        AppSettings? received = null;
        service.SettingsChanged += (_, s) => received = s;

        var settings = new AppSettings { Theme = "DarkBlue" };
        await service.SaveAsync(settings);

        received.Should().NotBeNull();
        received!.Theme.Should().Be("DarkBlue");
    }

    // ── Test 5: SaveAsync persists to disk ──
    [Fact]
    public async Task SaveAsync_WritesToDisk()
    {
        var service = CreateService();
        await service.SaveAsync(new AppSettings { Theme = "DiskTest" });

        File.Exists(_settingsPath).Should().BeTrue("SaveAsync must create the settings file");
        var content = await File.ReadAllTextAsync(_settingsPath);
        content.Should().Contain("\"Theme\"");
        content.Should().Contain("DiskTest");
    }
}
