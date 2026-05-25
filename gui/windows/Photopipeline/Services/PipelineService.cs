using Microsoft.Extensions.Logging;
using Photopipeline.Models;

namespace Photopipeline.Services;

public sealed class PipelineService : IPipelineService
{
    private readonly GrpcClientService _grpc;
    private readonly ILogger<PipelineService> _logger;

    public PipelineService(GrpcClientService grpc, ILogger<PipelineService> logger)
    {
        _grpc = grpc;
        _logger = logger;
    }

    public async Task<string> CreatePipelineAsync(PipelineSpec spec, CancellationToken ct = default)
    {
        var protoSpec = ToProtoSpec(spec);
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
        var result = await client.CreatePipelineAsync(protoSpec, cancellationToken: ct);
        _logger.LogInformation("Pipeline created: {Id}", result.Id);
        return result.Id;
    }

    public async Task<ValidationResult> ValidateAsync(PipelineSpec spec, CancellationToken ct = default)
    {
        var protoSpec = ToProtoSpec(spec);
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
        var protoResult = await client.ValidateAsync(protoSpec, cancellationToken: ct);

        return new ValidationResult
        {
            Valid = protoResult.Valid,
            Issues = protoResult.Issues.Select(i => new ValidationIssue
            {
                Severity = i.Severity switch
                {
                    global::Photopipeline.Pipeline.ValidationIssue.Types.Severity.Info => ValidationSeverity.Info,
                    global::Photopipeline.Pipeline.ValidationIssue.Types.Severity.Warning => ValidationSeverity.Warning,
                    global::Photopipeline.Pipeline.ValidationIssue.Types.Severity.Error => ValidationSeverity.Error,
                    _ => ValidationSeverity.Info
                },
                Param = i.Param,
                Message = i.Message
            }).ToList()
        };
    }

    public async IAsyncEnumerable<ExecuteProgress> ExecuteAsync(
        string pipelineId, string imagePath, string outputPath,
        [System.Runtime.CompilerServices.EnumeratorCancellation] CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
        var request = new global::Photopipeline.Pipeline.ExecuteRequest
        {
            PipelineId = pipelineId,
            ImagePath = imagePath,
            OutputPath = outputPath
        };

        using var call = client.Execute(request, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            var p = call.ResponseStream.Current;
            yield return new ExecuteProgress
            {
                Stage = p.Stage switch
                {
                    global::Photopipeline.Pipeline.ExecuteProgress.Types.Stage.Loading => ExecuteStage.Loading,
                    global::Photopipeline.Pipeline.ExecuteProgress.Types.Stage.Decoding => ExecuteStage.Decoding,
                    global::Photopipeline.Pipeline.ExecuteProgress.Types.Stage.Processing => ExecuteStage.Processing,
                    global::Photopipeline.Pipeline.ExecuteProgress.Types.Stage.Encoding => ExecuteStage.Encoding,
                    global::Photopipeline.Pipeline.ExecuteProgress.Types.Stage.Done => ExecuteStage.Done,
                    global::Photopipeline.Pipeline.ExecuteProgress.Types.Stage.Error => ExecuteStage.Error,
                    _ => ExecuteStage.Processing
                },
                NodeId = p.NodeId,
                NodeLabel = p.NodeLabel,
                Fraction = p.Fraction,
                Message = p.Message,
                ElapsedMs = p.ElapsedMs
            };
        }
    }

    public async Task<NodeSchema> GetNodeSchemaAsync(string pluginId, CancellationToken ct = default)
    {
        var channel = await _grpc.GetChannelAsync(ct);
        var client = new global::Photopipeline.Pipeline.PipelineService.PipelineServiceClient(channel);
        var protoSchema = await client.GetNodeSchemaAsync(
            new global::Photopipeline.Pipeline.PluginId { Id = pluginId },
            cancellationToken: ct);

        return new NodeSchema
        {
            PluginId = protoSchema.PluginId,
            Name = protoSchema.Name,
            Version = protoSchema.Version,
            Category = protoSchema.Category,
            Description = protoSchema.Description,
            ParameterSchema = ConvertStruct(protoSchema.ParameterSchema),
            GuiSchema = ConvertStruct(protoSchema.GuiSchema)
        };
    }

    private static global::Photopipeline.Pipeline.PipelineSpec ToProtoSpec(PipelineSpec spec)
    {
        var proto = new global::Photopipeline.Pipeline.PipelineSpec { Name = spec.Name };
        foreach (var node in spec.Nodes)
        {
            proto.Nodes.Add(new global::Photopipeline.Pipeline.PipelineNode
            {
                Id = node.Id,
                PluginId = node.PluginId,
                Label = node.Label,
                Enabled = node.Enabled
            });
        }
        foreach (var edge in spec.Edges)
        {
            proto.Edges.Add(new global::Photopipeline.Pipeline.PipelineEdge
            {
                From = edge.From,
                To = edge.To
            });
        }
        return proto;
    }

    private static Dictionary<string, object> ConvertStruct(Google.Protobuf.WellKnownTypes.Struct? s)
    {
        var result = new Dictionary<string, object>();
        if (s is null) return result;
        foreach (var (key, value) in s.Fields)
            result[key] = ConvertValue(value);
        return result;
    }

    private static object ConvertValue(Google.Protobuf.WellKnownTypes.Value v) => v.KindCase switch
    {
        Google.Protobuf.WellKnownTypes.Value.KindOneofCase.StringValue => v.StringValue,
        Google.Protobuf.WellKnownTypes.Value.KindOneofCase.NumberValue => v.NumberValue,
        Google.Protobuf.WellKnownTypes.Value.KindOneofCase.BoolValue => v.BoolValue,
        Google.Protobuf.WellKnownTypes.Value.KindOneofCase.StructValue => ConvertStruct(v.StructValue),
        Google.Protobuf.WellKnownTypes.Value.KindOneofCase.ListValue => v.ListValue.Values.Select(ConvertValue).ToList(),
        _ => string.Empty
    };
}
