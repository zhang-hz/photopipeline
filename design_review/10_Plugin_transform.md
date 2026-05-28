# GUI 设计审查 — #10 Transform 插件

---

## 基本信息

| 属性 | Rust 定义 | C# UI 显示 |
|------|----------|-----------|
| 插件 ID | `photopipeline.plugins.transform` | `resize` / `crop` / `rotate`（**分拆为3个**） |
| 名称 | `Transform` | `Resize` / `Crop` / `Rotate`（**分拆为3个**） |
| 分类 | Transform | Transform |
| 描述 | Resize, rotate, and crop images with configurable filters | 各自对应功能（**分拆但描述一致**） |

**关键发现：** Rust 端 Transform 是一个**复合插件**（包含 resize/rotation/crop/filter 四个参数组），而 C# 端将其拆为了 3 个独立插件。这并不是 UI 渲染的问题，而是 C# PluginService 硬编码数据与 Rust 不一致的表现。

## 一、界面设计（UI Design）

### GuiSchema 定义（Rust）

```rust
GuiSchema {
    layout: GuiLayout::Standard {
        sections: vec![
            GuiSection { id: "resize",  title_visible: true, style: Card },
            GuiSection { id: "rotation", title_visible: true, style: Card },
            GuiSection { id: "crop",    title_visible: true, style: Card },
            GuiSection { id: "filter",  title_visible: true, style: CollapsibleCard },
        ],
    },
    icon: Some("maximize"),
    color: Some("#06b6d4"),     // 青色主题
    preview: PreviewMode::BeforeAfter {
        default_split: 0.5,
        orientation: Horizontal,
        lock_zoom: false,
    },
    aux_views: vec![AuxView::Histogram],   // 🔴 C# UI 未实现
    min_panel_width: 340,
}
```

**设计预期：**
- 4 个参数卡片分区：resize / rotation / crop / filter（filter 可折叠）
- 最大化图标 + 青色主题色
- Before/After 默认水平分割预览
- **直方图辅助视图（AuxView::Histogram）**
- 最小面板宽度 340px

### C# UI 显示（分拆后）

以 "Resize" 为例：

```
┌──────────────────────────────────┐
│ Resize                       v1.0│
│ Resize image with configurable...│
│            [Transform]           │
├──────────────────────────────────┤
│ Parameters — Resize              │
│ ┌──────────────────────────────┐ │
│ │ Width:    [1920  ▲▼]        │ │ ← int
│ │ Height:   [1080  ▲▼]        │ │ ← int
│ │ Filter:   [Lanczos3    ▼]   │ │ ← enum
│ │ Keep Aspect: [✓]            │ │ ← bool toggle
│ └──────────────────────────────┘ │
│ [Reset] [Add to Pipeline]        │
└──────────────────────────────────┘
```

### 1.1 GuiSchema 利用程度

| GuiSchema 定义 | C# UI 实现 | 匹配度 |
|---------------|-----------|:-----:|
| icon = "maximize" | 未使用 | ❌ |
| color = "#06b6d4"（青色） | 未使用 | ❌ |
| 4 个 Section（resize/rotation/crop/filter） | 分拆为 3 个独立插件，不包含 filter | ❌ |
| CollapsibleCard（filter） | 不支持折叠 | ❌ |
| PreviewMode::BeforeAfter | PreviewView 支持分屏（但与此插件无关） | ⚠️ 预览模式存在但不关联 GuiSchema |
| AuxView::Histogram | **未实现** | 🔴 缺失 |
| min_panel_width = 340 | 未使用 | ❌ |

### 1.2 参数控件对比（以 Resize 为例）

| 参数 | 类型（Rust Resize） | C# 控件 | 匹配度 |
|------|-------------------|---------|:-----:|
| width | int + min/max | NumberBox | ✅ |
| height | int + min/max | NumberBox | ✅ |
| filter | enum + values | ComboBox | ✅ |
| keep_aspect | bool | ToggleSwitch | ✅ |

Resize 的 4 个参数控件均匹配。但 Rust 端的 Transform 是包含所有 4 组参数的复合插件，C# 的分拆方式改变了插件架构。

---

## 二、功能设计

### 数据流

Transform 是最常用的图像处理插件之一。Rust 端设计为在**一次管线执行中**同时完成 resize + rotate + crop + filter。C# 端将其分拆意味着用户需要添加多个节点才能完成相同的变换操作。

### 工作流对比

| 场景 | Rust 端（设计） | C# UI（当前） |
|------|---------------|-------------|
| 调整大小+旋转+裁剪 | 1 个 Transform 节点 | 3 个独立节点 + 3 倍连接操作 |
| 参数分组 | 4 个命名卡片分区 | 3 个独立参数面板 |
| 直方图预览 | 在辅助视图查看 | **不支持** |

---

## 三、实现程度

### 3.1 已实现

- ✅ 3 个独立插件卡片（Resize / Crop / Rotate）
- ✅ 各插件参数控件完整

### 3.2 未实现

| 缺失项 | 来源 | 严重度 |
|-------|------|:-----:|
| ❌ **Transform 作为复合插件** | Rust 架构 | 🔴 架构不一致 |
| ❌ **Filter 参数组（CollapsibleCard）** | Rust 定义 | 📝 缺失 |
| ❌ **直方图辅助视图** | GuiSchema.aux_views | 🔴 严重 |
| ❌ **插件图标/主题色** | GuiSchema | 📝 视觉缺失 |
| ❌ **参数卡片分区（Section）** | GuiLayout | 🔴 严重 |
| ❌ **可折叠面板** | CollapsibleCard | 📝 交互缺失 |

### 3.3 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 插件卡片展示 | 70% | 文字信息完整，缺图标/颜色 |
| 参数控件 | 80% | 各插件参数控件类型正确 |
| 复合插件集成 | 0% | **分拆为 3 个独立插件** |
| GuiSchema 利用 | 5% | 仅参数类型被使用 |
| 辅助视图（直方图） | 0% | **完全未实现** |
| **综合** | **35%** | **参数控件 OK，但插件架构与设计了严重不一致，辅助视图缺失** |
