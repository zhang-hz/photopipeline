# 插件 #02: transform — 详细设计规格书

> 后端来源: `feat/unified-binary` · `crates/plugins/src/transform.rs`  
> 前端框架: Fluent 2 + React · 面板宽度: 420px

---

## 1. 插件元数据

| 字段 | 值 | 来源 |
|------|-----|------|
| 名称 | Transform | `fn name()` |
| 插件 ID | `photopipeline.plugins.transform` | `fn id()` |
| 版本 | 1.0.0 | `fn version()` |
| 分类 | Transform | `PluginCategory::Transform` |
| 描述 | Resize, rotate, and crop images with configurable filters | `fn description()` |
| 标签 | resize, rotate, crop, filter, transform | `TAGS` |
| 能力 Trait | PixelProcessor | `impl PixelProcessor` |
| 最低内存 | 256 MB | `HardwareRequirement.min_ram_mb` |
| 像素输入 | 是 | `requires_pixel_access() → true` |
| 像素输出 | 是 | `produces_pixel_output() → true` |

---

## 2. GuiSchema

| 字段 | 值 |
|------|-----|
| layout | `GuiLayout::Standard` |
| icon | `"maximize"` |
| color | `"#06b6d4"` (青) |
| preview | `PreviewMode::BeforeAfter { default_split: 0.5, orientation: Horizontal, lock_zoom: false }` |
| aux_views | `[AuxView::Histogram]` |
| min_panel_width | 340px |

### 分区结构

| 分区 ID | 标题 | 样式 | 默认 |
|---------|------|------|:---:|
| `resize` | Resize | Card | 展开 |
| `rotation` | Rotation | Card | 展开 |
| `crop` | Crop | Card | 展开 |
| `filter` | Filter | **CollapsibleCard** | 折叠 |

---

## 3. 参数定义 & 控件映射

### 3.1 控件分配原则

| 语义类型 | 例子 | 控件 | 原因 |
|---------|------|:---:|------|
| 精确像素值 | Width, Height, Crop X/Y/W/H, Long Edge | **SpinButton** | 需要精确输入，无连续调节意义 |
| 连续调节 | Scale%, Angle | **Slider** + 数值显示 | 需要可视连续调节 |
| 离散选项 | Mode, Filter | **Dropdown** | 有限选项 |
| 布尔开关 | Flip H/V, Enable Crop | **Switch** | 二值选择 |

### 3.2 分区: Resize (Card, 展开)

| 参数 | 类型 | 默认 | 控件 | 说明 |
|------|------|------|:---:|------|
| `resize_mode` | Enum (6选项) | `"none"` | Dropdown | Long Edge(★)/Absolute/Percentage/Short Edge/Megapixels/None |
| `target_width` | Integer (1–65535) | 1920 | **SpinButton + px** | 目标宽度 |
| `target_height` | Integer (1–65535) | 1080 | **SpinButton + px** | 目标高度 |
| `scale_percent` | Float (1–1000, step=1) | 100.0 | **Slider + %显示** | 缩放百分比 |
| `long_edge_px` | Integer (1–65535) | 2048 | **SpinButton + px** | 长边目标像素 |

### 3.3 分区: Rotation (Card, 展开)

| 参数 | 类型 | 默认 | 控件 | 说明 |
|------|------|------|:---:|------|
| `angle` | Float (-360~360, step=0.1) | 0.0 | **Slider + °显示** | 顺时针旋转角度 |
| `flip_horizontal` | Boolean | false | Switch (Normal/Flipped) | 水平翻转 |
| `flip_vertical` | Boolean | false | Switch (Normal/Flipped) | 垂直翻转 |

### 3.4 分区: Crop (Card, 展开)

| 参数 | 类型 | 默认 | 控件 | 说明 |
|------|------|------|:---:|------|
| `crop_enabled` | Boolean | false | Switch (Disabled/Enabled) | 启用裁剪 |
| `crop_x` | Integer (0–65535) | 0 | **SpinButton + px** | 裁剪左边界 |
| `crop_y` | Integer (0–65535) | 0 | **SpinButton + px** | 裁剪上边界 |
| `crop_width` | Integer (1–65535) | 1920 | **SpinButton + px** | 裁剪宽度 |
| `crop_height` | Integer (1–65535) | 1080 | **SpinButton + px** | 裁剪高度 |

### 3.5 分区: Filter (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认 | 控件 | 说明 |
|------|------|------|:---:|------|
| `filter_type` | Enum (3选项) | `"bilinear"` | Dropdown | Bilinear / Lanczos3(★, Halide) / Nearest Neighbor |

- `advanced: true` — 标签显示 `[advanced]`

---

## 4. UI 布局

```
┌──────────────────────────────────────┐
│ Plugin Details · transform           │
├──────────────────────────────────────┤
│ [All] [Template] [High ISO] [img]   │ ← ctx-bar
├──────────────────────────────────────┤
│ ↔ Transform                     v1.0 │ ← plugin-header (青色图标)
│ photopipeline.plugins.transform      │
│ [Transform] [resize] [rotate] [crop] │ ← tags
│ PixelProcessor · RAM ≥ 256 MB        │
├──────────────────────────────────────┤
│ ▼ Resize                       5 params│
│  Mode          [Long Edge ★ ▾]  ⬜  │ ← Dropdown
│  Width         [  1920 ▲▼] px  ⬜  │ ← SpinButton
│  Height        [  1080 ▲▼] px  ⬜  │ ← SpinButton
│  Scale         ═══○══════ 100.0% ⬜ │ ← Slider
│  Long Edge     [  3000 ▲▼] px  🟡  │ ← SpinButton (overridden)
├──────────────────────────────────────┤
│ ▼ Rotation                    3 params│
│  Angle         ════○═══════ 1.5° 🟡 │ ← Slider (overridden)
│  Flip H        [Normal ●――○]    ⬜  │ ← Switch
│  Flip V        [Normal ●――○]    ⬜  │ ← Switch
├──────────────────────────────────────┤
│ ▼ Crop                        5 params│
│  Enable        [Disabled ●――○]   ⬜  │ ← Switch
│  X             [     0 ▲▼] px  ⬜  │ ← SpinButton
│  Y             [     0 ▲▼] px  ⬜  │ ← SpinButton
│  Width         [  1920 ▲▼] px  ⬜  │ ← SpinButton
│  Height        [  1080 ▲▼] px  ⬜  │ ← SpinButton
├──────────────────────────────────────┤
│ ▶ Filter            default collapsed│ ← CollapsibleCard
├──────────────────────────────────────┤
│ 𝑓 Expression                        │ ← 表达式编辑器
├──────────────────────────────────────┤
│ 📊 Histogram                         │ ← GuiSchema.aux_views
│ ▂▃▅▆▇▆▅▃▂▁                         │
├──────────────────────────────────────┤
│ PREVIEW                              │
│ PreviewMode::BeforeAfter            │ ← GuiSchema.preview
├──────────────────────────────────────┤
│ [🗑 Remove from Pipeline]           │
└──────────────────────────────────────┘
```

---

## 5. 数值参数的控件选择规则

```
selectWidget(field):
  match (field.field_type, semantics):
    Integer(min, max) where precision matters (px coords)
      → <SpinButton min={min} max={max} suffix={unit} />
      显示 ▲▼ 步进按钮，可直接键入精确值
    
    Integer/Float(min, max, step) where continuous adjustment matters
      → <Slider min={min} max={max} step={step} /> + 数值显示
      适合 Scale%、Angle、强度等
    
    Enum(options)
      → <Dropdown>
      推荐项加 ★，显示 label + description
    
    Boolean(label_true, label_false)
      → <Switch label={value ? label_true : label_false} />
      
    Boolean (无 label_true/label_false)
      → <Switch /> 简单开关
```

## 6. 与 raw_input 的关键差异

| 对比项 | raw_input | transform |
|--------|-----------|-----------|
| 参数数 | 6 | 14 |
| 控件类型 | Dropdown + Switch + Input | Dropdown + **SpinButton** + **Slider** + Switch |
| 预览 | None | **BeforeAfter (水平分屏)** |
| 辅助视图 | 无 | **Histogram** |
| 表达式 | 不支持 | 支持 |
| 高级参数 | White Balance, dcraw | Filter (collapsed) |
| min_panel_width | 320px | 340px |

## 7. 后端适配确认清单

- [x] 插件名称、ID、版本与 `fn name/id/version` 一致
- [x] 分类 Transform 与 `PluginCategory::Transform` 一致
- [x] 4 个分区 ID/标题/样式与 GuiSchema 一致
- [x] resize_mode 6 个枚举值（含推荐标记）与后端一致
- [x] filter_type 3 个枚举值（含推荐标记 Lanzos3）与后端一致
- [x] target_width/height 的 min/max/default 与后端一致
- [x] scale_percent 的 min/max/step/precision 与后端一致
- [x] angle 的 min/max/step/precision 与后端一致
- [x] crop_x/y/width/height 的 min/max/default 与后端一致
- [x] Boolean 字段的 label_true/label_false 与后端一致
- [x] filter_type 的 advanced=true
- [x] GuiSchema icon/color/preview/aux_views/min_panel_width 全部对应
