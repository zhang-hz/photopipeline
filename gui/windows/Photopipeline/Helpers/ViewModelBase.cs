using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;

namespace Photopipeline.Helpers;

public abstract partial class ViewModelBase : ObservableObject
{
    [ObservableProperty]
    private bool _isBusy;

    [ObservableProperty]
    private string? _errorMessage;

    [ObservableProperty]
    private string _statusMessage = "Ready";

    protected readonly ILogger Logger;

    private readonly CancellationTokenSource _shutdownCts = new();
    protected CancellationToken ShutdownToken => _shutdownCts.Token;

    protected ViewModelBase(ILogger logger)
    {
        Logger = logger;
    }

    protected async Task ExecuteAsync(Func<CancellationToken, Task> operation,
        string? errorContext = null, CancellationToken ct = default)
    {
        if (IsBusy) return;
        try
        {
            IsBusy = true;
            ErrorMessage = null;
            using var linked = CancellationTokenSource.CreateLinkedTokenSource(ct, _shutdownCts.Token);
            await operation(linked.Token);
        }
        catch (OperationCanceledException) { }
        catch (Exception ex)
        {
            ErrorMessage = $"Operation failed: {ex.Message}";
            Logger.LogError(ex, "Error during {Context}", errorContext ?? "operation");
        }
        finally
        {
            IsBusy = false;
        }
    }

    protected void ExecuteSync(Action action, string? errorContext = null)
    {
        try
        {
            ErrorMessage = null;
            action();
        }
        catch (Exception ex)
        {
            ErrorMessage = $"Operation failed: {ex.Message}";
            Logger.LogError(ex, "Error during {Context}", errorContext ?? "operation");
        }
    }

    public virtual void Shutdown()
    {
        _shutdownCts.Cancel();
    }
}
