# 插件 #13: tiff_encoder

> 后端: `feat/unified-binary` · `crates/plugins/src/tiff_encoder.rs`

| 字段 | 值 |
|------|-----|
| 名称 | TIFF Encoder |
| ID | `photopipeline.plugins.tiff_encoder` |
| 分类 | Format · 256 MB |
| GuiSchema | icon=`"file"`, color=`"#64748b"`(灰蓝), preview=None, aux=[], min=320px |

## Encoding (Card)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `compression` | Enum {None★, LZW, Deflate, PackBits} | `"none"` | Dropdown |
| `bigtiff` | Boolean (BigTIFF/Classic) | `true` | Switch |

## Metadata (Card)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `embed_icc` | Boolean (Embed/Skip) | `true` | Switch |
| `pixel_format` | Enum {16★, 32(f32), 8} | `"u16"` | Dropdown |

## 特点

- 最简洁的编码器：仅 4 个参数，2 个 Card，无 Advanced 折叠区
- 无 Slider — TIFF 是无损容器，不需要质量滑块
- 颜色: #64748b（灰蓝，"文件柜"色）
