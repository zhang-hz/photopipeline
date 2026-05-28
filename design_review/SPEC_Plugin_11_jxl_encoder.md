# 插件 #11: jxl_encoder — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/jxl_encoder.rs`

## 1. 元数据

| 字段 | 值 |
|------|-----|
| 名称 | JPEG XL Encoder |
| ID | `photopipeline.plugins.jxl_encoder` |
| 分类 | Format |
| 描述 | Encode images in JPEG XL 16-bit format via libjxl native FFI |
| 标签 | jxl, jpeg-xl, encode, format, 16bit |
| 能力 | FormatProcessor · 512 MB |
| GuiSchema | icon=`"file-image"`, color=`"#f97316"`(橙), preview=None, aux=[], min=320px |

## 2. 参数

### Quality (Card, 展开)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `quality` | Slider (-1~100, ticks[-1,25,50,75,100]) | 90.0 | Slider + 特殊刻度 |
| `lossless` | Boolean (Lossless/Lossy) | `false` | Switch |
| `bit_depth` | Enum {16★(hdr), 12, 10, 8} | `"16"` | Dropdown |

> quality=-1 等同于 lossless=true。Slider 首个刻度显示 "lossless"。

### Advanced (Card, 展开)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `effort` | Integer (1~9) | 7 | SpinButton + `[advanced]` |
| `modular` | Boolean (Modular/VarDCT) | `false` | Switch(VarDCT/Modular) + `[advanced]` |

## 3. jxl vs heif 差异

| | heif | jxl |
|------|------|-----|
| Quality 范围 | 0-100 | **-1~100** |
| Bit depth | 8/10 | 8/10/12/**16** |
| 默认 bit depth | 10 | 16 |
| Advanced | Chroma, Effort, Tune | Effort, Mode(VarDCT/Modular) |
| 颜色 | #14b8a6 | #f97316 |
