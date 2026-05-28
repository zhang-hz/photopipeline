# GUI 设计审查 — #12 Metadata 插件组（EXIF / GPS / TimeShift）

---

## 概述

三个元数据处理插件的 GuiSchema 对比。

### Rust ↔ C# 存在情况

| Rust 插件 | C# 表示 | 状态 |
|-----------|--------|:----:|
| EXIF Reader/Writer | "EXIF Reader" + "EXIF Writer"（2个分离） | ⚠️ 拆分 |
| GPS Coordinate Manager | **C# 中不存在** | ❌ 缺失 |
| Time Shift | **C# 中不存在** | ❌ 缺失 |

---

## 一、EXIF Reader/Writer

### GuiSchema 定义

```rust
GuiSchema {
    sections: [
        read_options (Default),
        write_options (Default),
        exiftool (CollapsibleCard),       // 可折叠
    ],
    icon: "tag", color: "#3b82f6" (蓝),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

### C# 拆分为 2 个插件

```
EXIF Reader:                    EXIF Writer:
┌─────────────────────────┐    ┌─────────────────────────┐
│ EXIF Reader         v1.0│    │ EXIF Writer         v1.0│
│ Extract EXIF/XMP/ICC... │    │ Embed EXIF/XMP/copy... │
│        [Metadata]       │    │        [Metadata]       │
├─────────────────────────┤    ├─────────────────────────┤
│ □ extract_exif [✓]     │    │ □ preserve_source [✓]  │
│ □ extract_xmp  [✓]     │    │ copyright: [________]  │
│ □ extract_icc  [✓]     │    │ artist:    [________]  │
│                         │    │ □ embed_icc   [✓]     │
│ [Reset] [Add to P.]    │    │ [Reset] [Add to P.]    │
└─────────────────────────┘    └─────────────────────────┘
```

### 控件匹配

| 参数 | Rust 类型 | C# 控件 | 匹配 |
|------|----------|---------|:----:|
| extract_exif | bool | ToggleSwitch | ✅ |
| extract_xmp | bool | ToggleSwitch | ✅ |
| extract_icc | bool | ToggleSwitch | ✅ |
| preserve_source | bool | ToggleSwitch | ✅ |
| copyright | string | TextBox | ✅ |
| artist | string | TextBox | ✅ |
| embed_icc | bool | ToggleSwitch | ✅ |

### 评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 插件卡片 | 70% | 文字完整，缺图标/颜色 |
| 参数控件 | 100% | 全部匹配 |
| GuiSchema 利用 | 5% | icon/color/sections 全忽略 |
| **综合** | **55%** | **参数 OK，但拆分方式改变插件架构，GuiSchema 设计几乎未用** |

---

## 二、GPS Coordinate Manager

### GuiSchema 定义

```rust
GuiSchema {
    sections: [
        source (Card),          // GPS 来源选择（EXIF/手动/GPX插值）
        manual_coords (Card),   // 手动坐标输入
        gpx_options (CollapsibleCard),  // GPX 选项
    ],
    icon: "map-pin", color: "#10b981" (绿),
    preview: None,
    aux_views: [AuxView::Map],   // 🔴 地图辅助视图
    min_panel_width: 340,
}
```

### C# 实现状态

| 元素 | C# 状态 | 严重度 |
|------|---------|:-----:|
| 插件卡片 | ❌ **完全不存在** | 🔴 |
| 参数控件 | ❌ 完全不存在 | 🔴 |
| 地图辅助视图（AuxView::Map） | ❌ 不支持 | 🔴 |
| GPS 坐标输入控件（lat/lon） | ❌ Coordinate 控件类型不存在 | 🔴 |

### 用户影响

用户无法通过 GUI 使用 GPS 坐标设置功能。GPX 轨迹插值、手动坐标输入等摄影工作流中常用的功能完全不可用。

### 评分

| 维度 | 评分 |
|------|:---:|
| 插件卡片 | **0%** |
| 参数控件 | **0%** |
| 辅助视图（地图） | **0%** |
| **综合** | **0%** |

---

## 三、Time Shift

### GuiSchema 定义

```rust
GuiSchema {
    sections: [
        time_shift (Card),     // 时间偏移量（小时/分钟/秒）
        timezone (Card),       // 时区设置
        batch (CollapsibleCard), // 批量模式选项
    ],
    icon: "clock", color: "#f59e0b" (琥珀),
    preview: None, aux_views: [],
    min_panel_width: 320,
}
```

### C# 实现状态

| 元素 | C# 状态 | 严重度 |
|------|---------|:-----:|
| 插件卡片 | ❌ **完全不存在** | 🔴 |
| 参数控件 | ❌ 完全不存在 | 🔴 |
| 时区选择器 | ❌ 不存在 | 📝 |

### 用户影响

摄影中常见的时间偏移调整（跨时区旅行、相机时间设置错误修正）无法通过 GUI 完成。

### 评分

| 维度 | 评分 |
|------|:---:|
| 插件卡片 | **0%** |
| 参数控件 | **0%** |
| **综合** | **0%** |

---

## 总结

| 插件 | Rust 完成度 | C# 卡片 | C# 参数 | C# AuxView | C# 综合 |
|------|:----------:|:-------:|:-------:|:----------:|:-------:|
| EXIF Reader/Writer | 100% | 70% | 100% | N/A | **55%** |
| GPS Coordinate Manager | 100% | 0% | 0% | ❌ Map | **0%** |
| Time Shift | 100% | 0% | 0% | N/A | **0%** |

**3 个元数据插件中有 2 个完全无法从 GUI 使用。** 这是摄影后处理应用中的重大功能缺失。
