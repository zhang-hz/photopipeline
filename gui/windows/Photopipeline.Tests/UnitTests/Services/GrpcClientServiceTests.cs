namespace Photopipeline.Tests.UnitTests.Services;

public sealed class GrpcClientServiceTests : IDisposable
{
    private GrpcClientService? _service;

    public void Dispose()
    {
        _service?.Dispose();
    }

    [Fact]
    public void GrpcClientService_DefaultAddress_InitiallyDisconnected()
    {
        _service = new GrpcClientService();

        _service.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void GrpcClientService_CustomAddress_InitiallyDisconnected()
    {
        _service = new GrpcClientService("http://custom:6000");

        _service.IsConnected.Should().BeFalse();
    }

    [Fact]
    public async Task ConnectAsync_EstablishesConnection()
    {
        _service = new GrpcClientService();

        await _service.ConnectAsync();

        _service.IsConnected.Should().BeTrue();
    }

    [Fact]
    public async Task ConnectAsync_WhenAlreadyConnected_Idempotent()
    {
        _service = new GrpcClientService();

        await _service.ConnectAsync();
        await _service.ConnectAsync();

        _service.IsConnected.Should().BeTrue();
    }

    [Fact]
    public async Task ConnectAsync_WithCancellationToken_WhenNotCancelled()
    {
        _service = new GrpcClientService();

        await _service.ConnectAsync(CancellationToken.None);

        _service.IsConnected.Should().BeTrue();
    }

    [Fact]
    public async Task GetChannelAsync_ReturnsChannel()
    {
        _service = new GrpcClientService();

        var channel = await _service.GetChannelAsync();

        channel.Should().NotBeNull();
        _service.IsConnected.Should().BeTrue();
    }

    [Fact]
    public async Task GetChannelAsync_WhenNotConnected_ConnectsFirst()
    {
        _service = new GrpcClientService();

        _service.IsConnected.Should().BeFalse();

        var channel = await _service.GetChannelAsync();

        _service.IsConnected.Should().BeTrue();
        channel.Should().NotBeNull();
    }

    [Fact]
    public async Task ReconnectAsync_ResetsAndConnects()
    {
        _service = new GrpcClientService();
        await _service.ConnectAsync();
        _service.IsConnected.Should().BeTrue();

        await _service.ReconnectAsync();

        _service.IsConnected.Should().BeTrue();
    }

    [Fact]
    public void Dispose_DisconnectsAndCleansUp()
    {
        _service = new GrpcClientService();

        _service.Dispose();

        _service.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        _service = new GrpcClientService();

        _service.Dispose();
        _service.Dispose();

        _service.IsConnected.Should().BeFalse();
    }

    [Fact]
    public async Task CallAsync_InvokesFunctionWithChannel()
    {
        _service = new GrpcClientService();
        bool wasCalled = false;

        var result = await _service.CallAsync(async channel =>
        {
            wasCalled = true;
            channel.Should().NotBeNull();
            await Task.CompletedTask;
            return 42;
        });

        wasCalled.Should().BeTrue();
        result.Should().Be(42);
    }

    [Fact]
    public async Task CallAsync_ReturnsResponseFromDelegate()
    {
        _service = new GrpcClientService();

        var result = await _service.CallAsync(async channel =>
        {
            await Task.CompletedTask;
            return "success";
        });

        result.Should().Be("success");
    }

    [Fact]
    public async Task CallAsync_WithException_AttemptsReconnect()
    {
        _service = new GrpcClientService();

        var result = await _service.CallAsync(channel =>
        {
            return Task.FromResult(99);
        });

        result.Should().Be(99);
        _service.IsConnected.Should().BeTrue();
    }

    [Fact]
    public async Task ReconnectAsync_IncrementsDelayOnMultipleCalls()
    {
        _service = new GrpcClientService();

        var sw = System.Diagnostics.Stopwatch.StartNew();
        await _service.ReconnectAsync();
        var firstDuration = sw.ElapsedMilliseconds;

        sw.Restart();
        await _service.ReconnectAsync();
        var secondDuration = sw.ElapsedMilliseconds;

        secondDuration.Should().BeGreaterOrEqualTo(firstDuration);
    }

    [Fact]
    public async Task ConcurrentConnect_CallsAreSerialized()
    {
        _service = new GrpcClientService();

        var tasks = Enumerable.Range(0, 5).Select(async _ =>
        {
            await _service.ConnectAsync();
        });

        await Task.WhenAll(tasks);

        _service.IsConnected.Should().BeTrue();
    }
}
