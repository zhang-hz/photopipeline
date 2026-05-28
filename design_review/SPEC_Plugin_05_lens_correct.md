# 插件 #05: lens_correct — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/lens_correct.rs`

---

## 1. 元数据

| 字段 | 值 | 来源 |
|------|-----|------|
| 名称 | Lens Correction | `fn name()` |
| ID | `photopipeline.plugins.lens_correct` | `fn id()` |
| 版本 | 1.0.0 | `fn version()` |
| 分类 | Correct | `PluginCategory::Enhance` |
| 描述 | Correct lens distortion, chromatic aberration, and vignetting via LensFun | `fn description()` |
| 标签 | lens, distortion, tca, vignetting, lensfun | `TAGS` |
| 能力 | PixelProcessor | |
| 内存 | 256 MB | |

## 2. GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `"aperture"` |
| color | `"#6366f1"` (靛蓝) |
| preview | BeforeAfter (lock_zoom: true) |
| aux_views | **[Histogram, StatusText]** |
| min_panel_width | 340px |

### 分区

| 分区 ID | 标题 | 样式 | 默认 |
|---------|------|------|:---:|
| `lens_detection` | Lens Detection | Card | 展开 |
| `corrections` | Corrections | Card | 展开 |
| `lensfun` | LensFun | CollapsibleCard | 折叠 |

## 3. 参数

### 3.1 Lens Detection (1 param)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `correction_mode` | Enum {auto(★), manual, off} | `"auto"` | Dropdown |

### 3.2 Corrections (4 params, all Boolean Correct/Skip)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `correct_distortion` | Boolean | `true` | Switch |
| `correct_tca` | Boolean | `true` | Switch |
| `correct_vignetting` | Boolean | `true` | Switch |
| `correct_geometry` | Boolean | `false` | Switch + `[advanced]` |

所有 Boolean 的 label_true="Correct", label_false="Skip"。

### 3.3 LensFun (4 params, all advanced, CollapsibleCard 默认折叠)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `lensfun_db_path` | **FilePath(kind=Directory)** | `/usr/share/lensfun` | Input+📂 + `[advanced]` |
| `camera_make` | String(max=128) | `""` | Input("Sony") + `[advanced]` |
| `camera_model` | String(max=128) | `""` | Input("ILCE-7RM5") + `[advanced]` |
| `lens_model` | String(max=256) | `""` | Input("FE 24-70mm F2.8 GM II") + `[advanced]` |

## 4. 新增: FilePath(kind=Directory)

`lensfun_db_path` 是第一个目录选择器。不同于之前的文件选择器（colorspace 的 ICC 文件、lut3d 的 LUT 文件），这里选择 LensFun XML 数据库所在目录。

## 5. 新增辅助视图: StatusText

| 属性 | 值 |
|------|-----|
| 类型 | `AuxView::StatusText` |
| 显示 | 文本信息 |

用于显示镜头检测结果："Detected: Sony ILCE-7RM5 + FE 24-70mm F2.8 GM II · LensFun XML 2025-01-01"

## 6. 控件累计

| 控件 | #01 | #02 | #03 | #04 | #05 |
|------|:---:|:---:|:---:|:---:|:---:|
| Dropdown | ✅ | ✅ | ✅ | ✅ | ✅ |
| Switch | ✅ | ✅ | ✅ | ✅ | ✅ |
| Input (text) | ✅ | — | — | — | ✅ |
| SpinButton | — | ✅ | — | — | — |
| Slider (from Int/Float) | — | ✅ | — | — | — |
| ParameterType::Slider | — | — | — | ✅ | — |
| FilePath (file) | — | — | ✅ | ✅ | — |
| **FilePath (dir)** | — | — | — | — | ✅ |
| Histogram | — | ✅ | ✅ | ✅ | ✅ |
| GamutDiagram | — | — | ✅ | — | — |
| Vectorscope | — | — | — | ✅ | — |
| **StatusText** | — | — | — | — | ✅ |

## 7. 后端适配确认

- [x] correction_mode 3 个枚举值 (auto★/manual/off)
- [x] 4 个 Boolean 全部 label_true="Correct", label_false="Skip"
- [x] correct_geometry advanced=true, default=false
- [x] lensfun_db_path FilePathKind::Directory, must_exist=true
- [x] camera_make/camera_model/lens_model 全部 String + advanced
- [x] GuiSchema: icon/color/preview/aux_views=[Histogram,StatusText]
