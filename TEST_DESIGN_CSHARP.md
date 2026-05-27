# Photopipeline C# 测试系统详细设计 (Layer 3-5)

**版本**: 1.0  
**日期**: 2026-05-26  
**基于**: TEST_CASE_SPECIFICATION.md Layer 3-5 + 六条铁律  
**总量**: C# 端 ~405 条 (Layer 3: 120 + Layer 4: 120 + Layer 5: 105 + Layer 6: 60)

---

## 六条铁律合规矩阵

每条铁律在后续各章节中均有对应实现。本矩阵列出每条铁律的核心合规手段：

| 铁律 | Layer 3 合规 | Layer 4 合规 | Layer 5 合规 |
|------|-------------|-------------|-------------|
| **铁律1: 每个测试必须有至少一个能 FAIL 的断言** | 每个测试结尾至少有 FluentAssertions `.Should().Be()` / Moq `.Verify()` 等断言 | 每个测试结尾调用 ImageAssert 系列方法 (PixelsEqual/PSNRAbove/SSIMAbove/IsValidFormat) | 每个测试结尾从磁盘读取输出文件，调用 ImageAssert 验证 |
| **铁律2: 禁止静默跳过** | try-catch 中 catch 块必须 Assert.Fail | ApiTestBase 中没有任何 try-catch-return 模式；连接失败直接 Assert.Fail | UiTestBase 中窗口找不到直接 Assert.Fail；不吞异常 |
| **铁律3: 基础设施必须先有消费者** | Moq Mock 跟随 ViewModel 测试同步创建 | SharedTestCaseLoader 在实现写第一个测试前即创建 | UiTestDriver 所有方法随第一个 E2E 测试实现 |
| **铁律4: UI 测试必须真正启动进程** | 不适用 (单元测试) | 不适用 (gRPC 调用) | FlaUI 启动真实 Photopipeline.exe 进程，操作真实 WPF 窗口 |
| **铁律5: 对抗性自查** | 每个 PR review checklist 包含 "改变 ViewModel 逻辑后这几个测试会 FAIL 吗?" | 每个 gRPC 用例的 golden 图像故意损坏一次验证 FAIL | 每写完一个 E2E 用例，在 Pipeline 空跑情况下验证测试确实 FAIL |
| **铁律6: 回归测试必须有黄金参考图像** | 不适用 (状态机验证) | Layer 2 生成的 golden 图像作为基准 | ReferenceImageGenerator 生成 golden 图像，逐像素比对 |

---

## A. 目录结构设计

### A.1 目录树

```
gui/windows/
├── Photopipeline.Tests/                          # 现有测试项目 (Layer 3 + Layer 4)
│   ├── Photopipeline.Tests.csproj                 # 修改: 添加 SkiaSharp, Moq, FluentAssertions
│   ├── UnitTests/                                 # Layer 3: C# 单元测试 (~120条)
│   │   ├── ViewModels/
│   │   │   ├── MainViewModelTests.cs              # 15条 (重写现有)
│   │   │   ├── FilmstripViewModelTests.cs          # 15条 (重写现有)
│   │   │   ├── PreviewViewModelTests.cs            # 12条 (重写现有)
│   │   │   ├── PipelineEditorViewModelTests.cs     # 18条 (重写现有)
│   │   │   ├── PluginBrowserViewModelTests.cs      # 10条 (重写现有)
│   │   │   ├── BatchViewModelTests.cs              # 15条 (重写现有)
│   │   │   └── SettingsViewModelTests.cs           # 8条 (重写现有)
│   │   └── Services/
│   │       ├── PipelineServiceTests.cs             # 4条 (新建, 替代现有反射检查)
│   │       ├── GrpcClientServiceTests.cs           # 4条 (新建)
│   │       ├── ImageServiceTests.cs                # 4条 (新建)
│   │       ├── BatchServiceTests.cs                # 4条 (新建)
│   │       ├── PluginServiceTests.cs               # 4条 (重写现有反射检查)
│   │       ├── BackendServiceTests.cs              # 4条 (新建)
│   │       └── SettingsServiceTests.cs             # 4条 (新建)
│   │
│   ├── FunctionalTests/                           # Layer 4: gRPC 集成测试
│   │   ├── ApiChannel/
│   │   │   ├── ApiTestBase.cs                     # 重写: 移除 try-catch-return
│   │   │   ├── PluginGrpcTests.cs                 # 44条 (重写现有)
│   │   │   ├── PipelineGrpcTests.cs               # 30条 (重写现有)
│   │   │   ├── FormatGrpcTests.cs                 # 15条 (重写现有)
│   │   │   ├── BatchGrpcTests.cs                  # 15条 (重写现有)
│   │   │   ├── ErrorPathGrpcTests.cs              # 10条 (重写现有)
│   │   │   └── ConcurrencyGrpcTests.cs            # 6条 (重写现有)
│   │   │
│   │   ├── CrossChannel/                          # Layer 6: 交叉验证测试
│   │   │   ├── CrossChannelTestBase.cs             # 重写: 异常传播，不吞异常
│   │   │   ├── PluginCrossTests.cs                # 20条 (重写现有)
│   │   │   ├── PipelineCrossTests.cs              # 15条 (重写现有)
│   │   │   ├── FormatCrossTests.cs                # 10条 (重写现有)
│   │   │   ├── BatchCrossTests.cs                 # 8条 (重写现有)
│   │   │   └── RegressionCrossTests.cs            # 7条 (重写现有)
│   │   │
│   │   ├── Infrastructure/                        # 共享基础设施
│   │   │   ├── ImageAssert.cs                     # 修改: 实现 MetadataMatches
│   │   │   ├── TestImageGenerator.cs              # 保留 (高质量, 完整)
│   │   │   ├── TestPipelineBuilder.cs             # 保留 (高质量, 完整)
│   │   │   ├── TestCaseDefinition.cs              # 保留 (已完整)
│   │   │   ├── TestCaseCatalog.cs                 # 保留 (已完整)
│   │   │   ├── TestDataCatalog.cs                 # 保留 (已完整)
│   │   │   ├── TestOutputManager.cs               # 保留 (已完整)
│   │   │   ├── CrossChannelVerifier.cs            # 保留 (高质量, 完整)
│   │   │   ├── SharedTestCaseLoader.cs            # 新建: JSON 反序列化
│   │   │   ├── ReferenceImageGenerator.cs          # 新建: 生成 golden 图像
│   │   │   ├── GrpcTestServerManager.cs           # 新建: 启动/管理 Rust server
│   │   │   ├── ImageMetadata.cs                   # 新建: 元数据模型
│   │   │   └── ResourceMonitor.cs                 # 保留 (质量一般但可用)
│   │   │
│   │   └── UiChannel/                             # 删除旧文件 (改为 UIAutomationTests)
│   │       └── (全部删除, 移入 Photopipeline.UIAutomationTests/)
│   │
│   ├── TestData/input/                            # 测试输入图像目录
│   │   └── manifest.json                          # TestImageGenerator 生成
│   │
│   └── ScenarioTests/                             # 删除 (旧垃圾代码)
│       └── (全部删除)
│
├── Photopipeline.UIAutomationTests/               # Layer 5: GUI FlaUI E2E 测试 (~105条)
│   ├── Photopipeline.UIAutomationTests.csproj      # 重写: 添加 FlaUI 依赖
│   ├── Framework/
│   │   ├── UiTestBase.cs                          # 新建: 进程启动/窗口就绪/清理
│   │   ├── UiTestDriver.cs                        # 新建: FlaUI UI 操作封装
│   │   ├── UiElementLocator.cs                    # 新建: 控件查找策略
│   │   └── FlaUIExtensions.cs                    # 新建: 扩展方法
│   │
│   ├── Tests/
│   │   ├── SinglePlugin/                          # 单插件工作流 (40条)
│   │   │   ├── RawInputTests.cs                   # GE2E-001~003 (3条)
│   │   │   ├── TransformTests.cs                  # GE2E-004~007 (4条)
│   │   │   ├── ColorspaceTests.cs                 # GE2E-008~011 (4条)
│   │   │   ├── Lut3DTests.cs                      # GE2E-012~015 (4条)
│   │   │   ├── LensCorrectTests.cs                # GE2E-016~019 (4条)
│   │   │   ├── AiDenoiseTests.cs                  # GE2E-020~022 (3条)
│   │   │   ├── ExifRwTests.cs                     # GE2E-023~025 (3条)
│   │   │   ├── GpsSetTests.cs                     # GE2E-026~028 (3条)
│   │   │   ├── TimeShiftTests.cs                  # GE2E-029~031 (3条)
│   │   │   └── EncoderTests.cs                    # GE2E-032~040 (9条)
│   │   │
│   │   ├── MultiPlugin/                           # 多插件工作流 (30条)
│   │   │   └── RealWorldWorkflowTests.cs           # GE2E-041~070
│   │   │
│   │   ├── FormatConversion/                      # 格式转换 (15条)
│   │   │   └── FormatConversionTests.cs            # GE2E-071~085
│   │   │
│   │   ├── Batch/                                 # 批处理 (10条)
│   │   │   └── BatchWorkflowTests.cs              # GE2E-086~095
│   │   │
│   │   └── ErrorPath/                             # 错误路径 & 边界 (10条)
│   │       └── ErrorPathTests.cs                  # GE2E-096~105
│   │
│   └── Fixtures/
│       ├── TestAppFixture.cs                      # 新建: 集合级 fixture
│       └── TestDataFixture.cs                     # 新建: 测试数据准备
│
└── shared/test_cases/                             # 共享测试用例 JSON
    ├── grpc_cases.json                            # Layer 2/4 共享 (120条)
    ├── cross_chain_cases.json                     # Layer 6 (60条)
    └── schema.json                                # JSON Schema 验证
```

### A.2 项目依赖关系

```
Photopipeline.Tests.csproj
  ├── Photopipeline.csproj (项目引用)
  ├── xunit 2.9.3
  ├── Moq 4.20.72
  ├── FluentAssertions 7.0.0
  ├── SkiaSharp 3.119.2
  ├── Grpc.Net.Client
  ├── Google.Protobuf
  └── Microsoft.Extensions.Logging

Photopipeline.UIAutomationTests.csproj
  ├── Photopipeline.csproj (项目引用)
  ├── FlaUI.UIA3 4.0.1
  ├── FlaUI.Core 4.0.1
  ├── xunit 2.9.3
  ├── FluentAssertions 7.0.0
  └── SkiaSharp 3.119.2
```

## B. Layer 3: C# Unit Test Detailed Design
### B.1 设计目标

Layer 3 单元测试覆盖 7 个 ViewModel + 7 个 Service 类，共~120 条测试。

**目标**:
- 每个测试至少一个能 FAIL 的 FluentAssertions 断言
- 使用 Moq 隔离外部依赖（Service、I/O、Dialog）
- 不启动真实进程，不依赖后端
- 测试执行时间 < 50ms/条
- 先写断言，再写 Arrange 和 Act

### B.2 测试基础设施模式

**Base test class pattern** — 每个 ViewModel 测试类使用以下模式:

```csharp
public sealed class FilmstripViewModelTests : IDisposable
{
    private readonly ITestOutputHelper _output;
    private readonly Mock<IImageService> _imageServiceMock;
    // ... mocks for all constructor dependencies
    
    public FilmstripViewModelTests(ITestOutputHelper output)
    {
        _output = output;
        _imageServiceMock = new Mock<IImageService>(MockBehavior.Strict);
        // MockBehavior.Strict ensures unexpected calls cause test FAIL
    }
    
    public void Dispose()
    {
        // Verify all Moq expectations were met
        Mock.VerifyAll(_imageServiceMock, _loggerMock);
    }
}
```

**Iron Rule 1 保障**: MockBehavior.Strict 在构造函数设置。任何未预期的 Mock 调用会立即让测试 FAIL。每个测试方法的结尾必须包含 FluentAssertions `.Should().Should().Be()` 断言或 Moq `.Verify()`。

**Iron Rule 2 保障**: 无 try-catch 块。如果构造函数抛出异常（如 Mock 配置错误），测试框架直接捕获并标记为 Failed。禁止任何 catch-and-return 模式。

### B.3 各 ViewModel 测试文件设计

#### B.3.1 MainViewModelTests.cs (15 条)

**依赖注入模式**: MainViewModel 有 10 个构造函数参数（ILogger, ISettingsService, IBackendService, 6 个子 VM, 可选 IThemeService）。

**Mock 工厂模式**:

```csharp
private static MainViewModel Create(
    bool backendHealthy = true,
    Action<Mock<ISettingsService>>? configureSettings = null)
{
    var logger = Mock.Of<ILogger<MainViewModel>>();
    var settingsMock = new Mock<ISettingsService>(MockBehavior.Strict);
    settingsMock.Setup(s => s.Current).Returns(new AppSettings());
    configureSettings?.Invoke(settingsMock);
    var backendMock = new Mock<IBackendService>(MockBehavior.Strict);
    backendMock.Setup(b => b.IsHealthy).Returns(backendHealthy);
    // ... sub-VM creation ...
    return new MainViewModel(logger, settingsMock.Object, backendMock.Object, ...);
}
```

**关键测试分类**:
- 导航命令 (4 条): NavigateCommand 切换 CurrentView
- 事件订阅 (2 条): PluginAdded → PipelineEditor.AddNodeAt
- 主题切换 (2 条): ToggleThemeCommand 切换 Light/Dark
- 后端状态 (2 条): IsHealthy 影响 BackendStatus
- 子 VM 交互 (3 条): 事件传播、Dispose、异常冒泡
- 初始状态 (2 条): 构造后所有属性非 null

#### B.3.2 FilmstripViewModelTests.cs (15 条)

**关键依赖**: IImageService (LoadImageInfoAsync)

**测试分类**:
- 导入/删除 (6 条): 文件存在/不存在/重复/多选/无选
- 选择操作 (3 条): 全选/清除/反选
- 排序 (3 条): 按名字/日期/大小
- 过滤 (2 条): 文本过滤/日期过滤
- 边界 (1 条): SelectedImage=null 时 CopyPath 不崩溃

**失败验证**: 每个测试验证 Images.Count 或 SelectedImages.Count 的变更, 使用 `.Should().Be(expected)`。

#### B.3.3 PreviewViewModelTests.cs (12 条)

**关键依赖**: IImageService (SaveAsync), IPipelineService

**测试分类**:
- 缩放 (4 条): ZoomIn/ZoomOut/Fit/Zoom100 修改 Zoom 属性
- 平移 (1 条): Pan(dx, dy) 更新 Offset
- 导出 (2 条): Export 调用 SaveAsync, 检查 PixelFormat
- 图像分析 (1 条): ComputeHistogram 返回正确 histogram
- 生命周期 (3 条): 图像切换时 Zoom 重置、Dispose 释放 Bitmap、无图像时 IsEmpty=true

**Iron Rule 5 保障**: 每个缩放测试都验证 Zoom 属性在预期范围。如果 ZoomInCommand 不改变 Zoom, 测试会 FAIL。

#### B.3.4 PipelineEditorViewModelTests.cs (18 条)

**关键依赖**: IPipelineService (ExecuteAsync, ValidateAsync, CreatePipelineAsync)

**测试分类**:
- 节点操作 (4 条): AddNode/RemoveNode/RemoveNodeWithReconnect
- 边操作 (5 条): Connect/RejectCycle/RejectSelfLoop/RejectDuplicate
- 管线执行 (4 条): Run/EmptyPipeline/NoImage/Cancel
- 命令行 (3 条): New/Save/Toggle
- 进度/参数 (2 条): Progress callback, SetParam

**Iron Rule 1 保障**: 循环检测测试验证 AddNodeAt 抛出异常或返回 false。如果 cycle detection 不工作, 测试 FAIL。

#### B.3.5 PluginBrowserViewModelTests.cs (10 条)

**关键依赖**: IPluginService (GetPluginsAsync)

**测试分类**:
- 搜索 (3 条): 匹配/无匹配/空搜索
- 分类过滤 (2 条): Format/Pixel category
- 交互 (3 条): AddToPipeline 事件、MainVM 订阅、选中点击
- 拖放 (1 条): Drag 开始 DoDragDrop
- 初始加载 (1 条): Plugins.Count > 0

#### B.3.6 BatchViewModelTests.cs (15 条)

**关键依赖**: IBatchService (StartAsync, PauseAsync, ResumeAsync, CancelAsync)

**测试分类**:
- 队列管理 (3 条): Add/Remove/验证错误
- 执行控制 (8 条): Start/Pause/Resume/Cancel/Monitor/Complete/Failure
- 集成 (2 条): PipelineConfigPath 来自 MainVM、Serialization
- 生命周期 (1 条): Dispose 时取消

**关键模式**: Progress monitoring 测试通过 mock `IProgress<BatchProgress>` 回调来验证进度更新。

#### B.3.7 SettingsViewModelTests.cs (8 条)

**关键依赖**: ISettingsService (Load, Save, Reset)

**测试分类**:
- 加载 (2 条): 文件存在/不存在时的默认值
- 保存/重置 (2 条): SaveCommand/ResetCommand
- 验证 (3 条): 无效 Theme/Language/BackendUrl
- 错误处理 (1 条): IOException 保存失败

### B.4 Service 测试设计 (28 条)

所有 Service 测试遵循同一模式: 实例化 Service → Mock gRPC channel/HTTP client → 调用方法 → 验证结果。

| 文件 | 测试模式 | 关键 Mock |
|------|---------|-----------|
| PipelineServiceTests.cs | gRPC CallAsync 调用验证 | Mock<IPipelines.PipelinesClient> |
| GrpcClientServiceTests.cs | 连接/重连/断开状态机 | Mock<Channel> |
| ImageServiceTests.cs | 文件系统操作 + SkiaSharp | Mock<IFileSystem> |
| BatchServiceTests.cs | gRPC batch client 调用 | Mock<Batch.BatchClient> |
| PluginServiceTests.cs | 本地 catalog 返回 | — (不依赖外部) |
| BackendServiceTests.cs | HTTP Client 健康检查 | Mock<HttpMessageHandler> |
| SettingsServiceTests.cs | JSON 序列化/反序列化 | Mock<IFileSystem> |

**关键模式 — gRPC Service 验证**:

```csharp
[Fact]
public async Task PipelineService_ToProtoSpec_SerializesAllParams()
{
    var mockClient = new Mock<Pipelines.PipelinesClient>(MockBehavior.Strict);
    // Configure mock to return expected response
    mockClient
        .Setup(c => c.ExecuteAsync(It.IsAny<PipelineExecuteRequest>(), ...))
        .Returns(new AsyncUnaryCall<PipelineExecuteResponse>(...));
    
    var service = new PipelineService(mockClient.Object);
    var result = await service.ExecuteAsync("valid-input.dng");
    
    result.Should().NotBeNull();
    result.OutputPath.Should().NotBeNullOrEmpty();
    // If ExecuteAsync doesn't call gRPC, Verify fails
    mockClient.Verify(c => c.ExecuteAsync(It.IsAny<PipelineExecuteRequest>(), ...), Times.Once);
}
```

### B.5 铁律合规声明 (Layer 3)

| 铁律 | 实施 | 验证方法 |
|------|------|---------|
| 铁律1: FAIL 断言 | 每个测试结尾有 .Should().Be() / Verify() | 删除断言后测试应通过? 不应该 — 审查捕获 |
| 铁律2: 禁止静默 | MockBehavior.Strict, 无 try-catch | 任何 catch 块必须有 Assert.Fail |
| 铁律3: 消费者先 | Moq Mock 与 ViewModel 测试同步实现 | 没有 Mock 独立存在 |
| 铁律4: UI 进程 | 不适用 (单元测试) | — |
| 铁律5: 对抗自查 | 每个测试提交前问"改变 ViewModel 逻辑后这测试会 FAIL?" | PR checklist |
| 铁律6: Golden 图像 | 不适用 (状态机验证) | — |
## C. Layer 4: C# gRPC Integration Test Detailed Design

This is a test of the heredoc approach.

### C.1 设计目标

Layer 4 集成测试通过 C# gRPC 客户端与真实 Rust 后端通信。

**与旧的 ApiChannel 关键区别**:
- 旧代码: try-catch-return 静默跳过, 仅 File.Exists 验证
- 新设计: Assert.Fail 传播所有异常, ImageAssert 逐像素验证

### C.2 基础设施设计

#### C.2.1 GrpcTestServerManager

管理 Rust 后端的启动/停止:

```csharp
public sealed class GrpcTestServerManager : IAsyncLifetime
{
    private Process? _serverProcess;
    private readonly int _port;
    private readonly string _serverExePath;

    public GrpcTestServerManager()
    {
        _port = FindAvailablePort();
        _serverExePath = LocateServerBinary();
    }

    public string Address => $"http://localhost:{_port}";

    public async Task InitializeAsync()
    {
        var psi = new ProcessStartInfo(_serverExePath)
        {
            Arguments = $"--port {_port} --test-mode",
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
        };
        _serverProcess = Process.Start(psi);
        // 轮询 /health 端点, 超时 30s
        await WaitForReadyAsync(TimeSpan.FromSeconds(30));
    }

    public async Task DisposeAsync()
    {
        if (_serverProcess is { HasExited: false })
        {
            _serverProcess.Kill(entireProcessTree: true);
            await _serverProcess.WaitForExitAsync();
        }
        _serverProcess?.Dispose();
    }

    private async Task WaitForReadyAsync(TimeSpan timeout)
    {
        using var cts = new CancellationTokenSource(timeout);
        using var httpClient = new HttpClient { BaseAddress = new Uri(Address) };
        while (DateTime.UtcNow < DateTime.UtcNow + timeout)
        {
            try
            {
                var response = await httpClient.GetAsync("/health", cts.Token);
                if (response.IsSuccessStatusCode) return;
            }
            catch { }
            await Task.Delay(500);
        }
        Assert.Fail($"Server did not become ready within {timeout}");
    }
}
```

**Iron Rule 2**: WaitForReadyAsync 超时后 Assert.Fail, 不吞异常。

#### C.2.2 ApiTestBase — 测试基类 (重写)

```csharp
public abstract class ApiTestBase : IClassFixture<GrpcTestServerManager>, IDisposable
{
    protected readonly GrpcTestServerManager Server;
    protected readonly TestOutputManager Output;
    protected readonly Pipelines.PipelinesClient PipelineClient;
    protected readonly Batch.BatchClient BatchClient;

    protected ApiTestBase(GrpcTestServerManager server, ITestOutputHelper output)
    {
        Server = server;
        Output = new TestOutputManager();
        var channel = GrpcChannel.ForAddress(server.Address);
        PipelineClient = new Pipelines.PipelinesClient(channel);
        BatchClient = new Batch.BatchClient(channel);
    }

    public virtual void Dispose() => Output.Dispose();

    protected async Task<string> ExecutePipelineAndGetOutputAsync(
        PipelineSpec spec, string inputPath)
    {
        var request = new PipelineExecuteRequest
        {
            PipelineSpec = spec,
            InputPath = inputPath,
            OutputPath = Output.CreatePath("output"),
        };
        var response = await PipelineClient.ExecuteAsync(request);
        response.OutputPath.Should().NotBeNullOrEmpty();
        File.Exists(response.OutputPath).Should().BeTrue();
        return response.OutputPath;
    }
}
```

**关键设计**: 没有 try-catch 块。gRPC 异常自然传播, 断言失败直接标记为 Failed。

#### C.2.3 SharedTestCaseLoader — JSON 用例加载器

```csharp
public sealed record TestCaseDefinition
{
    public string Id { get; init; } = "";
    public string Name { get; init; } = "";
    public string Category { get; init; } = "";
    public string InputImage { get; init; } = "";
    public PipelineSpecDef PipelineSpec { get; init; } = new();
    public AssertionDef Assertions { get; init; } = new();
}

public sealed record AssertionDef
{
    public int TolerancePerChannel { get; init; } = 0;
    public string ExpectedFormat { get; init; } = "";
    public int ExpectedWidth { get; init; }
    public int ExpectedHeight { get; init; }
    public double MinPSNR { get; init; }
    public double MinSSIM { get; init; }
    public double MaxDeltaE { get; init; }
}
```

**Iron Rule 3**: SharedTestCaseLoader 在第一个集成测试实现前创建。JSON 与 Rust Layer 2 共享。

### C.3 测试类设计

| 文件 | 数量 | 模式 | 验证 |
|------|------|------|------|
| PluginGrpcTests.cs | 44 | Theory+MemberData, 加载 JSON 用例 | ImageAssert.PixelsEqual/PSNR/SSIM |
| PipelineGrpcTests.cs | 30 | 多节点链, 验证中间+最终输出 | 同上, 每节点验证 |
| FormatGrpcTests.cs | 15 | 跨格式 decode→encode | IsValidFormat + PixelsEqual |
| BatchGrpcTests.cs | 15 | Submit→轮询→验证各输出 | 每输出文件验证 |
| ErrorPathGrpcTests.cs | 10 | 无效请求→捕获 RpcException | Assert.Throws + StatusCode |
| ConcurrencyGrpcTests.cs | 6 | Task.WhenAll 并发请求 | 各请求独立验证 |

### C.4 铁律合规声明 (Layer 4)

| 铁律 | 实施 |
|------|------|
| 铁律1: FAIL 断言 | 每测试调用 ImageAssert 像素验证 |
| 铁律2: 禁止静默 | ApiTestBase 无 try-catch-return, 连接失败 Assert.Fail |
| 铁律3: 消费者先 | SharedTestCaseLoader 创建于测试之前 |
| 铁律4: UI 进程 | 不适用 (gRPC 调用) |
| 铁律5: 对抗自查 | 故意损坏 golden 文件验证 FAIL |
| 铁律6: Golden 图像 | 每测试有 golden 参考, 逐像素比对 |
## D. Layer 5: C# GUI FlaUI E2E Test Detailed Design (105 tests)

### D.1 设计目标

Layer 5 通过真实 WPF 窗口操作验证 GUI 端到端工作流.

**旧代码问题**: UiTestDriver 全部 10 个方法为空桩, UIAutomationTests 全部 Skip + 空体。

### D.2 基础设施设计

#### D.2.1 UiTestDriver — UI 操作封装 (12 个方法, 全部真实实现)

```csharp
public sealed class UiTestDriver : IDisposable
{
    private readonly Application _app;
    private readonly Window _mainWindow;

    public UiTestDriver(string appPath, string testDataDir, ITestOutputHelper output)
    {
        _app = Application.Launch(appPath);
        _mainWindow = _app.GetMainWindow(new UIA3Automation(),
            waitTimeout: TimeSpan.FromSeconds(15));
        _mainWindow.Should().NotBeNull("Main window must appear within 15s");
        TestDataDir = testDataDir;
        Output = output;
    }
```

**铁律 4**: Application.Launch 启动真实 Photopipeline.exe。如果窗口 15s 内不出现, NotBeNull 断言 FAIL。

    // 12 个操作方法

    public void ImportImage(string imagePath)   // FlaUI 点击 Import + 文件对话框
    {
        var btn = _mainWindow.FindDescendant(c => c.AutomationId == "ImportButton");
        if (btn == null) throw new InvalidOperationException("ImportButton not found");
        btn.Click();
        // UIA3 操作文件打开对话框, 输入路径, 点击 Open
        var dlg = _mainWindow.FindDescendant(c => c.Name.Contains("Open"),
            waitTimeout: TimeSpan.FromSeconds(3));
        var edit = dlg?.FindDescendant(c => c.Name.Contains("File name"));
        if (edit == null) throw new InvalidOperationException("File name edit not found");
        edit.Enter(imagePath);
        dlg?.FindDescendant(c => c.Name == "Open")?.Click();
    }

    public void SelectImage(string name)        // 在 Filmstrip 点击缩略图
    {
        var item = _mainWindow.FindDescendant(c => c.Name.Contains(name));
        if (item == null) throw new InvalidOperationException("Image not in filmstrip");
        item.Click();
    }

    public void NavigateToPipelineEditor()      // NavigationView 切换
    {
        var nav = _mainWindow.FindDescendant(c => c.Name.Contains("Pipeline Editor"));
        if (nav == null) throw new InvalidOperationException("Nav item not found");
        nav.Click();
    }

    public void AddPluginToPipeline(string plugin)
    {
        var item = _mainWindow.FindDescendant(c => c.Name.Contains(plugin));
        if (item == null) throw new InvalidOperationException("Plugin not found");
        var btn = item.FindDescendant(c => c.AutomationId == "AddToPipelineButton");
        if (btn == null) throw new InvalidOperationException("Add button not found");
        btn.Click();
    }

    public void RunPipeline(TimeSpan timeout)
    {
        var btn = _mainWindow.FindDescendant(c => c.AutomationId == "RunButton");
        if (btn == null) throw new InvalidOperationException("RunButton not found");
        btn.Click();
        var prog = _mainWindow.FindDescendant(c => c.AutomationId == "ProgressBar");
        var deadline = DateTime.UtcNow + timeout;
        while (DateTime.UtcNow < deadline)
        {
            if (prog != null && (double)prog.GetCurrentPropertyValue(RangeValuePattern.ValueProperty) >= 100)
                return;
            Thread.Sleep(500);
        }
        Assert.Fail("Pipeline did not complete within " + timeout);
    }

    public void ExportImage(string outputPath)  // 点 Export + 保存对话框
    {
        var btn = _mainWindow.FindDescendant(c => c.AutomationId == "ExportButton");
        if (btn == null) throw new InvalidOperationException("ExportButton not found");
        btn.Click();
        var dlg = _mainWindow.FindDescendant(c => c.Name.Contains("Save As"),
            waitTimeout: TimeSpan.FromSeconds(3));
        var edit = dlg?.FindDescendant(c => c.Name.Contains("File name"));
        if (edit == null) throw new InvalidOperationException("Save file name edit not found");
        edit.Enter(outputPath);
        dlg?.FindDescendant(c => c.Name == "Save")?.Click();
    }

    public void PauseBatch() { /* similar pattern */ }
    public void ResumeBatch() { /* similar pattern */ }
    public void CancelOperation() { /* similar pattern */ }
    public void ToggleNode(string label) { /* similar pattern */ }
    public void DeleteNode(string label) { /* similar pattern */ }

    public void Dispose()
    {
        try { _app?.Close(); } catch { }
        try { _app?.Dispose(); } catch { }
    }
}
```

**铁律 1/2**: 每个方法在控件找不到时抛出异常, 不静默跳过。找不到 RunButton 直接让测试 FAIL。

#### D.2.2 UiElementLocator — 控件查找策略

```csharp
public static class UiElementLocator
{
    // 策略 1: AutomationId (最可靠, WPF 控件显式设置)
    public static AutomationElement? ById(Window window, string id) =>
        window.FindDescendant(c => c.AutomationId == id);

    // 策略 2: Name (用于按钮和标签)
    public static AutomationElement? ByName(Window window, string name) =>
        window.FindDescendant(c => c.Name.Contains(name));

    // 策略 3: ControlType + Name (用于列表项)
    public static AutomationElement? ByTypeAndName(Window window,
        ControlType type, string name) =>
        window.FindDescendant(c => c.ControlType == type && c.Name.Contains(name));
}
```

**WPF AutomationId 推荐命名规范**: ImportButton, ExportButton, RunButton, CancelButton,
PauseButton, ResumeButton, ProgressBar, AddToPipelineButton, PipelineCanvas, FilmstripList,
PluginBrowserList, ParameterPanel, NavigationView.

#### D.2.3 TestAppFixture — 集合级 Fixture

```csharp
[CollectionDefinition("FlaUITests")]
public sealed class FlaUITestCollection : ICollectionFixture<TestAppFixture> { }

public sealed class TestAppFixture : IDisposable
{
    public string AppPath { get; }
    public string TestDataDir { get; }

    public TestAppFixture()
    {
        // 定位 Photopipeline.exe (dotnet publish 产物)
        AppPath = Path.GetFullPath(
            Path.Combine("..", "Photopipeline", "bin", "Release", "net9.0-windows",
                "Photopipeline.exe"));
        if (!File.Exists(AppPath))
            Assert.Fail($"Photopipeline.exe not found at {AppPath}");

        TestDataDir = Path.GetFullPath(
            Path.Combine("..", "Photopipeline.Tests", "TestData", "input"));
    }

    public UiTestDriver CreateDriver(ITestOutputHelper output) =>
        new UiTestDriver(AppPath, TestDataDir, output);

    public void Dispose() { }
}
```

**Iron Rule 2**: 如果 exe 不存在, Assert.Fail 立即失败, 不静默跳过。

#### D.2.4 11 步标准测试流程

每条 GUI E2E 测试遵循相同流程, 由 UiTestBase.RunStandardWorkflowAsync 封装:

```
1. [启动] Photopipeline.exe, 等待主窗口就绪 (15s 超时)
2. [导入] ImportButton → 文件对话框 → 选测试图像 → 确认缩略图
3. [选图] Filmstrip 点击目标图像
4. [导航] NavigationView 切换到 Pipeline Editor
5. [添加节点] Plugin Browser → AddToPipelineButton
6. [连线] 自动边连接 (或手动添加后续节点)
7. [设置参数] 参数面板 → 输入值
8. [运行] RunButton → 轮询 ProgressBar 至 100%
9. [导出] ExportButton → 保存对话框 → 选路径
10. [验证] 从磁盘读取输出 → ImageAssert 像素验证
11. [清理] 关闭应用, 删除临时文件

### D.3 测试文件设计

五个测试目录, 对应五类测试场景:

| 目录 | 文件 | 用例 | 范围 |
|------|------|------|------|
| SinglePlugin/ | RawInputTests.cs | GE2E-001~003 (3) | raw_input 参数组合 |
| SinglePlugin/ | TransformTests.cs | GE2E-004~007 (4) | crop/resize/rotate/flip |
| SinglePlugin/ | ColorspaceTests.cs | GE2E-008~011 (4) | sRGB/AdobeRGB/P3/Gray |
| SinglePlugin/ | Lut3DTests.cs | GE2E-012~015 (4) | warm/cool/film/extreme |
| SinglePlugin/ | LensCorrectTests.cs | GE2E-016~019 (4) | barrel/pincushion/TCA |
| SinglePlugin/ | AiDenoiseTests.cs | GE2E-020~022 (3) | light/medium/heavy |
| SinglePlugin/ | ExifRwTests.cs | GE2E-023~025 (3) | preserve/write/clear |
| SinglePlugin/ | GpsSetTests.cs | GE2E-026~028 (3) | manual/gpx/clear |
| SinglePlugin/ | TimeShiftTests.cs | GE2E-029~031 (3) | +1h/-24h/timezone |
| SinglePlugin/ | EncoderTests.cs | GE2E-032~040 (9) | AVIF/JXL/HEIF/TIFF/PNG |
| MultiPlugin/ | RealWorldWorkflowTests.cs | GE2E-041~070 (30) | 2-5 节点真实用例 |
| FormatConversion/ | FormatConversionTests.cs | GE2E-071~085 (15) | 跨格式转换 |
| Batch/ | BatchWorkflowTests.cs | GE2E-086~095 (10) | 批处理 |
| ErrorPath/ | ErrorPathTests.cs | GE2E-096~105 (10) | 错误路径 & 边界 |

### D.4 测试示例

#### D.4.1 单插件测试 (RawInputTests.cs)

```csharp
[Collection("FlaUITests")]
public sealed class RawInputTests : UiTestBase
{
    public RawInputTests(TestAppFixture fixture, ITestOutputHelper output)
        : base(fixture, output) { }

    [Fact]
    public void GE2E_001_RawInput_AutoToTiff()
    {
        // Arrange
        var inputImage = "solid_color_1920.png";
        var goldenPath = Catalog.GetGoldenPath("GE2E-001");

        // Act: 11 步标准流程
        var outputPath = RunStandardWorkflowAsync(
            inputImage,
            plugins: new[] { "raw_input", "tiff_encoder" },
            configureParams: d =>
            {
                d.SetParameter("raw_mode", "auto");
                d.SetParameter("apply_white_balance", "true");
            });

        // Assert — Iron Rule 1&6
        ImageAssert.PixelsEqual(goldenPath, outputPath, tolerancePerChannel: 0);
        ImageAssert.IsValidFormat(outputPath, "TIFF", 1920, 1080);
        // If raw_input not applied or pipeline fails, PixelsEqual throws
    }

    [Fact]
    public void GE2E_003_RawInput_U16Output()
    {
        var outputPath = RunStandardWorkflowAsync(
            "high_bitdepth_1920.tiff",
            plugins: new[] { "raw_input", "tiff_encoder" },
            configureParams: d =>
            {
                d.SetParameter("output_format", "u16");
                d.SetParameter("half_size", "false");
            });

        ImageAssert.IsValidFormat(outputPath, "TIFF", 1920, 1080, bitDepth: 16);
        // If half_size=true, dimensions would be 960x540 -> test FAILs
    }
}
```

#### D.4.2 多插件工作流测试 (RealWorldWorkflowTests.cs)

```csharp
[Fact]
public void GE2E_041_FullRawWorkflow()
{
    // RAW → AdobeRGB → 16bit TIFF (专业 RAW 开发流程)
    var outputPath = RunStandardWorkflowAsync(
        "high_bitdepth_1920.tiff",
        plugins: new[] { "raw_input", "colorspace", "tiff_encoder" },
        configureParams: d =>
        {
            d.SetParameter("raw_mode", "auto");
            d.SetParameter("source_color_space", "sRGB");
            d.SetParameter("target_color_space", "AdobeRGB");
            d.SetParameter("embed_icc", "true");
            d.SetParameter("compression", "deflate");
        });

    ImageAssert.PixelsEqual(Catalog.GetGoldenPath("GE2E-041"), outputPath, 0);
    ImageAssert.IsValidFormat(outputPath, "TIFF", 1920, 1080, bitDepth: 16);
}
```

#### D.4.3 错误路径测试 (ErrorPathTests.cs)

```csharp
[Fact]
public void GE2E_096_RunWithoutNodes_ShowsError()
{
    Driver.ImportImage(Path.Combine(Driver.TestDataDir, "solid_color_1920.png"));
    Driver.SelectImage("solid_color_1920");
    Driver.NavigateToPipelineEditor();
    // 不添加任何节点, 直接点 Run
    var runButton = _mainWindow.FindDescendant(c => c.AutomationId == "RunButton");
    runButton?.Click();
    Thread.Sleep(2000);
    // 验证错误消息出现
    var errorMsg = _mainWindow.FindDescendant(c => c.Name.Contains("No nodes"));
    errorMsg.Should().NotBeNull("Error message should appear when running empty pipeline");
}

[Fact]
public void GE2E_105_CycleDetection_Warning()
{
    Driver.ImportImage(Path.Combine(Driver.TestDataDir, "solid_color_1920.png"));
    Driver.SelectImage("solid_color_1920");
    Driver.NavigateToPipelineEditor();
    Driver.AddPluginToPipeline("transform");
    Driver.AddPluginToPipeline("colorspace");
    // 尝试创建 A → B → A 循环
    Driver.ConnectNodes("transform", "colorspace");  // A → B
    Thread.Sleep(500);
    // 再次连接应该被 UI 阻止
    var warning = _mainWindow.FindDescendant(c => c.Name.Contains("cycle"),
        waitTimeout: TimeSpan.FromSeconds(2));
    warning.Should().NotBeNull("UI should show cycle detection warning");
    // If cycle detection is broken, test FAILs
}
```

### D.5 铁律合规声明 (Layer 5)

| 铁律 | 实施 |
|------|------|
| 铁律1: FAIL 断言 | 每个测试从磁盘读输出文件, 调用 ImageAssert.PixelsEqual |
| 铁律2: 禁止静默 | UiTestDriver 构造函数 FindDescendant 失败直接抛异常 |
| 铁律3: 消费者先 | UiTestDriver 12 个方法在第一个 E2E 测试实现时同步实现 |
| 铁律4: 真实进程 | FlaUI Application.Launch 启动真实 exe, 操作 WPF 控件 |
| 铁律5: 对抗自查 | 完成后故意设空管线, 验证测试确实 FAIL |
| 铁律6: Golden 图像 | ReferenceImageGenerator 提供基准, 逐像素比对 |

### D.6 验证方法使用矩阵

每个 GUI E2E 测试根据操作类型使用不同的验证:

| 操作类型 | 验证方法 | 适用 GE2E 范围 |
|---------|---------|---------------|
| 无损操作 (raw_input, transform, colorspace) | PixelsEqual(golden, output, 0) | 001~011, 016~031, 041~095 |
| LUT 应用 | SSIMAbove(golden, output, 0.98) | 012~015 |
| 降噪 | PSNRAbove(input, output, minPSNR) | 020~022 |
| 格式编码 | IsValidFormat(output, fmt, w, h, bitDepth) | 032~040, 071~085 |
| 批处理 | 每个输出文件独立 PixelsEqual | 086~095 |
| 错误路径 | Assert.Throws + UI 警告检测 | 096~105 |

## E. 需重写的文件清单

### E.1 删除/替换文件 (41 个文件)

这些文件被评估为垃圾代码 (空桩/静默跳过/空断言), 需要删除并由新实现替换:

**旧 ScenarioTests (全部删除)**:
| 文件 | 问题 | 替换方案 |
|------|------|---------|
| ScenarioTests/ErrorRecoveryScenarioTests.cs | Mock 设了但不触发断言 | 删除, 用例由新 UnitTests 覆盖 |
| ScenarioTests/* (其他) | 全部空断言 | 删除全部 |

**旧 UnitTests ViewModel (7 文件重写)**:
| 文件 | 问题 | 替换 |
|------|------|------|
| UnitTests/ViewModels/MainViewModelTests.cs | ZoomCommands 无断言 | 重写为 15 条含断言测试 |
| UnitTests/ViewModels/* (其他 6 文件) | 不完整/无断言 | 重写全部 |

**旧 UnitTests Services (1 文件重写)**:
| 文件 | 问题 | 替换 |
|------|------|------|
| UnitTests/Services/PluginServiceTests.cs | 纯反射检查 | 重写为 4 条行为测试 |
| UnitTests/Services/* (其他 6 文件新建) | 不存在 | 新建 |

**旧 UiChannel (10 文件删除)**:
| 文件 | 问题 |
|------|------|
| FunctionalTests/UiChannel/UiTestDriver.cs | 10 个空桩方法 |
| FunctionalTests/UiChannel/UiTestBase.cs | 仅 Thread.Sleep(5000) |
| FunctionalTests/UiChannel/*Tests.cs (8 文件) | 全部仅 File.Exists |

**旧 CrossChannel (5 文件重写)**:
| 文件 | 问题 |
|------|------|
| CrossChannel/CrossChannelTestBase.cs | 异常被吞, 返回 null |
| CrossChannel/PluginCrossTests.cs | ResourceMonitor.ShouldSkipLargeTest |
| CrossChannel/PipelineCrossTests.cs | 仅 File.Exists |
| CrossChannel/FormatCrossTests.cs | 仅 File.Exists |
| CrossChannel/BatchCrossTests.cs | 仅 File.Exists |
| CrossChannel/RegressionCrossTests.cs | 不存在 (新建) |

**旧 UIAutomationTests (11 文件删除)**:
| 文件 | 问题 |
|------|------|
| MainWindowUITests.cs | 6 个 Skip + 空体 |
| FilmstripUITests.cs | 5 个 Skip + 空体 |
| PipelineEditorUITests.cs | 全部 Skip |
| BatchUITests.cs | 全部 Skip |
| SettingsUITests.cs | 全部 Skip |
| ... (其他 6 文件) | 全部 Skip |

### E.2 新建文件 (42 个文件)

**Infrastructure (4 新建)**:
| 文件 | 用途 |
|------|------|
| Infrastructure/GrpcTestServerManager.cs | 启动/管理 Rust server |
| Infrastructure/SharedTestCaseLoader.cs | JSON 用例加载 |
| Infrastructure/ReferenceImageGenerator.cs | Golden 参考图像生成 |
| Infrastructure/ImageMetadata.cs | 元数据模型 record |

**UnitTests/ViewModels (7 重写)**:
MainViewModelTests.cs, FilmstripViewModelTests.cs, PreviewViewModelTests.cs,
PipelineEditorViewModelTests.cs, PluginBrowserViewModelTests.cs,
BatchViewModelTests.cs, SettingsViewModelTests.cs

**UnitTests/Services (7 文件: 1 重写 + 6 新建)**:
PipelineServiceTests.cs, GrpcClientServiceTests.cs, ImageServiceTests.cs,
BatchServiceTests.cs, PluginServiceTests.cs, BackendServiceTests.cs,
SettingsServiceTests.cs

**ApiChannel (6 重写)**:
ApiTestBase.cs, PluginGrpcTests.cs, PipelineGrpcTests.cs, FormatGrpcTests.cs,
BatchGrpcTests.cs, ErrorPathGrpcTests.cs, ConcurrencyGrpcTests.cs

**CrossChannel (5 重写 + 1 新建 = 6)**:
CrossChannelTestBase.cs, PluginCrossTests.cs, PipelineCrossTests.cs,
FormatCrossTests.cs, BatchCrossTests.cs, RegressionCrossTests.cs

**UIAutomationTests (14 新建)**:
| 目录 | 文件 |
|------|------|
| Framework/ | UiTestBase.cs, UiTestDriver.cs, UiElementLocator.cs, FlaUIExtensions.cs |
| Fixtures/ | TestAppFixture.cs, TestDataFixture.cs |
| Tests/SinglePlugin/ | RawInputTests.cs, TransformTests.cs, ColorspaceTests.cs, Lut3DTests.cs, LensCorrectTests.cs, AiDenoiseTests.cs, ExifRwTests.cs, GpsSetTests.cs, TimeShiftTests.cs, EncoderTests.cs |
| Tests/MultiPlugin/ | RealWorldWorkflowTests.cs |
| Tests/FormatConversion/ | FormatConversionTests.cs |
| Tests/Batch/ | BatchWorkflowTests.cs |
| Tests/ErrorPath/ | ErrorPathTests.cs |

**shared/test_cases (3 新建)**:
grpc_cases.json, cross_chain_cases.json, schema.json

### E.3 修改文件 (3 个文件)

| 文件 | 修改内容 |
|------|---------|
| Photopipeline.Tests.csproj | 添加 SkiaSharp 3.119.2, Moq 4.20.72, FluentAssertions 7.0.0, Grpc.Net.Client, Google.Protobuf 包引用 |
| Photopipeline.UIAutomationTests.csproj | 删除 Appium.WebDriver 5.1.0, 添加 FlaUI.UIA3 4.0.1, FlaUI.Core 4.0.1, FluentAssertions, SkiaSharp |
| Infrastructure/ImageAssert.cs | 实现 MetadataMatches 方法 (当前为空壳, 需实现真实 EXIF/XMP 读取) |

## F. 实施步骤

### F.1 实施阶段总览

五个阶段, 按依赖关系排列。每个阶段产出可独立验证。

```
Phase 1: 基础设施与修复
Phase 2: Layer 3 单元测试 (120 条)
Phase 3: Layer 4 gRPC 集成测试 (120 条)
Phase 4: Layer 5 GUI FlaUI E2E 测试 (105 条)
Phase 5: Layer 6 交叉验证 + 整体收尾
```

### F.2 Phase 1: 基础设施与修复

**优先级: P0 (阻塞所有后续测试)**

| # | 任务 | 产出 | 依赖 | 验证标准 |
|---|------|------|------|---------|
| 1.1 | 更新 Photopipeline.Tests.csproj 添加 NuGet 包 | 可编译的 csproj | 无 | dotnet restore 成功 |
| 1.2 | 创建 Infrastructure/GrpcTestServerManager.cs | GrpcTestServerManager 类 | 1.1 | 单元测试启动/停止 server |
| 1.3 | 创建 Infrastructure/SharedTestCaseLoader.cs | SharedTestCaseLoader | 1.1 | 加载 grpc_cases.json 成功 |
| 1.4 | 创建 Infrastructure/ReferenceImageGenerator.cs | ReferenceImageGenerator | 1.1 | 生成 golden 图像成功 |
| 1.5 | 实现 ImageAssert.MetadataMatches | MetadataMatches 方法 | 1.1 | 真实 EXIF 读取验证 |
| 1.6 | 更新 UIAutomationTests.csproj (FlaUI 替换 Appium) | 可编译的 csproj | 1.1 | FlaUI 测试项目构建成功 |
| 1.7 | 创建 shared/test_cases/*.json | 120+60 条 JSON 定义 | 无 | JSON Schema 验证通过 |
| 1.8 | 创建 Framework/UiTestBase.cs + UiTestDriver.cs | UiTestBase, UiTestDriver | 1.6 | 启动 exe + 找窗口成功 |

**Phase 1 完成标志**: `dotnet build` 无错误, GrpcTestServerManager 能启动/停止 Rust server,
UiTestDriver 能启动 Photopipeline.exe 并找到主窗口。

### F.3 Phase 2: Layer 3 单元测试 (120 条)

**优先级: P1 (无外部依赖, 可独立执行)**

| # | 任务 | 产出 | 用例数 | 依赖 | 验证 |
|---|------|------|--------|------|------|
| 2.1 | 重写 MainViewModelTests.cs | 文件 | 15 | 1.1 | dotnet test |
| 2.2 | 重写 FilmstripViewModelTests.cs | 文件 | 15 | 1.1 | dotnet test |
| 2.3 | 重写 PreviewViewModelTests.cs | 文件 | 12 | 1.1 | dotnet test |
| 2.4 | 重写 PipelineEditorViewModelTests.cs | 文件 | 18 | 1.1 | dotnet test |
| 2.5 | 重写 PluginBrowserViewModelTests.cs | 文件 | 10 | 1.1 | dotnet test |
| 2.6 | 重写 BatchViewModelTests.cs | 文件 | 15 | 1.1 | dotnet test |
| 2.7 | 重写 SettingsViewModelTests.cs | 文件 | 8 | 1.1 | dotnet test |
| 2.8 | 重写 PluginServiceTests.cs + 新建 6 Service 测试 | 7 文件 | 28 | 1.1 | dotnet test |

**关键规则**: 每个测试文件必须: (a) 使用 MockBehavior.Strict, (b) 每测试结尾有 FluentAssertions,
(c) 无 try-catch-return, (d) 无空方法体。

**Phase 2 完成标志**: `dotnet test --filter "Category=Unit"` → 120/120 通过。

### F.4 Phase 3: Layer 4 gRPC 集成测试 (120 条)

**优先级: P2 (依赖 Phase 1.2 + Rust 后端可执行)**

| # | 任务 | 产出 | 用例数 | 依赖 | 验证 |
|---|------|------|--------|------|------|
| 3.1 | 重写 ApiTestBase.cs | 文件 | — | 1.2 | 启动 server + gRPC 连接 |
| 3.2 | 重写 PluginGrpcTests.cs | 文件 | 44 | 3.1, 1.7 | dotnet test |
| 3.3 | 重写 PipelineGrpcTests.cs | 文件 | 30 | 3.1, 1.7 | dotnet test |
| 3.4 | 重写 FormatGrpcTests.cs | 文件 | 15 | 3.1, 1.7 | dotnet test |
| 3.5 | 重写 BatchGrpcTests.cs | 文件 | 15 | 3.1, 1.7 | dotnet test |
| 3.6 | 重写 ErrorPathGrpcTests.cs | 文件 | 10 | 3.1, 1.7 | dotnet test |
| 3.7 | 新建 ConcurrencyGrpcTests.cs | 文件 | 6 | 3.1, 1.7 | dotnet test |

**关键规则**: 每个测试必须: (a) 使用 SharedTestCaseLoader 加载用例,
(b) 输出路径通过 TestOutputManager 管理, (c) 调用 ImageAssert 像素验证。

**Phase 3 完成标志**: `dotnet test --filter "Category=GrpcIntegration"` → 120/120 通过。

### F.5 Phase 4: Layer 5 GUI FlaUI E2E 测试 (105 条)

**优先级: P2 (依赖 Phase 1.6 + 1.8)**

| # | 任务 | 产出 | 用例数 | 依赖 | 验证 |
|---|------|------|--------|------|------|
| 4.1 | 实现 RawInputTests.cs | 文件 | 3 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.2 | 实现 TransformTests.cs | 文件 | 4 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.3 | 实现 ColorspaceTests.cs | 文件 | 4 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.4 | 实现 Lut3DTests.cs | 文件 | 4 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.5 | 实现 LensCorrectTests.cs | 文件 | 4 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.6 | 实现 AiDenoiseTests.cs | 文件 | 3 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.7 | 实现 ExifRwTests.cs | 文件 | 3 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.8 | 实现 GpsSetTests.cs | 文件 | 3 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.9 | 实现 TimeShiftTests.cs | 文件 | 3 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.10 | 实现 EncoderTests.cs | 文件 | 9 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.11 | 实现 RealWorldWorkflowTests.cs | 文件 | 30 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.12 | 实现 FormatConversionTests.cs | 文件 | 15 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.13 | 实现 BatchWorkflowTests.cs | 文件 | 10 | 1.6, 1.8 | FlaUI 测试通过 |
| 4.14 | 实现 ErrorPathTests.cs | 文件 | 10 | 1.6, 1.8 | FlaUI 测试通过 |

**执行模式**: 每个测试文件使用 `[Collection("FlaUITests")]`, 共享 TestAppFixture。

**Phase 4 完成标志**: `dotnet test --filter "Category=GuiE2E"` → 105/105 通过。

### F.6 Phase 5: Layer 6 交叉验证 + 整体收尾

**优先级: P2 (依赖 Phase 3 + 4 全部完成)**

| # | 任务 | 产出 | 用例数 | 依赖 | 验证 |
|---|------|------|--------|------|------|
| 5.1 | 重写 CrossChannelTestBase.cs | 文件 | — | 3.2, 4.1 | 三通道执行全部成功 |
| 5.2 | 重写 PluginCrossTests.cs | 文件 | 20 | 5.1 | 三通道像素一致 |
| 5.3 | 重写 PipelineCrossTests.cs | 文件 | 15 | 5.1 | 三通道像素一致 |
| 5.4 | 重写 FormatCrossTests.cs | 文件 | 10 | 5.1 | 三通道像素一致 |
| 5.5 | 重写 BatchCrossTests.cs | 文件 | 8 | 5.1 | 三通道像素一致 |
| 5.6 | 新建 RegressionCrossTests.cs | 文件 | 7 | 5.1 | Golden baseline 比对 |
| 5.7 | 删除旧 ScenarioTests 目录 | 清理 | — | 2.x 完成 | 目录不存在 |
| 5.8 | 删除旧 UiChannel 目录 | 清理 | — | 3.x 完成 | 目录不存在 |
| 5.9 | 全局对抗性审查 | 清单 | — | 全部完成 | 6 条铁律逐条检查 |

### F.7 依赖图

```
Phase 1 (Infrastructure)
  ├── 1.1 csproj 更新 ──────────────── 所有后续任务依赖
  ├── 1.2 GrpcTestServerManager ────── Phase 3 依赖
  ├── 1.6 FlaUI csproj ─────────────── Phase 4 依赖
  └── 1.8 UiTestBase+Driver ────────── Phase 4 依赖

Phase 2 (Unit Tests) ─── 仅依赖 1.1, 可并行于 Phase 1.2~1.8

Phase 3 (gRPC Integration) ─── 依赖 1.1, 1.2, 1.3, 1.7

Phase 4 (GUI E2E) ─── 依赖 1.1, 1.6, 1.8; 可并行于 Phase 2+3

Phase 5 (Cross-Channel) ─── 依赖 Phase 3+4 完成; 最后清理
```

### F.8 验证标准汇总

| 检查项 | 命令 | 预期 |
|--------|------|------|
| C# 构建 | dotnet build -c Release | 0 error, 0 warning |
| Layer 3 单元测试 | dotnet test --filter Category=Unit | 120/120 pass |
| Layer 4 gRPC 集成 | dotnet test --filter Category=GrpcIntegration | 120/120 pass |
| Layer 5 GUI E2E | dotnet test --filter Category=GuiE2E | 105/105 pass |
| Layer 6 Cross-Channel | dotnet test --filter Category=CrossChannel | 60/60 pass |
| 总计 | dotnet test | ~405/405 pass |

### F.9 实施顺序推荐

基于依赖关系和风险, 推荐按以下顺序实施:

1. **Day 1-2**: Phase 1 (基础设施) — 这是最大风险项, GrpcTestServerManager 和
   UiTestDriver 的实现难度最高
2. **Day 3-4**: Phase 2 (单元测试) — 最简单, 可快速获得 120 条通过测试
3. **Day 5-7**: Phase 3 (gRPC 集成) — 依赖 Rust 后端可用, 每条测试有 golden 图像
4. **Day 8-12**: Phase 4 (GUI E2E) — 最复杂, 每条测试需要完整的 11 步流程
5. **Day 13-14**: Phase 5 (交叉验证 + 收尾) — 依赖前面全部完成
