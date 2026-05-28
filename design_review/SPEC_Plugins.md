# Photopipeline 插件 UI 设计规格书

> 每个插件定义 ParameterSchema（后端参数定义，gRPC 传输）+ GuiSchema（前端面板布局）  
> 前端从 `PluginService.ListPlugins()` 获取插件列表，`GetNodeSchema()` 获取完整 schema

---

## 插件分类总览

| 类别 | 插件 | ID | 能力 Trait | 预览 |
|------|------|-----|-----------|:---:|
| Input | raw_input | `photopipeline.plugins.raw_input` | FormatProcessor | None |
| Transform | transform | `photopipeline.plugins.transform` | PixelProcessor | BeforeAfter |
| Color | colorspace | `photopipeline.plugins.colorspace` | PixelProcessor | BeforeAfter |
| Color | lut3d | `photopipeline.plugins.lut3d` | PixelProcessor | BeforeAfter |
| Correct | lens_correct | `photopipeline.plugins.lens_correct` | PixelProcessor | BeforeAfter |
| Enhance | ai_denoise | `photopipeline.plugins.ai_denoise` | PixelProcessor + AiProcessor | BeforeAfter(lockZoom) |
| Metadata | exif_rw | `photopipeline.plugins.exif_rw` | MetadataProcessor | None |
| Metadata | gps_set | `photopipeline.plugins.gps_set` | MetadataProcessor | None |
| Metadata | time_shift | `photopipeline.plugins.time_shift` | MetadataProcessor | None |
| Export | heif_encoder | `photopipeline.plugins.heif_encoder` | FormatProcessor | None |
| Export | jxl_encoder | `photopipeline.plugins.jxl_encoder` | FormatProcessor | None |
| Export | avif_encoder | `photopipeline.plugins.avif_encoder` | FormatProcessor | None |
| Export | tiff_encoder | `photopipeline.plugins.tiff_encoder` | FormatProcessor | None |
| Export | png_encoder | `photopipeline.plugins.png_encoder` | FormatProcessor | None |

---

## 通用组件结构

每个插件面板由以下区域组成（从上到下）：

```
┌──────────────────────────────────────┐
│ [上下文栏]  Template / Group / Image  │ ← 覆盖层级切换
├──────────────────────────────────────┤
│ [插件信息]  名称 · 版本 · 描述 · 分类  │ ← 插件元数据
├──────────────────────────────────────┤
│ [参数分区1]  Card / Default 样式      │ ← 始终展开
│  ├ 参数行 × N                        │
│  └ 覆盖标记 ⬜🟡🔵                   │
├──────────────────────────────────────┤
│ [参数分区2]  CollapsibleCard 样式     │ ← 可折叠
│  ├ 参数行 × N                        │
│  └ ...                               │
├──────────────────────────────────────┤
│ [表达式编辑器]  条件显示              │ ← supports_expression=true 时
├──────────────────────────────────────┤
│ [辅助视图]  Histogram / Map / etc.    │ ← GuiSchema.aux_views
├──────────────────────────────────────┤
│ [Remove 按钮]                        │
└──────────────────────────────────────┘
```

---

## 1. raw_input — RAW 输入

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.raw_input` |
| 名称 | RAW Input |
| 分类 | Input |
| 版本 | 1.0.0 |
| 描述 | Read RAW camera files (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF) |
| 能力 | FormatProcessor |
| 最低内存 | 512 MB |
| 像素输入 | 否 |
| 像素输出 | 是 |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `camera` |
| color | `#ef4444` (红) |
| preview | None |
| aux_views | [] |
| min_panel_width | 320px |

### 参数分区

#### Section: "RAW Format" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `raw_mode` | Enum | `"auto"` | Dropdown | 解码引擎: Auto / dcraw / LibRaw / RawTherapee |

#### Section: "Output" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `output_format` | Enum | `"u16"` | Dropdown | 输出像素格式: 16-bit / 32-bit float |
| `half_size` | Boolean | `false` | Switch | 是否解码为半分辨率 (快速预览) |
| `apply_white_balance` | Boolean | `true` | Switch | 是否应用相机白平衡 (隐藏高级选项) |

#### Section: "dcraw Options" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `dcraw_path` | String | `"dcraw"` | Input | dcraw 二进制路径 |
| `dcraw_extra_args` | String | `""` | Input | 额外 dcraw 命令行参数 |

---

## 2. transform — 几何变换

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.transform` |
| 名称 | Transform |
| 分类 | Transform |
| 描述 | Resize, rotate, and crop images with configurable filters |
| 能力 | PixelProcessor |
| 最低内存 | 256 MB |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `maximize` |
| color | `#06b6d4` (青) |
| preview | **BeforeAfter** (default_split: 0.5, Horizontal, lock_zoom: false) |
| aux_views | **[Histogram]** |
| min_panel_width | 340px |

### 参数分区

#### Section: "Resize" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `width` | Integer | `1920` | Slider (1-65535) | 目标宽度 |
| `height` | Integer | `1080` | Slider (1-65535) | 目标高度 |
| `keep_aspect` | Boolean | `true` | Switch | 保持宽高比 |
| `filter` | Enum | `"Lanczos3"` | Dropdown | 重采样: Nearest/Bilinear/CatmullRom/Lanczos3/Mitchell/Gaussian |
| `fit_mode` | Enum | `"fit"` | Dropdown | 缩放模式: Fit/Fill/Crop/Distort |

#### Section: "Rotation" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `angle` | Float | `0.0` | Slider (-360~360, step=0.1) | 旋转角度 (度) |
| `flip_horizontal` | Boolean | `false` | Switch | 水平翻转 |
| `flip_vertical` | Boolean | `false` | Switch | 垂直翻转 |
| `auto_orient` | Boolean | `true` | Switch | 自动方向 (EXIF) |
| `background_color` | Color | `#000000` | ColorPicker | 旋转背景填充色 |

#### Section: "Crop" (Card, 始终展开)  

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `crop_x` | Integer | `0` | Input | 裁剪 X 起点 |
| `crop_y` | Integer | `0` | Input | 裁剪 Y 起点 |
| `crop_width` | Integer | 0 | Input | 裁剪宽度 (0=不限) |
| `crop_height` | Integer | 0 | Input | 裁剪高度 (0=不限) |
| `aspect_ratio` | Enum | `"free"` | ButtonGroup | 约束比例: Free/1:1/4:3/3:2/16:9/16:10 |

#### Section: "Filter" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `sharpen_amount` | Float | `0.0` | Slider (0~1) | 锐化量 |
| `sharpen_radius` | Float | `1.0` | Slider (0.3~5.0) | 锐化半径 |

---

## 3. colorspace — 色彩空间转换

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.colorspace` |
| 名称 | Color Space |
| 分类 | Color |
| 描述 | Convert between color spaces with ICC profile support and rendering intents |
| 能力 | PixelProcessor |
| 最低内存 | 256 MB |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `palette` |
| color | `#8b5cf6` (紫) |
| preview | **BeforeAfter** (lock_zoom: true) |
| aux_views | **[Histogram, GamutDiagram]** |
| min_panel_width | 340px |

### 参数分区

#### Section: "Conversion" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `source` | Enum | `"sRGB"` | Dropdown (8选项) | 源色彩空间 |
| `target` | Enum | `"sRGB"` | Dropdown (7选项) | 目标色彩空间 |
| `intent` | Enum | `"Relative"` | Dropdown | 渲染意图: Perceptual/Relative/Saturation/Absolute |

#### Section: "Rendering" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `black_point_compensation` | Boolean | `true` | Switch | 黑点补偿 |
| `gamut_clipping` | Boolean | `true` | Switch | 色域裁剪 |
| `linearize` | Boolean | `false` | Switch | 线性化输出 |

#### Section: "ICC Profile" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `icc_profile_path` | FilePath | `""` | Input+浏览 | 外部 ICC 文件路径 |
| `embed_icc` | Boolean | `true` | Switch | 嵌入 ICC 到输出 |

---

## 4. lut3d — 3D LUT 应用

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.lut3d` |
| 名称 | 3D LUT |
| 分类 | Color |
| 描述 | Apply 3D Look-Up Tables for color grading and film emulation |
| 能力 | PixelProcessor |
| GPU | 推荐 |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `grid3x3` |
| color | `#ec4899` (粉) |
| preview | **BeforeAfter** (lock_zoom: true) |
| aux_views | **[Histogram, Vectorscope]** |
| min_panel_width | 320px |

### 参数分区

#### Section: "LUT File" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `lut_file` | FilePath | `""` | Input+浏览 | .cube / .3dl LUT 文件 |
| `lut_format` | Enum | `"auto"` | Dropdown | 格式: Auto/Cube/3DL/SPI/KiPro |
| `lut_size` | Enum | `"32"` | Dropdown | LUT 精度: 17/33/65 |

#### Section: "LUT Transform" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `intensity` | Float | `1.0` | Slider (0~1, step=0.01) | LUT 混合强度 |
| `input_color_space` | Enum | `"auto"` | Dropdown | 输入色彩空间 |
| `output_color_space` | Enum | `"auto"` | Dropdown | 输出色彩空间 |

#### Section: "Interpolation" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `interpolation` | Enum | `"Trilinear"` | Dropdown | 插值: Trilinear/Tetrahedral/Pyramid |
| `cubic_b_spline` | Boolean | `false` | Switch | 使用三次B样条平滑 |

---

## 5. lens_correct — 镜头校正

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.lens_correct` |
| 名称 | Lens Correction |
| 分类 | Correct |
| 描述 | Correct lens distortion, chromatic aberration, and vignetting via LensFun |
| 能力 | PixelProcessor |
| 最低内存 | 256 MB |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `aperture` |
| color | `#6366f1` (靛) |
| preview | **BeforeAfter** (lock_zoom: true) |
| aux_views | **[Histogram, StatusText]** |
| min_panel_width | 340px |

### 参数分区

#### Section: "Lens Detection" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `camera_make` | String | `""` | Input (自动检测) | 相机品牌 (EXIF) |
| `camera_model` | String | `""` | Input (自动检测) | 相机型号 (EXIF) |
| `lens_name` | String | `""` | Input (自动检测) | 镜头名称 (EXIF) |
| `auto_detect` | Boolean | `true` | Switch | 从 EXIF 自动检测 |

#### Section: "Corrections" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `distortion` | Boolean | `true` | Switch | 校正畸变 |
| `tca` | Boolean | `true` | Switch | 校正横向色差 (TCA) |
| `vignetting` | Boolean | `true` | Switch | 校正暗角 |
| `geometry_scale` | Float | `1.0` | Slider (0.5~2.0) | 几何校正强度 |

#### Section: "LensFun" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `lensfun_db_path` | FilePath | `""` | Input+浏览 | 自定义 LensFun 数据库 |
| `update_db` | Boolean | `false` | Switch | 自动更新数据库 |

---

## 6. ai_denoise — AI 降噪

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.ai_denoise` |
| 名称 | AI Denoise |
| 分类 | Enhance |
| 描述 | AI-based noise reduction using ONNX models |
| 能力 | PixelProcessor + AiProcessor |
| 最低内存 | 2048 MB |
| GPU | 推荐 (ONNX Runtime GPU) |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `sparkles` |
| color | `#a855f7` (紫罗兰) |
| preview | **BeforeAfter** (lock_zoom: true) |
| aux_views | **[Histogram, ProgressBar, StatusText]** |
| min_panel_width | 360px |

### 参数分区

#### Section: "Model" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `model` | Enum | `"standard_v2"` | Dropdown (含model元数据) | AI 模型选择 |
| `model_source` | Enum | `"huggingface"` | Dropdown | 模型来源: HuggingFace/Local/URL |
| `model_path` | FilePath | `""` | Input+浏览 | 本地模型路径 (source=local时显示) |
| `download_model` | Button | — | Button | 下载/更新模型 |

**ModelInfo 卡片（模型选择后动态显示）：**
- 模型名称、版本、来源仓库
- 输入/输出张量形状 (e.g. [1,3,1024,1024])
- 内存需求 (e.g. 2GB VRAM)
- 描述文本

#### Section: "Strength" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `strength` | Float | `0.5` | Slider (0~1, step=0.01) | 总体降噪强度 |
| `luma_strength` | Float | `0.5` | Slider (0~1, step=0.01) | 亮度通道降噪 |
| `chroma_strength` | Float | `0.5` | Slider (0~1, step=0.01) | 色度通道降噪 |
| `noise_model` | Enum | `"auto"` | Dropdown | 噪声模型: Auto/Gaussian/Poisson/混合 |
| `color_noise_reduction` | Enum | `"balanced"` | Dropdown | 色噪处理: Aggressive/Balanced/Conservative |

#### Section: "Hardware" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `backend` | Enum | `"auto"` | Dropdown | 后: Auto/CPU/GPU(NPU) |
| `gpu_device` | Enum | `"0"` | Dropdown | GPU 设备选择 |
| `tile_size` | Integer | `1024` | Input (256~4096) | 分块大小 (VRAM 限制) |
| `tile_overlap` | Integer | `64` | Input (0~tile_size/2) | 分块重叠 |

---

## 7. exif_rw — EXIF 读写

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.exif_rw` |
| 名称 | EXIF Reader/Writer |
| 分类 | Metadata |
| 描述 | Read and write EXIF, XMP, IPTC, and GPS metadata via exiftool |
| 能力 | MetadataProcessor |
| 最低内存 | 128 MB |
| 像素访问 | 否 |
| 像素输出 | 否 |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `tag` |
| color | `#3b82f6` (蓝) |
| preview | None |
| aux_views | [] |
| min_panel_width | 320px |

### 参数分区

#### Section: "Read Options" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `extract_exif` | Boolean | `true` | Switch | 提取 EXIF |
| `extract_xmp` | Boolean | `true` | Switch | 提取 XMP |
| `extract_iptc` | Boolean | `true` | Switch | 提取 IPTC |
| `extract_icc` | Boolean | `true` | Switch | 提取 ICC Profile |

#### Section: "Write Options" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `preserve_source` | Boolean | `true` | Switch | 保留源元数据 |
| `copyright` | String | `""` | Input | 版权信息 |
| `artist` | String | `""` | Input | 作者/摄影师 |
| `description` | String | `""` | Input (多行) | 图像描述 |
| `keywords` | String | `""` | Input | 关键词 (逗号分割) |
| `rating` | Integer | `0` | Slider (0~5) | 评分 |

#### Section: "ExifTool" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `exiftool_path` | FilePath | `"exiftool"` | Input+浏览 | exiftool 路径 |
| `exiftool_args` | String | `""` | Input | 额外 exiftool 参数 |
| `skip_exiftool` | Boolean | `false` | Switch | 仅用内置 parser (轻量模式) |

---

## 8. gps_set — GPS 坐标设置

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.gps_set` |
| 名称 | GPS Coordinate Manager |
| 分类 | Metadata |
| 描述 | Set GPS coordinates manually or interpolated from GPX track logs |
| 能力 | MetadataProcessor |
| 最低内存 | 64 MB |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `map-pin` |
| color | `#10b981` (绿) |
| preview | None |
| aux_views | **[Map]** |
| min_panel_width | 340px |

### 参数分区

#### Section: "Source" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `mode` | Enum | `"exif_preserve"` | Dropdown | GPS 模式: 保留EXIF/手动输入/GPX插值/EXIF覆盖/清除GPS |

#### Section: "Manual Coordinates" (Card, mode="manual" 时显示)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `latitude` | Float | `0.0` | Input (-90~90) | 纬度 |
| `longitude` | Float | `0.0` | Input (-180~180) | 经度 |
| `altitude` | Float | `0.0` | Input | 海拔 (米) |

#### Section: "GPX Options" (CollapsibleCard, mode="gpx" 时显示)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `gpx_file` | FilePath | `""` | Input+浏览 | GPX 轨迹文件 |
| `time_offset` | Integer | `0` | Input | 时间偏移 (秒) |

---

## 9. time_shift — 时间偏移

| 属性 | 值 |
|------|-----|
| ID | `photopipeline.plugins.time_shift` |
| 名称 | Time Shift |
| 分类 | Metadata |
| 描述 | Adjust DateTimeOriginal by hours, minutes, and seconds with timezone support |
| 能力 | MetadataProcessor |
| 最低内存 | 64 MB |

### GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `clock` |
| color | `#f59e0b` (琥珀) |
| preview | None |
| aux_views | [] |
| min_panel_width | 320px |

### 参数分区

#### Section: "Time Shift" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `hours` | Integer | `0` | Input (-23~23) | 小时偏移 |
| `minutes` | Integer | `0` | Input (-59~59) | 分钟偏移 |
| `seconds` | Integer | `0` | Input (-59~59) | 秒偏移 |

#### Section: "Timezone" (Card, 始终展开)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `timezone_from` | Enum | `"UTC"` | Dropdown (全球时区) | 源时区 |
| `timezone_to` | Enum | `"UTC"` | Dropdown (全球时区) | 目标时区 |

#### Section: "Batch" (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认值 | 控件 | 说明 |
|------|------|--------|------|------|
| `offset_mode` | Enum | `"fixed"` | Dropdown | 偏移模式: 固定/递增/从文件名提取 |
| `increment_seconds` | Integer | `60` | Input (mode="递增"时) | 每张图片递增秒数 |

---

## 10-14. 编码器插件组 (heif / jxl / avif / tiff / png)

5 个编码器共享相似的参数结构。以下列出各编码器的独有参数。

### 通用结构

每个编码器有 2 个参数分区：
- **Quality** (Card, 始终展开): 质量/格式相关参数
- **Advanced / Metadata** (CollapsibleCard, 默认折叠): 高级选项

| 编码器 | GuiSchema.icon | GuiSchema.color | 输出格式 | 特点 |
|--------|:---:|------|----------|------|
| heif_encoder | image | #14b8a6 (青) | HEIF/HEIC 10-bit | x265/x264 编码器, chroma 子采样 |
| jxl_encoder | file-image | #f97316 (橙) | JPEG XL 16-bit | libjxl effort 级别, 模块化编码 |
| avif_encoder | image | #22c55e (绿) | AVIF | rav1e/alliance, YUV 444/422/420 |
| tiff_encoder | file | #64748b (灰蓝) | TIFF | LZW/ZIP 压缩, ICC嵌入 |
| png_encoder | image | #0ea5e9 (蓝) | PNG 8/16-bit | 压缩级别, 滤镜策略, ICC iCCP |

### heif_encoder 独有参数

| 参数 | 类型 | 默认值 | 控件 |
|------|------|--------|------|
| `encoder` | Enum | `"x265"` | Dropdown: x265/x264/aom |
| `preset` | Enum | `"veryslow"` | Dropdown (x265): ultrafast~placebo |
| `crf` | Integer | `18` | Slider (0~51) |
| `bit_depth` | Enum | `"10"` | Dropdown: 8/10/12 |
| `chroma_subsampling` | Enum | `"444"` | Dropdown: 444/422/420 |
| `lossless` | Boolean | `false` | Switch |
| `tune` | Enum | `"ssim"` | Dropdown (x265): psnr/ssim/grain/fastdecode |

### jxl_encoder 独有参数

| 参数 | 类型 | 默认值 | 控件 |
|------|------|--------|------|
| `effort` | Integer | `7` | Slider (1~9) |
| `distance` | Float | `1.0` | Slider (0~15, Butteraugli) |
| `modular` | Boolean | `true` | Switch (无损模式) |
| `lossless` | Boolean | `false` | Switch |
| `bit_depth` | Enum | `"16"` | Dropdown: 8/16/32 |
| `progressive` | Boolean | `true` | Switch (渐进式) |
| `coding_speed` | Enum | `"default"` | Dropdown: Lightning/Thunder/Falcon/Cheetah/Hare/Wombat/Squirrel/Kitten/Tortoise/Glacier |

### avif_encoder 独有参数

| 参数 | 类型 | 默认值 | 控件 |
|------|------|--------|------|
| `encoder` | Enum | `"rav1e"` | Dropdown: rav1e/aom/svt-av1 |
| `speed` | Integer | `4` | Slider (0~10) |
| `quality` | Integer | `80` | Slider (0~100) |
| `bit_depth` | Enum | `"10"` | Dropdown: 8/10/12 |
| `yuv_format` | Enum | `"444"` | Dropdown: 444/422/420 |
| `color_primaries` | Enum | `"bt709"` | Dropdown |
| `transfer` | Enum | `"bt709"` | Dropdown |
| `matrix` | Enum | `"bt709"` | Dropdown |

### tiff_encoder 独有参数

| 参数 | 类型 | 默认值 | 控件 |
|------|------|--------|------|
| `compression` | Enum | `"lzw"` | Dropdown: None/LZW/ZIP/PackBits |
| `predictor` | Enum | `"horizontal"` | Dropdown: None/Horizontal/Float |
| `bit_depth` | Enum | `"16"` | Dropdown: 8/16/32 |
| `big_tiff` | Boolean | `false` | Switch (输出 >4GB 时启用) |
| `embed_icc` | Boolean | `true` | Switch (嵌入ICC标签) |
| `rows_per_strip` | Integer | `0` | Input (0=自动) |

### png_encoder 独有参数

| 参数 | 类型 | 默认值 | 控件 |
|------|------|--------|------|
| `compression_level` | Integer | `6` | Slider (0~9) |
| `filter_strategy` | Enum | `"adaptive"` | Dropdown: None/Sub/Up/Average/Paeth/Adaptive |
| `bit_depth` | Enum | `"16"` | Dropdown: 8/16 |
| `color_type` | Enum | `"rgb"` | Dropdown: RGB/RGBA/Grayscale/GrayscaleAlpha |
| `embed_icc` | Boolean | `true` | Switch (嵌入iCCP chunk) |
| `interlaced` | Boolean | `false` | Switch (Adam7 隔行) |

---

## 控件映射参考

| ParameterType | Fluent 组件 | 用法 |
|--------------|-----------|------|
| `Enum` | `<Dropdown>` + `<Option>` | 显示 label，标记 recommended |
| `Integer` (有 min/max) | `<Slider>` + `<Input>` | Slider 为主，Input 精确输入 |
| `Integer` (无范围) | `<Input type="number">` | `<SpinButton>` |
| `Float` (有 min/max) | `<Slider>` | 显示单位 (unit) |
| `Boolean` | `<Switch>` | 如有 label_true/label_false 作为开关文本 |
| `String` | `<Input>` | 如有 placeholder 作为提示 |
| `FilePath` | `<Input>` + `<Button icon={<FolderOpen/>}>` | 浏览按钮 |
| `Color` | `<ColorPicker>` | 自定义组件 |
| `Coordinate` | 双 `<Input>` (lat/lon) | GPS 坐标专用 |

## 条件显示逻辑

- `mode="gpx"` → 显示 GPX Options section
- `mode="manual"` → 显示 Manual Coordinates section
- `source="local"` → 显示 model_path 输入
- `supports_expression=true` → 显示表达式编辑器
- `advanced=true` → 仅在 CollapsibleCard 展开时可见
