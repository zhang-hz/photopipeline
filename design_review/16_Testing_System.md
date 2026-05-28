# GUI 设计审查 — #16 GUI 测试体系

---

## 一、测试体系设计概述

测试体系分为三层，设计文档 TEST_CASE_SPECIFICATION.md 和 TEST_DESIGN_CSHARP.md 定义了完整规格：

| 层 | 名称 | 测试类型 | 设计目标 |
|:--:|------|---------|---------|
| L5 | GUI E2E | FlaUI 自动化 | 44+ 条真实 WPF 窗口操作测试 |
|  | UI Channel | C# 功能测试 | 通过 UI 层执行管线并验证输出 |
| L6 | Cross-Channel | 三通道验证 | Rust 引擎 / gRPC / GUI 三通道逐像素比对 |

## 二、实际实现状态

### 2.1 FlaUI 自动化测试（UIAutomationTests）

| 项目 | 状态 |
|------|------|
| 测试文件数 | 10 个 .cs 文件 |
| 测试方法数 | 44 个 `[Fact]` |
| 其中跳过 | **44 个全部 `[Fact(Skip)]`** |
| 实现的方法体 | **全部为空** |
| 测试框架 | FlaUI.UIA3 已引用 |

**结论：** 100% 未实现。

### 2.2 UI Channel 测试

```
gui/windows/Photopipeline.Tests/FunctionalTests/UiChannel/
├── PipelineUiTests.cs
├── BatchUiTests.cs
├── FormatUiTests.cs
├── MetadataUiTests.cs
├── ContentUiTests.cs
├── PluginUiTests.cs
├── InteractionUiTests.cs
└── UiTestDriver.cs
```

| 组件 | 状态 | 严重度 |
|------|------|:-----:|
| UiTestDriver（10 个操作方法） | **全部为空桩**，返回 Task.CompletedTask | 🔴 |
| PipelineUiTests | 框架存在，无真实执行 | 🔴 |
| BatchUiTests | 框架存在，无真实执行 | 🔴 |
| FormatUiTests | 框架存在，无真实执行 | 🔴 |
| MetadataUiTests | 框架存在，无真实执行 | 🔴 |
| ContentUiTests | 框架存在，无真实执行 | 🔴 |
| PluginUiTests | 框架存在，无真实执行 | 🔴 |
| InteractionUiTests | **4 个测试调用 Assert.Fail("not implemented")** | 🔴 |

**`UiTestDriver` 中的空桩操作：**
- `ImportImageAsync` → `return Task.CompletedTask`
- `SelectImageAsync` → `return Task.CompletedTask`
- `AddNodeToPipelineAsync` → `return Task.CompletedTask`
- `SetParameterAsync` → `return Task.CompletedTask`
- `RunPipelineAsync` → `return Task.CompletedTask`
- `ExportOutputAsync` → `throw new NotImplementedException("Skia drag not yet implemented")`
- `RunBatchAsync` → `return Task.CompletedTask`
- `CheckOutputDirectoryAsync` → `return Task.CompletedTask`

### 2.3 Cross-Channel 测试

```
gui/windows/Photopipeline.Tests/FunctionalTests/CrossChannel/
├── CrossChannelTestBase.cs
├── CrossChannelVerifyTests.cs
├── CrossChannelPluginTests.cs
├── CrossChannelEdgeTests.cs
├── CrossChannelConsistencyTests.cs
└── CrossChannelServiceTests.cs
```

| 组件 | 状态 | 严重度 |
|------|------|:-----:|
| 测试用例定义（CCV-001~060） | 全部编写 | ✅ |
| CrossChannelVerifier 实现 | 质量高（逐像素比对） | ✅ |
| Channel 1（Rust 引擎） | 真实执行 | ✅ |
| Channel 2（gRPC C#） | 真实执行 | ✅ |
| **Channel 3（GUI 通道）** | **始终返回 null** | 🔴 |
| **VerifyEquivalence 调用** | **从未实际执行（UI 通道为 null）** | 🔴 |

### 2.4 测试统计数据

| 测试类型 | 总数 | 有效 | 跳过/桩 | 有效占比 |
|---------|:---:|:----:|:-------:|:--------:|
| ViewModel 单元测试 | ~73 | ~68 | ~5 | 93% |
| API Channel | ~60 | ~60 | 0 | 100% |
| FlaUI 自动化 | 44 | 0 | 44 | **0%** |
| UI Channel | ~30 | 0 | ~30 | **0%** |
| Cross-Channel | 60 | 0 | 60（UI 通道为 null） | **0%** |

## 三、测试体系完成度

| 层 | 设计规格 | 实际实现 | 完成度 |
|:--:|---------|---------|:-----:|
| L5 FlaUI | 44 条测试 | 全部跳过 | **0%** |
| L5 UI Channel | ~30 条测试 | 全部空桩 | **0%** |
| L6 Cross-Channel | 60 条测试 | 框架存在但 UI 通道为 null | **0%** |
| **GUI 测试总计** | **~134 条** | **0 条真实执行** | **0%** |

## 四、最终总结

> **整个 GUI 自动化测试体系处于骨架/空桩阶段。** 虽然测试文件结构完整、命名规范、CrossChannelVerifier 实现质量高，但没有一条测试真实执行过 GUI 操作。这意味任何 GUI 变更都没有自动化回归保障。
