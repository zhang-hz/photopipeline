# Changelog

All notable changes to the Photopipeline project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## v0.1.0 (2026-05-23)

### Added

#### Core Crate — 核心类型体系 (`photopipeline-core`)
- **ImageBuffer / PixelBuffer**: 16bit+ 像素缓冲区，含 AlignedBuffer（页对齐支持 GPU 映射）
- **PixelFormat 枚举**: U8 / U16 / U32 / F16 / F32 五种格式
- **ChannelLayout**: Gray / GrayAlpha / RGB / RGBA / Planar(n) / Custom(n)
- **ColorSpace 类型**: 原色 + 传递函数 + 白点 + HDR nit 值
  - 内置预设: sRGB / Adobe RGB / Display P3 / Rec.2020 PQ / ACEScg / Linear sRGB
  - 10 种原色: BT.709 / BT.2020 / Display P3 / sRGB / Adobe RGB / ProPhoto / ACES / ACEScg / CIE XYZ / DCI P3 / Rec.2100
  - 11 种传递函数: Linear / sRGB / Gamma 2.2-2.8 / PQ / HLG / SLog3 / LogC / Custom
  - 8 种白点: D50 / D55 / D60 / D65 / D75 / DCI / E / Custom
- **Metadata 类型体系**: ExifData / XmpData / IptcData / GpsData / CustomTag
- **GpxTrack / GpxPoint**: GPX 轨迹解析与时间戳插值
- **TileLayout**: 分块布局及迭代器（支持重叠）

#### Plugin System — 插件框架 (`photopipeline-plugin`)
- **6 种能力 Trait**: Plugin / MetadataProcessor / PixelProcessor / FormatProcessor / GpuProcessor / AiProcessor / ExternalToolProcessor
- **Schema 驱动**: ParameterSchema / ParameterSection / ParameterField
- **18 种参数类型**: String / Integer / Float / Boolean / Enum / Color / FilePath / Coordinate / Slider / ComboSlider / Expression / Preset / Array / MapWidget / BeforeAfter / Separator / Section
- **GuiSchema**: 声明式 GUI 布局（Standard + Custom）、预览模式（Live / BeforeAfter / Tiled）、辅助视图（Histogram / Waveform / Vectorscope / Map 等 10 种）
- **ParameterSet**: 参数值容器（支持 JSON 值存取与浅合并）
- **Registry**: 线程安全的全局插件注册表（基于 DashMap）
- **PluginQuery**: 按分类/标签/关键词/像素需求查询插件
- **PluginLoader 体系**: BuiltinPluginLoader / NativePluginLoader / ExternalToolPluginLoader
- **PluginLoaderManager**: 自动发现并加载插件
- **ProgressSink**: 进度报告与取消 trait

#### Pipeline Engine — 管线引擎 (`photopipeline-engine`)
- **PipelineGraph**: DAG 有向无环图结构
  - `add_node` / `remove_node` — 节点增删
  - `connect` / `disconnect` — 边操作（含自环/重复检测）
  - `topological_order` — 拓扑排序（Kahn 算法，检测环）
  - `validate_graph` — 图完整性校验
  - `from_template` — 从 TOML 模板构建图
- **PipelineTemplate**: TOML 管线配置的可序列化结构
  - 节点 / 边 / 图像覆盖 / 分组规则 / 批量配置
  - `validate()` 方法
- **NodeExecutor**: 管线执行引擎
  - 按拓扑顺序执行节点
  - 像素/元数据节点分离处理
  - 参数校验 + 表达式求值
- **ParameterResolver**: 四级参数优先级系统
  - 插件默认 < 模板默认 < 分组覆盖 < 图像覆盖
  - 分组条件: ExifEq / ExifGte / ExifLte / GpsNear / Always / And / Or / Expression
  - Haversine 公式 GPS 距离计算
- **ExpressionEngine**: 参数表达式引擎
  - 变量命名空间: `exif.*`（iso / aperture / shutter / focal_length / make / model / lens）+ `image.*`（filename / width / height / filesize）
  - 比较运算符: `>` / `<` / `>=` / `<=` / `==` / `!=`
  - 三元运算符: `condition ? true_value : false_value`
  - 字符串字面值与数值字面值支持
- **TileEngine**: 分块处理引擎
  - 默认 1024px tile + 64px 重叠
  - 自动并行度检测

#### 14 个内置插件
1. **exif_rw**: EXIF/XMP/IPTC/GPS 元数据读写（ExifTool 后端）
2. **gps_set**: GPS 坐标管理（手动 / GPX 轨迹插值 / 清除）
3. **time_shift**: 拍摄时间调整与时区转换
4. **colorspace**: 色彩空间转换（ICC Profile + 渲染意图）
5. **lut3d**: 3D LUT 色彩调色（.cube / .3dl / .look / .csp）
6. **transform**: 缩放/旋转/裁剪/翻转（双线性/Lanczos3/最近邻）
7. **lens_correct**: 镜头畸变/TCA/暗角校正（LensFun）
8. **ai_denoise**: AI 降噪（ONNX Runtime，支持 CUDA/TensorRT/CoreML/OpenVINO）
9. **raw_input**: RAW 文件输入（ARW/CR2/CR3/NEF/DNG/RAF/ORF/RW2/PEF 等 15 种格式）
10. **heif_encoder**: HEIF/HEIC 10-bit 编码（libheif + x265）
11. **jxl_encoder**: JPEG XL 16-bit 编码（libjxl，支持无损）
12. **avif_encoder**: AVIF（AV1）编码（libavif + aom）
13. **tiff_encoder**: TIFF/BigTIFF 编码（无压缩/LZW/Deflate/PackBits）
14. **png_encoder**: PNG 16-bit 编码（Deflate，含 ICC/EXIF chunk）

#### CLI 应用 (`photopipeline-cli`)
- **pipeline 子命令**: `run`（执行管线）/ `validate`（验证配置）
- **plugin 子命令**: `list`（列出插件）/ `info`（插件详情）
- **batch 子命令**: `run`（批量处理）/ `validate`（批量验证）
- TOML 配置加载，进度条指示

#### gRPC Server (`photopipeline-server`)
- **Protobuf 3 定义**: pipeline.proto / image.proto / batch.proto
- **3 个服务**: PipelineService / ImageService / BatchService
- **tonic 实现**: 流式 RPC（Execute / Decode / Encode / GetProgress）
- 基于 tokio 的异步运行时，优雅关闭支持

#### Halide 生成器源文件（C++）
- `colorspace_generator.cpp`: 色彩空间转换 Halide 管线
- `resize_generator.cpp`: 图像缩放 Halide 管线（Lanczos3）
- `tonemap_generator.cpp`: HDR 色调映射 Halide 管线
- `CMakeLists.txt`: 构建配置（CI 编译）

#### OIIO FFI 绑定
- `crates/oiio/`: feature-gated OpenImageIO FFI
- `OIIO_read_image` / `OIIO_write_image` / `OIIO_free_image` 外部函数声明

#### External Crate
- `crates/external/`: 外部工具封装入口（ExifTool / libvips / 商业 API stubs）

#### GUI
- **WinUI 3 GUI**（Windows）：.NET 8 项目骨架
- **GTK4 + Rust GUI**（Linux）：项目骨架
- **SwiftUI GUI**（macOS）：项目骨架

#### CI/CD
- **build-halide.yml**: Halide 平台矩阵构建（linux/windows/macos）
- **build-rust.yml**: Rust workspace 跨平台构建
- **release.yml**: 全平台发布打包（AppImage / MSIX / DMG）

#### 测试
- **Core**: 67 个单元测试（PixelBuffer/AlignedBuffer/TileLayout/ColorSpace/ColorRGB/Metadata/GPS/GPX 插值/Version/Format/Backend 等）
- **Plugin**: 20 个单元测试（Registry 注册/查询/注销/Manifest/Schema 默认值/字段查找/ParameterSet 存取合并）
- **Engine**: 43 个单元测试（Graph 增删改连/拓扑排序/环检测/校验/序列化/Template/ParameterResolver 解析合并/条件求值/三元表达式/各组条件匹配/优先级）
- **Plugins**: 所有 14 个插件内嵌校验逻辑
- **CLI 集成测试**: `integration_test.rs` 包含 10 个集成测试（插件注册数/分类/图构建/序列化/校验/查询/显示）
- **总计**: 168+ 个单元测试

### Infrastructure
- Cargo workspace 结构（10 个 crate）
- `justfile` 任务运行器（build / check / test / lint / fmt / clean / run-pipeline / run-server）
- tracing 结构化日志
- tokio 异步运行时
- 依赖矩阵: serde（序列化）、clap（CLI）、tonic（gRPC）、dashmap（并发 Map）、parking_lot（同步原语）、chrono（时间）、uuid（标识符）、regex（正则）、glob（文件匹配）、indicatif（进度条）、bytemuck（零拷贝转换）

---

[0.1.0]: https://github.com/zhang-hz/photopipeline/releases/tag/v0.1.0
