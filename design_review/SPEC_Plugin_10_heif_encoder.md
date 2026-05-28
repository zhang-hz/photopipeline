# 插件 #10: heif_encoder — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/heif_encoder.rs`

## 1. 元数据

| 字段 | 值 |
|------|-----|
| 名称 | HEIF Encoder |
| ID | `photopipeline.plugins.heif_encoder` |
| 分类 | Format |
| 描述 | Encode images in HEIF/HEIC 10-bit format using libheif native FFI |
| 标签 | heif, heic, encode, format, 10bit |
| 能力 | FormatProcessor · 512 MB |
| GuiSchema | icon=`"image"`, color=`"#14b8a6"`, preview=None, aux=[], min=320px |

## 2. 参数

### Quality (Card, 展开)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `quality` | Slider (0–100, ticks[0,25,50,75,100]) | 95.0 | Slider + 刻度 |
| `lossless` | Boolean (Lossless/Lossy) | `false` | Switch |
| `bit_depth` | Enum {10★(hdr), 8} | `"10"` | Dropdown |

### Advanced (Card, 展开)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `chroma_subsampling` | Enum {444★, 422, 420} | `"444"` | Dropdown + `[advanced]` |
| `encoder_effort` | Integer (0~10) | 4 | SpinButton + `[advanced]` |
| `tune` | Enum {ssim★, psnr, grain, fastdecode} | `"ssim"` | Dropdown + `[advanced]` |

## 3. 编码器插件模板

heif_encoder 是所有 5 个编码器插件的基础模板:

```
Quality (Card, 展开) — 质量/格式参数
Advanced (Card, 展开) — 编码器专有高级选项
```

- 无预览（Export 节点）
- 无辅助视图
- 参数紧凑，3+3 结构
