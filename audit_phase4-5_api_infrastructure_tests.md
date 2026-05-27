# Photopipeline Phase 4-5 测试基础设施与 API 通道测试深度审计报告

**审计日期:** 2026-05-26
**审计范围:** 10 个基础设施文件 + 9 个 API 通道测试文件
**审计方法:** 逐文件、逐方法读码分析，追踪每条逻辑流与操作流

---

## 总体结论

### 基础设施层: 9.1/10 (高质量)
基础设施层质量**非常高**。ImageAssert 提供完整的像素级比较工具链，TestImageGenerator 生成覆盖广泛的测试图像，TestCaseCatalog 定义了约 393 个结构化用例，CrossChannelVerifier 和 TestPipelineBuilder 都是完整实现。

### API 测试层: 2.4/10 (严重不足)
与基础设施层形成尖锐对比。**所有测试都会运行管线（如果后端可用），但没有任何测试验证管线是否产生了正确的结果。**

---

## 第一部分: 基础设施审计

### ImageAssert.cs — 8.5/10

| 方法 | 评分 | 说明 |
|---|---|---|
| PixelsEqual | 10/10 | 完整实现。SKCodec 解码，逐像素 RGBA 通道对比 |
| PSNRAbove | 10/10 | 完整实现。逐像素 MSE → PSNR |
| SSIMAbove | 9/10 | 完整实现。Luminance 公式使用简单平均而非标准权重 |
| HistogramSimilarityAbove | 10/10 | 64-bin Pearson 相关系数 |
| DeltaEBelow | 8/10 | CIEDE2000 完整实现，但仅采样 1/10000 像素 |
| IsValidFormat | 9/10 | 完整的格式/尺寸/位深度检查 |
| **MetadataMatches** | **1/10** | **严重空壳方法！** 创建空 `new ImageMetadata()`，完全不读取文件元数据 |
| ApiEqualsUi | 10/10 | 正确委托给 PixelsEqual |

### TestImageGenerator.cs — 10/10
全部方法真实生成有意义的像素数据到磁盘。确定性 Random(42)，覆盖固体色/渐变/棋盘格/色条/灰度阶/波带片/纹理/Alpha/边界/多格式/高位深共 11 类。

### TestCaseCatalog.cs — 8/10
约 393 个用例，Pipeline 构建正确。但**错误路径仅 3 个用例**，ExpectedErrorMessage 从未被填充。

### CrossChannelVerifier.cs — 10/10
完整实现。但**零调用**（没有任何测试使用它）。

### TestCaseDefinition.cs — 10/10 (数据结构设计良好)
但 TolerancePerChannel, MinPSNR, MinSSIM, MaxDeltaE, ExpectedErrorMessage 全部为**死代码**。

### 其他基础设施

| 文件 | 评分 | 说明 |
|---|---|---|
| TestPipelineBuilder.cs | 10/10 | Fluent API 完整 |
| TestDataCatalog.cs | 7/10 | manifest 缺失时静默退化 |
| TestOutputManager.cs | 10/10 | 完整实现 |
| ResourceMonitor.cs | 9/10 | 简单但正常 |
| **ReferenceImageGenerator.cs** | **N/A** | **文件不存在！** |

---

## 第二部分: API 通道测试审计

### 系统性缺陷 1: "静默跳过"模式 (CRITICAL)

**所有 9 个 API 测试类使用完全相同模式：**
```csharp
try { await EnsureConnectedAsync(); }
catch { _output.WriteLine("Backend not available -- skipping ..."); return; }
```
如果 gRPC 后端未运行，**全部 383+ 个测试静默通过**。

### 系统性缺陷 2: 像素级断言完全缺失 (CRITICAL)

| 测试类 | PixelsEqual? | PSNR? | SSIM? | 实际验证内容 |
|---------|-------------|-------|-------|-------------|
| PluginApiTests | 否 | 否 | 否 | 文件大小 > 0 |
| FormatApiTests | 否 | 否 | 否 | 格式 + 大小 |
| PipelineApiTests | 否 | 否 | 否 | 格式 + 位深 |
| BatchApiTests | 否 | 否 | 否 | 文件大小 > 0 |
| MetadataApiTests | 否 | 否 | 否 | 文件大小 > 0 |
| RegressionApiTests | 否 | 否 | 否 | 格式 + 大小 |
| ContentApiTests | 否 | 否 | 否 | 文件大小 > 0 |
| ErrorPathApiTests | 否 | 否 | 否 | 仅 AnyException |

ImageAssert 的完整工具链**未被任何 API 测试使用**。

### 系统性缺陷 3: 测试名与行为严重脱节

| 测试类 | 名称暗示 | 实际行为 |
|---|---|---|
| RegressionApiTests | 与黄金参考图像快照比对 | 仅检查文件大小 > 0 |
| ContentApiTests | 内容验证 | 仅检查文件大小 > 0 |
| MetadataApiTests | 元数据传透检查 | 仅检查文件大小 > 0 |

### 逐类评分

| 文件 | 评分 | 核心问题 |
|---|---|---|
| ApiTestBase.cs | 9/10 | PluginService 非 gRPC |
| PluginApiTests.cs | 3/10 | 空 catch; 无像素断言; ExpectError 缺陷 |
| FormatApiTests.cs | 3/10 | MinSSIM 死代码 |
| PipelineApiTests.cs | 4/10 | 无拓扑验证 |
| BatchApiTests.cs | 4/10 | 仅文件存在检查 |
| MetadataApiTests.cs | 1/10 | 完全不验证元数据 |
| RegressionApiTests.cs | 1/10 | 无参考图像比较 |
| ContentApiTests.cs | 1/10 | 完全不验证内容 |
| ErrorPathApiTests.cs | 2/10 | ThrowsAny 太宽泛; ExpectedMessage 未用 |

### 其他严重缺陷

- **ReferenceImageGenerator.cs 不存在** — 回归测试的黄金参考图像生成器从未被创建
- **ExpectError 逻辑缺陷** — PluginApiTests.cs:37-41 管线应失败却成功时不会报告失败
- **MetadataMatches 是空壳** — 创建空 ImageMetadata 对象，不读取任何文件数据
- **CrossChannelVerifier 零调用** — 完整实现但任何测试都未使用

---

## 改进建议（优先级排序）

1. **修复静默跳过**: gRPC 连接失败应使用 `Assert.Fail()` 或 `Skip.Always()`
2. **添加像素断言**: 使用 ImageAssert.PixelsEqual/PSNRAbove 将输出与参考图像比较
3. **创建 ReferenceImageGenerator**: 为回归/内容测试生成黄金参考输出
4. **重新实现 MetadataMatches**: 使用真实库读取 EXIF/XMP 数据
5. **修复 ExpectError 逻辑**: 使用 `Assert.ThrowsAsync<SpecificException>`
6. **实现 ExpectedErrorMessage 检查**: 验证后端返回的错误消息内容
7. **集成 CrossChannelVerifier**: 在交叉验证测试中调用 VerifyEquivalence
8. **扩展错误路径测试**: 覆盖超时/取消/无效路径/权限/并发/资源耗尽
9. **移除空壳方法**: 真正实现或删除 MetadataMatches

---

*审计报告结束。*
