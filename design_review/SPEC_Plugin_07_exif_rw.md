# 插件 #07: exif_rw — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/exif_rw.rs`  
> **注意**: UI 设计的 43 个可编辑字段超出当前后端 ParameterSchema。后端需扩展以支持逐字段编辑。

---

## 1. 插件元数据

| 字段 | 值 |
|------|-----|
| 名称 | EXIF Reader/Writer |
| ID | `photopipeline.plugins.exif_rw` |
| 分类 | Metadata |
| 描述 | Read and write EXIF, XMP, IPTC, and GPS metadata via exiftool |
| 标签 | exif, metadata, xmp, iptc, exiftool |
| 能力 | MetadataProcessor · 128 MB · 无像素访问 |
| GuiSchema | icon=`"tag"`, color=`"#3b82f6"`, preview=None, aux=[], min=320px |

## 2. 核心设计

exif_rw 是**元数据编辑器**，不是简单的读写开关。用户期望看到所有常见 EXIF 字段并逐条编辑。字段分为 8 个可折叠分区，42+ 个字段。

## 3. 字段分类

### 3.1 Camera (5 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Make | RO | 从 EXIF 读取，灰色斜体 |
| Model | RO | 从 EXIF 读取 |
| Lens | RO | 镜头型号 |
| Serial Number | ✅ | 可覆盖 |
| Firmware | ✅ | 相机固件版本 |

### 3.2 Author & Copyright (5 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Artist | ✅ | EXIF:Artist · XMP:dc:creator |
| Copyright | ✅ | EXIF:Copyright · XMP:dc:rights |
| Image Description | ✅ | EXIF:ImageDescription |
| Rating | ✅ | ★★★★★ (0-5) |
| Instructions | ✅ | 使用说明 |

### 3.3 Date & Time (5 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Date Taken | ✅ | datetime-local 控件 |
| Digitized | RO | 拍摄日期（只读） |
| Offset Time | ✅ | +08:00 格式 |
| SubSec | ✅ | 亚秒时间 |
| Modified | RO | 最后修改（只读） |

### 3.4 Exposure (8 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Shutter Speed | ✅ | 1/250 格式 |
| Aperture | ✅ | f/8.0 格式 |
| ISO | ✅ | 数值 |
| Exposure Bias | ✅ | EV |
| Metering Mode | ✅ | Dropdown: Multi/Center/Spot |
| Flash | ✅ | Dropdown: Off/On/Auto |
| Exposure Program | ✅ | Dropdown: Aperture/Manual/Shutter/Auto |
| Scene Type | RO | 只读 |

### 3.5 Lens (4 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Focal Length | ✅ | mm |
| 35mm Equivalent | ✅ | mm |
| Max Aperture | ✅ | f/ |
| Lens ID | RO | 镜头识别码 |

### 3.6 GPS (5 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Latitude | ✅ | 十进制度数 |
| Longitude | ✅ | 十进制度数 |
| Altitude | ✅ | 米 |
| Reference | ✅ | Dropdown: N/E, S/W |
| Direction | ✅ | 度数 |

### 3.7 Image (6 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Orientation | ✅ | Dropdown: 1/3/6/8 |
| X Resolution | ✅ | DPI |
| Y Resolution | ✅ | DPI |
| Resolution Unit | ✅ | Dropdown: inches/cm |
| Color Space | ✅ | Dropdown: sRGB/AdobeRGB/Uncalibrated |
| Software | ✅ | 处理软件名称 |

### 3.8 Keywords & Location (5 fields)

| 字段 | 可编辑 | 说明 |
|------|:---:|------|
| Keywords | ✅ | 逗号分割 |
| City | ✅ | XMP:photoshop:City |
| Province/State | ✅ | |
| Country | ✅ | XMP:photoshop:Country |
| Country Code | ✅ | ISO 3166 |

## 4. 字段状态

| 状态 | 样式 | 含义 |
|:---:|------|------|
| 可编辑 | 正常 Input/Select | 用户可修改 |
| 只读 (RO) | 灰色斜体 | 从源文件提取，不可覆盖 |
| 已覆盖 🟡 | 黄色圆点标记 | 在当前层级被覆盖的值 |

## 5. 现有后端参数（需扩展）

当前 ParameterSchema 仅有读/写开关，缺少逐字段定义：

```
read_options: read_exif, read_xmp, read_iptc, read_gps (Boolean)
write_options: overwrite_original, preserve_makernote, write_exif
exiftool (CollapsibleCard): exiftool_path, exiftool_args
```

**建议**: 后端增加 `exif_fields` 分区，包含 42+ 个 ParameterField，每个字段定义其 EXIF tag 名称、可编辑性、默认值来源（from_source / custom）。

## 6. 布局特点

- **面板宽度**: 520px（比其他插件宽 100px，因为字段多）
- **网格布局**: `grid-template-columns: 84px 1fr 10px` — 标签/控件/覆盖标记
- **8 个可折叠分区**: 全部默认展开（Camera 部分字段只读可折叠）
- **无预览、无辅助视图**: 纯元数据节点
