# Photopipeline Phase 1-3 单元测试 & 场景测试 深度审计报告

**审计日期:** 2026-05-26
**审计范围:** 14 个测试文件, 111 个测试方法
**审计方法:** 逐方法逐行阅读，追踪逻辑流和操作流

---

## 汇总统计

| 文件 | 测试方法数 | 平均分 | 短路测试数 | 主要问题 |
|------|-----------|--------|-----------|---------|
| MainViewModelTests.cs | 6 | 5.5 | 1 | ZoomCommands 无断言 |
| BatchViewModelTests.cs | 14 | 6.6 | 0 | 后两个测试过于简单 |
| FilmstripViewModelTests.cs | 16 | 6.2 | 2 | CopyPath/OpenInExplorer 无断言 |
| PreviewViewModelTests.cs | 11 | 7.3 | 0 | 缩放使用模糊断言 |
| PipelineEditorViewModelTests.cs | 18 | 7.4 | 0 | 质量最高 |
| PluginBrowserViewModelTests.cs | 4 | 6.5 | 0 | SearchText 存在 flaky 风险 |
| SettingsViewModelTests.cs | 4 | 6.3 | 0 | 体量太小 |
| PluginServiceTests.cs | 6 | 1.8 | 6 | 全部为反射检查，无实质测试 |
| EndToEndScenarioTests.cs | 4 | 4.8 | 0 | FullWorkflow 名不副实 |
| ErrorRecoveryScenarioTests.cs | 6 | 1.5 | 6 | 全文件严重缺陷 |
| BatchScenarioTests.cs | 6 | 6.7 | 0 | 与单元测试大量重复 |
| CrossPanelScenarioTests.cs | 4 | 6.5 | 0 | FilmstripToBatch 名不副实 |
| FilmstripScenarioTests.cs | 6 | 6.8 | 0 | MultiSelect_ThenRemove 名不副实 |
| PreviewScenarioTests.cs | 6 | 7.5 | 0 | 场景测试中质量最高 |
| **总计** | **111** | **5.7** | **15** | |

---

## 关键发现

### 最严重问题

1. **ErrorRecoveryScenarioTests.cs (评分 1.5)** — 全文件需要重写。6 个测试方法中有 4 个名不副实，2 个完全没有断言。Mock 中正确设置了 `ThrowsAsync` 但从未被触发。该文件冠以"错误恢复"之名，却未触发任何错误路径。

2. **PluginServiceTests.cs (评分 1.8)** — 6 个测试全部使用反射检查接口类型元数据，不测试任何具体实现。这些测试在编译时就能被完全确定，在测试套件中无实际价值。

3. **15 个短路测试** — 其中 11 个缺少断言（如 `MainViewModelTests.ZoomCommands_DelegateToPreview`、`FilmstripViewModelTests.CopyPath_NullImage_Noop` 等），4 个名称与行为严重脱节（如 `ErrorRecoveryScenarioTests.ViewModelBase_IsBusy_PreventsConcurrentOperation`）。

4. **端到端判定缺失** — 未发现任何测试验证了最终的图像输出或业务结果。所有测试均停留在 ViewModel 状态属性层面，没有测试验证图像解码/编码是否生成正确的像素数据，流水线执行是否输出有效的图像文件，批处理是否生成正确的输出文件，或任何实际的 gRPC 服务通信。Phase 1-3 的测试本质上都是 ViewModel 状态转换测试和接口契约检查。

### 亮点

- **PipelineEditorViewModelTests.cs (评分 7.4)** — 测试质量最高，18 个测试方法全部有效，全面覆盖了节点操作、边连接、环检测、缩放和验证
- **PreviewScenarioTests.cs (评分 7.5)** — 场景测试中质量最高，缩放步进使用精确值断言
- 大部分 ViewModel 的初始状态验证是充分的

---

*完整的逐方法审计细节见代理输出。*
