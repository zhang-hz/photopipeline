# Photopipeline Phase 5-6 UI 通道测试/交叉验证测试/UIAutomation 测试 深度审计报告

**审计日期:** 2026-05-26
**审计范围:** 22 个测试文件 (实际存在 19 个源文件)
**审计方法:** 逐方法逐行阅读，追踪逻辑流和操作流

---

## 总体结论

整个 UI 测试体系处于**骨架/桩阶段**。UiTestDriver 中的所有 UI 操作方法全部是空桩（stub），包含 `// TODO Phase 5` 注释，全部返回 `Task.CompletedTask`。UIAutomationTests 全部 44 个测试方法被 `[Fact(Skip)]` 跳过且方法体为空。**交叉验证从未真正发生过。**

---

## 第一部分: UI Channel 测试审计

### UiTestDriver.cs — 界面操作驱动 ⭐ 核心问题

**所有操作全部是空桩 (stubs):**

| 方法 | 实际执行内容 | 是否有 UI 交互 |
|---|---|---|
| `RunFullWorkflowAsync` | 调用全部空桩方法 | **否** |
| `ImportImageAsync` | `// TODO Phase 5: Click Import button` → `Task.CompletedTask` | **否** |
| `SelectImageAsync` | `// TODO Phase 5: Click image in filmstrip` → `Task.CompletedTask` | **否** |
| `AddNodeToPipelineAsync` | `// TODO Phase 5: Find plugin in browser` → `Task.CompletedTask` | **否** |
| `SetNodeParameterAsync` | `// TODO Phase 5: Find parameter control` → `Task.CompletedTask` | **否** |
| `RunPipelineAsync` | `// TODO Phase 5: Click Run button` → `Task.CompletedTask` | **否** |
| `WaitForPipelineCompletionAsync` | `// TODO Phase 5: Monitor progress` → `Task.CompletedTask` | **否** |
| `ExportOutputAsync` | `// TODO Phase 5: Click Export button` → `Task.CompletedTask` | **否** |
| `CancelPipelineAsync` | `// TODO Phase 5: Click Cancel button` → `Task.CompletedTask` | **否** |
| `ToggleSplitViewAsync` | `// TODO Phase 5: Click split view toggle` → `Task.CompletedTask` | **否** |

**致命缺陷:** `RunFullWorkflowAsync` 看起来像完整的 UI 工作流（导入 → 添加节点 → 设置参数 → 运行 → 等待 → 导出），但所有步骤都是空方法的 `await`，瞬间返回。应用虽通过 `Process.Start()` 启动，但没有执行任何 UI 交互。

### 短路逻辑分析

8 个 UiChannel 测试文件全部包含:
```csharp
if (!File.Exists(AppPath)) { _output.WriteLine("..."); return; }
```
这使得测试在应用不存在时**静默通过**（无断言失败），而非明确标记为跳过。

### 评分总表

| 文件 | 每方法平均评分 | 核心问题 |
|---|---|---|
| UiTestBase.cs | 5/10 | 进程管理正确，缺窗口就绪验证 |
| UiTestDriver.cs | 0/10 | 全部空桩 |
| PluginUiTests.cs | 1/10 | 短路 + 空桩 |
| FormatUiTests.cs | 1/10 | 同上 |
| PipelineUiTests.cs | 1/10 | 同上 |
| BatchUiTests.cs | 1/10 | 同上 |
| MetadataUiTests.cs | 1/10 | 同上 |
| RegressionUiTests.cs | 1/10 | 同上 |
| ContentUiTests.cs | 1/10 | 同上 |
| InteractionUiTests.cs | 0/10 | 空桩 + 2 个 case 无任何操作 |

---

## 第二部分: Cross Channel 测试审计

### CrossChannelTestBase.cs — 交叉验证基类

**API 通道** (`RunApiChannelAsync`): 通过 gRPC 连接后端服务，创建管线、执行、返回输出路径 — **真实的后端调用**。

**UI 通道** (`RunUiChannelAsync`): 使用 UiTestDriver，所有操作是空桩。异常被 `catch` 吞噬并返回 `null`。

**交叉验证** (`VerifyCrossChannelAsync`):
```csharp
var apiOutput = await RunApiChannelAsync(...);   // 真实输出
var uiOutput = await RunUiChannelAsync(...);      // null (异常被吞)
if (uiOutput == null) return false;               // "跳过"
CrossChannelVerifier.VerifyEquivalence(...)        // ← 从未执行!
```

**致命缺陷:** API 通道产生真实输出；UI 通道不产生输出。异常被吞噬使 UI 失败被当作"跳过"，测试总是打印 "UI channel skipped" 后通过。`CrossChannelVerifier.VerifyEquivalence` 从未被实际执行到。

### 评分

| 文件 | 评分 | 核心问题 |
|---|---|---|
| CrossChannelTestBase.cs | 2/10 | API 真实, UI 空桩, 异常被吞 |
| CrossChannelVerifier.cs | 8/10 | 实现优秀但从未被调用至实质比较 |
| PluginCrossTests.cs | 1/10 | UI 永远跳过 |
| FormatCrossTests.cs | 1/10 | 同上 |
| PipelineCrossTests.cs | 1/10 | 同上 |
| BatchCrossTests.cs | 1/10 | 同上 + early return 缺陷 |
| RegressionCrossTests.cs | 1/10 | 同上 |

---

## 第三部分: UIAutomationTests 审计

**全部 44 个测试方法**均标记 `[Fact(Skip = "Requires UI context with WinAppDriver")]`，方法体全部为空 `{ }`。

**无 FlaUI 引用:** 项目不引用 FlaUI NuGet 包。仅引用 `Appium.WebDriver` 但也未被任何代码使用。

| 文件 | 测试方法数 | 实际执行 | 评分 |
|---|---|---|---|
| StartupSmokeTests.cs | 5 | 0 | 0/10 |
| MainWindowUITests.cs | 6 | 0 | 0/10 |
| FilmstripUITests.cs | 5 | 0 | 0/10 |
| PreviewUITests.cs | 5 | 0 | 0/10 |
| PipelineEditorUITests.cs | 5 | 0 | 0/10 |
| PluginPanelUITests.cs | 5 | 0 | 0/10 |
| BatchPanelUITests.cs | 4 | 0 | 0/10 |
| CrossPanelUITests.cs | 3 | 0 | 0/10 |
| ErrorHandlingUITests.cs | 3 | 0 | 0/10 |
| EndToEndWorkflowTests.cs | 3 | 0 | 0/10 |
| **总计** | **44** | **0** | **0/10** |

---

## 按审计维度的总体评估

| 审计维度 | 评估 | 详情 |
|---|---|---|
| **是否真正模拟界面交互?** | **否** | UiTestDriver 全部操作是空桩。UIAutomationTests 全部被跳过 |
| **是否外部读取输出图片断言?** | **部分** | ImageAssert 和 CrossChannelVerifier 实现正确，但从未被 UI 测试路径实际触及 |
| **完整操作流覆盖?** | **否** | UiChannel 测试方法启动进程后不执行任何 UI 操作 |
| **短路检测?** | **严重** | 8 个 UiChannel 文件包含短路径使测试在没有应用时虚假通过 |
| **交叉验证的真实性?** | **虚假** | UI 通道异常被吞，永远返回 null，未发生任何交叉比对 |
| **UIAutomationTests 的 FlaUI 使用?** | **无** | 44 个测试全部 Skip，无代码体 |

## 改进优先级

1. **最高优先级: 实现 UiTestDriver 的操作方法。** 引入 FlaUI NuGet 包，实现每个桩方法
2. **移除短路逻辑。** 改为 `Skip.IfNot()` 确保测试在无应用时明确标记为跳过
3. **实现 UIAutomationTests。** 移除 Skip，编写实际的 FlaUI 自动化代码
4. **修复 CrossChannel 异常吞噬。** UI 错误不应被吞掉
5. **InteractionUiTests 空 case 添加实现**

## 最终结论

当前 Phase 5-6 的 UI 通道测试、交叉验证测试和 UIAutomation 测试**处于不可用状态**。所有 UI 交互操作都是空桩（`// TODO Phase 5`），所有 UIAutomationTests 被 `Skip` 跳过，交叉验证虽然 API 通道工作正常但 UI 通道从不产生有效结果。`ImageAssert` 和 `CrossChannelVerifier` 等基础设施类实现质量良好，但从未被 UI 测试路径实际执行到。**整个 Phase 5-6 的 UI 测试体系需要从零开始实现。**

---

*报告结束。*
