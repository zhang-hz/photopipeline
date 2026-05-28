# GUI 设计审查 — #03 PluginBrowserView 插件浏览器

---

## 一、界面设计（UI Design）

### 1.1 设计文档定义的界面

ARCHITECTURE.md 第 8.1 节将插件浏览器放在 **右栏（插件控制面板）** 中，作为"选中节点的完整详情"。

第 8.2 节定义了每个插件的面板结构：

```
每个插件面板包含：
┌──────────────────────────────────┐
│ 上下文栏                         │
│ [模板▼] [分组▼] [覆盖切换]       │ ← 上下文栏
├──────────────────────────────────┤
│ 参数控件（ParameterSchema 驱动）  │
│ ┌──────────────────────────────┐ │
│ │ Parameter 1: [___________]   │ │
│ │ Parameter 2: [===○=======]  │ │
│ │ Parameter 3: [▼ Option ]   │ │
│ │ 🟡 override 标记              │ │
│ │ ✎ 表达式编辑器（条件启用）    │ │
│ └──────────────────────────────┘ │
├──────────────────────────────────┤
│ Preview 区域（GuiSchema 定义）    │
│ 辅助视图（直方图/地图/波形图）    │
└──────────────────────────────────┘
```

**设计预期布局（右栏）：**
- 顶部：上下文栏（模板/分组/图像覆盖切换按钮）
- 中部：Schema 驱动的参数控件面板（17 种控件类型）
  - 每个字段带覆盖状态标记（🟡 override / ⬜ inherited）
  - `supports_expression=true` 时显示表达式编辑器 ✎
- 下部：预览区域 + 辅助视图（直方图、地图、波形图等）

### 1.2 实际实现的界面

PluginBrowserView.xaml 实现（实际在 **左栏底部**）：

```
┌────────────────────────────┐
│ [Search plugins...       ] │  ← 搜索框
│ [All Categories       ▼]  │  ← 分类筛选
├────────────────────────────┤
│ ┌────────────────────────┐ │
│ │ transform              │ │  ← 每个插件卡片
│ │ v1.0                   │ │
│ │ Geometric transform... │ │
│ │ [Adjust]               │ │  ← 分类标签
│ ├────────────────────────┤ │
│ │ colorspace             │ │
│ │ v1.0                   │ │
│ │ Convert between col... │ │
│ │ [Adjust]               │ │
│ ├────────────────────────┤ │
│ │ ...                    │ │
│ └────────────────────────┘ │
├────────────────────────────┤  ← 以下仅选中插件时显示
│ Parameters — transform     │
│ ┌────────────────────────┐ │
│ │ (动态生成参数控件)       │ │
│ │                        │ │
│ └────────────────────────┘ │
│ [Reset] [Add to Pipeline]  │
└────────────────────────────┘
```

**控件布局（3 行）：**
1. 搜索/筛选栏 Row 0：搜索 TextBox + 分类 ComboBox
2. 插件列表 Row 1：ListBox + VirtualizingPanel，每项含名称/版本/描述/分类标签
3. 参数面板 Row 2（条件显示）：SelectedPlugin!=null 时显示，含动态生成的参数控件 + Reset 和 Add 按钮

### 1.3 视觉元素

| 元素 | 实际实现 | 评估 |
|------|---------|:----:|
| 插件卡片 | Border + 圆角 4px + 阴影背景色 | ✅ |
| 插件名称 | 12px SemiBold | ✅ |
| 版本号 | v{0} 格式，9px，三级文字色 | ✅ |
| 描述 | 10px，二级文字色，字符截断 | ✅ |
| 分类标签 | AccentFillColorDisabledBrush 背景，9px | ✅ |
| 搜索框 | TextBox + PlaceholderText + 清除按钮 | ✅ |
| 分类筛选 | ComboBox 下拉 | ✅ |
| 参数面板 | 空 StackPanel 动态填充，MaxHeight=200 | ✅ |
| 参数面板标题 | "Parameters — {PluginName}" | ✅ |
| Reset/Add 按钮 | 底部操作栏 | ✅ |

### 1.4 状态覆盖

| 状态 | 设计文档 | 实际实现 | 匹配度 |
|------|---------|---------|:-----:|
| **空态**（无插件） | 未指定 | 空列表 | ❌ |
| **搜索无结果** | 未指定 | 空列表，无提示 | ❌ |
| **插件选中 - 参数面板** | 参数控件 + 覆盖标记 + 表达式编辑器 | 仅有参数控件 | ⚠️ |
| **插件未选中** | 未指定 | 参数面板隐藏（NullToVisibility） | ✅ |
| **拖放状态** | 未指定 | 鼠标按下 → 拖拽 → DragDrop | ✅ |

---

## 二、功能设计（Functional Design）

### 2.1 职责边界

PluginBrowserViewModel 负责：
- 列出可用插件（来源：PluginService — 实际是硬编码本地数据）
- 按分类筛选 + 文字搜索
- 选中插件时显示其参数 Schema
- 参数编辑 → 维护 CurrentParameters 字典
- 添加到 PipelineEditor（按钮 + 拖放）

### 2.2 数据流向

```
PluginService.GetAllAsync() ← 硬编码数据（非 gRPC）
  → FilteredPlugins（搜索+筛选后） → ListBox

用户选中插件 → SelectedPlugin
  → 参数 Schema 读取
  → ParameterControlFactory.CreateControl() → 动态生成 WPF 控件
  → 用户编辑 → CurrentParameters 字典更新

用户点击 "Add to Pipeline" / 拖放
  → MainViewModel中介事件
  → PipelineEditorViewModel.AddNode()
```

### 2.3 用户操作流程

| 步骤 | 操作 | 系统响应 | 状态变化 |
|------|------|---------|---------|
| 1 | 打开应用 | PluginService 加载插件列表 | 加载 → 显示列表 |
| 2 | 在搜索框输入 | FilteredPlugins 实时过滤 | 列表动态缩小 |
| 3 | 选择分类 | 按分类筛选 | 列表仅显示该类 |
| 4 | 点击插件 | 选中 → 参数面板出现 | 参数面板动态生成 |
| 5 | 调整参数 | CurrentParameters 更新 | 参数状态变更 |
| 6 | 点击 Reset | 恢复默认参数 | 参数面板重新生成 |
| 7 | 点击 "Add to Pipeline" | 发往管线编辑器 | 管线添加节点 |
| 8 | 拖放插件到管线编辑器 | 拖拽 → PipelineEditorView.OnNodeDropped | 管线添加节点 |

### 2.4 与其他组件的协作

| 协作方 | 关系 | 实现 |
|-------|------|------|
| PluginService | 获取插件列表 | **非 gRPC，硬编码数据** |
| PipelineEditor | 添加节点 | 中介事件 + 拖放 DragDrop |
| ParameterControlFactory | 动态生成参数控件 | 代码后置 RegenerateParameterControls() |

---

## 三、实现程度（Implementation Assessment）

### 3.1 已实现

- ✅ 插件列表显示（名称/版本/描述/分类）
- ✅ 文本搜索（实时过滤）
- ✅ 分类筛选（ComboBox）
- ✅ 选中插件后显示参数面板
- ✅ 参数动态生成（ParameterControlFactory）
- ✅ 参数编辑（CurrentParameters 字典）
- ✅ Reset to Defaults
- ✅ Add to Pipeline 按钮
- ✅ 拖放插件到管线（DragDrop）

### 3.2 未实现

| 缺失项 | 设计依据 | 严重度 |
|-------|---------|:-----:|
| ❌ **上下文栏（模板/分组/覆盖切换按钮）** | 第 8.2 节"上下文栏" | 🔴 严重 |
| ❌ **覆盖状态标记（🟡override/⬜inherited）** | 第 8.2 节"每个字段的覆盖状态标记" | 🔴 严重 |
| ❌ **表达式编辑器**（supports_expression=true） | 第 8.2 节 | 🔴 严重 |
| ❌ **辅助视图（直方图/地图/波形图/矢量图）** | 第 8.2 节 + GuiSchema AuxView | 🔴 严重 |
| ❌ **插件图标/颜色显示** | GuiSchema 定义了 icon 和 color 字段 | 📝 视觉缺失 |
| ❌ **参数控件类型不全** | 17 种 ParameterType 仅约 6-7 种有 C# 控件 | 🔴 严重 |

### 3.3 设计偏离汇总

| 偏离项 | 设计文档 | 实际 | 影响评估 |
|--------|---------|------|---------|
| **插件浏览器位置** | 右栏（插件控制面板） | 左栏底部 | ⚪ 布局变更 |
| **插件数据来源** | 后端 gRPC 注册表 | 本地硬编码 | 🔴 架构问题 |
| **参数面板位置** | 右栏全高度 | 插件浏览器底部折叠区 | ⚪ 布局有限 |
| **参数面板滚动** | 期望全高度展开 | MaxHeight=200 受限 | 📝 体验受限 |

### 3.4 关键交互问题

1. **参数面板高度受限** — ScrollViewer MaxHeight=200 限制了参数较多时的可视区域
2. **参数控件类型严重不足** — 仅实现了 bool（ToggleSwitch）、int/float（TextBox+步进）、enum（ComboBox）、path（TextBox+浏览）、string（TextBox）。Color/Slider/Expression/Preset/Map/Coordinate 等均缺失
3. **无参数校验反馈** — 输入无效值时无视觉提示
4. **无插件刷新机制** — 列表在构造函数中一次性加载，无法从后端重新获取
5. **插件元数据缺失** — GuiSchema 中定义的 icon、color、min_panel_width 完全未被使用

### 3.5 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 插件列表浏览 | 90% | 列表/搜索/筛选完整，缺图标/颜色显示 |
| 参数控件 | 40% | 仅基础类型实现，Color/Slider/Expression 等缺失 |
| 上下文栏 | 0% | **完全未实现** |
| 覆盖状态标记 | 0% | **完全未实现** |
| 表达式编辑器 | 0% | **完全未实现** |
| 辅助视图 | 0% | **完全未实现** |
| 拖放交互 | 100% | 拖放流畅 |
| 视觉设计 | 60% | 插件卡片设计良好，但缺高级 UI 元素 |
| **综合** | **35%** | **设计文档承诺的高级插件 UI 功能大多未实现** |
