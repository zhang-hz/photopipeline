using System.Windows;
using System.Windows.Threading;

namespace Photopipeline.Helpers;

public sealed class DispatcherHelper
{
    private static DispatcherHelper? _current;
    public static DispatcherHelper Current => _current ??= CreateCurrent();

    private readonly Dispatcher _dispatcher;

    public DispatcherHelper(Dispatcher dispatcher)
    {
        _dispatcher = dispatcher;
    }

    public void Invoke(Action action)
    {
        if (_dispatcher.CheckAccess())
            action();
        else
            _dispatcher.Invoke(action);
    }

    public Task InvokeAsync(Action action)
    {
        if (_dispatcher.CheckAccess())
        {
            action();
            return Task.CompletedTask;
        }
        return _dispatcher.InvokeAsync(action, DispatcherPriority.Normal).Task;
    }

    public Task<T> InvokeAsync<T>(Func<T> func)
    {
        if (_dispatcher.CheckAccess())
            return Task.FromResult(func());
        return _dispatcher.InvokeAsync(func, DispatcherPriority.Normal).Task;
    }

    public bool IsOnUIThread => _dispatcher.CheckAccess();

    private static DispatcherHelper CreateCurrent()
    {
        var app = Application.Current;
        if (app is null)
            return new DispatcherHelper(Dispatcher.CurrentDispatcher);
        return new DispatcherHelper(app.Dispatcher);
    }
}
