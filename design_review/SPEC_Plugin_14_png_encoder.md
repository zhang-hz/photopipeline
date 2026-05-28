# 插件 #14: png_encoder

> 后端: `feat/unified-binary` · `crates/plugins/src/png_encoder.rs`

| 字段 | 值 |
|------|-----|
| 名称 | PNG Encoder |
| ID | `photopipeline.plugins.png_encoder` |
| 分类 | Format · 128 MB |
| GuiSchema | icon=`"image"`, color=`"#0ea5e9"`(蓝), preview=None, aux=[], min=320px |

## Encoding (Card)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `compression_level` | Integer (0~9) | 6 | SpinButton (0=store, 9=best) |
| `bit_depth` | Enum {16★[hdr], 8} | `"16"` | Dropdown |

## Metadata (Card)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `embed_icc` | Boolean (Embed/Skip) | `true` | Switch |
| `include_exif` | Boolean (Include/Skip) | `false` | Switch |
| `color_type` | Enum {RGB★, RGBA, Grayscale, Gray+Alpha} | `"rgb"` | Dropdown |

## 特点

- 最轻量：128 MB RAM（所有编码器中最低）
- 5 个参数，2 个 Card，无 Advanced 折叠
- Compression Level 使用 SpinButton（非 Slider）
- Color Type 含 4 种 PNG 标准类型
- 颜色: #0ea5e9（天蓝）

---

## 全部 14 个插件完成

| # | 插件 | 类别 | 参数 | 独特控件 |
|---|------|------|:---:|------|
| 01 | raw_input | Input | 6 | — |
| 02 | transform | Transform | 14 | SpinButton + Slider |
| 03 | colorspace | Color | 8 | FilePath, GamutDiagram |
| 04 | lut3d | Color | 7 | ParamType::Slider, Vectorscope |
| 05 | lens_correct | Correct | 8 | FilePath(Dir), StatusText |
| 06 | ai_denoise | Enhance | 7 | ModelInfo, ProgressBar |
| 07 | exif_rw | Metadata | 43 | 可编辑字段网格 |
| 08 | gps_set | Metadata | 7 | Map Picker, 供应商切换 |
| 09 | time_shift | Metadata | 7 | Live Preview |
| 10 | heif_encoder | Format | 6 | Chroma 444/422/420 |
| 11 | jxl_encoder | Format | 5 | Quality -1=lossless |
| 12 | avif_encoder | Format | 5 | Speed 0-10 |
| 13 | tiff_encoder | Format | 4 | 最简洁 |
| 14 | png_encoder | Format | 5 | Color Type |
