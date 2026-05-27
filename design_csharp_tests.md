# Photopipeline C# GUI 测试系统详细设计
> 编写日期：2026-05-26
> 基于代码库现状分析，涵盖测试各层（L3 单元测试 / L4 gRPC 集成测试 / L5 GUI E2E 测试）的完整设计方案。
---
## 目录
1. [现状分析](#1-现状分析)
2. [总体文件结构](#2-总体文件结构)
3. [Layer 3：C# 单元测试（60条）](#3-layer-3c-单元测试60条)
4. [Layer 4：C# gRPC 集成测试（60条）](#4-layer-4c-grpc-集成测试60条)
5. [Layer 5：GUI E2E 测试（40条）](#5-layer-5gui-e2e-测试40条)
6. [UiTestBase / UiTestDriver 完全重写设计](#6-uitestbase--uitestdriver-完全重写设计)
7. [基础设施修复清单](#7-基础设施修复清单)
8. [测试包依赖更新](#8-测试包依赖更新)
9. [与现有可重用基础设施的集成方式](#9-与现有可重用基础设施的集成方式)
---
## 1. 现状分析
### 1.1 已有单元测试（UnitTests/）
| 文件 | 现有测试数 | 关注点 |
|------|-----------|--------|
| FilmstripViewModelTests.cs | 18 | 初始状态、CRUD、选择、排序选项、格式过滤、缩略图大小 |
| PreviewViewModelTests.cs | 13 | 默认值、缩放上下限、重置、FitToWindow、ToggleSplit、Pan |
| PipelineEditorViewModelTests.cs | 19 | 初始状态、节点增删、连接/去重/环检测、画布操作、验证 |
| BatchViewModelTests.cs | 16 | 初始状态、队列管理、Start/Stop/Pause、清除完成项 |
| PluginBrowserViewModelTests.cs | 4 | 初始状态、选择插件提取默认参数、事件、搜索过滤 |
| SettingsViewModelTests.cs | 4 | 从服务加载、预定义列表、默认值 |
| MainViewModelTests.cs | 8 | 子VM存在性、Backend状态、Zoom委托、窗口大小 |
| **合计** | **~82** | |
### 1.2 已有测试基础设施
| 文件 | 状态 | 备注 |
---
## 2. 总体文件结构

```
Photopipeline.Tests/
  UnitTests/                        # Layer 3
    ViewModels/
      ViewModelBaseTests.cs         # 新增 - 5 tests
      FilmstripViewModelTests.cs    # 增补 - 从18到22+ tests
      PreviewViewModelTests.cs      # 增补 - 从13到18+ tests
      PipelineEditorViewModelTests.cs  # 增补 - 从19到25+ tests
      BatchViewModelTests.cs        # 增补 - 从16到22+ tests
      PluginBrowserViewModelTests.cs  # 增补 - 从4到8+ tests
      SettingsViewModelTests.cs      # 增补 - 从4到8+ tests
      MainViewModelTests.cs         # 保持

  FunctionalTests/
    ApiChannel/                     # Layer 4 (现有文件增补)
      ApiTestBase.cs                # 修复 PluginService 构造
      PipelineApiTests.cs           # 增补像素级断言
      BatchApiTests.cs              # 增补断言
      PluginApiTests.cs             # 增补
      FormatApiTests.cs             # 增补
      ContentApiTests.cs            # 增补
      MetadataApiTests.cs           # 增补
      ErrorPathApiTests.cs          # 增补
      RegressionApiTests.cs         # 增补

    UiChannel/                      # Layer 5 (重写)
      UiTestBase.cs                 # 完全重写 - FlaUI替代Thread.Sleep
      UiTestDriver.cs               # 完全重写 - 10方法真实FlaUI操作
      PipelineUiTests.cs            # 重写 - 从文件存在到像素级比较
      BatchUiTests.cs               # 重写
      ContentUiTests.cs             # 重写
      FormatUiTests.cs              # 重写
      PluginUiTests.cs              # 重写
      MetadataUiTests.cs            # 重写
      RegressionUiTests.cs          # 重写
      InteractionUiTests.cs         # 重写

    CrossChannel/                   # 不变
      CrossChannelTestBase.cs
      ...

    Infrastructure/                 # 修复
      ImageAssert.cs                # 修复 MetadataMatches
      ...
```
---
## 3. Layer 3：C# 单元测试（60条）

### 3.1 ViewModelBaseTests.cs - 5条（新建文件）

```csharp
// 文件: UnitTests/ViewModels/ViewModelBaseTests.cs
// 测试目标: Photopipeline.Helpers.ViewModelBase
// 使用: Moq + xUnit
// 需要一个测试桩继承 ViewModelBase 以便测试 protected 成员
```
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
extglob
