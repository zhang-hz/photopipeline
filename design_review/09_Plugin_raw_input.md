# GUI 设计审查 — #09 RAW Input 插件

---

## 基本信息

| 属性 | Rust 定义 | C# UI 显示 |
|------|----------|-----------|
| 插件 ID | `photopipeline.plugins.raw_input` | `raw_decoder`（**不匹配**） |
| 名称 | `RAW Input` | `Raw Decoder`（**不匹配**） |
| 版本 | 1.0.0 | 1.0.0 |
| 分类 | Input | Input |
| 描述 | Read RAW camera files (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF) | Decode camera raw files (DNG, NEF, CR2, ARW, ORF) into linear RGB pixel data（**描述不一致**） |

## 一、界面设计（UI Design）

### GuiSchema 定义（Rust）

```rust
GuiSchema {
    layout: GuiLayout::Standard {
        sections: vec![
            // 解马赛克设置（Card 样式）
            GuiSection { id: "raw_format", title_visible: true, style: Card },
            // 输出设置（Card 样式）
            GuiSection { id: "output", title_visible: true, style: Card },
            // DCRAW 高级选项（可折叠）
            GuiSection { id: "dcraw_options", title_visible: true, style: CollapsibleCard },
        ],
    },
    icon: Some("camera"),       // 🔴 C# UI 未使用
    color: Some("#ef4444"),     // 🔴 C# UI 未使用（红色主题色）
    preview: PreviewMode::None,
    aux_views: vec![],         // 无辅助视图
    min_panel_width: 320,      // 🔴 C# UI 未使用
}
```

**设计预期：**
- 三个参数卡片分区：raw_format、output、dcraw_options（最后一个可折叠）
- 相机图标 + 红色主题色
- 最小面板宽度 320px
- 无预览模式（输入节点）
- 无辅助视图

### C# UI 实际显示

```
┌──────────────────────────────────┐
│ Raw Decoder                  v1.0│
│ Decode camera raw files...       │
│            [Input]               │ ← 分类标签
├──────────────────────────────────┤
│ Parameters — Raw Decoder         │
│ ┌──────────────────────────────┐ │
│ │ Demosaic: [AMaZE         ▼]  │ │ ← enum → ComboBox
│ │ Border:   [3   ▲▼]          │ │ ← int → NumberBox
│ │ White Balance:[Camera ▼]    │ │ ← enum → ComboBox
│ └──────────────────────────────┘ │
│ [Reset] [Add to Pipeline]        │
└──────────────────────────────────┘
```

### 1.1 视觉设计对比

| GuiSchema 定义 | C# UI 实现 | 匹配度 |
|---------------|-----------|:-----:|
| icon = "camera" | 未使用（插件卡片无图标） | ❌ |
| color = "#ef4444"（红色） | 未使用（插件卡片无颜色标识） | ❌ |
| 3 个命名 Section | 所有参数合并在一个面板，无分区 | ❌ |
| CollapsibleCard（可折叠） | 不支持折叠 | ❌ |
| min_panel_width = 320 | 未使用 | ❌ |

### 1.2 参数控件对比

| 参数 | 类型（Rust） | C# 控件 | 匹配度 |
|------|------------|---------|:-----:|
| demosaic | Enum | ComboBox（enum→values） | ✅ |
| border | Integer | NumberBox（int→min/max） | ✅ |
| white_balance | Enum | ComboBox（enum→values） | ✅ |

所有 3 个参数的基础控件类型匹配。但缺少 Rust `ParameterSchema` 中定义的高级元数据：units、display names、grouping。

---

## 二、功能设计

### Rust 端参数（实际）

Rust 端的 raw_input 插件有 DCRAW 级别的大量复杂参数，包括黑白平衡、去马赛克算法、输出色彩空间等。但 C# 端 `PluginService` 的 `raw_decoder` 只有 3 个简单参数（demosaic/border/white_balance）。

**这意味着 C# UI 显示的参数与后端实际插件的能力存在严重脱节。**

### 用户操作流程

| 步骤 | 操作 | 预期行为 | 实际行为 |
|------|------|---------|---------|
| 1 | 从插件浏览器选中 Raw Decoder | 显示其参数 | 显示 3 个参数（缺少后端实际支持的参数） |
| 2 | 调整参数 | 参数更新 | 参数值更新到 CurrentParameters 字典 |
| 3 | Add to Pipeline | 节点添加到管线 | 节点添加到 DAG |
| 4 | 执行管线 | 后端使用参数处理 | 参数可能不匹配后端期望 |

---

## 三、实现程度

### 3.1 已实现

- ✅ 插件卡片显示（名称/版本/描述/分类标签）
- ✅ 参数控件（3 个参数全部有正确控件类型）
- ✅ Add to Pipeline 按钮 + 拖放支持

### 3.2 未实现

| 缺失项 | 来源 | 严重度 |
|-------|------|:-----:|
| ❌ **插件图标显示** | GuiSchema.icon | 📝 视觉缺失 |
| ❌ **插件主题色显示** | GuiSchema.color | 📝 视觉缺失 |
| ❌ **参数分区（Section）** | GuiLayout.Standard.sections | 🔴 严重 |
| ❌ **可折叠卡片** | SectionStyle::CollapsibleCard | 📝 交互缺失 |
| ❌ **C# 参数与 Rust 参数不匹配** | 架构不一致 — C# 只有 3 个参数，Rust 有更多 | 🔴 架构问题 |
| ❌ **参数元数据（unit/step/display_name）** | ParameterSchema | 📝 体验缺失 |

### 3.3 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 插件卡片展示 | 70% | 文字信息完整，缺图标/颜色 |
| 参数控件 | 60% | 基础类型 OK，缺分区/折叠 |
| GuiSchema 利用 | 10% | icon/color/section/min_width 全部忽略 |
| 后端一致性 | 20% | C# 参数列表与 Rust 严重不一致 |
| **综合** | **40%** | **卡片显示 OK 但参数与后端脱节，GuiSchema 设计几乎未被使用** |
