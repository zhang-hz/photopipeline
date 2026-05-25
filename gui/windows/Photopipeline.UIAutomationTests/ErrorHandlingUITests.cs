namespace Photopipeline.UIAutomationTests;

public sealed class ErrorHandlingUITests
{
    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void BackendDisconnect_IndicatorTurnsRed() { }

    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void InvalidOperation_ShowsErrorNotification() { }

    [Fact(Skip = "Requires UI context with WinAppDriver")]
    public void Timeout_ShowsRetryPrompt() { }
}
