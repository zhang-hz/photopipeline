using Microsoft.Extensions.Logging;
using Moq;
using Photopipeline.Helpers;
using Photopipeline.Models;
using Photopipeline.Services;
using Photopipeline.ViewModels;

namespace Photopipeline.Tests.UnitTests.ViewModels;

/// <summary>
/// Layer 3 unit tests for SettingsViewModel.
/// Uses MockBehavior.Strict for service mocks. Every test has a FAIL-able assertion.
/// </summary>
public sealed class SettingsViewModelTests : IDisposable
{
    private readonly List<Mock> _strictMocks = new();

    public void Dispose()
    {
        foreach (var mock in _strictMocks)
            mock.VerifyAll();
    }

    private Mock<T> Strict<T>() where T : class
    {
        var mock = new Mock<T>(MockBehavior.Strict);
        _strictMocks.Add(mock);
        return mock;
    }

    private static ILogger<T> AnyLogger<T>() => Mock.Of<ILogger<T>>();

    private SettingsViewModel CreateVm(Mock<ISettingsService>? settingsMock = null)
    {
        var sm = settingsMock ?? Strict<ISettingsService>();
        if (settingsMock is null)
            sm.Setup(s => s.Current).Returns(new AppSettings());
        return new SettingsViewModel(AnyLogger<SettingsViewModel>(), sm.Object);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 001: InitialState_LoadsFromCurrentSettings
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_001_InitialState_LoadsFromCurrentSettings()
    {
        var mock = Strict<ISettingsService>();
        mock.Setup(s => s.Current).Returns(new AppSettings
        {
            Theme = "Light",
            ServerPort = 6000,
            DefaultOutputFormat = "JPEG",
            JpegQuality = 85,
            ThumbnailSize = 128,
            MaxRecentFiles = 20
        });

        var vm = new SettingsViewModel(AnyLogger<SettingsViewModel>(), mock.Object);

        vm.Theme.Should().Be("Light");
        vm.ServerPort.Should().Be(6000);
        vm.DefaultOutputFormat.Should().Be("JPEG");
        vm.JpegQuality.Should().Be(85);
        vm.ThumbnailSize.Should().Be(128);
        vm.MaxRecentFiles.Should().Be(20);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 002: DefaultValues_FromFreshAppSettings
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_002_DefaultValues_FromFreshAppSettings()
    {
        var mock = Strict<ISettingsService>();
        mock.Setup(s => s.Current).Returns(new AppSettings());

        var vm = new SettingsViewModel(AnyLogger<SettingsViewModel>(), mock.Object);

        vm.Theme.Should().Be("Dark");
        vm.ServerPort.Should().Be(50051);
        vm.AutoStartServer.Should().BeTrue();
        vm.ThumbnailSize.Should().Be(256);
        vm.MaxRecentFiles.Should().Be(10);
        vm.EmbedMetadata.Should().BeTrue();
        vm.DefaultOutputFormat.Should().Be("TIFF");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 003: SaveCommand_CallsSettingsServiceSaveAsync
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_003_SaveCommand_CallsSettingsServiceSaveAsync()
    {
        var mock = Strict<ISettingsService>();
        mock.Setup(s => s.Current).Returns(new AppSettings { Theme = "Dark" });
        mock.Setup(s => s.SaveAsync(It.IsAny<AppSettings>(), It.IsAny<CancellationToken>()))
            .Returns(Task.CompletedTask);

        var vm = new SettingsViewModel(AnyLogger<SettingsViewModel>(), mock.Object);
        vm.Theme = "Light";
        vm.ServerPort = 12345;

        vm.SaveCommand.Execute(null);

        mock.Verify(s => s.SaveAsync(
            It.Is<AppSettings>(a => a.Theme == "Light" && a.ServerPort == 12345),
            It.IsAny<CancellationToken>()), Times.Once);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 004: ResetCommand_CallsSettingsServiceResetAsync
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_004_ResetCommand_CallsSettingsServiceResetAsync()
    {
        var mock = Strict<ISettingsService>();
        mock.Setup(s => s.Current).Returns(new AppSettings
        {
            Theme = "Light",
            ServerPort = 6000
        });
        mock.Setup(s => s.ResetAsync(It.IsAny<CancellationToken>()))
            .Callback<CancellationToken>(_ => { })
            .Returns(Task.CompletedTask);

        var vm = new SettingsViewModel(AnyLogger<SettingsViewModel>(), mock.Object);
        vm.Theme.Should().Be("Light"); // Precondition

        vm.ResetCommand.Execute(null);

        mock.Verify(s => s.ResetAsync(It.IsAny<CancellationToken>()), Times.Once);
    }

    // ═════════════════════════════════════════════════════════════
    // Test 005: Theme_Property_CanBeChanged
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_005_ThemeProperty_CanBeChanged()
    {
        var vm = CreateVm();

        vm.Theme = "Dark";
        vm.Theme.Should().Be("Dark");
        vm.Theme = "Light";
        vm.Theme.Should().Be("Light");
    }

    // ═════════════════════════════════════════════════════════════
    // Test 006: Themes_ContainsDarkAndLight
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_006_Themes_ContainsDarkAndLight()
    {
        var vm = CreateVm();

        vm.Themes.Should().Contain(new[] { "Dark", "Light" });
    }

    // ═════════════════════════════════════════════════════════════
    // Test 007: OutputFormats_ContainsAllExpectedFormats
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_007_OutputFormats_ContainsAllExpectedFormats()
    {
        var vm = CreateVm();

        vm.OutputFormats.Should().Contain(new[] { "TIFF", "JPEG", "PNG", "WebP", "HEIF", "AVIF", "JPEG XL" });
    }

    // ═════════════════════════════════════════════════════════════
    // Test 008: ServerPath_AndPort_CanBeChanged
    // ═════════════════════════════════════════════════════════════
    [Fact]
    public void Test_008_ServerPathAndPort_CanBeChanged()
    {
        var vm = CreateVm();

        var propertyChangedNames = new List<string>();
        vm.PropertyChanged += (s, e) => propertyChangedNames.Add(e.PropertyName!);

        vm.ServerPath = @"C:\tools\server.exe";
        vm.ServerPort = 9999;
        vm.AutoStartServer = false;

        vm.ServerPath.Should().Be(@"C:\tools\server.exe");
        vm.ServerPort.Should().Be(9999);
        vm.AutoStartServer.Should().BeFalse();
        propertyChangedNames.Should().Contain(nameof(vm.ServerPath),
            "ServerPath setter should fire PropertyChanged");
        propertyChangedNames.Should().Contain(nameof(vm.ServerPort),
            "ServerPort setter should fire PropertyChanged");
        propertyChangedNames.Should().Contain(nameof(vm.AutoStartServer),
            "AutoStartServer setter should fire PropertyChanged");
    }
}
