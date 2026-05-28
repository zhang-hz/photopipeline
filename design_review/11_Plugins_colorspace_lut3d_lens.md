# GUI 设计审查 — #11 Color / LUT / Lens 插件组

---

## 概述

Colorspace、3D LUT、Lens Correction 三个插件同属"图像调整"类，GuiSchema 模式高度相似。

### 命名/ID 一致性

| Rust 插件 | C# 表示 | 匹配度 |
|-----------|--------|:-----:|
| Colorspace（ID: `photopipeline.plugins.colorspace`） | "Color Space Transform"（ID: `color_space`） | ⚠️ 名称相近但 ID 不同 |
| 3D LUT（ID: `photopipeline.plugins.lut3d`） | "3D LUT"（ID: `lut3d`） | ✅ 基本一致 |
| Lens Correction（ID: `photopipeline.plugins.lens_correct`） | **C# 中不存在!** | ❌ 完全缺失 |

**lens_correct 在 C# 的 PluginService 中没有对应条目，这意味着用户无法从 GUI 使用镜头校正功能。**

---

## 一、界面设计对比

### Colorspace

| GuiSchema 定义 | 值 | C# UI |
|---------------|-----|-------|
| Sections | conversion(Card), rendering(Card), icc_profile(CollapsibleCard) | ❌ 无分区 |
| icon | "palette" | ❌ 未使用 |
| color | "#8b5cf6"（紫色） | ❌ 未使用 |
| aux_views | Histogram, **GamutDiagram** | ❌ 均未实现 |
| preview | BeforeAfter(lock_zoom) | ✅ Preview 支持分屏 |
| min_panel_width | 340px | ❌ 未使用 |

### 3D LUT

| GuiSchema 定义 | 值 | C# UI |
|---------------|-----|-------|
| Sections | lut_file(Card), lut_transform(Card), interpolation(CollapsibleCard) | ❌ 无分区 |
| icon | "grid3x3" | ❌ 未使用 |
| color | "#ec4899"（粉色） | ❌ 未使用 |
| aux_views | Histogram, **Vectorscope** | ❌ 均未实现 |
| preview | BeforeAfter(lock_zoom) | ✅ Preview 支持分屏 |
| min_panel_width | 320px | ❌ 未使用 |

### Lens Correction

| GuiSchema 定义 | 值 | C# UI |
|---------------|-----|-------|
| Sections | lens_detection(Card), corrections(Card), lensfun(CollapsibleCard) | ❌ **插件不存在** |
| icon | "aperture" | ❌ |
| color | "#6366f1"（靛蓝） | ❌ |
| aux_views | Histogram, StatusText | ❌ |
| preview | BeforeAfter(lock_zoom) | ❌ |
| min_panel_width | 340px | ❌ |

### C# 插件卡片显示（仅 Colorspace / LUT）

```
┌──────────────────────────────────┐
│ 3D LUT                       v1.0│
│ Apply 3D LUT for color grading...│
│            [Color]                │
├──────────────────────────────────┤
│ Parameters — 3D LUT              │
│ ┌──────────────────────────────┐ │
│ │ LUT Path: [________] [...] │ │ ← path
│ │ Intensity:[1.0   ▲▼]      │ │ ← float
│ └──────────────────────────────┘ │
│ [Reset] [Add to Pipeline]        │
└──────────────────────────────────┘
```

---

## 二、参数控件对比

### Colorspace 参数

| 参数 | 类型 | C# 控件 |
|------|------|---------|
| source | enum（8个值） | ComboBox ✅ |
| target | enum（7个值） | ComboBox ✅ |
| intent | enum（4个值） | ComboBox ✅ |

### 3D LUT 参数

| 参数 | 类型 | C# 控件 |
|------|------|---------|
| lut_path | path | TextBox+浏览 ✅ |
| intensity | float(0.0-1.0, step=0.01) | NumberBox ⚠️（无滑块，step不支持） |

### Lens Correction（C# 缺失）

Rust 参数：camera_make(string)、camera_model(string)、lens(string)、distortion(bool)、ca(bool)、vignetting(bool)

---

## 三、关键问题

### 1. Lens Correction 在 C# GUI 中不存在

用户无法通过 GUI 添加镜头校正节点，但后端 Rust 已实现完整功能。

### 2. 所有插件缺少辅助视图

三个插件都定义了辅助视图（Histogram、GamutDiagram、Vectorscope、StatusText），但 C# PreviewView 完全未实现这些视图。

### 3. 重复的 GuiSchema 特性未利用

icon、color、sections、min_panel_width 全部被忽略。

### 4. 完成度评分

| 维度 | Colorspace | 3D LUT | Lens Correct |
|------|:---------:|:------:|:-----------:|
| 插件卡片 | 70% | 70% | 0% |
| 参数控件 | 90% | 80% | 0% |
| GuiSchema 利用 | 5% | 5% | 0% |
| 辅助视图 | 0% | 0% | 0% |
| **综合** | **40%** | **40%** | **0%** |
