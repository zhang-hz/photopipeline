namespace Photopipeline.Tests.FunctionalTests.UiChannel;

public class UiTestDriver : UiTestBase
{
    // This class provides high-level UI operation wrappers for the test cases.
    // In Phase 5, FlaUI integration will be added for actual UI automation.

    public UiTestDriver() : base() { }

    public async Task<string> RunFullWorkflowAsync(
        string inputImagePath,
        PipelineSpec pipeline,
        Dictionary<string, Dictionary<string, object>>? nodeParameters = null,
        string? outputFormat = null,
        CancellationToken ct = default)
    {
        StartApp();
        try
        {
            // Step 1: Import image
            await ImportImageAsync(inputImagePath, ct);

            // Step 2: Add nodes to pipeline (one per plugin)
            foreach (var node in pipeline.Nodes)
            {
                await AddNodeToPipelineAsync(node.PluginId, ct);
            }

            // Step 3: Set node parameters if provided
            if (nodeParameters != null)
            {
                foreach (var (nodeId, parameters) in nodeParameters)
                {
                    foreach (var (key, value) in parameters)
                    {
                        await SetNodeParameterAsync(nodeId, key, value, ct);
                    }
                }
            }

            // Step 4: Run pipeline
            await RunPipelineAsync(ct);

            // Step 5: Wait for completion
            await WaitForPipelineCompletionAsync(ct);

            // Step 6: Export output
            var outputPath = Path.Combine(Path.GetTempPath(), $"pp_ui_test_{Guid.NewGuid():N}.tif");
            await ExportOutputAsync(outputPath, outputFormat, ct);

            return outputPath;
        }
        catch
        {
            StopApp();
            throw;
        }
    }

    // ── Operation stubs — to be implemented with FlaUI in Phase 5 ──

    public Task ImportImageAsync(string path, CancellationToken ct = default)
    {
        // TODO Phase 5: Click Import button, select file in dialog
        return Task.CompletedTask;
    }

    public Task SelectImageAsync(string fileName, CancellationToken ct = default)
    {
        // TODO Phase 5: Click image in filmstrip
        return Task.CompletedTask;
    }

    public Task AddNodeToPipelineAsync(string pluginId, CancellationToken ct = default)
    {
        // TODO Phase 5: Find plugin in browser, click "Add to Pipeline" or drag
        return Task.CompletedTask;
    }

    public Task SetNodeParameterAsync(string nodeId, string paramKey, object value, CancellationToken ct = default)
    {
        // TODO Phase 5: Find parameter control, set value
        return Task.CompletedTask;
    }

    public Task RunPipelineAsync(CancellationToken ct = default)
    {
        // TODO Phase 5: Click Run button
        return Task.CompletedTask;
    }

    public Task WaitForPipelineCompletionAsync(CancellationToken ct = default)
    {
        // TODO Phase 5: Monitor progress indicator until Done
        return Task.CompletedTask;
    }

    public Task ExportOutputAsync(string outputPath, string? format = null, CancellationToken ct = default)
    {
        // TODO Phase 5: Click Export button, enter path, confirm
        return Task.CompletedTask;
    }

    public Task CancelPipelineAsync(CancellationToken ct = default)
    {
        // TODO Phase 5: Click Cancel button
        return Task.CompletedTask;
    }

    public Task ToggleSplitViewAsync(CancellationToken ct = default)
    {
        // TODO Phase 5: Click split view toggle
        return Task.CompletedTask;
    }
}
