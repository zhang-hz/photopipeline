using Microsoft.Extensions.Logging;
using Moq;

namespace Photopipeline.Tests.UnitTests.ViewModels;

public sealed class SettingsViewModelTests
{
    private static SettingsViewModel Create(Mock<ISettingsService>? settingsServiceMock = null)
    {
        var logger = Mock.Of<ILogger<SettingsViewModel>>();
        var settingsService = settingsServiceMock ?? new Mock<ISettingsService>();
        if (settingsServiceMock is null)
            settingsService.Setup(s => s.Current).Returns(new AppSettings());
        return new SettingsViewModel(logger, settingsService.Object);
    }

    [Fact]
    public void InitialState_LoadsFromSettings()
    {
        var mock = new Mock<ISettingsService>();
        mock.Setup(s => s.Current).Returns(new AppSettings
        {
            Theme = "Light",
            ServerPort = 6000,
            DefaultOutputFormat = "JPEG",
            JpegQuality = 85
        });

        var vm = Create(mock);

        vm.Theme.Should().Be("Light");
        vm.ServerPort.Should().Be(6000);
        vm.DefaultOutputFormat.Should().Be("JPEG");
        vm.JpegQuality.Should().Be(85);
    }

    [Fact]
    public void Themes_ContainsDarkAndLight()
    {
        var vm = Create();

        vm.Themes.Should().Contain(new[] { "Dark", "Light" });
    }

    [Fact]
    public void OutputFormats_ContainsAllFormats()
    {
        var vm = Create();

        vm.OutputFormats.Should().Contain(new[] { "TIFF", "JPEG", "PNG", "WebP", "HEIF", "AVIF", "JPEG XL" });
    }

    [Fact]
    public void DefaultValues_FromNewSettings()
    {
        var vm = Create();

        vm.ServerPath.Should().Be("photopipeline-server.exe");
        vm.ServerPort.Should().Be(50051);
        vm.AutoStartServer.Should().BeTrue();
        vm.ThumbnailSize.Should().Be(256);
        vm.MaxRecentFiles.Should().Be(10);
        vm.EmbedMetadata.Should().BeTrue();
    }
}
