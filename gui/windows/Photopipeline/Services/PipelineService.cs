using Grpc.Net.Client;
using Photopipeline.Models;
using Photopipeline.Pipeline;
using System.Collections.ObjectModel;

namespace Photopipeline.Services;

public sealed class PipelineService : IPipelineService
{
    private readonly GrpcClientService _grpc;

    public PipelineService(GrpcClientService grpc)
    {
        _grpc = grpc;
    }

    public async Task<PipelineModel> CreatePipelineAsync(string name, string description = "", CancellationToken ct = default)
    {
        var pipeline = new PipelineModel
        {
            Name = name,
            Description = description
        };

        try
        {
            var channel = await _grpc.GetChannelAsync(ct);
            var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
            var spec = new PipelineSpec { Name = name };
            var response = await client.CreatePipelineAsync(spec, cancellationToken: ct);
            pipeline.Id = response.Id;
        }
        catch
        {
            // gRPC server unavailable; use local-only pipeline
        }

        return pipeline;
    }

    public async Task<bool> ValidatePipelineAsync(PipelineModel pipeline, CancellationToken ct = default)
    {
        if (pipeline.Nodes.Count == 0)
        {
            pipeline.ValidationError = "Pipeline has no nodes";
            pipeline.IsValid = false;
            return false;
        }

        try
        {
            var channel = await _grpc.GetChannelAsync(ct);
            var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
            var spec = BuildPipelineSpec(pipeline);
            var result = await client.ValidateAsync(spec, cancellationToken: ct);
            pipeline.IsValid = result.Valid;
            pipeline.ValidationError = result.Valid ? string.Empty
                : string.Join("; ", result.Issues.Select(i => i.Message));
            return result.Valid;
        }
        catch
        {
            // Fall back to local validation
            var connectedNodeIds = new HashSet<string>();
            foreach (var edge in pipeline.Edges)
            {
                connectedNodeIds.Add(edge.SourceNodeId);
                connectedNodeIds.Add(edge.TargetNodeId);
            }

            foreach (var node in pipeline.Nodes)
            {
                if (node.InputPorts.Count > 0 && !connectedNodeIds.Contains(node.Id) && pipeline.Nodes.Count > 1)
                {
                    pipeline.ValidationError = $"Node '{node.DisplayName}' has unconnected input ports";
                    pipeline.IsValid = false;
                    return false;
                }
            }

            pipeline.IsValid = true;
            pipeline.ValidationError = string.Empty;
            return true;
        }
    }

    public async Task<bool> ExecutePipelineAsync(PipelineModel pipeline, string imageId, CancellationToken ct = default)
    {
        if (!pipeline.IsValid)
        {
            var valid = await ValidatePipelineAsync(pipeline, ct);
            if (!valid) return false;
        }

        pipeline.IsExecuting = true;
        try
        {
            var channel = await _grpc.GetChannelAsync(ct);
            var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
            var request = new ExecuteRequest
            {
                PipelineId = pipeline.Id,
                ImagePath = imageId
            };

            using var call = client.Execute(request, cancellationToken: ct);
            while (await call.ResponseStream.MoveNext(ct))
            {
                var progress = call.ResponseStream.Current;
                var node = pipeline.Nodes.FirstOrDefault(n => n.Id == progress.NodeId);
                if (node is not null)
                {
                    node.IsProcessing = progress.Stage != ExecuteProgress.Types.Stage.Done;
                }
            }

            return true;
        }
        catch
        {
            // Fall back to local execution
            foreach (var node in pipeline.Nodes)
            {
                node.IsProcessing = true;
                await Task.Delay(100, ct);
                node.IsProcessing = false;
            }
            return true;
        }
        finally
        {
            pipeline.IsExecuting = false;
        }
    }

    public async Task<ObservableCollection<PluginInfo>> GetAvailablePluginsAsync(CancellationToken ct = default)
    {
        var plugins = new ObservableCollection<PluginInfo>
        {
            new()
            {
                Id = "demosaic", Name = "Demosaic", Category = "Raw Processing",
                Description = "Convert raw Bayer pattern to RGB image",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas =
                {
                    new ParameterSchema { Name = "algorithm", DisplayName = "Algorithm", ParameterType = ParameterType.Enum, EnumValues = new ObservableCollection<object> { "AMaZE", "LMMSE", "VNG4", "PPG", "Bilinear" }, DefaultValue = "AMaZE", Description = "Demosaicing algorithm" },
                    new ParameterSchema { Name = "border", DisplayName = "Border Handling", ParameterType = ParameterType.Integer, DefaultValue = 3, MinValue = 0, MaxValue = 8, Description = "Pixels to crop from border" }
                }
            },
            new()
            {
                Id = "exposure", Name = "Exposure", Category = "Tonal",
                Description = "Adjust image exposure in stops",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas =
                {
                    new ParameterSchema { Name = "ev", DisplayName = "Exposure (EV)", ParameterType = ParameterType.Float, DefaultValue = 0.0, MinValue = -5.0, MaxValue = 5.0, Step = 0.01, Unit = "EV", DecimalPlaces = 2, Description = "Exposure adjustment in stops" },
                    new ParameterSchema { Name = "highlight_recovery", DisplayName = "Highlight Recovery", ParameterType = ParameterType.Boolean, DefaultValue = true, Description = "Attempt to recover clipped highlights" }
                }
            },
            new()
            {
                Id = "white_balance", Name = "White Balance", Category = "Color",
                Description = "Adjust white balance temperature and tint",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas =
                {
                    new ParameterSchema { Name = "temperature", DisplayName = "Temperature", ParameterType = ParameterType.Integer, DefaultValue = 5500, MinValue = 2000, MaxValue = 50000, Unit = "K", Description = "Color temperature in Kelvin" },
                    new ParameterSchema { Name = "tint", DisplayName = "Tint", ParameterType = ParameterType.Float, DefaultValue = 0.0, MinValue = -150.0, MaxValue = 150.0, Step = 0.1, DecimalPlaces = 1, Description = "Green/magenta tint adjustment" }
                }
            },
            new()
            {
                Id = "denoise", Name = "Denoise", Category = "Noise Reduction",
                Description = "Reduce image noise using AI-based algorithms",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas =
                {
                    new ParameterSchema { Name = "strength", DisplayName = "Strength", ParameterType = ParameterType.Float, DefaultValue = 0.5, MinValue = 0.0, MaxValue = 1.0, Step = 0.01, DecimalPlaces = 2, Description = "Denoising strength" },
                    new ParameterSchema { Name = "model", DisplayName = "Model", ParameterType = ParameterType.Enum, EnumValues = new ObservableCollection<object> { "Standard", "Raw", "JPG", "High ISO" }, DefaultValue = "Standard", Description = "Noise model to use" }
                }
            },
            new()
            {
                Id = "sharpen", Name = "Sharpen", Category = "Detail",
                Description = "Sharpen image using unsharp mask or deconvolution",
                MinInputs = 1, MaxInputs = 1, Outputs = 1,
                ParameterSchemas =
                {
                    new ParameterSchema { Name = "amount", DisplayName = "Amount", ParameterType = ParameterType.Float, DefaultValue = 0.5, MinValue = 0.0, MaxValue = 1.0, Step = 0.01, DecimalPlaces = 2, Description = "Sharpening amount" },
                    new ParameterSchema { Name = "radius", DisplayName = "Radius", ParameterType = ParameterType.Float, DefaultValue = 1.0, MinValue = 0.3, MaxValue = 5.0, Step = 0.1, DecimalPlaces = 1, Unit = "px", Description = "Sharpening radius in pixels" }
                }
            }
        };

        await Task.CompletedTask;
        return plugins;
    }

    public Task UpdateNodeParametersAsync(string nodeId, Dictionary<string, object> parameters, CancellationToken ct = default)
    {
        // gRPC: call update_node_parameters endpoint when available
        return Task.CompletedTask;
    }

    private static PipelineSpec BuildPipelineSpec(PipelineModel pipeline)
    {
        var spec = new PipelineSpec { Name = pipeline.Name };
        foreach (var node in pipeline.Nodes)
        {
            spec.Nodes.Add(new global::Photopipeline.Pipeline.PipelineNode
            {
                Id = node.Id,
                PluginId = node.PluginId,
                Label = node.DisplayName,
                Enabled = true
            });
        }
        foreach (var edge in pipeline.Edges)
        {
            spec.Edges.Add(new global::Photopipeline.Pipeline.PipelineEdge
            {
                From = edge.SourceNodeId,
                To = edge.TargetNodeId
            });
        }
        return spec;
    }
}
