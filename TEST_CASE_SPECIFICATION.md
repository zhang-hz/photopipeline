# Photopipeline 全栈测试用例规格文档

**版本**: 2.0
**日期**: 2026-05-26
**原则**: 每条测试用例必须能 FAIL。禁止空桩、禁止静默跳过、禁止仅验证文件存在。
**总量**: ~1075 条 (Layer 0: 350 + Layer 1: 200 + Layer 2: 120 + Layer 3: 120 + Layer 4: 120 + Layer 5: 105 + Layer 6: 60)

---

## 目录

1. [测试工程铁律](#1-测试工程铁律)
2. [测试分层架构](#2-测试分层架构)
3. [测试输入图像矩阵](#3-测试输入图像矩阵)
4. [Layer 0: Rust 单元测试](#4-layer-0-rust-单元测试)
5. [Layer 1: Rust Pipeline 集成测试](#5-layer-1-rust-pipeline-集成测试)
6. [Layer 2: Rust gRPC E2E 测试](#6-layer-2-rust-grpc-e2e-测试)
7. [Layer 3: C# 单元测试](#7-layer-3-c-单元测试)
8. [Layer 4: C# gRPC 集成测试](#8-layer-4-c-grpc-集成测试)
9. [Layer 5: C# GUI FlaUI E2E 测试](#9-layer-5-c-gui-flaui-e2e-测试)
10. [Layer 6: Cross-Channel 交叉验证测试](#10-layer-6-cross-channel-交叉验证测试)
11. [共享测试用例 JSON 格式](#11-共享测试用例-json-格式)
12. [可复用基础设施](#12-可复用基础设施)
13. [需重写的垃圾代码清单](#13-需重写的垃圾代码清单)
14. [实施阶段](#14-实施阶段)
15. [验证标准与对抗性检查清单](#15-验证标准与对抗性检查清单)

---

## 1. 测试工程铁律

这六条铁律来自对历史垃圾测试代码的深度反思。**每写一条测试前必须大声朗读。**

| # | 铁律 | 反例 |
|---|------|------|
| 1 | 每个测试必须有至少一个能 FAIL 的断言 | 空方法体、仅 File.Exists |
| 2 | 禁止静默跳过 | try{catch{return;}} 吞噬异常 |
| 3 | 基础设施必须先有消费者 | 先建完美工具后"以后接线" |
| 4 | UI 测试必须真正启动进程 | 直接 new ViewModel 称"UI 测试" |
| 5 | 对抗性自查 | 不问"这个测试怎么被愚弄" |
| 6 | 回归测试必须有黄金参考图像 | 仅验证文件非空 |

---

## 2. 测试分层架构

```
Layer 0: Rust 单元测试 (~350 条) — cargo test --lib/--tests
  ├── 14 插件: ParameterSchema 验证、process/tile 边界
  ├── Engine: Graph/DAG、ParameterResolver、TileEngine、NodeExecutor
  └── gRPC service: 请求验证、错误传播、序列化往返

Layer 1: Rust Pipeline 集成测试 (~200 条) — 直接调用引擎，像素级验证
  ├── 单插件 × 多图像: 56 条
  ├── 单插件参数变异: 28 条
  ├── 多插件链: 90 条
  ├── 格式往返: 10 条
  └── 边界条件: 16 条

Layer 2: Rust gRPC E2E 测试 (~120 条) — 启动 server → gRPC 调用 → 像素验证
Layer 3: C# 单元测试 (~120 条) — 8 ViewModel + 7 Service → 状态机/命令/集成逻辑
Layer 4: C# gRPC 集成测试 (~120 条) — 与 Layer 2 共享用例定义，通过 C# gRPC 客户端执行
Layer 5: C# GUI FlaUI E2E 测试 (~105 条) — 真实 WPF 窗口操作 → 像素输出验证
Layer 6: Cross-Channel 交叉验证 (~60 条) — 多通道逐像素完全一致

总计: ~1075 条, GUI E2E 105 条 ≥ 100
```

### 2.1 端到端测试用例的一致性保证

Layer 2 (Rust gRPC)、Layer 4 (C# gRPC)、Layer 5 (C# GUI) 使用相同的测试用例定义。同一用例在不同通道下执行，产出必须**逐像素完全一致** (tolerancePerChannel=0)。

- Layer 2 和 Layer 4 共享 `shared/test_cases/` 下的 JSON 定义文件
- Layer 5 的每条 GUI E2E 用例有对应的 API 版本，用于 Layer 6 交叉验证
- Layer 6 的 60 条交叉验证用例是 Layer 2/4/5 共有用例的子集

---

## 3. 测试输入图像矩阵

共 20 张测试图像，由 `TestImageGenerator` (确定性种子 42) 生成，签入 `tests/fixtures/input/`。

| ID | 文件名 | 分辨率 | 色彩/位深 | 格式 | 内容描述 | 用途 |
|----|--------|--------|-----------|------|---------|------|
| I01 | solid_color_1920 | 1920×1080 | RGB U8 | PNG | 全彩场景 (天空+建筑+人物) | 通用验证、编码器基准 |
| I02 | adobergb_wide_1920 | 1920×1080 | AdobeRGB U8 | TIFF | 广色域风光照片 | 广色域转换验证 |
| I03 | web_photo_800 | 800×600 | sRGB U8 | JPEG | Web 尺寸照片 | Web 尺寸、有损格式源 |
| I04 | 4k_highres_3840 | 3840×2160 | sRGB U8 | PNG | 4K 高分辨率城市全景 | 高分辨率压力、瓦片引擎 |
| I05 | displayp3_wide_1920 | 1920×1080 | DisplayP3 U8 | PNG | P3 广色域日落场景 | P3 广色域、HDR 色调映射 |
| I06 | noisy_texture_1920 | 1920×1080 | sRGB U8 | PNG | ISO 6400 模拟噪声纹理 | AI 降噪验证 |
| I07 | barrel_distortion_1920 | 1920×1080 | sRGB U8 | PNG | 广角镜头桶形畸变网格 | 镜头校正 (桶形) |
| I08 | pincushion_vignette_1920 | 1920×1080 | sRGB U8 | PNG | 长焦镜头枕形畸变+暗角 | 复合光学校正 |
| I09 | grayscale_1024 | 1024×1024 | Gray U8 | PNG | 256 级灰阶渐变 | 灰度处理、单通道验证 |
| I10 | high_bitdepth_1920 | 1920×1080 | RGB U16 | TIFF | 16 位高位深渐变 | 高位深编解码、精度保持 |
| I11 | camera_jpeg_exif | 1920×1080 | sRGB U8 | JPEG | 相机直出 (完整 EXIF/GPS/XMP) | 元数据传递验证 |
| I12 | alpha_transparent_1024 | 1024×1024 | RGBA U8 | PNG | 棋盘格半透明覆盖 | Alpha 通道保持验证 |
| I13 | icon_tiny_256 | 256×256 | sRGB U8 | PNG | 小图标/缩略图 | 小尺寸、放大变换 |
| I14 | panorama_wide_8000 | 8000×4000 | sRGB U8 | PNG | 360° 全景图拼接 | 全景图、大尺寸瓦片 |
| I15 | cmyk_print_1920 | 1920×1080 | CMYK U8 | TIFF | 印刷用 CMYK 海报 | 印刷色域、CMYK 输入 |
| I16 | zone_plate_test_1920 | 1920×1080 | sRGB U8 | PNG | 正弦波带片 (圆频率递增) | 缩放滤波器/摩尔纹 |
| I17 | color_checker_1920 | 1920×1080 | sRGB U8 | PNG | 24 色 ColorChecker 标准色卡 | 色彩准确性基准 |
| I18 | gradient_all_1920 | 1920×1080 | RGB U8 | PNG | 水平+垂直+径向+对角渐变 | 色调曲线、LUT 映射 |
| I19 | single_pixel_1x1 | 1×1 | RGB U8 | PNG | 单个白色像素 | 极限最小尺寸 |
| I20 | extreme_aspect_100x65535 | 100×65535 | RGB U8 | PNG | 极端长宽比竖条 | 极端长宽比边界 |

---

## 4. Layer 0: Rust 单元测试

### 4.0 概述

- **总数**: ~350 条
- **执行**: `cargo test --workspace --lib`
- **验证方式**: `assert!` / `assert_eq!` / `should_panic`
- **原则**: 不涉及 I/O、不启动 server、每个测试 < 10ms

### 4.1 插件 ParameterSchema 验证 (140 条)

每个插件的 `PARAMETER_SCHEMA` 必须通过以下验证：

**通用验证 (14 插件 × 4 = 56 条):**

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-PS-001~014 | `{Plugin}_Schema_Is_NonEmpty` | 访问 `PLUGIN.schema()` | sections 非空 |
| U-PS-015~028 | `{Plugin}_Schema_All_Fields_Have_Defaults` | 遍历每个 field | field.default 非 null |
| U-PS-029~042 | `{Plugin}_Schema_Defaults_Match_Field_Types` | 检查 default 值的 JSON 类型 | 匹配 field_type (Int→Number, Enum→String...) |
| U-PS-043~056 | `{Plugin}_Schema_Enum_Options_Valid` | 遍历枚举字段的 options | 每个 option 有 value + label |

**参数类型专项验证 (14 插件 × 6 = 84 条):**

每个插件的关键参数测试：

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-PT-raw_input-01 | raw_input_RawMode_Enum_HasAllOptions | 验证 raw_mode 枚举 | 包含 auto/dcraw/libraw/rawtherapee |
| U-PT-raw_input-02 | raw_input_OutputFormat_Defaults_U16 | 验证 output_format 默认值 | default = "u16" |
| U-PT-transform-01 | transform_ScalePercent_Range | 验证 scale_percent 范围 | min=1, max=400 |
| U-PT-transform-02 | transform_Angle_Accepts_Negative | 验证 angle range | min=-360, max=360 |
| U-PT-transform-03 | transform_FilterType_Enum_Complete | 验证 filter_type 枚举 | bilinear/lanczos3/nearest |
| U-PT-colorspace-01 | colorspace_SourceSpace_8Options | 验证 source_color_space 枚举 | 8 个选项含 sRGB/AdobeRGB/ProPhoto/DisplayP3... |
| U-PT-colorspace-02 | colorspace_RenderingIntent_4Options | 验证 rendering_intent | perceptual/relative/absolute/saturation |
| U-PT-lut3d-01 | lut3d_LutFormat_4Formats | 验证 lut_format 枚举 | cube/3dl/look/csp |
| U-PT-lut3d-02 | lut3d_Intensity_Range | 验证 intensity 范围 | min=0, max=100 |
| U-PT-lens_correct-01 | lens_correct_Mode_3Options | 验证 correction_mode 枚举 | auto/manual/off |
| U-PT-ai_denoise-01 | ai_denoise_Strength_Range | 验证 denoise_strength | min=0, max=100 |
| U-PT-exif_rw-01 | exif_rw_WriteExif_Enum | 验证 write_exif 枚举 | preserve/custom/clear |
| U-PT-gps_set-01 | gps_set_Mode_3Options | 验证 gps_mode 枚举 | manual/gpx_track/clear |
| U-PT-gps_set-02 | gps_set_Latitude_Range | 验证 latitude 范围 | min=-90, max=90 |
| U-PT-time_shift-01 | time_shift_Hours_Range | 验证 shift_hours 范围 | min=-23, max=23 |
| U-PT-avif_encoder-01 | avif_Quality_Range | 验证 quality 范围 | min=0, max=100 |
| U-PT-avif_encoder-02 | avif_Chroma_Enum_3Options | 验证 chroma_subsampling | 444/422/420 |
| U-PT-jxl_encoder-01 | jxl_Quality_Range | 验证 quality 范围 | min=-1, max=100 |
| U-PT-jxl_encoder-02 | jxl_Effort_Range | 验证 effort 范围 | min=1, max=9 |
| U-PT-heif_encoder-01 | heif_Quality_Range | 验证 quality 范围 | min=0, max=100 |
| U-PT-tiff_encoder-01 | tiff_Compression_Enum | 验证 compression 枚举 | none/lzw/deflate/packbits |
| U-PT-png_encoder-01 | png_ColorType_Enum | 验证 color_type 枚举 | rgb/rgba/gray/graya |
| U-PT-png_encoder-02 | png_Compression_Range | 验证 compression_level | min=0, max=9 |

### 4.2 插件 process/tile 边界测试 (70 条)

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-PP-001~014 | `{Plugin}_Process_EmptyInput_ReturnsError` | 传入空 PixelBuffer | 返回 `Err(PluginError)` |
| U-PP-015~028 | `{Plugin}_Process_NullMetadata_NoCrash` | metadata=None | 不 panic，优雅处理 |
| U-PP-029~042 | `{Plugin}_Tile_SingleTile_SameAsProcess` | 单瓦片=全图 | tile 结果 = process 结果 |
| U-PP-043~056 | `{Plugin}_Tile_OutOfBounds_Clamped` | tile 坐标越界 | 被 clamp 到有效范围 |
| U-PP-057~070 | `{Plugin}_Process_Cancelled_MidExecution` | ProgressSink 触发 cancel | 返回 `Err(PluginError::Canceled)` |

### 4.3 Engine 核心单元测试 (105 条)

#### 4.3.1 Graph/DAG 拓扑排序 (30 条)

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-ENG-DAG-001 | Linear_3Nodes_CorrectOrder | A→B→C | 拓扑序 [A,B,C] |
| U-ENG-DAG-002 | Diamond_4Nodes_CorrectOrder | A→B, A→C, B→D, C→D | A 在 B/C 前, B/C 在 D 前 |
| U-ENG-DAG-003 | Branch_ThenMerge | A→B→C, A→D→C | 两个合法拓扑序之一 |
| U-ENG-DAG-004 | SingleNode_Trivial | 只有 A | 拓扑序 [A] |
| U-ENG-DAG-005 | EmptyGraph_EmptyResult | 空图 | 拓扑序 [] |
| U-ENG-DAG-006 | Disconnected_Components | A→B, C→D (无连接) | 两组互不干扰 |
| U-ENG-DAG-007 | Simple_Cycle_Detected | A→B→C→A | Err(CircularDependency) |
| U-ENG-DAG-008 | SelfLoop_Detected | A→A | Err(CircularDependency) |
| U-ENG-DAG-009 | Complex_Cycle_5Nodes | A→B→C→D→E→B | Err(CircularDependency) |
| U-ENG-DAG-010 | Cycle_With_Diamond | 钻石中有反向边 | Err(CircularDependency) |
| U-ENG-DAG-011 | Edge_With_MissingSource | 边引用不存在的 from | Err(InvalidEdge) |
| U-ENG-DAG-012 | Edge_With_MissingTarget | 边引用不存在的 to | Err(InvalidEdge) |
| U-ENG-DAG-013 | Duplicate_Edge | A→B 重复添加 | Err(DuplicateEdge) |
| U-ENG-DAG-014 | MaxNodes_1000Nodes_Linear | 1000 节点线性链 | 拓扑排序正确 (耗时 < 50ms) |
| U-ENG-DAG-015 | DeepDiamond_20Levels | 20 层钻石 | 拓扑序合法 |
| U-ENG-DAG-016 | WideFanOut_100Branches | A→B1..B100 | 所有 B 在 A 后 |
| U-ENG-DAG-017 | WideFanIn_100Sources | A1..A100→B | 所有 A 在 B 前 |
| U-ENG-DAG-018 | DisabledNode_Skipped_InOrder | A→B(disabled)→C | 拓扑序 [A,C] |
| U-ENG-DAG-019 | AllDisabled_EmptyOrder | 所有节点 disabled | 拓扑序 [] |
| U-ENG-DAG-020 | Mixed_EnabledDisabled_Diamond | 钻石中有 disabled 节点 | 只包含 enabled 节点的正确顺序 |

#### 4.3.2 ParameterResolver (30 条)

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-ENG-PR-001 | DefaultLayer_Wins_WhenNoOverride | 只设置默认值 | 输出 = 默认值 |
| U-ENG-PR-002 | TemplateLayer_Overrides_Default | 默认 x=1, 模板 x=2 | 输出 x=2 |
| U-ENG-PR-003 | GroupLayer_Overrides_Template | 模板 x=2, 组 x=3 | 输出 x=3 |
| U-ENG-PR-004 | ImageOverride_Wins_HighestPriority | 四层都设值 | 输出 = 图像覆盖层 |
| U-ENG-PR-005 | AllowOverride_False_BlocksHigherLayer | 默认 allow_override=false | 高层覆盖被忽略 |
| U-ENG-PR-006 | Expression_Variable_Substitution | `${exif.iso}` | 替换为实际 ISO 值 |
| U-ENG-PR-007 | Expression_Ternary_True | `${iso >= 400 ? 'high' : 'low'}`, iso=800 | 输出 "high" |
| U-ENG-PR-008 | Expression_Ternary_False | `${iso >= 400 ? 'high' : 'low'}`, iso=200 | 输出 "low" |
| U-ENG-PR-009 | Expression_Nested_Ternary | `${a ? (b ? 1 : 2) : 3}` | 正确求值嵌套三元 |
| U-ENG-PR-010 | Expression_Comparison_GT | `${width > 1000 ? 'large' : 'small'}` | 正确比较 |
| U-ENG-PR-011 | Expression_Arithmetic | `${exif.iso / 100}` | 正确算术运算 |
| U-ENG-PR-012 | Expression_UndefinedVariable_Error | `${nonexistent}` | 返回错误 (不 panic) |
| U-ENG-PR-013 | Expression_DivideByZero_Error | `${1 / 0}` | 返回错误 |
| U-ENG-PR-014 | Expression_Malformed_SyntaxError | `${???}` | 返回语法错误 |
| U-ENG-PR-015 | Expression_String_Concat | `${'IMG_' + exif.iso}` | 字符串拼接 |
| U-ENG-PR-016 | GroupCondition_ExifEq_Match | condition=ExifEq(make, "SONY") 且实际=Sony | 组覆盖被应用 |
| U-ENG-PR-017 | GroupCondition_ExifEq_NoMatch | condition=ExifEq(make, "SONY") 但实际=Canon | 组覆盖被跳过 |
| U-ENG-PR-018 | GroupCondition_GpsNear_Match | condition=GpsNear(39.9, 116.4, 100km) | 组覆盖被应用 |
| U-ENG-PR-019 | GroupCondition_GpsNear_NoMatch | condition=GpsNear(39.9, 116.4, 1km) 但 100km 外 | 组覆盖被跳过 |
| U-ENG-PR-020 | GroupCondition_And_BothTrue | And(ExifEq, GpsNear) 二者都满足 | 组覆盖被应用 |
| U-ENG-PR-021 | GroupCondition_Or_OneTrue | Or(ExifEq, GpsNear) 只有 ExifEq 满足 | 组覆盖被应用 |
| U-ENG-PR-022 | GroupCondition_ExifGte_Match | condition=ExifGte(iso, 800) | 组覆盖被应用 |
| U-ENG-PR-023 | Resolve_ReturnsAllParameters | 解析完整参数集 | 所有字段有值 |
| U-ENG-PR-024 | Resolve_RequiredField_Missing_Error | required=true 字段无任何层设值 | 返回 MissingRequiredField |
| U-ENG-PR-025 | Resolve_TypeValidation_IntField | 类型为 Int 的字段收到 String 值 | 类型错误 |

#### 4.3.3 TileEngine (25 条)

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-ENG-TE-001 | SingleTile_EqualsFullImage | 图像 ≤ 1024px | 单瓦片 = 全图 |
| U-ENG-TE-002 | TwoTiles_Horizontal | 2048×1024, tile=1024 | 左+右拼回 = 原图 |
| U-ENG-TE-003 | TwoTiles_Vertical | 1024×2048, tile=1024 | 上+下拼回 = 原图 |
| U-ENG-TE-004 | FourTiles_2x2Grid | 2048×2048, tile=1024 | 4 块拼回 = 原图 |
| U-ENG-TE-005 | Overlap_Region_Blended | 2048×1024, overlap=64 | 重叠区正确混合 |
| U-ENG-TE-006 | TileSize_LargerThanImage | tile > 图像尺寸 | 单瓦片处理 |
| U-ENG-TE-007 | NonDivisible_TileSize | 图像 1500px, tile=1024 | 最后一行/列正确裁剪 |
| U-ENG-TE-008 | Boundary_Pixels_NoArtifacts | 瓦片边界逐像素对比 | 无接缝伪影 |
| U-ENG-TE-009 | EmptyImage_0x0 | 0×0 图像 | 返回空或错误 |
| U-ENG-TE-010 | SingleRow_Image | 1×1024 | 正确处理单行 |
| U-ENG-TE-011 | SingleColumn_Image | 1024×1 | 正确处理单列 |
| U-ENG-TE-012 | TileCount_MaxResolution | 8000×4000, tile=1024 | 瓦片数 = ceil(8)*ceil(4) = 32 |
| U-ENG-TE-013 | Cancel_MidTile_StopsProcessing | 第 3 个瓦片 cancel | 返回 Canceled, 仅处理 2 个瓦片 |
| U-ENG-TE-014 | Progress_Sink_Reports_CorrectFraction | 4 瓦片 | Progress 报告 0.25, 0.50, 0.75, 1.00 |

#### 4.3.4 NodeExecutor (20 条)

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-ENG-NE-001 | SingleNode_ExecutesAndReturns | A → 执行 | 输出正确, NodeState=Completed |
| U-ENG-NE-002 | Linear_2Nodes_Sequential | A→B | 两个都执行, 顺序正确 |
| U-ENG-NE-003 | Node_Failure_Propagates | A(失败)→B | A=Failed, B=NotStarted |
| U-ENG-NE-004 | Node_Cancel_Propagates | A→B(取消)→C | B=Canceled, C=NotStarted |
| U-ENG-NE-005 | DisabledNode_Skipped | A→B(disabled)→C | B=Skipped, A→C 直接 |
| U-ENG-NE-006 | AllNodesDisabled_Completes | 全 disabled | 无错误, 无输出 |
| U-ENG-NE-007 | Concurrent_Branches | A→B, A→C (并行) | B 和 C 可并行执行 |
| U-ENG-NE-008 | Diamond_Merge_AfterBranches | A→B, A→C, B→D, C→D | D 在 B 和 C 都完成后执行 |
| U-ENG-NE-009 | Progress_Aggregation | 多节点 | 总进度跨所有节点聚合 |
| U-ENG-NE-010 | Metadata_Passthrough | 处理节点→编码节点 | 元数据经管线传递 |

### 4.4 gRPC Service 验证 (35 条)

| ID | 测试名称 | 流程 | 预期结果 |
|----|---------|------|---------|
| U-GRPC-001~010 | `{Service}_Request_Serialization_Roundtrip` | serialize→deserialize | 字段值不变 (pipeline/image/batch 3 服务 × 主要方法) |
| U-GRPC-011 | PipelineCreate_EmptyNodes_Error | nodes=[] 的 PipelineSpec | 返回 InvalidArgument |
| U-GRPC-012 | PipelineCreate_InvalidPluginId | 引用不存在的 plugin_id | 返回 NotFound 或 InvalidArgument |
| U-GRPC-013 | PipelineExecute_InvalidPipelineId | 不存在的 pipeline_id | 返回 NotFound |
| U-GRPC-014 | PipelineExecute_MissingInputPath | input="" | 返回 InvalidArgument |
| U-GRPC-015 | ImageDecode_UnsupportedFormat | 不支持的文件格式 | 返回 InvalidArgument |
| U-GRPC-016 | ImageEncode_InvalidOptions | 无效编码参数 | 返回 InvalidArgument |
| U-GRPC-017 | BatchCreate_NoFiles | files=[] | 返回 InvalidArgument |
| U-GRPC-018 | BatchCreate_MissingPipeline | pipeline_config_path="" | 返回 InvalidArgument |
| U-GRPC-019 | AllServices_Unauthenticated | 无认证凭证 (如果启用) | 返回 Unauthenticated |
| U-GRPC-020 | ConcurrentRequests_NoDataRace | 8 线程同时调用 CreatePipeline | 所有成功，无数据竞争 |

---

## 5. Layer 1: Rust Pipeline 集成测试

### 5.0 概述

- **总数**: ~200 条
- **执行**: `cargo test --test pipeline_integration`
- **基础设施**: 复用 `tests/test_harness/src/` 的 assertions/fixtures
- **每条用例流程**: 构建 PipelineTemplate → 加载输入图像 → NodeExecutor 执行 → 保存输出 → 像素验证
- **每测试必须**: 读取输出文件, 逐像素验证 (PixelsEqual/PSNR/SSIM/格式)

### 5.1 单插件 × 多图像 (56 条)

每插件 × 4 种代表性输入图像:

| ID | 插件 | 图像 | 参数 | 验证方法 | 如何 FAIL |
|----|------|------|------|---------|----------|
| IT-P01-001 | raw_input | I01 | raw_mode=auto | assert_pixels_eq(output, golden) | 与 golden 像素不同 |
| IT-P01-002 | raw_input | I02 | raw_mode=dcraw, manual_wb=5500K | assert_buffer_dimensions(1920,1080) | 尺寸错误 |
| IT-P01-003 | raw_input | I10 | output_format=u16 | assert_pixel_format(u16) | 位深不对 |
| IT-P01-004 | raw_input | I11 | raw_mode=libraw | assert_valid_tiff(output) | TIFF 结构无效 |
| IT-P02-001 | transform | I01 | crop 50% center | assert_buffer_dimensions(960,540) | 裁剪尺寸不对 |
| IT-P02-002 | transform | I03 | resize 200% lanczos3 | assert_buffer_dimensions(1600,1200) | 缩放尺寸不对 |
| IT-P02-003 | transform | I04 | rotate 90° | assert_buffer_dimensions(2160,3840) | 旋转后尺寸不对 |
| IT-P02-004 | transform | I13 | flip H+V | assert_pixels_eq(output, expected_flip) | 翻转结果不对 |
| IT-P03-001 | colorspace | I01 | sRGB→AdobeRGB, embed_icc=true | asset_icc_profile_present(output) | ICC 缺失 |
| IT-P03-002 | colorspace | I05 | sRGB→DisplayP3 | compute_delta_e(output, golden) < 2 | 色差过大 |
| IT-P03-003 | colorspace | I03 | sRGB→Gray, bp_comp=true | assert_channel_count(1) | 非单通道 |
| IT-P03-004 | colorspace | I09 | Gray→sRGB | assert_channel_count(3), R=G=B | 三通道不相等 |
| IT-P04-001 | lut3d | I01 | warm.cube, intensity=80 | assert_pixels_eq(output, golden_warm) | LUT 未生效 |
| IT-P04-002 | lut3d | I03 | film.cube, interp=tetrahedral | compute_ssim(output, golden) > 0.98 | SSIM 太低 |
| IT-P04-003 | lut3d | I06 | extreme.cube, clamp=true | assert_pixel_range(0, 255) | 像素越界 |
| IT-P04-004 | lut3d | I18 | cool.cube, intensity=50 | compute_psnr(output, input) < 50 | 变换幅度不对 |
| IT-P05-001 | lens_correct | I07 | correction_mode=auto | assert_pixels_eq(output, golden_corrected) | 校正结果不对 |
| IT-P05-002 | lens_correct | I08 | distortion+vignette full | compute_psnr(output, input) > 40 | 校正幅度不足 |
| IT-P05-003 | lens_correct | I07 | TCA only | compute_mae(output, input) | CA 校正无效 |
| IT-P05-004 | lens_correct | I08 | manual lensfun_db=custom | 不 panic, 正常执行 | panic |
| IT-P06-001 | ai_denoise | I06 | strength=20, detail=80 | compute_psnr(output, input) > input_psnr | PSNR 未改善 |
| IT-P06-002 | ai_denoise | I06 | strength=50, detail=50 | compute_ssim(output, input) > 0.8 | SSIM 过低 |
| IT-P06-003 | ai_denoise | I06 | strength=90, color_noise=true | compute_entropy(output) < compute_entropy(input) | 熵未降低 |
| IT-P06-004 | ai_denoise | I04 | strength=30, 4K input | assert_buffer_dimensions(3840, 2160) | 尺寸变化 |
| IT-P07-001 | exif_rw | I11 | read_all=true, overwrite=true | assert_exif_match(output, input) | EXIF 丢失 |
| IT-P07-002 | exif_rw | I01 | write custom tags | assert_exif_tag(output, "Artist", "Test") | 标签未写入 |
| IT-P07-003 | exif_rw | I11 | clear_existing=true | assert_no_exif(output) | EXIF 未清除 |
| IT-P07-004 | exif_rw | I11 | read_xmp=true, read_iptc=true | assert_xmp_present(output) | XMP 丢失 |
| IT-P08-001 | gps_set | I01 | manual 39.9042,116.4074 | assert_gps_coords(output, 39.9042, 116.4074) | GPS 坐标不对 |
| IT-P08-002 | gps_set | I11 | mode=clear | assert_no_gps(output) | GPS 未清除 |
| IT-P08-003 | gps_set | I11 | gpx_track=test.gpx | assert_gps_near(output, gpx_point) | GPX 插值不对 |
| IT-P08-004 | gps_set | I01 | mode=manual, altitude=100 | assert_gps_altitude(output, 100) | 海拔不对 |
| IT-P09-001 | time_shift | I11 | shift_hours=+1 | assert_exif_time_offset(output, +1h) | 时间偏移不对 |
| IT-P09-002 | time_shift | I11 | shift_hours=-24 | assert_exif_date_offset(output, -1d) | 日期偏移不对 |
| IT-P09-003 | time_shift | I11 | source_tz=UTC, target=Asia/Shanghai | assert_exif_timezone(output, +8h) | 时区不对 |
| IT-P09-004 | time_shift | I11 | shift_minutes=+30 | assert_exif_time_offset(output, +30m) | 分钟偏移不对 |
| IT-P10-001 | avif_encoder | I01 | Q=50, speed=5 | assert_valid_avif(output) | AVIF 容器无效 |
| IT-P10-002 | avif_encoder | I01 | Q=100, 10bit, chroma=444 | compute_psnr(output, decode(output)) > 45 | 编码质量差 |
| IT-P10-003 | avif_encoder | I01 | lossless=true | assert_pixels_eq(input, decode(output), 0) | 无损≠逐像素 |
| IT-P10-004 | avif_encoder | I05 | Q=75, 8bit | assert_valid_avif(output) | 格式错误 |
| IT-P11-001 | jxl_encoder | I01 | Q=50, effort=5 | assert_valid_jxl(output) | JXL 容器无效 |
| IT-P11-002 | jxl_encoder | I01 | lossless=true, effort=9 | assert_pixels_eq(input, decode(output), 0) | 无损≠逐像素 |
| IT-P11-003 | jxl_encoder | I10 | Q=100, bit_depth=16 | assert_pixel_format(decode(output), u16) | 位深降级 |
| IT-P11-004 | jxl_encoder | I14 | Q=80, effort=3, 8000×4000 | assert_buffer_dimensions(8000, 4000) | 尺寸错误 |
| IT-P12-001 | heif_encoder | I01 | Q=80 | assert_valid_heif(output) | HEIF 容器无效 |
| IT-P12-002 | heif_encoder | I01 | Q=100, bit_depth=10 | compute_psnr(output, decode(output)) > 48 | 编码质量差 |
| IT-P12-003 | heif_encoder | I01 | chroma=444 | 无 chroma 子采样伪影 | sub采样不对 |
| IT-P13-001 | tiff_encoder | I01 | compression=deflate, u8 | assert_valid_tiff(output) + assert_pixels_eq | TIFF 无效 |
| IT-P13-002 | tiff_encoder | I10 | compression=zip, u16 | assert_pixel_format(u16) | 位深降级 |
| IT-P13-003 | tiff_encoder | I04 | compression=lzw, 4K | assert_buffer_dimensions(3840, 2160) | 尺寸错误 |
| IT-P14-001 | png_encoder | I01 | color_type=rgb, compression=6 | assert_valid_png(output) | PNG 无效 |
| IT-P14-002 | png_encoder | I10 | bit_depth=16, color_type=rgb | assert_pixel_format(u16) | 位深降级 |
| IT-P14-003 | png_encoder | I12 | color_type=rgba | assert_channel_count(4) + alpha 通道完整 | alpha 丢失 |

### 5.2 单插件参数变异 (28 条)

| ID | 插件 | 参数变异 | 图像 | 验证 |
|----|------|---------|------|------|
| IT-PV-001 | transform | resize 50% vs 200% | I01 | 输出大小不同, 比例正确 |
| IT-PV-002 | transform | rotate 45° vs 180° | I01 | 旋转角度不同的输出不同 |
| IT-PV-003 | colorspace | 6 种 rendering_intent | I17 | 每种 intent 产生不同输出 |
| IT-PV-004 | colorspace | bp_comp on vs off | I05 | 不同黑点补偿结果 |
| IT-PV-005 | lut3d | intensity 0 vs 50 vs 100 | I01 | 渐变强度变化 |
| IT-PV-006 | lut3d | trilinear vs tetrahedral | I03 | 插值方法影响质量 |
| IT-PV-007 | lens_correct | auto vs manual vs off | I07 | off 时无校正, auto 自动检测 |
| IT-PV-008 | ai_denoise | strength 0 vs 50 vs 100 | I06 | 降噪强度递增 |
| IT-PV-009 | avif_encoder | chroma 444 vs 420 vs 422 | I01 | sub采样影响输出 |
| IT-PV-010 | jxl_encoder | modular true vs false | I01 | modular 模式输出不同 |
| IT-PV-011 | tiff_encoder | 4 种 compression mode | I01 | 不同压缩算法 |
| IT-PV-012 | png_encoder | compression 0 vs 9 | I01 | 压缩级别不影响像素 |
| IT-PV-013 | png_encoder | 4 种 color_type | I09 | 通道数正确 |
| IT-PV-014 | heif_encoder | quality 10 vs 50 vs 90 | I01 | 质量影响文件大小 |

### 5.3 多插件管道链 (90 条)

#### 两插件链 (40 条, 20 组合 × 2 图像):

| ID | 管线 | 图像 | 验证 |
|----|------|------|------|
| IT-M2-001 | raw_input→colorspace | I01, I03 | 每个中间节点输出都验证 |
| IT-M2-002 | raw_input→transform(crop) | I01, I03 | 裁剪后尺寸正确 |
| IT-M2-003 | raw_input→lut3d | I01, I03 | LUT 应用后色彩变化 |
| IT-M2-004 | raw_input→tiff_encoder | I10, I04 | TIFF 格式+像素 |
| IT-M2-005 | transform(rotate)→colorspace | I01, I04 | rotate 后色彩空间转换 |
| IT-M2-006 | transform(resize)→png_encoder | I01, I13 | 缩放后 PNG 输出 |
| IT-M2-007 | colorspace→tiff_encoder | I01, I05 | TIFF 含 ICC profile |
| IT-M2-008 | colorspace→avif_encoder | I01, I05 | AVIF 含色彩空间 |
| IT-M2-009 | lut3d→png_encoder | I01, I03 | PNG 保留 LUT 效果 |
| IT-M2-010 | lens_correct→tiff_encoder | I07, I08 | TIFF 含校正后图像 |
| IT-M2-011 | ai_denoise→jxl_encoder | I06, I04 | JXL 压缩降噪后图像 |
| IT-M2-012 | exif_rw→tiff_encoder | I11, I01 | TIFF 保留元数据 |
| IT-M2-013 | gps_set→png_encoder | I01, I11 | PNG 的 GPS 数据传递 |
| IT-M2-014 | time_shift→tiff_encoder | I11, I01 | TIFF 含时间偏移后 EXIF |
| IT-M2-015 | colorspace→jxl_encoder(lossless) | I01, I02 | lossless JXL 像素完整 |
| IT-M2-016 | ai_denoise→colorspace | I06, I06 | 降噪后色彩空间转换 |
| IT-M2-017 | transform(crop+resize)→avif_encoder | I01, I04 | 复合变换后 AVIF |
| IT-M2-018 | lens_correct→colorspace | I07, I08 | 校正后色彩转换 |
| IT-M2-019 | colorspace→lut3d | I01, I03 | 先转色彩再套 LUT |
| IT-M2-020 | raw_input→heif_encoder | I10, I11 | RAW→HEIF 直出 |

#### 三插件链 (30 条, 15 组合 × 2 图像):

| ID | 管线 | 验证 |
|----|------|------|
| IT-M3-001 | transform→colorspace→tiff_encoder | 全链像素验证 |
| IT-M3-002 | raw_input→colorspace→png_encoder | RAW 显影到 PNG |
| IT-M3-003 | ai_denoise→colorspace→jxl_encoder | 降噪→色彩→JXL |
| IT-M3-004 | lens_correct→colorspace→tiff_encoder | 校正→色彩→TIFF |
| IT-M3-005 | raw_input→transform→avif_encoder | 裁剪后导出 |
| IT-M3-006 | exif_rw→colorspace→tiff_encoder | 元数据经全链传递 |
| IT-M3-007 | colorspace→lut3d→png_encoder | LUT 效果持久化 |
| IT-M3-008 | raw_input→ai_denoise→tiff_encoder | 降噪后归档 |
| IT-M3-009 | transform→lut3d→jxl_encoder | 缩放+LUT+JXL |
| IT-M3-010 | gps_set→time_shift→tiff_encoder | 元数据链加工 |
| IT-M3-011 | colorspace→transform(crop)→avif_encoder | 色彩→裁剪→导出 |
| IT-M3-012 | lens_correct→ai_denoise→png_encoder | 光学→降噪→PNG |
| IT-M3-013 | raw_input→lens_correct→heif_encoder | RAW 光学校正后 HEIF |
| IT-M3-014 | transform(resize)→colorspace→jxl_encoder | 缩放→色彩→无损 |
| IT-M3-015 | ai_denoise→lut3d→tiff_encoder | 降噪+LUT+归档 |

#### 四+插件链 (20 条, 10 组合 × 2 图像):

| ID | 管线 | 验证 |
|----|------|------|
| IT-M4-001 | raw_input→lens_correct→colorspace→tiff_encoder | 全 RAW 工作流 |
| IT-M4-002 | raw_input→ai_denoise→colorspace→lut3d→png_encoder | 5 节点降噪→色彩→LUT |
| IT-M4-003 | transform→colorspace→lut3d→jxl_encoder(lossless) | 变换→色彩→LUT→压缩 |
| IT-M4-004 | raw_input→colorspace→transform(crop)→avif_encoder | 数码处理链 |
| IT-M4-005 | ai_denoise→lens_correct→colorspace→tiff_encoder | 修复→校正→色彩→输出 |
| IT-M4-006 | exif_rw→gps_set→time_shift→colorspace→tiff_encoder | 5 节点全元数据处理 |
| IT-M4-007 | raw_input→lens_correct→colorspace→lut3d→jxl_encoder | 专业 RAW 后期 |
| IT-M4-008 | transform(rotate→crop→resize)→colorspace→png_encoder | 多重变换→PNG |
| IT-M4-009 | ai_denoise→colorspace→transform(resize)→lut3d→avif_encoder | 5 节点降噪+发布 |
| IT-M4-010 | raw_input→lens_correct→ai_denoise→colorspace→tiff_encoder(16bit) | 5 节点全流程 16bit |

### 5.4 格式往返测试 (10 条)

| ID | 流程 | 验证 |
|----|------|------|
| IT-FR-001 | PNG→decode→PNG encode | 像素完全相同 |
| IT-FR-002 | PNG→decode→TIFF encode→decode→PNG encode | 像素完全相同 |
| IT-FR-003 | TIFF→decode→TIFF encode | 像素完全相同 |
| IT-FR-004 | JPEG→decode→PNG encode | 跳过有损 |
| IT-FR-005 | PNG→decode→AVIF(lossless)→decode→PNG encode | 像素完全相同 |
| IT-FR-006 | PNG→decode→JXL(lossless)→decode→PNG encode | 像素完全相同 |
| IT-FR-007 | 16bit TIFF→decode→16bit PNG encode | 位深保持 |
| IT-FR-008 | RGBA PNG→decode→TIFF encode→decode | alpha 通道保持 |
| IT-FR-009 | Gray PNG→decode→RGB→decode→Gray PNG | 灰度数值往返保真 |
| IT-FR-010 | CMYK TIFF→decode→sRGB PNG encode | 色彩转换正确 |

### 5.5 边界条件 (16 条)

| ID | 场景 | 流程 | 预期 |
|----|------|------|------|
| IT-BD-001 | 空管线 | nodes=[] 执行 | 错误/无输出 |
| IT-BD-002 | 单像素图像 | I19 经 transform(resize 200%) | 输出 2×2 |
| IT-BD-003 | 极端长宽比 | I20 经 colorspace | 输出 100×65535 |
| IT-BD-004 | 最大管道: 100 节点 | 线性链 A1→A2→...→A100 | 全部完成, 最后节点输出正确 |
| IT-BD-005 | 巨型参数: LUT 64³ | 大型 Cube LUT 文件 | 内存可控, 不 OOM |
| IT-BD-006 | 并行执行: 4 并发管线 | 4 条不同管线同时执行 | 全部正确完成 |
| IT-BD-007 | 并行执行: 8 并发管线 | 8 条管线 | 全部正确, 无数据竞争 |
| IT-BD-008 | 超大图像: 8000×4000 | I14 完整处理 | 瓦片正确拼接 |
| IT-BD-009 | 全 disabled 管线 | 所有节点 disabled | 输入直通输出 |
| IT-BD-010 | 无编码器管线 | 只有 transform 节点 | 输出 = 最后处理节点的 buffer |
| IT-BD-011 | 混合 pixel+metadata 处理器 | raw_input→exif_rw→colorspace | 两类处理器共存 |
| IT-BD-012 | 取消中间节点 | 5 节点链, 在第 3 个 cancel | 前 2 完成, 后 2 未启动 |
| IT-BD-013 | 钻石节点 disabled | A→B(disabled)→C, A→D→C | D 路径正常, B 跳过 |
| IT-BD-014 | 多次执行同一管线 | 同 pipeline 执行 10 次 | 每次结果相同 (确定性) |
| IT-BD-015 | 不同图像执行同管线 | I01 和 I03 都执行同 pipeline | 各自正确 |
| IT-BD-016 | 参数值边界: 最大/最小值 | 每个 Int/Float 参数设 min 和 max | 正确执行, 不 crash |

---

## 6. Layer 2: Rust gRPC E2E 测试

### 6.0 概述

- **总数**: ~120 条
- **执行**: 启动 tonic TestServer (随机端口) → gRPC client → 管线执行 → 像素验证
- **每测试流程**: start_server → connect → create_pipeline → execute → decode_output → assert_pixels

### 6.1 单插件 gRPC (40 条)

与 Layer 4 共享 JSON 定义。每插件 2-4 条代表性用例，通过 gRPC 协议完整执行。

| ID | 类别 | 数量 | 说明 |
|----|------|------|------|
| GRPC-P01~P04 | raw_input | 4 | auto/dcraw/libraw + 2 种输出格式 |
| GRPC-P05~P08 | transform | 4 | crop/resize/rotate/flip |
| GRPC-P09~P12 | colorspace | 4 | sRGB→AdobeRGB/sRGB→Gray/Gray→RGB/sRGB→P3 |
| GRPC-P13~P15 | lut3d | 3 | warm/cool/film LUT |
| GRPC-P16~P18 | lens_correct | 3 | barrel/pincushion/auto |
| GRPC-P19~P21 | ai_denoise | 3 | light/medium/heavy |
| GRPC-P22~P24 | exif_rw | 3 | preserve/write/clear |
| GRPC-P25~P27 | gps_set | 3 | manual/gpx/clear |
| GRPC-P28~P30 | time_shift | 3 | +1h/-24h/timezone |
| GRPC-P31~P33 | avif_encoder | 3 | Q50/Q100/lossless |
| GRPC-P34~P36 | jxl_encoder | 3 | Q50/Q100/lossless |
| GRPC-P37~P38 | heif_encoder | 2 | Q50/Q100 |
| GRPC-P39~P41 | tiff_encoder | 3 | deflate/zip/lzw |
| GRPC-P42~P44 | png_encoder | 3 | rgb8/rgb16/rgba |

### 6.2 多插件 gRPC (30 条)

| ID | 管线 | 节点数 |
|----|------|--------|
| GRPC-M01~M05 | raw_input→colorspace→encoder (5 编码器各一) | 3 |
| GRPC-M06~M10 | transform→colorspace→encoder (5 编码器各一) | 3 |
| GRPC-M11~M15 | raw_input→transform→colorspace→encoder | 4 |
| GRPC-M16~M20 | ai_denoise→colorspace→encoder | 3 |
| GRPC-M21~M25 | lens_correct→colorspace→lut3d→encoder | 4 |
| GRPC-M26~M30 | raw_input→lens_correct→ai_denoise→colorspace→encoder | 5 |

### 6.3 格式 gRPC (15 条)

| ID | 流程 | 验证 |
|----|------|------|
| GRPC-F01~F05 | {input_fmt}→decode→pipeline→{output_fmt} encode | cross-format 像素保真 |
| GRPC-F06~F10 | {input_fmt}→decode→pipeline→{output_fmt} encode | 格式头验证 |
| GRPC-F11~F15 | 位深转换: 8→16, 16→8, 8→10, 16→32 | 位深正确 |

### 6.4 批处理 gRPC (15 条)

| ID | 场景 | 验证 |
|----|------|------|
| GRPC-B01~B05 | 1/2/5/10/20 文件批处理 | 全部成功 |
| GRPC-B06~B08 | 暂停/恢复/取消 | 状态正确 |
| GRPC-B09~B10 | 混合格式输入 | 各格式独立正确 |
| GRPC-B11~B12 | 部分失败: 混合有效+损坏文件 | 部分成功统计正确 |
| GRPC-B13~B15 | 大文件 (4K/8K/16bit) 批处理 | 内存不泄露 |

### 6.5 错误路径 gRPC (10 条)

| ID | 场景 | 预期 gRPC Status |
|----|------|-----------------|
| GRPC-E01 | 无效 pipeline ID | NotFound |
| GRPC-E02 | 不存在的输入文件路径 | InvalidArgument |
| GRPC-E03 | 损坏的图像文件 | InvalidArgument |
| GRPC-E04 | 不支持的插件 ID | InvalidArgument |
| GRPC-E05 | 空管线 (无节点) | InvalidArgument |
| GRPC-E06 | 管线含循环依赖 | InvalidArgument |
| GRPC-E07 | 请求超时 | DeadlineExceeded |
| GRPC-E08 | Cancel 正在执行的管线 | Cancelled |
| GRPC-E09 | 缺失必填参数 | InvalidArgument |
| GRPC-E10 | 服务器内部错误 (模拟) | Internal |

### 6.6 性能/并发 gRPC (10 条)

| ID | 场景 | 验证 |
|----|------|------|
| GRPC-PF01 | 4 并发请求, 不同管线 | 全部成功, 无交叉污染 |
| GRPC-PF02 | 8 并发请求, 相同管线 | 全部正确, 输出一致 |
| GRPC-PF03 | 16 并发请求, 混合管线 | 全部成功, 无超时 |
| GRPC-PF04 | 100 节点管线, 单请求 | 正确完成 (验证无栈溢出) |
| GRPC-PF05 | 串行 100 次请求 | 全部成功 (验证无内存泄漏) |
---

## 7. Layer 3: C# 单元测试

### 7.0 概述

- **总数**: ~120 条
- **执行**: `dotnet test --filter "Category=Unit"`
- **框架**: xUnit + Moq + FluentAssertions

### 7.1 MainViewModel (15 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-MVM-001 | Mock IServiceProvider | NavigateCommand("Filmstrip") | CurrentView = FilmstripView |
| CS-U-MVM-002 | Mock IServiceProvider | NavigateCommand("PipelineEditor") | CurrentView = PipelineEditorView |
| CS-U-MVM-003 | Mock IServiceProvider | NavigateCommand("Batch") | CurrentView = BatchView |
| CS-U-MVM-004 | Mock IDialogService | NavigateCommand("Settings") | ShowDialog<SettingsDialog>() called |
| CS-U-MVM-005 | PluginBrowser raises PluginAdded | event handler | PipelineEditor.AddNodeAt called |
| CS-U-MVM-006 | Check construction | — | PluginAdded not subscribed before loaded |
| CS-U-MVM-007 | Mock ISettingsService | ToggleThemeCommand() | ApplicationThemeManager.Apply called |
| CS-U-MVM-008 | Current theme=Dark | ToggleThemeCommand() | theme = Light |
| CS-U-MVM-009 | Mock BackendService.IsConnected=true | CheckConnection | BackendStatus="Connected" |
| CS-U-MVM-010 | Mock BackendService.IsConnected=false | CheckConnection | BackendStatus="Disconnected" |
| CS-U-MVM-011 | Filmstrip.SelectedImage changed | — | PreviewVM.CurrentImage updated |
| CS-U-MVM-012 | PipelineEditor.PipelineId changed | — | BatchVM.PipelineConfigPath updated |
| CS-U-MVM-013 | All child VMs initialized | Dispose() | All child Dispose() called |
| CS-U-MVM-014 | Child VM throws exception | operation triggers error | Snackbar shows error |
| CS-U-MVM-015 | New MainViewModel | — | All ViewModel properties non-null |

### 7.2 FilmstripViewModel (15 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-FS-001 | File.Exists=true | ImportCommand(path) | Images.Count += 1 |
| CS-U-FS-002 | Image already in collection | ImportCommand(samePath) | Images.Count unchanged |
| CS-U-FS-003 | File.Exists=false | ImportCommand(badPath) | ErrorMessage non-null |
| CS-U-FS-004 | 1 image selected | RemoveCommand() | Images.Count -= 1 |
| CS-U-FS-005 | 3 images selected | RemoveCommand() | Images.Count -= 3 |
| CS-U-FS-006 | No selection | RemoveCommand() | Images.Count unchanged |
| CS-U-FS-007 | 5 images loaded | SelectAllCommand() | SelectedImages.Count = 5 |
| CS-U-FS-008 | 3 images selected | ClearSelectionCommand() | SelectedImages.Count = 0 |
| CS-U-FS-009 | 2 of 5 selected | InvertSelectionCommand() | Other 3 selected |
| CS-U-FS-010 | Images B,A,C loaded | Sort("Name") | Order = A,B,C |
| CS-U-FS-011 | Images with EXIF dates | Sort("Date") | Ordered by EXIF date |
| CS-U-FS-012 | Images sizes 1,3,2 MB | Sort("Size", desc) | Order = 3,2,1 MB |
| CS-U-FS-013 | Mixed PNG+JPEG | FilterText=".png" | FilteredImages only PNG |
| CS-U-FS-014 | Various date ranges | FilterDate set | FilteredImages in range |
| CS-U-FS-015 | SelectedImage=null | CopyPathCommand() | No exception thrown |

### 7.3 PreviewViewModel (12 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-PV-001 | Zoom=1.0 | ZoomInCommand() | Zoom > 1.0, <= max |
| CS-U-PV-002 | Zoom=2.0 | ZoomOutCommand() | Zoom < 2.0, >= min |
| CS-U-PV-003 | Image 3840x2160, viewport 1920x1080 | FitCommand() | Zoom fits viewport |
| CS-U-PV-004 | Zoom=0.5 | Zoom100Command() | Zoom = 1.0 |
| CS-U-PV-005 | Zoom=2.0 | Pan(dx, dy) | Offset updated |
| CS-U-PV-006 | Mock IImageService | Export(path) | imageService.SaveAsync called |
| CS-U-PV-007 | SKBitmap.ColorType=Bgra1010102 | Export(path) | PixelFormat = "U16" |
| CS-U-PV-008 | SKBitmap.ColorType=Bgra8888 | Export(path) | PixelFormat = "U8" |
| CS-U-PV-009 | Known pixel data | ComputeHistogram() | HistogramBins correct |
| CS-U-PV-010 | Zoom=3.0, Offset=(100,100) | CurrentImage=new | Zoom reset, Offset=(0,0) |
| CS-U-PV-011 | SKBitmap non-null | Dispose() | Bitmap reference released |
| CS-U-PV-012 | CurrentImage=null | — | IsEmpty=true |

### 7.4 PipelineEditorViewModel (18 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-PE-001 | Empty pipeline | AddNode(plugin, x, y) | Nodes.Count=1, Node.X=x, Node.Y=y |
| CS-U-PE-002 | Has raw_input node | AddNode(raw_input) | Nodes.Count=2 (allows duplicate) |
| CS-U-PE-003 | A→B→C, select B | RemoveNodeCommand(B) | B removed, edges reconnected |
| CS-U-PE-004 | Only node A exists | RemoveNodeCommand(A) | Nodes=0, Edges=0 |
| CS-U-PE-005 | A, B unconnected | Connect(A, B) | Edges contains A→B |
| CS-U-PE-006 | A→B→C, attempting C→A | Connect(C, A) | Edge rejected (cycle) |
| CS-U-PE-007 | A exists alone | Connect(A, A) | Self-loop rejected |
| CS-U-PE-008 | A→B exists | Connect(A, B) | Duplicate edge rejected |
| CS-U-PE-009 | Valid A→B pipeline | RunCommand() | pipelineService.ExecuteAsync called |
| CS-U-PE-010 | Empty pipeline | RunCommand() | ErrorMessage non-null |
| CS-U-PE-011 | A→B, SelectedImage=null | RunCommand() | ErrorMessage "no image" |
| CS-U-PE-012 | Pipeline executing | CancelCommand() | IsRunning=false, progress=0 |
| CS-U-PE-013 | Pipeline A→B→C exists | NewPipelineCommand() | Nodes=0, Edges=0, PipelineId=null |
| CS-U-PE-014 | New pipeline A→B | SaveCommand() | pipelineService.CreatePipelineAsync called |
| CS-U-PE-015 | A(enabled)→B | Toggle(A) | A.Enabled=false |
| CS-U-PE-016 | A(disabled) | Toggle(A) | A.Enabled=true |
| CS-U-PE-017 | During execution | Progress callback | Progress property updated |
| CS-U-PE-018 | Parameter set on node | SetParam(node, key, value) | Node.Params[key] = value |

### 7.5 PluginBrowserViewModel (10 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-PB-001 | 3 plugins loaded | SearchText="raw" | VisiblePlugins only raw_input |
| CS-U-PB-002 | 3 plugins | SearchText="zzz" | VisiblePlugins empty |
| CS-U-PB-003 | 3 plugins | SearchText="" | VisiblePlugins = All |
| CS-U-PB-004 | All plugins | SelectedCategory="Format" | Only encoders visible |
| CS-U-PB-005 | All plugins | SelectedCategory="Pixel" | Only processors visible |
| CS-U-PB-006 | raw_input selected | AddToPipelineCommand() | PluginAdded event raised |
| CS-U-PB-007 | MainVM subscribed | AddToPipelineCommand() | MainVM.OnPluginAdded called |
| CS-U-PB-008 | Click raw_input | SelectCommand(plugin) | SelectedPlugin.Description non-null |
| CS-U-PB-009 | Drag raw_input | OnDragStart(plugin) | DragDrop.DoDragDrop called |
| CS-U-PB-010 | PluginService.GetPluginsAsync | — | Plugins.Count > 0 |

### 7.6 BatchViewModel (15 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-BV-001 | File.Exists=true | AddImageCommand(path) | Queue.Count += 1 |
| CS-U-BV-002 | File.Exists=false | AddImageCommand(path) | ErrorMessage non-null |
| CS-U-BV-003 | 1 item selected | RemoveCommand() | Queue.Count -= 1 |
| CS-U-BV-004 | Queue has 3 items | StartCommand() | batchService.StartAsync called |
| CS-U-BV-005 | Queue empty | StartCommand() | ErrorMessage "add images" |
| CS-U-BV-006 | PipelineConfigPath empty | StartCommand() | ErrorMessage "configure pipeline" |
| CS-U-BV-007 | IsRunning=true | PauseCommand() | IsPaused=true |
| CS-U-BV-008 | IsPaused=true | ResumeCommand() | IsPaused=false, IsRunning=true |
| CS-U-BV-009 | IsRunning=true | CancelCommand() | batchService.CancelAsync called |
| CS-U-BV-010 | Batch 5 items executing | Monitor progress | CompletedCount increments |
| CS-U-BV-011 | All 3 items complete | — | CompletedCount=3, IsRunning=false |
| CS-U-BV-012 | Item 2 fails | Continue processing | Item 3 still processed |
| CS-U-BV-013 | MainVM.PipelineId="pid-1" | — | BatchVM.PipelineConfigPath="pid-1" |
| CS-U-BV-014 | Batch executing | Dispose() | Cancellation token triggered, no exception |
| CS-U-BV-015 | Queue+Pipeline set | Serialize BatchSpec | Proto roundtrip correct |

### 7.7 SettingsViewModel (8 条)

| ID | Arrange | Act | Assert |
|----|---------|-----|--------|
| CS-U-SV-001 | settings.json exists | Load on init | Theme/Language/BackendUrl loaded |
| CS-U-SV-002 | Modified Theme+Language | SaveCommand() | settingsService.Save called |
| CS-U-SV-003 | Modified values | ResetCommand() | Values restored to defaults |
| CS-U-SV-004 | Invalid theme "blue" | Validate | ValidationError |
| CS-U-SV-005 | Invalid language code | Validate | ValidationError |
| CS-U-SV-006 | BackendUrl="not a url" | Validate | ValidationError |
| CS-U-SV-007 | No settings.json | Load | Theme=Light, Lang=en (defaults) |
| CS-U-SV-008 | Mock IOException on Save | SaveCommand() | Snackbar shows error |

### 7.8 Service 单元测试 (28 条)

| ID | Service | Test | Assert |
|----|---------|------|--------|
| CS-U-SRV-001 | PipelineService | ToProtoSpec serializes all node.Params | proto.parameters contains quality=90 |
| CS-U-SRV-002 | PipelineService | ToProtoSpec with empty Params | proto.parameters empty |
| CS-U-SRV-003 | PipelineService | ExecuteAsync valid input | output path non-null |
| CS-U-SRV-004 | PipelineService | ExecuteAsync invalid input | Exception propagated |
| CS-U-SRV-005 | GrpcClientService | ConnectAsync server available | IsConnected=true |
| CS-U-SRV-006 | GrpcClientService | ConnectAsync unavailable, retry 3x | IsConnected=false after retries |
| CS-U-SRV-007 | GrpcClientService | Connected then disconnect | IsConnected=false |
| CS-U-SRV-008 | GrpcClientService | Disconnected then reconnect | IsConnected=true |
| CS-U-SRV-009 | ImageService | ImportImage copies to working dir | File copied |
| CS-U-SRV-010 | ImageService | ImportImage duplicate | File renamed |
| CS-U-SRV-011 | ImageService | GetImageInfo valid image | Width/Height/Format correct |
| CS-U-SRV-012 | ImageService | Dispose cleans temp files | Temp files deleted |
| CS-U-SRV-013 | BatchService | CreateBatch valid spec | batch_id non-null |
| CS-U-SRV-014 | BatchService | StartBatch valid id | grpc.StartBatch called |
| CS-U-SRV-015 | BatchService | CancelBatch while running | grpc.CancelBatch called |
| CS-U-SRV-016 | BatchService | GetProgress during execution | completed/total correct |
| CS-U-SRV-017 | PluginService | GetPlugins returns all | plugins.Count = 14 |
| CS-U-SRV-018 | PluginService | GetPlugin valid id | plugin non-null |
| CS-U-SRV-019 | PluginService | GetPlugin invalid id | result = null |
| CS-U-SRV-020 | PluginService | GetSchema valid id | schema.sections non-empty |
| CS-U-SRV-021 | BackendService | CheckHealth available | IsHealthy=true |
| CS-U-SRV-022 | BackendService | CheckHealth unavailable | IsHealthy=false |
| CS-U-SRV-023 | BackendService | GetVersion | version string non-empty |
| CS-U-SRV-024 | BackendService | Ping roundtrip | latency < 5s |
| CS-U-SRV-025 | SettingsService | Save writes to file | settings.json content correct |
| CS-U-SRV-026 | SettingsService | Load from existing file | Settings match file |
| CS-U-SRV-027 | SettingsService | Load file not found | Returns default Settings |
| CS-U-SRV-028 | SettingsService | Save no permission | Exception contains "permission" |

---

## 8. Layer 4: C# gRPC 集成测试

### 8.0 概述

- **总数**: ~120 条
- **执行**: `dotnet test --filter "Category=GrpcIntegration"`
- **共享定义**: 与 Layer 2 共享 `shared/test_cases/grpc_cases.json`
- **关键修复**: 旧代码静默跳过 (try{catch{return;}}) 改为 Assert.Fail

### 8.1 测试类结构

| 文件 | 用例数 | 说明 |
|------|--------|------|
| PluginGrpcTests.cs | 44 | 14 插件 × 2-4 参数组合, 通过 C# gRPC client |
| PipelineGrpcTests.cs | 30 | 多插件链 (2-5 节点) |
| FormatGrpcTests.cs | 15 | 跨格式转换 (10 format pairs + 5 bit-depth) |
| BatchGrpcTests.cs | 15 | 批处理 1/2/5/10/20 files + pause/resume/cancel |
| ErrorPathGrpcTests.cs | 10 | 无效ID/损坏文件/超时/取消/缺参数 |
| ConcurrencyGrpcTests.cs | 6 | 4/8/16 并发 + 100node + 100x serial |

### 8.2 每测试的验证方法使用

所有输出必须通过 ImageAssert 验证:
- 无损操作: `PixelsEqual(output, golden, 0)`
- 有损编码: `PSNRAbove(output, reference, minPSNR)`
- 降噪处理: `SSIMAbove(output, reference, minSSIM)`
- 色彩变换: `DeltaEBelow(output, reference, maxDeltaE)`
- 格式转换: `IsValidFormat(output, expectedFormat)`

---

## 9. Layer 5: C# GUI FlaUI E2E 测试 (105 条)

### 9.0 概述与完整操作流程

**每条 GUI E2E 测试的 11 步标准流程:**

```
1. [启动] 启动 Photopipeline.exe (dotnet publish 产物), 等待主窗口 FlaUI 可交互
2. [导入] 点击 Filmstrip Import 按钮 → 文件对话框选择测试图像 → 确认缩略图显示
3. [选图] 在 Filmstrip 列表中点击选中目标图像
4. [导航] NavigationView 切换到 Pipeline Editor 视图
5. [添加节点] 从 Plugin Browser 点击 "Add to Pipeline" 添加目标插件
6. [连线] 重复步骤 5 添加后续节点 (自动边连接)
7. [设置参数] 在右侧属性面板中找到参数控件, 设置值
8. [运行] 点击 Run 按钮 → 轮询等待进度条完成
9. [导出] 点击 Export → 选择输出路径 → 保存文件
10. [验证] 从磁盘读取导出图像 → ImageAssert 验证像素
11. [清理] 关闭应用, 删除临时输出文件
```

**关键实现要求:**
- 使用 FlaUI.UIA3 (不是 Appium.WebDriver)
- UiTestDriver 所有方法真实操作 WPF 控件, 不允许空桩
- 连接失败: Assert.Fail (禁止静默跳过)
- 每测试结束后必须 Dispose 进程, 清理临时文件

### 9.1 单插件工作流 (40 条)

**raw_input (GE2E-001~003):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-001 | I01 | raw_mode=auto, apply_wb=true, →tiff_encoder | PixelsEqual(output, golden, 0), 3ch RGB, 1920×1080 |
| GE2E-002 | I02 | raw_mode=dcraw, manual_wb=5500K, →tiff_encoder | 与 golden_manual_wb.tif 逐像素一致 |
| GE2E-003 | I10 | output_format=u16, half_size=false, →tiff_encoder | IsValidFormat(TIFF, 1920, 1080, 16) |

**transform (GE2E-004~007):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-004 | I01 | crop_enabled=true, crop_rect=25%,25%,50%,50%, →png | 960×540 (50%), 内容为原图中心 |
| GE2E-005 | I03 | scale_percent=200, filter=lanczos3, →png | 1600×1200, PSNR>35 |
| GE2E-006 | I04 | angle=90, resize_mode=expand, →png | 2160×3840, 宽高互换 |
| GE2E-007 | I01 | flip_h=true, flip_v=true, →png | PixelsEqual(output, expected_flip, 0) |

**colorspace (GE2E-008~011):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-008 | I01 | sRGB→AdobeRGB, embed_icc=true, →tiff | ICC AdobeRGB 嵌入 |
| GE2E-009 | I05 | sRGB→DisplayP3, gamut=clip, →tiff | DeltaE<2 vs golden |
| GE2E-010 | I03 | sRGB→Gray, bp_comp=true, →tiff | 单通道灰度 |
| GE2E-011 | I09 | Gray→sRGB, rendering=perceptual, →png | 3ch RGB, R=G=B 每个像素 |

**lut3d (GE2E-012~015):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-012 | I01 | warm.cube, intensity=80, →png | PixelsEqual(output, golden_warm, 0) |
| GE2E-013 | I01 | cool.cube, intensity=50, →png | 色温偏冷调 |
| GE2E-014 | I03 | film.cube, interp=tetrahedral, →png | SSIM>0.98 vs golden |
| GE2E-015 | I06 | extreme.cube, clamp=true, →png | 所有像素在 [0,255] |

**lens_correct (GE2E-016~019):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-016 | I07 | auto mode, correct_distortion=true, →png | 几何校正后与 golden 一致 |
| GE2E-017 | I08 | distortion+vignette full, →tiff | 畸变+暗角同时修复 |
| GE2E-018 | I07 | TCA only, →png | 色差减少, 畸变保持 |
| GE2E-019 | I08 | manual lensfun_db=custom, →tiff | 自定义数据库校正成功 |

**ai_denoise (GE2E-020~022):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-020 | I06 | strength=20, detail=80, →png | PSNR改善>2dB vs 原图 |
| GE2E-021 | I06 | strength=50, detail=50, →png | PSNR明显改善 (>5dB) |
| GE2E-022 | I06 | strength=90, color_noise=true, →png | 强降噪+色噪减少 |

**exif_rw (GE2E-023~025):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-023 | I11 | read_all=true, overwrite=true, →tiff | 输出EXIF与输入一致 |
| GE2E-024 | I01 | write_exif=custom, tags={"Artist":"Test"}, →tiff | Artist="Test" |
| GE2E-025 | I11 | clear_existing=true, →tiff | 输出无EXIF/XMP/IPTC |

**gps_set (GE2E-026~028):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-026 | I01 | manual, lat=39.9042, lon=116.4074, →tiff | GPS=(39.9042,116.4074) |
| GE2E-027 | I11 | mode=clear, →tiff | 输出无GPS数据 |
| GE2E-028 | I11 | gpx_track, gpx_file=test.gpx, →tiff | GPS从GPX轨道提取 |

**time_shift (GE2E-029~031):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-029 | I11 | shift_hours=+1, shift_minutes=0, →tiff | EXIF时间偏移+1h |
| GE2E-030 | I11 | shift_hours=-24, →tiff | EXIF日期前一天 |
| GE2E-031 | I11 | tz: UTC→Asia/Shanghai, →tiff | EXIF时间+8h |

**格式编码器 (GE2E-032~040):**
| ID | 输入 | 参数 | 预期 |
|----|------|------|------|
| GE2E-032 | I01 | avif: Q=50, speed=5, 8bit | IsValidFormat(AVIF) |
| GE2E-033 | I01 | avif: Q=100, 10bit, chroma=444 | PSNR>45dB vs input |
| GE2E-034 | I01 | avif: lossless | PixelsEqual(input, decoded, 0) |
| GE2E-035 | I01 | jxl: Q=50, effort=5, modular=false | IsValidFormat(JXL) |
| GE2E-036 | I01 | jxl: lossless, effort=9 | PixelsEqual(input, decoded, 0) |
| GE2E-037 | I01 | heif: Q=80 | IsValidFormat(HEIF) |
| GE2E-038 | I10 | tiff: compression=deflate, u16 | IsValidFormat(TIFF, 1920, 1080, 16) |
| GE2E-039 | I12 | png: color_type=rgba, compression=6 | alpha 通道不变 |
| GE2E-040 | I10 | png: bit_depth=16, color_type=rgb | IsValidFormat(PNG, 1920, 1080, 16) |

### 9.2 多插件真实世界工作流 (30 条, GE2E-041~070)

每条管线都有明确的实际用途 (RAW开发/胶片模拟/社交发布/归档/Web发布等):

| ID | 管线 | 输入 | 预期 |
|----|------|------|------|
| GE2E-041 | raw_input→colorspace(sRGB→AdobeRGB)→tiff(16bit ZIP) | I10 | 完整RAW显影, ICC嵌入 |
| GE2E-042 | raw_input→colorspace→lut3d(warm)→jxl(Q=90) | I01 | 胶片模拟流程 |
| GE2E-043 | raw_input→ai_denoise(med)→colorspace→png | I06 | 降噪→色彩→PNG |
| GE2E-044 | raw_input→lens_correct(full)→colorspace→tiff | I07 | 镜头→色彩→TIFF |
| GE2E-045 | raw_input→transform(crop 50%)→colorspace→avif(Q=75) | I01 | Web发布 |
| GE2E-046 | raw_input→transform(200%)→ai_denoise(light)→jxl(lossless) | I03 | 放大+降噪+无损存档 |
| GE2E-047 | transform(rotate 90°)→heif(Q=85) | I04 | 旋转+高效存储 |
| GE2E-048 | raw_input→lens_correct→colorspace→lut3d(film)→tiff | I08 | 5节点完整专业RAW流程 |
| GE2E-049 | ai_denoise(med)→colorspace→lut3d(cool)→jxl | I06 | 降噪+风格化 |
| GE2E-050 | colorspace(sRGB→P3)→transform(50%)→lut3d→png | I01 | 社交媒体发布 |
| GE2E-051 | colorspace(sRGB→Gray)→tiff(16bit) | I03 | 黑白转换 |
| GE2E-052 | transform(flip H+V)→png | I01 | 镜像翻转 |
| GE2E-053 | colorspace(sRGB→DisplayP3)→avif(Q=90,10bit) | I05 | 广色域输出 |
| GE2E-054 | lens_correct(barrel)→colorspace→jxl | I07 | 修复桶形畸变 |
| GE2E-055 | lens_correct(pincushion+vignette)→tiff | I08 | 完整光学校正 |
| GE2E-056 | transform(crop 25%)→colorspace→lut3d(warm)→png | I09 | 缩略图+风格 |
| GE2E-057 | colorspace→transform(resize 50%)→avif(Q=60) | I10 | Web优化 |
| GE2E-058 | ai_denoise(med)→colorspace→tiff(16bit) | I11 | 存档级修复 |
| GE2E-059 | colorspace(sRGB→Gray)→transform(rotate 180°)→png(RGBA) | I12 | Alpha通道处理 |
| GE2E-060 | transform(scale 400%)→colorspace→jxl | I13 | 大倍率放大 |
| GE2E-061 | ai_denoise(heavy)→lens_correct→colorspace→tiff | I06 | 完整修复流程 |
| GE2E-062 | colorspace→lut3d(cool)→transform(crop 75%)→avif | I01 | 风格化裁剪 |
| GE2E-063 | transform(rotate→crop→resize)→colorspace→png | I04 | 复合变换链 |
| GE2E-064 | colorspace(AdobeRGB→sRGB)→jxl(Q=100) | I02 | 色域规范化 |
| GE2E-065 | colorspace→transform(resize 25%)→tiff(8bit) | I10 | 位深降低 |
| GE2E-066 | exif_rw(read_all)→colorspace→tiff | I11 | 元数据保存链 |
| GE2E-067 | gps_set(manual)→time_shift(+8h)→colorspace→jxl | I01 | GPS+时间元数据链 |
| GE2E-068 | lens_correct→ai_denoise(light)→colorspace→lut3d→tiff | I03 | 5节点完整后期 |
| GE2E-069 | ai_denoise→colorspace→transform(crop+resize)→avif(Q=75) | I06 | 降噪打码社交 |
| GE2E-070 | colorspace(CMYK→sRGB)→png | I15 | CMYK→RGB印刷转Web |

### 9.3 格式转换 (15 条, GE2E-071~085)

| ID | 源格式 | 目标格式 | 验证 |
|----|--------|---------|------|
| GE2E-071 | PNG | TIFF | IsValidFormat(TIFF), 像素等同 |
| GE2E-072 | JPEG | PNG | PixelsEqual(input_rgb, output, 2) |
| GE2E-073 | TIFF | JPEG(通过JXL/AVIF) | IsValidFormat |
| GE2E-074 | WebP | PNG | PixelsEqual(input, output, 0) |
| GE2E-075 | BMP | TIFF | IsValidFormat(TIFF) |
| GE2E-076 | AVIF | PNG | PSNR>40dB vs 原始PNG |
| GE2E-077 | JXL(lossless) | TIFF | PixelsEqual(input, output, 0) |
| GE2E-078 | 8bit PNG | 16bit TIFF | IsValidFormat(TIFF, w, h, 16) |
| GE2E-079 | 16bit TIFF | 8bit PNG | 值域正确映射 0-255 |
| GE2E-080 | RGB PNG | Gray TIFF | 单通道, 亮度正确 |
| GE2E-081 | Gray PNG | RGB TIFF | R=G=B 所有像素 |
| GE2E-082 | RGBA PNG | RGB TIFF | 3通道, 无alpha |
| GE2E-083 | RGB PNG | RGBA TIFF | Alpha=255 全通道 |
| GE2E-084 | CMYK TIFF | RGB PNG | 颜色空间转换正确 |
| GE2E-085 | PNG+JPEG+TIFF | AVIF+JXL+PNG | 3输入→3输出各自正确 |

### 9.4 批处理 (10 条, GE2E-086~095)

| ID | 场景 | 预期 |
|----|------|------|
| GE2E-086 | Batch 3 images, crop 50%→png | 3个输出, 每个50%尺寸 |
| GE2E-087 | Batch 5 images, sRGB→Gray→tiff | 5个灰度TIFF |
| GE2E-088 | Pause at image 3/5, Resume | 暂停后恢复, 5/5完成 |
| GE2E-089 | Cancel mid-execution | 部分输出+状态正确 |
| GE2E-090 | 2 images, different encoders each | 格式各自正确 |
| GE2E-091 | 10 images stress test | 全部完成, 内存不泄漏 |
| GE2E-092 | No images selected | 明确错误提示 |
| GE2E-093 | Empty pipeline | 验证阶段报错 |
| GE2E-094 | 1 valid + 1 corrupt image | 部分成功, 失败原因日志 |
| GE2E-095 | Resume after app restart | 断点续传正确 |

### 9.5 错误路径 & 边界 (10 条, GE2E-096~105)

| ID | 操作 | 预期 |
|----|------|------|
| GE2E-096 | 不添加节点, 点击Run | 错误提示, 状态不变 |
| GE2E-097 | resize=-50% | 参数验证报错, 不执行 |
| GE2E-098 | 不运行直接Export | 错误提示"请先运行" |
| GE2E-099 | Run→立即Cancel | 状态正确回滚, 资源释放 |
| GE2E-100 | 选图A→建管线→选图B→Run | 管线适配新图像 |
| GE2E-101 | A→B(disabled)→C, Run | B被跳过, A→C直通 |
| GE2E-102 | A→B→C, 删除B, 重新添加B | 管线状态恢复正确 |
| GE2E-103 | 快速添加/删除10个节点 | 不崩溃, 内存稳定 |
| GE2E-104 | 禁用全部节点后Run | 明确错误提示 |
| GE2E-105 | A→B→C→A (环形) | UI阻止连线, 显示警告 |

**Layer 5 总计: 40 + 30 + 15 + 10 + 10 = 105 条**

---

## 10. Layer 6: Cross-Channel 交叉验证测试

### 10.0 概述

- **总数**: ~60 条
- **三通道比对**: Rust gRPC ≡ C# gRPC ≡ C# GUI (逐像素 tolerance=0)
- **铁律**: API 输出 = 黄金基准; UI 输出必须完全一致; 失败生成差异图像; 异常必须传播

### 10.1 用例详细

| ID | 类别 | 管线 | 输入 | 三通道比对 |
|----|------|------|------|-----------|
| CCV-001 | plugin | raw_input(auto)→tiff | I01 | Rust gRPC = C# gRPC = GUI |
| CCV-002 | plugin | raw_input(dcraw)→tiff | I10 | 三通道像素一致 |
| CCV-003 | plugin | transform(crop 50%)→png | I01 | 三通道像素一致 |
| CCV-004 | plugin | transform(rotate 90°)→png | I04 | 三通道像素一致 |
| CCV-005 | plugin | colorspace(sRGB→AdobeRGB)→tiff | I01 | 三通道像素一致 |
| CCV-006 | plugin | colorspace(sRGB→Gray)→tiff | I03 | 三通道像素一致 |
| CCV-007 | plugin | lut3d(warm, intensity=80)→png | I01 | 三通道像素一致 |
| CCV-008 | plugin | lut3d(film, tetrahedral)→png | I03 | 三通道像素一致 |
| CCV-009 | plugin | lens_correct(auto)→png | I07 | 三通道像素一致 |
| CCV-010 | plugin | lens_correct(full)→tiff | I08 | 三通道像素一致 |
| CCV-011 | plugin | ai_denoise(med)→png | I06 | 三通道像素一致 |
| CCV-012 | plugin | ai_denoise(light)→tiff | I06 | 三通道像素一致 |
| CCV-013 | plugin | exif_rw(preserve)→tiff | I11 | 三通道像素+元数据一致 |
| CCV-014 | plugin | gps_set(manual 39.9,116.4)→tiff | I01 | 三通道像素+GPS一致 |
| CCV-015 | plugin | time_shift(+1h)→tiff | I11 | 三通道像素+EXIF时间一致 |
| CCV-016 | plugin | avif_encoder(lossless)→decode | I01 | 三通道像素一致 |
| CCV-017 | plugin | jxl_encoder(lossless)→decode | I01 | 三通道像素一致 |
| CCV-018 | plugin | heif_encoder(Q=80)→decode | I01 | 三通道像素一致(有损容差) |
| CCV-019 | plugin | tiff_encoder(16bit ZIP)→decode | I10 | 三通道像素一致 |
| CCV-020 | plugin | png_encoder(RGBA)→decode | I12 | 三通道像素+alpha一致 |
| CCV-021 | pipeline | raw→colorspace→tiff (3节点) | I01 | 三通道像素一致 |
| CCV-022 | pipeline | raw→ai_denoise→colorspace→png (4节点) | I06 | 三通道像素一致 |
| CCV-023 | pipeline | raw→lens→colorspace→lut→tiff (5节点) | I08 | 三通道像素一致 |
| CCV-024 | pipeline | transform→colorspace→jxl (3节点) | I01 | 三通道像素一致 |
| CCV-025 | pipeline | raw→transform→colorspace→avif (4节点) | I01 | 三通道像素一致 |
| CCV-026 | pipeline | raw→lens→ai_denoise→colorspace→tiff (5节点) | I06 | 三通道像素一致 |
| CCV-027 | pipeline | exif→gps→time→colorspace→tiff (5节点) | I11 | 三通道像素+元数据一致 |
| CCV-028 | pipeline | ai_denoise→colorspace→lut→tiff (4节点) | I06 | 三通道像素一致 |
| CCV-029 | pipeline | colorspace→lut→transform→png (4节点) | I01 | 三通道像素一致 |
| CCV-030 | pipeline | raw→colorspace→lut→jxl (4节点) | I01 | 三通道像素一致 |
| CCV-031 | pipeline | transform(crop+resize)→colorspace→png (3节点) | I01 | 三通道像素一致 |
| CCV-032 | pipeline | lens→ai_denoise→colorspace→heif (4节点) | I07 | 三通道像素一致(有损容差) |
| CCV-033 | pipeline | raw→transform(flip)→colorspace→tiff (4节点) | I01 | 三通道像素一致 |
| CCV-034 | pipeline | disabled middle node: A→B(disabled)→C | I01 | 三通道像素一致(B被跳过) |
| CCV-035 | pipeline | all disabled except encoder: 验证穿透 | I01 | 三通道像素一致 |
| CCV-036 | format | PNG→decode→TIFF encode | I01 | 三通道像素一致 |
| CCV-037 | format | JPEG→decode→PNG encode | I03 | 三通道像素一致 |
| CCV-038 | format | 16bit TIFF→decode→16bit PNG encode | I10 | 三通道像素一致 |
| CCV-039 | format | RGBA PNG→decode→TIFF encode | I12 | 三通道像素+alpha一致 |
| CCV-040 | format | Gray PNG→decode→RGB→decode→Gray | I09 | 三通道像素一致 |
| CCV-041 | format | CMYK TIFF→decode→sRGB PNG | I15 | 三通道像素一致 |
| CCV-042 | format | AVIF→decode→PNG encode | I01 | 三通道像素一致(有损容差) |
| CCV-043 | format | JXL(lossless)→decode→TIFF encode | I01 | 三通道像素一致 |
| CCV-044 | format | 8bit→16bit promotion | I01 | 三通道像素一致(位深提升) |
| CCV-045 | format | 16bit→8bit truncation | I10 | 三通道像素一致(位深降低) |
| CCV-046 | batch | 3 images, 1 pipeline (crop→png) | I01,I03,I09 | 三通道每个输出一致 |
| CCV-047 | batch | 5 images, 1 pipeline (color→tiff) | I01~I05 | 三通道每个输出一致 |
| CCV-048 | batch | 2 images, different encoders | I01,I10 | 三通道每个输出一致 |
| CCV-049 | batch | Pause at 3/5, Resume | I01~I05 | 三通道每个输出一致 |
| CCV-050 | batch | Cancel after 2/5 | I01~I05 | 三通道部分输出一致 |
| CCV-051 | batch | 10 images stress | I01~I10 | 三通道全部10个一致 |
| CCV-052 | batch | Mixed format inputs (PNG+JPEG+TIFF) | mixed | 三通道每个输出一致 |
| CCV-053 | batch | Single image batch (= non-batch) | I01 | 三通道像素一致 |
| CCV-054 | regression | raw→colorspace→tiff (golden baseline) | I01 | 三通道 = golden reference |
| CCV-055 | regression | transform(crop 50%)→png (golden) | I01 | 三通道 = golden reference |
| CCV-056 | regression | colorspace(sRGB→Gray)→tiff (golden) | I03 | 三通道 = golden reference |
| CCV-057 | regression | lut3d(film)→png (golden) | I17 | 三通道 = golden reference |
| CCV-058 | regression | ai_denoise(med)→png (golden) | I06 | 三通道 = golden reference |
| CCV-059 | regression | lens_correct→colorspace→tiff (golden) | I07 | 三通道 = golden reference |
| CCV-060 | regression | 5-node full RAW workflow (golden) | I08 | 三通道 = golden reference |

**Layer 6 总计: 20 + 15 + 10 + 8 + 7 = 60 条**

---

## 11. 共享测试用例 JSON 格式

跨语言共享 (Rust `crates/test-defs/` + C# `SharedTestCaseLoader.cs`):

```json
{
  "id": "GE2E-001",
  "name": "RawInput_AutoExposure_SdrOutput",
  "category": "plugin",
  "input_image": "solid_color_1920",
  "pipeline_spec": {
    "nodes": [
      {"id": "n1", "plugin": "raw_input", "params": {"raw_mode": "auto", "apply_white_balance": true}},
      {"id": "n2", "plugin": "tiff_encoder"}
    ],
    "edges": [{"from": "n1", "to": "n2"}]
  },
  "assertions": {
    "tolerance_per_channel": 0,
    "expected_format": "TIFF",
    "expected_width": 1920,
    "expected_height": 1080
  }
}
```

创建目录: `shared/test_cases/` 存放:
- `grpc_cases.json` — Layer 2/4 共享 (120 条)
- `cross_chain_cases.json` — Layer 6 (60 条)
- `schema.json` — JSON Schema 验证

---

## 12. 可复用基础设施

### Rust (tests/test_harness/src/)
- `assertions/golden.rs` — assert_golden_bytes + PHOTOPIPELINE_GENERATE_GOLDEN
- `assertions/image.rs` — assert_pixels_eq, assert_buffer_dimensions
- `assertions/quality.rs` — compute_psnr, compute_ssim, compute_mae
- `assertions/structural.rs` — PNG/TIFF/HEIF/AVIF 容器验证
- `assertions/tiff.rs` — assert_valid_tiff, assert_tiff_tag
- `fixtures/image.rs` — ImageFixture builder
- `fixtures/metadata.rs` — 预配置 EXIF/GPS/GPX
- `fixtures/pipeline.rs` — minimal_pipeline 工厂

### C# (Photopipeline.Tests/FunctionalTests/Infrastructure/)
- `ImageAssert.cs` — 7 种像素验证方法
- `TestImageGenerator.cs` — 确定性图像生成 (种子 42)
- `CrossChannelVerifier.cs` — VerifyEquivalence + SaveDiffImage
- `TestPipelineBuilder.cs` — Fluent API
- `TestCaseDefinition.cs` — 用例定义 record (需扩展 JSON 反序列化)

---

## 13. 需重写的垃圾代码清单

| 文件 | 旧评分 | 问题 | 行动 |
|------|--------|------|------|
| ErrorRecoveryScenarioTests.cs | 1.5/10 | Mock 设置但从未触发 | **重写** |
| PluginServiceTests.cs | 1.8/10 | 全部反射检查 | **重写** |
| UiTestDriver.cs | 0/10 | 全部空桩 | **重写**为 FlaUI 实现 |
| UIAutomationTests/*.cs (11文件) | 0/10 | 全部 Skip + 空体 | **重写**为真实 FlaUI 自动化 |
| UiChannel/*Tests.cs (10文件) | 1/10 | 短路静默通过 | **重写** |
| CrossChannel/*Tests.cs (5文件) | 1/10 | UI 通道异常被吞 | **重写** |
| ImageAssert.MetadataMatches | 空壳 | 创建空对象 | **实现**真实 EXIF/XMP 读取 |
| ReferenceImageGenerator.cs | 不存在 | — | **新建** |
| ApiTestBase.cs | — | try{catch{return;}} | **修复**为 Assert.Fail |

---

## 14. 实施阶段

### Phase A: Rust 测试扩展 (670 条, Layers 0-2)

| # | 内容 | 产出 |
|---|------|------|
| A1 | 补充 14 插件 ParameterSchema 单元测试 (140) | crates/plugins/tests/ |
| A2 | 补充 Engine 核心单元测试 (110) | crates/engine/tests/ |
| A3 | 补充 gRPC service 单元测试 (35) | crates/server/tests/ |
| A4 | 生成 20 张测试输入图像 | tests/fixtures/input/ |
| A5 | 实现单插件集成测试 (84) | tests/pipeline_integration/ |
| A6 | 实现多插件链集成测试 (90) | tests/pipeline_integration/ |
| A7 | 实现格式+边界集成测试 (26) | tests/pipeline_integration/ |
| A8 | 实现 gRPC E2E 测试 (120) | tests/grpc_e2e/ |
| A9 | 创建 shared/test_cases/ JSON 定义 | shared/test_cases/ |
| A10 | 创建 Rust JSON 解析 crate | crates/test-defs/ |

### Phase B: C# 测试扩展 (405 条, Layers 3-6)

| # | 内容 | 产出 |
|---|------|------|
| B1 | 重写 PluginServiceTests (28) | UnitTests/Services/ |
| B2 | 补充 ViewModel 测试 (92) | UnitTests/ViewModels/ |
| B3 | 重写 ApiTestBase (修复静默跳过) | ApiChannel/ApiTestBase.cs |
| B4 | 实现 C# gRPC 集成测试 (120) | ApiChannel/ (重写所有) |
| B5 | 集成 FlaUI NuGet 包 | UIAutomationTests.csproj |
| B6 | 重写 UiTestDriver (FlaUI 真实操作) | UiChannel/UiTestDriver.cs |
| B7 | 重写 UiTestBase (进程启动+窗口就绪) | UiChannel/UiTestBase.cs |
| B8 | 实现 105 条 GUI E2E 测试 | UIAutomationTests/ (重写所有) |
| B9 | 重写 CrossChannel 测试 | CrossChannel/ (重写所有) |
| B10 | 实现 SharedTestCaseLoader | Infrastructure/SharedTestCaseLoader.cs |
| B11 | 实现 ReferenceImageGenerator | Infrastructure/ReferenceImageGenerator.cs |
| B12 | 实现 ImageAssert.MetadataMatches | Infrastructure/ImageAssert.cs |

---

## 15. 验证标准与对抗性检查清单

### 15.1 构建标准
- `cargo build --workspace` → 0E 0W
- `dotnet build Photopipeline.sln -c Release` → 0E 0W

### 15.2 测试通过标准
- `cargo test --workspace` → 350/350 (Layer 0)
- `cargo test --test pipeline_integration` → 200/200 (Layer 1)
- `cargo test --test grpc_e2e` → 120/120 (Layer 2)
- `dotnet test --filter "Category=Unit"` → 120/120 (Layer 3)
- `dotnet test --filter "Category=GrpcIntegration"` → 120/120 (Layer 4)
- `dotnet test --filter "Category=GuiE2E"` → 105/105 (Layer 5)
- `dotnet test --filter "Category=CrossChannel"` → 60/60 (Layer 6)
- **总计: 1075/1075**

### 15.3 对抗性检查清单 (每写完一个文件必问)

1. **如果后端返回全黑图像，测试会 FAIL 吗？**
   - 只有逐像素验证的测试会 FAIL
   - 仅 File.Exists 的测试会 PASS (这是对抗性代码!)

2. **如果管线完全没有执行，测试会发现吗？**
   - 输出路径为空或不存在 → 会 FAIL (如果做了 File.Exists 检查)
   - 但如果静默跳过 → 不会 FAIL (对抗性代码!)

3. **如果交换输入和输出路径，测试会 FAIL 吗？**
   - 像素比对会发现输出=输入 → 会 FAIL
   - 但如果用 golden images → 会 FAIL (内容不对)

4. **如果参数被静默忽略，测试会 FAIL 吗？**
   - 输出与预期 golden 不同 → 会 FAIL
   - 但如果只验证文件存在 → 不会 FAIL (对抗性代码!)

5. **这个测试被愚弄的最简单方法是什么？**
   - 返回固定大小但内容随机的文件
   - 让管线直接复制输入到输出
   - 静默跳过所有处理步骤
