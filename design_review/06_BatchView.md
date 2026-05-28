# GUI 设计审查 — #06 BatchView 批量处理

---

## 一、界面设计（UI Design）

### 1.1 设计文档定义的界面

ARCHITECTURE.md 第 8.1 节的布局图将批量进度放在 **左栏底部**（胶片条下方），但实际实现中将批量放在 **右栏底部**。

第 9 章"批量处理"定义的批量工作流：

```
1. 加载图片 → 自动提取 EXIF 快照
2. 加载 GPX 轨迹（可选）→ 按时间戳自动插值 GPS
3. 自动分组：按 ISO、GPS 聚类、时间间隔 → 应用分组预设
4. 选择单张图片 → 按需细调覆盖参数
5. 验证 → 检查参数完整性
6. 导出 → 并行处理、进度流、断点续传支持
```

第 9.2 节定义的 TOML 配置结构（即预期 UI 应支持的配置项）：

```toml
[metadata]
name = "我的 HDR 管线"
version = "1.0"

[[nodes]]
...（节点定义）

# 逐图覆盖（仅与模板的差异）
[[overrides]]
image = "DSC0003.ARW"
params.gps = { lat = 30.5728, lon = 104.0668 }

# 自动分组
[[groups]]
name = "高 ISO"
condition = "exif.iso >= 1600"
params.ai_denoise = { strength = 0.9 }

[batch]
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

**设计预期的 UI 功能：**
- 自动分组管理（按 ISO / GPS 聚类 / 时间间隔）
- 逐图覆盖参数编辑
- 输出模板 `{date}/ {filename}/{camera}` 占位符
- 并发控制
- 输出冲突策略选择（skip / overwrite）
- 断点续传开关
- 实时进度流

### 1.2 实际实现的界面

BatchView.xaml 实现（右栏底部）：

```
┌──────────────────────────────────────┐
│ [▶ Start] [⏸ Pause] [▶ Resume] [⏹ Stop] │
│ [Clear Done]       12 img/min        │ ← 控制栏 + 速度
├──────────────────────────────────────┤
│ ┌──────────────────────────────────┐ │
│ │ ● photo001.arw           Queued  │ │ ← 队列项
│ │   1920x1080 ARW                  │ │
│ ├──────────────────────────────────┤ │
│ │ ● photo002.arw             Done  │ │
│ │   6000x4000 ARW                  │ │
│ ├──────────────────────────────────┤ │
│ │ ● photo003.arw           Failed  │ │
│ │   2400x1600 JPEG                 │ │
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ ████████████████░░░░░ 72%            │
│ 18 done, 1 failed, 25 total         │
│                      00:03:15        │
│                      ~00:01:20 left  │ ← 进度+计时
├──────────────────────────────────────┤
│ Output Settings                      │
│ ┌──────────────────────────────────┐ │
│ │ Directory: [___________] [...]  │ │
│ │ Template:  [___________]        │ │
│ │ Format:    [PNG           ▼]    │ │
│ │ Quality:   [===○========] 80%   │ │
│ │ Parallel:  [4  ▲▼]             │ │
│ └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

**布局结构（4 行）：**
1. **控制栏** Row 0：Start / Pause / Resume / Stop / Clear Done + SpeedInfo
2. **队列列表** Row 1：ListBox + AllowDrop，每项含状态圆点 + 文件名/分辨率/格式 + 状态文字
3. **进度区域** Row 2：ProgressBar + 完成/失败/总计计数 + 已用时间 + 预计剩余
4. **输出设置** Row 3：Border，5 行配置（Directory / Template / Format / Quality / Parallel）

### 1.3 视觉元素

| 元素 | 实际实现 | 评估 |
|------|---------|:----:|
| 控制按钮 | Start/Pause/Resume/Stop/Clear Done + SymbolIcon | ✅ |
| 速度信息 | "12 img/min" 格式 | ✅ |
| 队列项 | 状态圆点（灰/绿/红）+ 文件名 + 分辨率 + 格式 + 状态文字 | ✅ |
| 状态圆点颜色 | None=Gray / Overridden=LimeGreen / Error=Red | ✅ |
| 进度条 | Value 绑定 OverallProgress，Max=100 | ✅ |
| 计时器 | 已用时间 + 预计剩余 | ✅ |
| 输出设置面板 | 5 行 Grid 布局，标签固定宽度 64px | ✅ |

### 1.4 状态覆盖

| 状态 | 设计文档 | 实际实现 | 匹配度 |
|------|---------|---------|:-----:|
| **空态**（无队列项） | 未指定 | 空 ListBox，无占位提示 | ❌ |
| **队列就绪** | 等待开始 | 列表显示 + Start 按钮可用 | ✅ |
| **执行中** | 进度流 | ProgressBar + 计时 + 速度信息 | ✅ |
| **暂停态** | 断点续传 | Resume 按钮可见（IsPaused → BoolToVisibility） | ✅ |
| **完成态** | 所有项 Done | 进度 100% + 计时停止 | ✅ |
| **错误态** | 逐项失败 | 红色圆点 + "Failed" 文字 | ✅ |
| **空输出设置** | 未指定 | Directory/Template 可空，无校验 | ❌ |

---

## 二、功能设计（Functional Design）

### 2.1 职责边界

BatchViewModel 负责：
- 批量队列管理（添加/移除/清除已完成）
- 批量执行（通过 gRPC BatchService）
- 进度追踪（OverallProgress + 逐项状态）
- 计时/速度/预计剩余
- 输出设置（目录/模板/格式/质量/并行数）
- 暂停/恢复/停止

### 2.2 数据流向

```
胶片条 → SendToBatch → BatchVM.BatchQueue

用户点击 Start → StartBatch() → BatchService.SubmitAsync()
  → gRPC SubmitBatch → 后端开始处理
  → GetProgressAsync() stream → 实时更新进度

用户点击 Pause → 取消 gRPC 流（客户端侧）
用户点击 Resume → 重新提交全部（非仅剩余）
用户点击 Stop → CancelAsync() gRPC → 后端取消
```

### 2.3 用户操作流程

| 步骤 | 操作 | 系统响应 | 状态变化 |
|------|------|---------|---------|
| 1 | 从胶片条选择图像 → To Batch | 图像加入队列 | 空态 → 队列就绪 |
| 2 | 设置输出参数 | 参数绑定 | 配置就绪 |
| 3 | 点击 Start | 提交给后端 | 执行中 |
| 4 | 后端处理中 | 实时进度流 → ProgressBar + 计时 | 进度变化 |
| 5a | 点击 Pause | 取消流（客户端侧） | 暂停态 |
| 5b | 点击 Resume | **重新提交全部**（非仅剩余） | 执行中 |
| 5c | 点击 Stop | gRPC Cancel | 停止 |
| 6 | 全部完成 | ProgressBar 100% | 完成态 |
| 7 | 点击 Clear Done | 移除已完成项 | 完成项清除 |

### 2.4 与其他组件的协作

| 协作方 | 关系 | 实现 |
|-------|------|------|
| FilmstripViewModel | 接收图像到队列 | SendToBatch 命令 |
| PipelineEditorViewModel | 获取当前管线作为批量处理模板 | CurrentPipeline 属性 |
| BatchService | 提交/进度/取消 | gRPC 客户端 |

---

## 三、实现程度（Implementation Assessment）

### 3.1 已实现

- ✅ 队列列表（拖放支持 + 状态圆点 + 文件名/分辨率/格式/状态）
- ✅ Start / Pause / Resume / Stop / Clear Done 按钮
- ✅ 整体进度条（Value=OverallProgress%）
- ✅ 完成/失败/总计计数
- ✅ 已用时间 + 预计剩余时间
- ✅ 速度（img/min）
- ✅ 输出设置（Directory / Template / Format / Quality / Parallel）
- ✅ 输出目录浏览（BrowseOutputDirectoryCommand）
- ✅ JpegQuality Slider（0-100）
- ✅ ParallelCount NumberBox（1-32）
- ✅ 拖放添加队列项

### 3.2 未实现

| 缺失项 | 设计依据 | 严重度 |
|-------|---------|:-----:|
| ❌ **自动分组管理 UI**（ISO/GPS/时间条件） | 第 9.1 节"自动分组" | 🔴 严重 |
| ❌ **逐图覆盖参数编辑 UI** | 第 9.1 节"选择单张图片 → 按需细调覆盖参数" | 🔴 严重 |
| ❌ **输出冲突策略选择**（skip / overwrite） | 第 9.2 节 on_conflict 配置 | 📝 功能缺失 |
| ❌ **断点续传开关 UI**（resume = true） | 第 9.2 节 batch.resume | 📝 功能缺失 |
| ❌ **输出模板占位符提示**（{date}/{filename}/{camera}） | 第 9.2 节 output_pattern | 📝 体验缺失 |
| ❌ **GPX 轨迹加载 UI** | 第 9.1 节"加载 GPX 轨迹" | 📝 功能缺失 |
| ❌ **管线模板选择**（而非仅当前管线） | 第 9.2 节 TOML 配置 | 📝 功能缺失 |
| ❌ **空队列占位提示** | UX 标准 | 📝 体验缺失 |

### 3.3 关键交互问题

1. **暂停/恢复功能损坏** — 暂停仅取消客户端 gRPC 流，恢复重新提交全部项目（含已完成）。用户感知为"恢复后重复处理"
2. **FileNameTemplate 未传递到后端** — 界面上有 Template 输入框，但 `BatchSpec.FilePattern` 只是文件路径 join(";")，模板字符串从未使用
3. **EmbedMetadata 设置丢失** — 设置对话框中有此选项但从未传递到 BatchSpec
4. **进度更新粒度粗** — gRPC 进度流更新总体计数，但队列中单个 `ImageEntry.Status` 仅在完成后批量更新，非实时
5. **暂停状态仅前端** — 后端没有收到暂停信号，仍在处理

### 3.4 设计偏离汇总

| 偏离项 | 设计文档 | 实际 | 影响评估 |
|--------|---------|------|---------|
| 批量位置 | 左栏（胶片条下方） | 右栏（管线编辑器下方） | ⚪ 布局决策变更 |
| 自动分组 | ISO/GPS/时间条件式分组 | 无分组 UI | 🔴 完全缺失 |
| 逐图覆盖 | 每图可调参数 | 无逐图编辑 UI | 🔴 完全缺失 |
| 冲突策略 | skip/overwrite 选择 | 无选择 UI | 📝 缺失 |
| 断点续传 | resume = true | 无 UI 开关，且功能损坏 | 🔴 功能损坏 |
| 输出模板 | {date}/{filename} 等 | 有输入框但不生效 | 🔴 功能损坏 |

### 3.5 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 队列管理 | 90% | 添加/显示/拖放全部实现 |
| 进度追踪 | 90% | 进度条/计时/计数/速度完整 |
| 暂停/恢复/停止 | 30% | UI 有按钮但功能损坏 |
| 输出设置 | 80% | UI 字段完整但部分不传递到后端 |
| 自动分组 | 0% | **完全未实现** |
| 逐图覆盖 | 0% | **完全未实现** |
| 冲突/续传/GPX | 0% | **完全未实现** |
| 空态/提示 | 0% | 无任何引导提示 |
| **综合** | **40%** | **基础框架和进度展示好，但高级批量功能缺失，暂停/恢复/模板损坏** |
