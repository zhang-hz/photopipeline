# 插件 #12: avif_encoder — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/avif_encoder.rs`

## 1. 元数据

| 字段 | 值 |
|------|-----|
| 名称 | AVIF Encoder |
| ID | `photopipeline.plugins.avif_encoder` |
| 分类 | Format |
| 描述 | Encode images in AVIF format using ravif (pure-Rust AV1) |
| 标签 | avif, encode, format |
| 能力 | FormatProcessor · 512 MB |
| GuiSchema | icon=`"image"`, color=`"#22c55e"`(绿), preview=None, aux=[], min=320px |

## 2. 参数

### Quality (Card, 展开, 2 params)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `quality` | Slider (0–100, ticks[0,25,50,75,100]) | 85.0 | Slider + 刻度 |
| `speed` | Integer (0~10) | 6 | SpinButton (0=slow/best, 10=fast) |

### Format Options (Card, 展开, 2 params)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `bit_depth` | Enum {10★, 12[hdr], 8} | `"10"` | Dropdown |
| `chroma_subsampling` | Enum {444★, 422, 420} | `"444"` | Dropdown + `[advanced]` |

### Advanced (CollapsibleCard, 默认折叠, 1 param)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `lossless` | Boolean (Lossless/Lossy) | `false` | Switch + `[advanced]` |

## 3. 编码器对比总结

| | heif | jxl | avif |
|------|------|------|------|
| Quality 范围 | 0–100 | **-1**–100 | 0–100 |
| Bit depth | 8/10 | 8/10/12/**16** | 8/10/12 |
| 默认 quality | 95 | 90 | **85** |
| 独有 | Chroma, Effort, Tune | Effort, Modular | **Speed** |
| 颜色 | #14b8a6 | #f97316 | **#22c55e** |
| Lossless 位置 | Quality section | Quality section | **Advanced (collapsed)** |
