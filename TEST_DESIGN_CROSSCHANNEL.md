# Photopipeline Layer 6 Cross-Channel 验证 + Shared JSON + CI/CD 测试编排 详细设计

**设计日期:** 2026-05-26
**对应规格:** TEST_CASE_SPECIFICATION.md Layer 6 (60 条 CCV + Shared JSON + CI/CD)
**设计原则:** 6 条铁律 (feedback_test_antagonism.md)

---

## A. CrossChannel 测试架构设计

### A.1 三通道数据流图

```
                        ┌──────────────────────────────────────────────────┐
                        │              Shared JSON Test Cases             │
                        │        shared/test_cases/cross_chain_cases.json │
                        └──────────────┬───────────────────────────────────┘
                                       │ 加载
                  ┌────────────────────┼────────────────────┐
                  ▼                    ▼                     ▼
         ┌────────────────┐  ┌────────────────┐  ┌──────────────────┐
         │ Channel 1:     │  │ Channel 2:     │  │ Channel 3:       │
         │ Rust Engine    │  │ Rust gRPC →    │  │ C# GUI via FlaUI │
         │ (direct call)  │  │ C# gRPC client │  │ (WPF automation) │
         │                │  │                │  │                  │
         │ Layer 1 tests  │  │ Layer 4 tests  │  │ Layer 5 tests    │
         └───────┬────────┘  └───────┬────────┘  └────────┬─────────┘
                 │                   │                    │
                 ▼                   ▼                    ▼
         ┌─────────────────────────────────────────────────────┐
         │            CrossChannelVerifier.VerifyEquivalence   │
         │                                                   │
         │  1. 三输出文件路径收集                                │
         │  2. 尺寸一致性检查 (宽/高/通道数/位深)                 │
         │  3. 逐像素比对 (Channel 1 vs Channel 2)             │
         │  4. 逐像素比对 (Channel 1 vs Channel 3)             │
         │  5. 逐像素比对 (Channel 2 vs Channel 3)             │
         │  6. 差异图像生成 (故障时)                            │
         └─────────────────────┬───────────────────────────────┘
                               ▼
                     PASS / FAIL + 差异图像
```

### A.2 每条通道的输入/输出/验证点

| 属性 | Channel 1 (Rust Engine) | Channel 2 (gRPC C#) | Channel 3 (GUI FlaUI) |
|------|------------------------|---------------------|----------------------|
| 执行器 | `cargo test --test pipeline_integration` | `GrpcClientService` 通过 gRPC 调用 Rust server | WPF 进程 + FlaUI.UIA3 自动化 |
| 输入源 | `tests/fixtures/input/` 直接磁盘读 | 同左 (路径通过 gRPC ExecuteRequest 传递) | 同左 (GUI Import → 文件对话框) |
| 管线定义 | `PipelineTemplate` Rust struct | `PipelineSpec` → protobuf → gRPC | PipelineSpec → 自动 GUI 操作 |
| 参数设置 | Rust 代码直接赋值 | C# `PipelineSpec.Params` dict → protobuf Struct | FlaUI 找到属性面板控件 → 输入值 |
| 执行方式 | `NodeExecutor::execute()` 直接调用 | `PipelineService.ExecuteAsync()` 流式进度 | 点击 Run → 轮询进度条 Done |
| 输出格式 | 最后节点输出文件 | gRPC Execute 写到 output_path | 点击 Export → 文件对话框保存 |
| 验证点 | 输出文件像素 | 输出文件像素 | 输出文件像素 |

### A.3 交叉验证的数学定义

**一致性定义 (等价关系):**

- **弱一致性** (用于有损编码): `∀ch ∈ {R,G,B}: |A.ch(x,y) - B.ch(x,y)| ≤ tolerancePerChannel` 且 `SSIM(A,B) ≥ minSSIM`
- **强一致性** (用于无损管线): `∀ch ∈ {R,G,B}: A.ch(x,y) ≡ B.ch(x,y) ≡ C.ch(x,y)`, 即 `tolerancePerChannel = 0`
- **全一致** (所有三通道): `PixelsEqual(A, B, 0) ∧ PixelsEqual(A, C, 0) ∧ PixelsEqual(B, C, 0)`

---

## B. Layer 6: Cross-Channel 验证测试 (60 条) 详细设计

### B.1 20 条单插件跨通道测试 (CCV-001~020)

| ID | 插件 | 输入 | 参数 | 管线 (2节点) | 预期容差 | 特殊验证 |
|----|------|------|------|-------------|---------|---------|
| CCV-001 | raw_input | I01 | raw_mode=auto, apply_wb=true | raw_input→tiff_encoder | 0 | 三通道 RGB 1920x1080 |
| CCV-002 | raw_input | I10 | raw_mode=dcraw, output_format=u16 | raw_input→tiff_encoder | 0 | 16bit 输出 |
| CCV-003 | transform | I01 | crop_enabled=true, crop_rect=25,25,50,50 | transform→png_encoder | 0 | 960x540 |
| CCV-004 | transform | I04 | angle=90, resize_mode=expand | transform→png_encoder | 0 | 2160x3840 |
| CCV-005 | colorspace | I01 | source=sRGB, target=AdobeRGB | colorspace→tiff_encoder | 0 | ICC profile 嵌入 |
| CCV-006 | colorspace | I03 | source=sRGB, target=Gray | colorspace→tiff_encoder | 0 | 单通道灰度 |
| CCV-007 | lut3d | I01 | lut_path=warm.cube, intensity=80 | lut3d→png_encoder | 0 | golden warm 比对 |
| CCV-008 | lut3d | I03 | lut_path=film.cube, interp=tetrahedral | lut3d→png_encoder | 0 | SSIM>0.98 |
| CCV-009 | lens_correct | I07 | correction_mode=auto | lens_correct→png_encoder | 0 | 几何校正一致性 |
| CCV-010 | lens_correct | I08 | correct_distortion+vignetting=true | lens_correct→tiff_encoder | 0 | 复合校正一致性 |
| CCV-011 | ai_denoise | I06 | strength=50, detail_preservation=50 | ai_denoise→png_encoder | PSNR>45 | GPU 差异容差 |
| CCV-012 | ai_denoise | I06 | strength=20, detail_preservation=80 | ai_denoise→tiff_encoder | PSNR>50 | 轻降噪一致性 |
| CCV-013 | exif_rw | I11 | read_all=true, overwrite_original=true | exif_rw→tiff_encoder | 0 | MetadataMatches |
| CCV-014 | gps_set | I01 | mode=manual, lat=39.9042, lon=116.4074 | gps_set→tiff_encoder | 0 | GPS 完全一致 |
| CCV-015 | time_shift | I11 | shift_hours=+1 | time_shift→tiff_encoder | 0 | EXIF 时间偏移一致 |
| CCV-016 | avif_encoder | I01 | lossless=true | avif_encoder→decode | 0 | 无损往返 |
| CCV-017 | jxl_encoder | I01 | lossless=true, effort=9 | jxl_encoder→decode | 0 | 无损往返 |
| CCV-018 | heif_encoder | I01 | quality=80, chroma=444 | heif_encoder→decode | PSNR>45 | 有损一致 |
| CCV-019 | tiff_encoder | I10 | compression=deflate, pixel_format=u16 | tiff_encoder→decode | 0 | 16bit 无损 |
| CCV-020 | png_encoder | I12 | color_type=rgba, compression=6 | png_encoder→decode | 0 | Alpha 完全一致 |

### B.2 15 条多插件管道跨通道测试 (CCV-021~035)

| ID | 管线 | 节点数 | 输入 | 验证方式 |
|----|------|--------|------|---------|
| CCV-021 | raw→colorspace→tiff | 3 | I01 | tolerance=0 |
| CCV-022 | raw→ai_denoise→colorspace→png | 4 | I06 | PSNR>45 |
| CCV-023 | raw→lens→colorspace→lut→tiff | 5 | I08 | tolerance=0 |
| CCV-024 | transform→colorspace→jxl(lossless) | 3 | I01 | tolerance=0 |
| CCV-025 | raw→transform→colorspace→avif(Q=90) | 4 | I01 | PSNR>48 |
| CCV-026 | raw→lens→ai_denoise→colorspace→tiff | 5 | I06 | PSNR>40 |
| CCV-027 | exif→gps→time→colorspace→tiff | 5 | I11 | tolerance=0 + MetadataMatches |
| CCV-028 | ai_denoise→colorspace→lut→tiff | 4 | I06 | PSNR>42 |
| CCV-029 | colorspace→lut→transform→png | 4 | I01 | tolerance=0 |
| CCV-030 | raw→colorspace→lut→jxl(lossless) | 4 | I01 | tolerance=0 |
| CCV-031 | transform(crop+resize)→colorspace→png | 3 | I01 | tolerance=0 |
| CCV-032 | lens→ai_denoise→colorspace→heif(Q=85) | 4 | I07 | PSNR>38 |
| CCV-033 | raw→transform(flip)→colorspace→tiff | 4 | I01 | tolerance=0 |
| CCV-034 | A(enabled)→B(disabled)→C | 3 (B穿透) | I01 | tolerance=0 |
| CCV-035 | 全disabled + 仅编码器 | N/A | I01 | tolerance=0 |

### B.3 10 条格式转换跨通道测试 (CCV-036~045)

| ID | 流程 | 验证方式 |
|----|------|---------|
| CCV-036 | PNG→decode→TIFF encode | tolerance=0 |
| CCV-037 | JPEG→decode→PNG encode | tolerance=2 (JPEG源有损) |
| CCV-038 | 16bit TIFF→decode→16bit PNG encode | tolerance=0, bit_depth=16 |
| CCV-039 | RGBA PNG→decode→TIFF encode | tolerance=0, alpha保持 |
| CCV-040 | Gray PNG→decode→sRGB→decode→Gray | tolerance=0 |
| CCV-041 | CMYK TIFF→decode→sRGB PNG | DeltaE<2 |
| CCV-042 | AVIF→decode→PNG encode | PSNR>45 |
| CCV-043 | JXL(lossless)→decode→TIFF encode | tolerance=0 |
| CCV-044 | 8bit→16bit promotion | tolerance=0, bit_depth=16 |
| CCV-045 | 16bit→8bit truncation | MAE<1.0 |

### B.4 8 条批处理跨通道测试 (CCV-046~053)

| ID | 文件数 | 管线 | 三通道执行差异 |
|----|--------|------|--------------|
| CCV-046 | 3 (I01,I03,I09) | transform(crop→png) | 每条通道处理3文件 |
| CCV-047 | 5 (I01~I05) | colorspace→tiff | 每条通道处理5文件 |
| CCV-048 | 2, 不同编码器 | 各自pipeline | 分别比对 |
| CCV-049 | 5, pause/resume@3/5 | colorspace→jxl | API无pause; GUI模拟 |
| CCV-050 | Cancel after 2/5 | colorspace→avif | API发Cancel; GUI点Cancel |
| CCV-051 | 10 images stress | transform(50%)→png | 全部10输出逐文件比对 |
| CCV-052 | 混合格式 | colorspace→tiff | 每个文件独立比对 |
| CCV-053 | 单图像批处理=非批处理 | transform→png | 一致性验证 |

### B.5 7 条回归快照跨通道测试 (CCV-054~060)

| ID | 管线 | Golden 文件 | 验证 |
|----|------|------------|------|
| CCV-054 | raw→colorspace→tiff(I01) | golden/ccv054_raw_colorspace_tiff.png | 三通道=golden |
| CCV-055 | transform(crop 50%)→png(I01) | golden/ccv055_crop50_png.png | 三通道=golden |
| CCV-056 | colorspace(sRGB→Gray)→tiff(I03) | golden/ccv056_gray_tiff.png | 三通道=golden |
| CCV-057 | lut3d(film)→png(I17) | golden/ccv057_film_lut_png.png | 三通道=golden |
| CCV-058 | ai_denoise(med)→png(I06) | golden/ccv058_denoise_med_png.png | 三通道=golden |
| CCV-059 | lens_correct→colorspace→tiff(I07) | golden/ccv059_lens_color_tiff.png | 三通道=golden |
| CCV-060 | 5-node full RAW workflow(I08) | golden/ccv060_full_raw_workflow_tiff.png | 三通道=golden |

---

## C. CrossChannelTestBase 重写方案

### C.1 新类结构

```
CrossChannelTestBase (abstract)
├── Channel 1: RunRustEngineAsync()      // 直接 Rust engine 调用 (进程外)
├── Channel 2: RunGrpcChannelAsync()     // 真实 gRPC 调用 (复用 ApiTestBase)
├── Channel 3: RunGuiChannelAsync()      // 真实 FlaUI 自动化 (不可空桩)
└── VerifyEquivalence()                  // 三路汇总比对
```

### C.2 API 通道修复 (修复静默跳过)

```csharp
// 铁律 2: 禁止静默跳过，后端不可用必须 Assert.Fail
protected async Task EnsureApiAvailableAsync(CancellationToken ct)
{
    if (_apiAvailable) return;
    try
    {
        await Api.EnsureConnectedAsync(ct);
        _apiAvailable = true;
    }
    catch (Exception ex)
    {
        Assert.Fail($"Backend gRPC unavailable: {ex.Message}");
    }
}
```

### C.3 UI 通道修复 (异常必须传播)

```csharp
protected async Task<string> RunUiChannelAsync(
    PipelineSpec pipeline, string inputImageName, string testName, ...)
{
    // 铁律 4: 必须启动真实进程 + FlaUI 操作
    // 铁律 2: 异常必须传播，不能 catch { return null; }
    
    var app = Application.Launch(AppPath);
    var window = app.GetMainWindow(AutomationObject);
    
    try
    {
        // Step 1-7: Import → Build Pipeline → Set Params → Run → Export
        // 所有异常直接传播
        return outputPath;
    }
    catch (Exception ex)
    {
        // 记录进程状态，然后重新抛出
        File.WriteAllText(crashLogPath, $"UI Channel FAILED: {ex}");
        throw; // 铁律 2
    }
    finally { app.Close(); }
}
```

### C.4 三通道汇总比对

```csharp
protected async Task VerifyCrossChannelAsync(...)
{
    var result = new CrossChannelResult(testName);
    
    // 三通道独立执行
    try { await result.RunChannel1Async(...); } catch (Exception ex) { result.Channel1Failed = ex; }
    try { await result.RunChannel2Async(...); } catch (Exception ex) { result.Channel2Failed = ex; }
    try { await result.RunChannel3Async(...); } catch (Exception ex) { result.Channel3Failed = ex; }
    
    var successfulChannels = result.GetSuccessfulChannels();
    
    if (successfulChannels.Count < 2)
        Assert.Fail($"Cross-channel FAILED: at least 2 channels required");
    
    // 两两比对所有成功通道
    for (int i = 0; i < successfulChannels.Count; i++)
        for (int j = i + 1; j < successfulChannels.Count; j++)
            CrossChannelVerifier.VerifyEquivalence(
                successfulChannels[i].OutputPath,
                successfulChannels[j].OutputPath,
                $"{testName}_ch{i}_vs_ch{j}");
}
```

### C.5 异常吞咽防护措施

| 防护层 | 措施 |
|--------|------|
| 编译期 | 禁止 `catch { return null; }` 模式 |
| CI 审计 | `grep -r "catch\s*{" path/to/CrossChannel/` |
| 运行时 | 主验证方法不包裹 try-catch，异常直达 xUnit |
| 审计日志 | 统计 "UI channel skipped" 行数，>0 则 CI 失败 |

---

## D. 跨通道失败处理

### D.1 部分通道失败汇总策略

- **>=2 通道成功**: 执行成功通道间比对，差异图像记录
- **1 通道成功**: 跳过 (至少需要 2 通道比对)
- **0 通道成功**: `Assert.Fail("所有通道均不可用")`

### D.2 差异图像生成

CrossChannelVerifier.SaveDiffImage 增强:
- 三通道差异: Ch1_vs_Ch2, Ch1_vs_Ch3, Ch2_vs_Ch3
- 存储路径: `%TEMP%/photopipeline_tests/diffs/{TestName}_ch{X}_vs_ch{Y}_diff.png`
- 元数据: 同目录 .json 记录差异像素数/百分比/首个差异点坐标

### D.3 失败日志格式

```
=== Cross-Channel Test Failure ===
Test: CCV-012 (ai_denoise_light→tiff)

Channel Status:
  [PASS] Channel 1 (Rust Engine): output OK
  [PASS] Channel 2 (gRPC):        output OK
  [FAIL] Channel 3 (GUI):         Application crash on Run

Pairwise Results:
  Ch1 vs Ch2: PASS (0 pixels differ, PSNR=∞)
  Ch1 vs Ch3: SKIPPED (Ch3 unavailable)
  Ch2 vs Ch3: SKIPPED (Ch3 unavailable)

Conclusion: PARTIAL FAILURE - Ch1/Ch2 match, Ch3 needs investigation
```

---

## E. Shared JSON 测试用例定义格式

### E.1 JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://photopipeline.dev/schemas/test-case-v1.json",
  "type": "object",
  "required": ["id", "name", "category", "pipeline_spec", "assertions"],
  
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^(CCV|GE2E|GRPC)-\\d{3}$"
    },
    "name": { "type": "string" },
    "category": {
      "type": "string",
      "enum": ["plugin", "pipeline", "format", "batch", "regression"]
    },
    "tags": { "type": "array", "items": { "type": "string" } },
    "input_images": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1, "maxItems": 20
    },
    "pipeline_spec": {
      "type": "object",
      "required": ["nodes", "edges"],
      "properties": {
        "nodes": {
          "type": "array",
          "items": { "$ref": "#/definitions/PipelineNode" },
          "minItems": 1, "maxItems": 20
        },
        "edges": {
          "type": "array",
          "items": { "$ref": "#/definitions/PipelineEdge" }
        }
      }
    },
    "assertions": {
      "type": "object",
      "required": ["tolerance_per_channel"],
      "properties": {
        "tolerance_per_channel": { "type": "integer", "minimum": 0, "maximum": 255 },
        "expected_format": { "type": "string" },
        "expected_width": { "type": "integer", "minimum": 1 },
        "expected_height": { "type": "integer", "minimum": 1 },
        "expected_bit_depth": { "type": "integer", "enum": [8, 10, 16, 32] },
        "min_psnr": { "type": "number", "minimum": 0 },
        "min_ssim": { "type": "number", "minimum": 0, "maximum": 1 },
        "max_delta_e": { "type": "number", "minimum": 0 },
        "check_metadata": { "type": "boolean" },
        "expect_error": { "type": "boolean" },
        "expected_error_message": { "type": "string" }
      }
    }
  }
}
```

### E.2 Rust 端解析器 (`crates/test-defs/`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub category: TestCategory,
    pub tags: Vec<String>,
    pub input_images: Vec<String>,
    pub pipeline_spec: PipelineSpec,
    pub assertions: Assertions,
}

impl TestCase {
    pub fn from_file(path: &str) -> Result<Self, TestDefError> { ... }
    pub fn validate(&self) -> Result<(), TestDefError> { ... }
}

pub fn load_all_test_cases(cases_dir: &str) -> Result<Vec<TestCase>, TestDefError> { ... }
```

### E.3 C# 端解析器 (`SharedTestCaseLoader.cs`)

```csharp
public static class SharedTestCaseLoader
{
    public static IEnumerable<TestCaseDefinition> LoadAll()
    {
        var sharedDir = FindSharedTestCasesDir();
        foreach (var jsonFile in Directory.GetFiles(sharedDir, "*.json"))
        {
            var sharedCase = JsonSerializer.Deserialize<SharedTestCase>(json, SharedJsonOptions);
            yield return ConvertToTestCaseDefinition(sharedCase);
        }
    }
}
```

### E.4 两端一致性保障

1. **单一真实来源**: `shared/test_cases/` 下 .json 是唯一来源
2. **JSON Schema 双重验证**: CI 阶段 Rust + C# 两端均校验
3. **结构体一致性测试**: 序列化→反序列化→再序列化，比对两次 JSON
4. **CI 交叉验证**: 比对两端解析结果的数量/ID 列表/断言字段

---

## F. ReferenceImageGenerator 设计

### F.1 生成流程

```csharp
public static class ReferenceImageGenerator
{
    public static async Task GenerateAllGoldenAsync()
    {
        var goldenDir = FindGoldenDirectory();
        var shouldWrite = Environment.GetEnvironmentVariable("PHOTOPIPELINE_GENERATE_GOLDEN") == "true";
        
        if (!shouldWrite) return; // DRY-RUN mode
        
        foreach (var (testId, pipeline, inputImage, goldenFileName) in GetRegressionCases())
        {
            var outputPath = Path.Combine(goldenDir, goldenFileName);
            await ExecutePipeline(pipeline, inputImage, outputPath);
            // 验证 golden 文件有效
        }
    }
}
```

### F.2 存储位置

```
tests/fixtures/golden/
├── ccv054_raw_colorspace_tiff.png
├── ccv055_crop50_png.png
├── ...
├── README.md
└── manifest.json
```

### F.3 环境变量机制

- 未设置: 测试模式，读取 golden 比对
- `"true"`: 生成模式，重新生成所有 golden
- `"verify"`: 验证模式，报告不一致但不失败

---

## G. CI/CD 测试编排设计

### G.1 7 阶段执行顺序

```
Stage 1: Build & Static Analysis
  ├── cargo build --workspace
  ├── dotnet build Photopipeline.sln -c Release
  └── clippy / roslyn analyzers

Stage 2: Layer 0 - Rust Unit Tests (并行 4 进程) [~350, <30s]
Stage 3: Layer 1 - Rust Pipeline Integration (串行) [~200, <5min]
Stage 4: Layer 2+4 - gRPC E2E (共享 server, 并行) [~240, <5min]
Stage 5: Layer 3 - C# Unit Tests (最大并行) [~120, <30s]
Stage 6: Layer 5 - GUI FlaUI E2E (串行, Windows only) [~105, <15min]
Stage 7: Layer 6 - Cross-Channel (最终验证) [~60, <10min]
```

### G.2 并行化策略

| 阶段 | 并行度 | 方法 |
|------|--------|------|
| Stage 1 | 2路 | cargo + dotnet 并行 |
| Stage 2 | 4路 | `--test-threads 4` |
| Stage 3 | 串行 | 资源密集 |
| Stage 4 | 2路 | Rust + C# 同时向同 server 发请求 |
| Stage 5 | 最大 | xUnit 默认 |
| Stage 6 | 串行 | GUI 独占桌面 |
| Stage 7 | 串行 | 等前序完成 |

### G.3 测试报告格式

```json
{
  "build_id": "20260526.1",
  "summary": {
    "total": 1075,
    "passed": 1070,
    "failed": 5,
    "pass_rate": "99.53%"
  },
  "layers": [
    { "name": "Layer 0: Unit Tests (Rust)", "total": 350, "passed": 350, ... },
    { "name": "Layer 6: Cross-Channel", "total": 60, "passed": 58, "failed": 2, ... }
  ]
}
```

---

## H. 实施步骤

### 优先级 P0 (阻塞项)

| 步骤 | 内容 | 验证方式 |
|------|------|---------|
| Step 1 | 创建 `shared/test_cases/` 目录和 JSON 定义 | JSON Schema validation 全通过 |
| Step 2 | 创建 `crates/test-defs/` Rust 解析 crate | `cargo test -p test-defs` 通过 |
| Step 3 | 实现 `SharedTestCaseLoader.cs` (C# 端) | 反序列化所有 JSON 无异常 |
| Step 4 | 修复 `CrossChannelTestBase.cs` 异常吞噬 | 异常传播 + Assert.Fail |

### 优先级 P1 (核心功能)

| 步骤 | 内容 | 验证方式 |
|------|------|---------|
| Step 5 | 实现 `UiTestDriver` FlaUI 操作 | 单条 CCV 完整执行 |
| Step 6 | 实现 `ReferenceImageGenerator.cs` | golden 文件生成有效 |
| Step 7 | 实现 20 条单插件 CCV 测试 | 60/60 全通过 |

### 优先级 P2-P3 (扩展 + CI)

| 步骤 | 内容 |
|------|------|
| Step 8-11 | 多插件/格式/批处理/回归 CCV |
| Step 12 | CI/CD 7-Stage 编排 |
| Step 13 | `ImageAssert.MetadataMatches` 实现 |

---

## 对抗性检查清单

1. 如果后端返回全黑图像 → CrossChannelVerifier.PixelsEqual 比对 golden 会 FAIL
2. 如果管线完全未执行 → FileNotFoundException + FlaUI 找不到进度条异常
3. 如果三通道返回同一错误图像 → 回归测试比对 golden 不一致
4. 如果 FlaUI 异常但不传播 → 铁律 2 强制 `throw;`，CI 审计检测
5. 如果 JSON 找不到 → `DirectoryNotFoundException` 抛出
6. 如果 Rust/C# Schema 不一致 → CI 交叉验证比对失败

---

**文档结束。**
