# GUI 设计审查 — #05 PipelineEditorView 管线编辑器

---

## 一、界面设计（UI Design）

### 1.1 设计文档定义的界面

ARCHITECTURE.md 第 8.1 节将右栏定义为"插件控制面板"，但实际布局中管线编辑器占据了右栏上半部分。

设计文档中关于管线编辑器的描述散见于第 8 章和第 9 章：

- **第 8.1 节布局图：** 右栏 = "插件控制面板（选中节点的完整详情，继承 schema）"
- **第 8.1 节布局图：** 中栏下方 = "管线总览"，含"迷你 DAG + 状态"
- **架构中隐含：** 完整 DAG 编辑能力（节点添加/删除/连接、参数编辑、执行）

### 1.2 实际实现的界面

PipelineEditorView.xaml 实现（右栏顶部）：

```
┌────────────────────────────────────┬────────┐
│ [New] [Save] [Validate] │ [Run]    │        │ ← 工具栏
│   🔍+ 🔍- 1.0x [Fit]    │ [Cancel] │        │
├────────────────────────────────────┤        │
│                                    │ Node:  │ ← 属性面板
│         SkiaDAGCanvas              │ transform│位置=右
│                                    │        │
│    ┌────────────┐                  │ Parameters│
│    │ raw_input  │                  │ ┌────┐ │
│    │  ○───┐     │                  │ │    │ │
│    └──────│     │                  │ └────┘ │
│           │     │                  │────────│
│    ┌──────│─────┴──┐               │[Remove]│
│    │      ○─→→→→→→○│trans...       │        │
│    │      │         │              │        │
│    └──────┴─────────┘              │        │
│                                    │        │
│     [MiniMap]                      │        │
├────────────────────────────────────┴────────┤
│ Validation: OK           Executing... ⟳     │ ← 状态栏
└─────────────────────────────────────────────┘
```

**布局结构（2 列 × 3 行）：**
- 工具栏 Row 0, ColSpan 2：New / Save / Validate / Separator / Run / Cancel / Separator / Zoom+ / Zoom- / Scale / Fit
- 左栏 Row 1：SkiaDAGCanvas（SkiaSharp 全功能 DAG 渲染）
- 右栏 Row 2：属性面板，宽度 240px，条件显示（SelectedNode!=null），含：
  - 节点标签 + PluginId
  - Parameters 标题 + 动态参数控件（ParameterPanel）
  - Separator + Remove Node 按钮（Danger 样式）
- 状态栏 Row 2, ColSpan 2：ValidationMessage + ExecutionStatus + ProgressRing

### 1.3 SkiaDAGCanvas 视觉元素

| 元素 | 实现 | 评估 |
|------|------|:----:|
| 网格背景 | 深浅交替网格线 | ✅ |
| 节点渲染 | 阴影 + 边框 + 标签 + 端口 (○) | ✅ |
| 边（连线） | 贝塞尔曲线 + 选中高亮 | ✅ |
| 端口连接 | 拖拽 ○ → ○ 建立连接 | ✅ |
| 鼠标滚轮缩放 | 以鼠标位置为中心 | ✅ |
| 中键平移 | 拖动平移 | ✅ |
| 迷你地图 | 右下角缩略图 + 视口矩形 | ✅ |
| 节点拖动 | 拖动改变位置 | ✅ |
| 禁用节点 | 半透明覆盖层 | ✅ |
| 框选（BoxSelecting） | 枚举值存在但**未实现** | ❌ |

### 1.4 状态覆盖

| 状态 | 设计文档 | 实际实现 | 匹配度 |
|------|---------|---------|:-----:|
| **空态**（无管线） | 未指定 | 空白画布，无"创建新管线"提示 | ❌ |
| **编辑态**（有节点） | 完整 DAG 编辑 | 节点/边/拖拽/参数编辑 | ✅ |
| **验证结果** | 未指定 | ValidationMessage 显示 | ✅ |
| **执行中** | 未指定 | ProgressRing + ExecutionStatus + Cancel 按钮 | ✅ |
| **错误态**（验证失败） | 未指定 | ValidationMessage 显示错误 + Run 禁用 | ✅ |
| **选中节点** | 完整详情面板 | 属性面板显示参数控件 | ✅ |

---

## 二、功能设计（Functional Design）

### 2.1 职责边界

PipelineEditorViewModel + SkiaDAGCanvas 负责：
- 管线创建/保存/验证/执行（通过 gRPC）
- DAG 图可视化（节点 + 有向边）
- 节点操作（添加/删除/连接/断开/参数编辑）
- 环检测
- 选中节点的参数编辑
- 执行进度和状态反馈
- 触发 Preview 更新

### 2.2 数据流向

```
PluginBrowser 拖放插件 → OnNodeDropped → AddNodeAt()
  → Nodes ObservableCollection 更新 → DAGCanvas 重绘

用户连接端口 → OnPortsConnected → ConnectNodesCommand
  → Edges ObservableCollection 更新 → DAGCanvas 重绘

用户编辑参数 → ParameterControl callback
  → UpdateNodeParameterCommand → node.Params 更新

点击 Validate → gRPC ValidateAsync → ValidationMessage
点击 Run → gRPC ExecuteAsync → ExecuteProgress 流 → ExecutionStatus
点击 Save → gRPC CreatePipelineAsync → 服务器保存管线
```

### 2.3 用户操作流程

| 步骤 | 操作 | 系统响应 | 状态变化 |
|------|------|---------|---------|
| 1 | 从插件浏览器拖放插件到画布 | 创建节点 | 空态 → 编辑态 |
| 2 | 拖拽节点输出端口到另一节点输入 | 创建边 | 边建立 |
| 3 | 点击选中节点 | 属性面板出现 | 选择态 |
| 4 | 在属性面板调整参数 | 参数更新 | 参数变化 |
| 5 | 点击 Validate | gRPC 验证 | 验证结果 |
| 6 | 点击 Run | gRPC 执行 | 执行中 → 完成 |
| 7 | 点击 Save | gRPC 创建 | 管线持久化 |
| 8 | 点击 Remove Node | 删除选中节点 | 节点移除 |
| 9 | 滚轮缩放/拖动平移 | 画布视角调整 | 视角变化 |

### 2.4 与其他组件的协作

| 协作方 | 关系 | 实现 |
|-------|------|------|
| PluginBrowserView | 接收拖放节点 | DragDrop Event → OnNodeDropped |
| PreviewViewModel | 执行后触发预览更新 | PreviewUpdateRequested 事件 |
| PipelineService | 管线 CRUD + 执行 | gRPC 客户端 |
| MainViewModel | 状态转发 | 验证结果 → 全局状态栏 |

---

## 三、实现程度（Implementation Assessment）

### 3.1 已实现

- ✅ DAG 画布（SkiaSharp 硬件加速，网格/节点/边/端口）
- ✅ 节点添加（拖放 + 程序化）
- ✅ 节点删除（Remove Node 按钮）
- ✅ 端口连接（贝塞尔曲线）
- ✅ 环检测
- ✅ 节点拖动重定位
- ✅ 画布缩放/平移
- ✅ 迷你地图
- ✅ 管线验证（gRPC）
- ✅ 管线执行（gRPC streaming + 进度反馈）
- ✅ 执行取消
- ✅ 选中节点参数编辑面板（基础类型）
- ✅ 验证/执行状态信息

### 3.2 未实现

| 缺失项 | 设计依据 | 严重度 |
|-------|---------|:-----:|
| ❌ **加载已保存管线** | 管线管理基本操作 | 🔴 严重 — 只能 Create 不能 Load |
| ❌ **断开单条边（DisconnectEdgeCommand 无 UI 绑定）** | 基本编辑操作 | 📝 交互缺失 |
| ❌ **撤消/重做** | DAG 编辑器标准功能 | 📝 体验缺失 |
| ❌ **多选/框选（BoxSelecting）** | 画布枚举定义了但未实现 | 📝 交互缺失 |
| ❌ **Delete 键删除节点/边** | DAG 编辑器标准快捷键 | 📝 交互缺失 |
| ❌ **迷你地图点击导航** | 仅显示不可操作 | 📝 交互缺失 |
| ❌ **上下文栏（模板/分组/覆盖切换）** | 第 8.2 节 | 🔴 严重 |
| ❌ **覆盖状态标记（override/inherited）** | 第 8.2 节 | 🔴 严重 |
| ❌ **表达式编辑器** | 第 8.2 节 | 🔴 严重 |
| ❌ **颜色选择器 / Slider / Preset / Map / Coordinate 控件** | 参数控件类型 | 🔴 严重 |
| ❌ **节点启用/禁用切换** | 常见 DAG 功能 | 📝 交互缺失 |

### 3.3 设计偏离汇总

| 偏离项 | 设计文档 | 实际 | 影响评估 |
|--------|---------|------|---------|
| DAG 位置 | 中栏下方（迷你 DAG 总览） | 右栏顶部（全尺寸编辑器） | ⚪ 功能更强 |
| 属性面板 | 右栏独立区域 | DAG 编辑器右侧内嵌 | ⚪ 布局更紧凑 |
| 参数控件 | 全部 17 种类型 | 仅 6-7 种基础类型 | 🔴 严重缺失 |
| 高级 UI | 上下文栏/覆盖/表达式 | 完全缺失 | 🔴 严重缺失 |

### 3.4 节点参数编辑的质量问题

1. **参数类型推断过于简化** — `RegenerateParameterControls` 中仅根据 .NET 类型（bool/int/float/string）推断 schema type，忽略了后端 ParameterSchema 中的丰富类型信息（enum/color/slider/preset/array/map 等）
2. **参数面板双重重建** — 当点击 DAG 画布上的节点时，SelectedNode 的 TwoWay 绑定和 NodeSelected 事件会触发两次参数面板重建
3. **无参数校验反馈** — 用户输入无效值时无即时视觉反馈
4. **参数 Schema 信息丢失** — 只传递了键值对，未传递完整的 ParameterSchema（min/max/step/unit/description 等）

### 3.5 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| DAG 画布渲染 | 95% | 节点/边/端口/网格/缩放/平移/迷你地图全部实现 |
| 节点操作 | 80% | 添加/删除/连接 OK；缺边删除/多选/键盘快捷键 |
| 管线 CRUD | 30% | 仅有 Create；缺 Load/List/Delete/Update |
| 参数编辑面板 | 40% | 基础类型 OK；缺高级控件/覆盖标记/表达式编辑器 |
| 上下文栏 | 0% | **完全未实现** |
| 执行/验证流程 | 90% | 完整的 gRPC 流 + 进度/状态反馈 |
| 交互完整性 | 50% | 缺撤消/多选/键盘快捷键/迷你地图交互 |
| 视觉设计 | 90% | Skia 画布渲染质量高 |
| **综合** | **55%** | **DAG 画布扎实但管线管理严重不完整、高级 UI 元素全缺** |
