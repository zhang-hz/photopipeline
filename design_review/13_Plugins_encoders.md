# GUI 设计审查 — #13 编码器插件组（AVIF / JXL / HEIF / TIFF / PNG）

---

## 概述

5 个输出编码器插件共享相似的 GuiSchema 模式。Rust 端设计为独立的专一编码器，每个针对特定格式优化。

### Rust ↔ C# 存在情况

| Rust 插件 | C# 表示 | 状态 |
|-----------|--------|:----:|
| AVIF Encoder | "Format Converter"（单一合并） | ❌ 架构不一致 |
| JPEG XL Encoder | "Format Converter"（单一合并） | ❌ 架构不一致 |
| HEIF Encoder | "Format Converter"（单一合并） | ❌ 架构不一致 |
| TIFF Encoder | "Format Converter"（单一合并） | ❌ 架构不一致 |
| PNG Encoder | "Format Converter"（单一合并） | ❌ 架构不一致 |

**C# 用一个 "Format Converter" 插件覆盖了全部 5 个专用编码器。** 这不是 UI 呈现的问题，而是 C# PluginService 硬编码数据与 Rust 架构的根本性不匹配。

---

## 一、GuiSchema 定义对比

### AVIF Encoder

```rust
GuiSchema {
    sections: [quality(Card), format(Card), advanced(CollapsibleCard)],
    icon: "image", color: "#22c55e" (绿),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

### JPEG XL Encoder

```rust
GuiSchema {
    sections: [quality(Card), advanced(CollapsibleCard)],
    icon: "file-image", color: "#f97316" (橙),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

### HEIF Encoder

```rust
GuiSchema {
    sections: [quality(Card), advanced(CollapsibleCard)],
    icon: "image", color: "#14b8a6" (青),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

### TIFF Encoder

```rust
GuiSchema {
    sections: [encoding(Card), metadata(Card)],
    icon: "file", color: "#64748b" (灰蓝),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

### PNG Encoder

```rust
GuiSchema {
    sections: [encoding(Card), metadata(Card)],
    icon: "image", color: "#0ea5e9" (蓝),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

**共性：** 都有 quality/advanced 或 encoding/metadata 分区，icon+color 不同，均无预览/辅助视图。

---

## 二、C# "Format Converter" 显示

```
┌──────────────────────────────────┐
│ Format Converter             v1.0│
│ Convert between image formats... │
│            [Format]               │
├──────────────────────────────────┤
│ Parameters — Format Converter    │
│ ┌──────────────────────────────┐ │
│ │ Format:  [TIFF          ▼]   │ │ ← enum (10个)
│ │ Quality: [95.0  ▲▼]         │ │ ← float
│ │ Lossless:[✓]                │ │ ← bool
│ │ Bit Depth:[8 ▼]             │ │ ← enum
│ │ Chroma:  [4:4:4 ▼]          │ │ ← enum
│ └──────────────────────────────┘ │
│ [Reset] [Add to Pipeline]        │
└──────────────────────────────────┘
```

## 三、架构差异分析

| 维度 | Rust 设计（5 个编码器） | C# UI（1 个 Format Converter） |
|------|----------------------|--------------------------------|
| 编码器粒度 | 每个格式独立插件 | 1 个插件统管所有格式 |
| 参数分组 | 各格式独有的参数面板 | 大而全的通用参数 |
| 格式特定选项 | 各有专属参数（如 jxl effort/tiff compression） | 缺失格式特定选项 |
| 视觉识别 | 各自 icon + color | 无差异化 |
| 后端匹配 | 与 server 端一一对应 | **不匹配后端注册表** |

### 关键问题

1. **格式特定参数丢失** — 每个编码器在 Rust 端有特有参数（如 JXL 的 effort 级别、HEIF 的 x265 预设、TIFF 的压缩算法），但这些在 C# 通用 Format Converter 中不存在
2. **用户需要选择格式而非选择编码器** — 在 Rust 设计中，用户添加"HEIF Encoder"节点就知道输出是 HEIF；在 C# 中，用户添加"Format Converter"后还需选择格式
3. **无法表达管线中的多格式输出** — 如果用户需要同时输出 JPEG XL 和 PNG，在 Rust 设计中加两个编码器节点即可；在 C# 中需要加两个 Format Converter 节点

### 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 插件卡片 | 70% | Format Converter 卡片显示 OK |
| 格式覆盖 | 40% | 参数覆盖了大部分常用格式但缺少专有选项 |
| 后端一致性 | 10% | 与 Rust 5 个独立编码器严重不一致 |
| GuiSchema 利用 | 5% | 忽略各编码器独有的 icon/color/sections |
| **综合** | **30%** | **一个合并的 Format Converter 与 Rust 专用编码器架构严重不符** |
