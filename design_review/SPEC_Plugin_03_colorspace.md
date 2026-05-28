# 插件 #03: colorspace — 详细设计规格书

> 后端来源: `feat/unified-binary` · `crates/plugins/src/colorspace.rs`  
> 前端框架: Fluent 2 + React · 面板宽度: 420px

---

## 1. 插件元数据

| 字段 | 值 | 来源 |
|------|-----|------|
| 名称 | Color Space | `fn name()` |
| 插件 ID | `photopipeline.plugins.colorspace` | `fn id()` |
| 版本 | 1.0.0 | `fn version()` |
| 分类 | Color | `PluginCategory::Color` |
| 描述 | Convert between color spaces with ICC profile support and rendering intents | `fn description()` |
| 标签 | color, colorspace, icc, profile, srgb, hdr | `TAGS` |
| 能力 | PixelProcessor | `impl PixelProcessor` |
| 最低内存 | 256 MB | `HardwareRequirement.min_ram_mb` |
| 像素输入 | 是 | `requires_pixel_access() → true` |
| 像素输出 | 是 | `produces_pixel_output() → true` |
| 支持格式 | U8/U16/F16/F32 (输入和输出相同) | `supported_input/output_formats()` |

---

## 2. GuiSchema

| 字段 | 值 | 说明 |
|------|-----|------|
| layout | `GuiLayout::Standard` | 标准分区布局 |
| icon | `"palette"` | 调色板图标 |
| color | `"#8b5cf6"` | 紫色主题 |
| preview | `BeforeAfter { default_split: 0.5, orientation: Horizontal, lock_zoom: true }` | 锁定缩放的分屏对比 |
| aux_views | `[Histogram, GamutDiagram]` | 直方图 + 色域图 |
| min_panel_width | 340px | |

### 分区结构

| 分区 ID | 标题 | 样式 | 默认状态 |
|---------|------|------|:---:|
| `conversion` | Color Space Conversion | Card | 展开 |
| `rendering` | Rendering | Card | 展开 |
| `icc_profile` | ICC Profile | CollapsibleCard | 折叠 |

---

## 3. 参数详细定义

### 3.1 分区: Color Space Conversion (Card, 展开, 2 params)

| 属性 | 值 |
|------|-----|
| 分区 ID | `conversion` |
| 标题 | Color Space Conversion |
| 描述 | Select source and target color spaces |
| 图标 | palette |

#### 参数: `source_color_space`

| 字段 | 值 |
|------|-----|
| 类型 | Enum (8 选项) |
| 默认值 | `"auto"` |
| 高级 | 否 |
| 允许覆盖 | 是 |

| 值 | 标签 | 描述 | 标签 | 推荐 |
|------|------|------|------|:---:|
| `"auto"` | Auto-detect | Detect from embedded profile | — | ★ |
| `"srgb"` | sRGB | Standard sRGB IEC61966-2.1 | — | |
| `"display_p3"` | Display P3 | Wide gamut P3 D65 | — | |
| `"adobe_rgb"` | Adobe RGB | Adobe RGB (1998) | — | |
| `"pro_photo"` | ProPhoto RGB | Kodak ProPhoto RGB | — | |
| `"bt2020"` | BT.2020 | Rec. 2020 UHDTV | [hdr] | |
| `"aces_cg"` | ACEScg | ACES CG linear | [cinema] | |
| `"linear_srgb"` | Linear sRGB | Linear-light sRGB | — | |

**Fluent 控件:** `<Dropdown>` · 显示格式: `"Auto-detect ★ — Detect from embedded profile"`

#### 参数: `target_color_space`

| 字段 | 值 |
|------|-----|
| 类型 | Enum (6 选项) |
| 默认值 | `"srgb"` |
| 高级 | 否 |
| 允许覆盖 | 是 |

| 值 | 标签 | 描述 | 标签 | 推荐 |
|------|------|------|------|:---:|
| `"srgb"` | sRGB | Standard sRGB | — | ★ |
| `"display_p3"` | Display P3 | Wide gamut P3 | — | |
| `"adobe_rgb"` | Adobe RGB | Adobe RGB (1998) | — | |
| `"pro_photo"` | ProPhoto RGB | Kodak ProPhoto | — | |
| `"bt2020_pq"` | BT.2020 PQ (HDR) | Rec. 2020 with PQ transfer, HDR 1000 nits | [hdr] | |
| `"linear_srgb"` | Linear sRGB | Linear-light working space | — | |

**Fluent 控件:** `<Dropdown>`

### 3.2 分区: Rendering (Card, 展开, 3 params)

| 属性 | 值 |
|------|-----|
| 分区 ID | `rendering` |
| 标题 | Rendering |
| 描述 | Rendering intent and gamut mapping options |
| 图标 | sliders |

#### 参数: `rendering_intent`

| 字段 | 值 |
|------|-----|
| 类型 | Enum (4 选项) |
| 默认值 | `"relative_colorimetric"` |

| 值 | 标签 | 描述 | 推荐 |
|------|------|------|:---:|
| `"relative_colorimetric"` | Relative Colorimetric | Clip out-of-gamut, preserve white point | ★ |
| `"perceptual"` | Perceptual | Compress gamut, preserve relationships | |
| `"saturation"` | Saturation | Preserve saturation at cost of accuracy | |
| `"absolute_colorimetric"` | Absolute Colorimetric | Preserve exact colors, clip | |

**Fluent 控件:** `<Dropdown>`

#### 参数: `black_point_compensation`

| 字段 | 值 |
|------|-----|
| 类型 | Boolean |
| 默认值 | `true` |
| 真标签 | "On" |
| 假标签 | "Off" |

**Fluent 控件:** `<Switch label={value ? "On" : "Off"}>`

#### 参数: `gamut_mapping`

| 字段 | 值 |
|------|-----|
| 类型 | Enum (3 选项) |
| 默认值 | `"compress"` |
| 高级 | **是** |

| 值 | 标签 | 描述 | 推荐 |
|------|------|------|:---:|
| `"compress"` | Compress | Smoothly compress into gamut | ★ |
| `"clip"` | Clip | Hard clip to target gamut | |
| `"luminance_preserve"` | Luminance Preserve | Preserve luminance, sacrifice chroma | |

**Fluent 控件:** `<Dropdown>` + `[advanced]` 标记

### 3.3 分区: ICC Profile (CollapsibleCard, 默认折叠, 2 params)

| 属性 | 值 |
|------|-----|
| 分区 ID | `icc_profile` |
| 标题 | ICC Profile |
| 描述 | ICC profile embedding and external profile usage |
| 图标 | file |

#### 参数: `embed_icc`

| 字段 | 值 |
|------|-----|
| 类型 | Boolean |
| 默认值 | `true` |
| 真标签 | "Embed" |
| 假标签 | "Skip" |

**Fluent 控件:** `<Switch label={value ? "Embed" : "Skip"}>`

#### 参数: `icc_profile_path`

| 字段 | 值 |
|------|-----|
| 类型 | **FilePath** |
| 默认值 | `""` |
| 文件过滤 | `*.icc`, `*.icm` |
| 必须存在 | **是** (后端验证) |
| 高级 | **是** |

**Fluent 控件:** `<Input placeholder="Path to .icc file">` + `<Button icon={FolderOpen}>` + `[advanced]` 标记

---

## 4. UI 布局

```
┌──────────────────────────────────────┐
│ Plugin Details · colorspace          │
├──────────────────────────────────────┤
│ [All] [Template] [High ISO] [img]  │
├──────────────────────────────────────┤
│ 🎨 Color Space                  v1.0 │ ← 紫色图标 #8b5cf6
│ photopipeline.plugins.colorspace     │
│ [Color] [icc] [srgb] [hdr]           │
│ PixelProcessor · RAM ≥ 256 MB        │
├──────────────────────────────────────┤
│ ▼ Color Space Conversion     2 params│
│  Source  [Auto-detect ★ — ... ▾] ⬜ │ ← Dropdown (8选项)
│  Target  [sRGB ★ — Standard  ▾] 🟡 │ ← Dropdown (6选项, overridden)
├──────────────────────────────────────┤
│ ▼ Rendering                  3 params│
│  Intent  [Relative Colormtc★ ▾] ⬜ │ ← Dropdown (4选项)
│  Black P [On ●――――○]           ⬜ │ ← Switch
│  Gamut M [Compress ★ — Sm.. ▾] ⬜ │ ← Dropdown + [advanced]
├──────────────────────────────────────┤
│ ▶ ICC Profile          default collapsed│
├──────────────────────────────────────┤
│ 𝑓 Expression                        │
├──────────────────────────────────────┤
│ 📊 Histogram                         │
│ ▂▃▅▆▇▆▅▃▂▁                         │
├──────────────────────────────────────┤
│ 🎯 Gamut Diagram                     │ ← 新增: CIE 1931 色域图
│ ┌──────────────────────────────┐    │
│ │ CIE 1931 xy chromaticity     │    │
│ │ diagram — gamut overlay      │    │
│ └──────────────────────────────┘    │
├──────────────────────────────────────┤
│ PREVIEW                              │
│ BeforeAfter (lock_zoom: true)        │
├──────────────────────────────────────┤
│ [🗑 Remove from Pipeline]           │
└──────────────────────────────────────┘
```

---

## 5. 新增控件类型: FilePath

colorspace 引入了第一个 **FilePath** 参数类型 `icc_profile_path`：

```
Custom ICC [advanced]  [________________] [📂]
```

| 组件 | 规格 |
|------|------|
| Input | flex:1, placeholder=描述文字, 默认值为空 |
| Button | 32×30px, FolderOpen 图标, 打开系统文件对话框 |
| 文件过滤 | `*.icc`, `*.icm` (来自 ParameterType::FilePath.filters) |
| 后端验证 | `must_exist: true` → 文件不存在时报 `ValidationIssue::Error` |

---

## 6. 新增辅助视图: GamutDiagram

| 属性 | 值 |
|------|-----|
| 视图类型 | `AuxView::GamutDiagram` |
| 显示内容 | CIE 1931 xy 色度图 + 源/目标色域叠加 |
| 尺寸 | 高度 ~80px，宽度填充面板 |

这是 raw_input(无aux) 和 transform(仅Histogram) 都没有的新视图。

---

## 7. 控件累计清单

| 控件类型 | raw_input | transform | colorspace |
|---------|:---:|:---:|:---:|
| Dropdown | ✅ | ✅ | ✅ |
| Switch | ✅ | ✅ | ✅ |
| Input (text) | ✅ | — | — |
| SpinButton | — | ✅ | — |
| Slider | — | ✅ | — |
| **FilePath** | — | — | ✅ (new) |

| 辅助视图 | raw_input | transform | colorspace |
|---------|:---:|:---:|:---:|
| Histogram | — | ✅ | ✅ |
| **GamutDiagram** | — | — | ✅ (new) |

---

## 8. 后端适配确认清单

- [x] 名称 "Color Space" / ID `photopipeline.plugins.colorspace` / 版本 v1.0.0
- [x] 分类 Color, 标签 color/colorspace/icc/profile/srgb/hdr
- [x] 3 个分区 ID/标题/样式与 GuiSchema 一致
- [x] source_color_space 8 个枚举值（含推荐/标签）与后端一致
- [x] target_color_space 6 个枚举值（含推荐/标签）与后端一致
- [x] rendering_intent 4 个枚举值与后端一致
- [x] gamut_mapping 3 个枚举值, advanced=true
- [x] black_point_compensation Boolean, label_true="On", label_false="Off"
- [x] embed_icc Boolean, label_true="Embed", label_false="Skip"
- [x] icc_profile_path FilePath, filters=[*.icc,*.icm], must_exist=true, advanced=true
- [x] GuiSchema icon/color/preview/aux_views/min_panel_width 全部对应
- [x] lock_zoom: true — BeforeAfter 预览锁定缩放
- [x] aux_views: [Histogram, GamutDiagram] — 两个辅助视图
