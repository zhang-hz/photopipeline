# Changelog

All notable changes to the Photopipeline project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-05-23

### Added

#### Core Crate (`photopipeline-core`)

- **ImageBuffer / PixelBuffer**: 16-bit+ pixel buffer with `AlignedBuffer` (page-aligned for GPU mapping).
- **PixelFormat** enum: `U8`, `U16`, `U32`, `F16`, `F32` — all with `bytes_per_channel()`, `is_float()`, `is_high_precision()`, `max_value_u16()` helpers.
- **ChannelLayout**: `Gray`, `GrayAlpha`, `RGB`, `RGBA`, `Planar(n)`, `Custom(n)` — with `channel_count()` and `is_interleaved()`.
- **ColorSpace** type with `ColorPrimaries` + `TransferFunction` + `WhitePoint` + optional `hdr_nits`.
  - 11 colour primaries: BT.709, BT.2020, Display P3, sRGB, Adobe RGB, ProPhoto, ACES, ACEScg, CIE XYZ, DCI P3, Rec.2100.
  - 11 transfer functions: Linear, sRGB, Gamma22–28, PQ, HLG, SLog3, LogC, Custom(f64).
  - 8 white points: D50, D55, D60, D65, D75, DCI, E, Custom(f32, f32).
  - Preset colour spaces: `SRGB`, `ADOBE_RGB`, `DISPLAY_P3`, `REC2020_PQ`, `ACES_CG`, `LINEAR_SRGB`.
- **RenderingIntent** and **GamutMapping** enums for ICC‑based colour conversion.
- **ColorConversionSpec**: full specification for colour space transforms including ICC profile and OCIO config.
- **Metadata** type hierarchy: `Metadata`, `ExifData`, `XmpData`, `IptcData`, `GpsData`, `CustomTag`, plus raw tag types for each standard.
- **GpxTrack / GpxPoint**: GPX track parsing with timestamp‑based linear interpolation (position, elevation, speed, bearing).
- **TileLayout**: tile grid computation with overlap support, `TileSpec` iterator, and edge‑tile boundary handling.
- **AlignedBuffer**: byte buffer with `as_u16_slice()` and `as_f32_slice()` accessors via `bytemuck`.
- **PluginError**: 23‑variant error enum (`NotFound`, `AlreadyLoaded`, `LoadFailed`, `VersionMismatch`, `InvalidParameter`, `MissingTool`, `GpuNotAvailable`, `GpuOutOfMemory`, `ExpressionError`, `Timeout`, `Internal`, `Canceled`, `Io`, `ValidationFailed`, `NodeExecutionFailed`, `CircularDependency`, `FileNotFound`, `UnsupportedFormat`, `EncodingFailed`, `DecodingFailed`, `Config`, `Other`).
- **ValidationIssue**: `Error`, `Warning`, `Info` with parameter reference and message.
- **PluginVersion**: semver struct with `new()`, `Display`, and comparison operators.
- **VersionRequirement**: semver range with `is_satisfied_by()`.
- **PluginCategory**: `Input`, `Metadata`, `Color`, `Transform`, `Enhance`, `Merge`, `Format`, `External`, `Custom(String)`.
- **GpuBackend** and **AiBackend**: enumerating CUDA, Metal, Vulkan, DirectX, OpenCL, ROCm, OpenVINO, ONNX, TensorRT, CoreML, Burn.
- **ImageInfo**: image metadata struct (id, path, filename, format, dimensions, file size, pixel format, colour space).
- **ProcessingStats**: timing and memory statistics for a processing operation.
- **ImageFormat** enum: HEIF, HEIC, AVIF, JXL, PNG, TIFF, JPEG, WEBP, OpenEXR, RAW, DNG, PPM, PGM, BMP, Unknown.
- **DecodeOptions / EncodeOptions**: format‑agnostic decode and encode parameter structs.
- **ChromaSubsampling**: `Yuv444`, `Yuv422`, `Yuv420`.
- **HardwareRequirement / PluginConfig / FormatProbe**: supporting types.
- **Tensor / TensorDtype**: AI inference tensor with F32, F16, I8, U8 dtypes.
- **GuiSchema** types: `GuiLayout`, `GuiSection`, `GuiRow`, `GuiCell`, `PreviewMode`, `AuxView`, `SectionStyle`, `RowHeight`, `LabelPosition`, `SplitOrientation`, `SliderStyle`, `SliderOrientation`, `FloatWidget`, `IntegerWidget`, `EnumDisplay`, `ColorMode`, `FilePathKind`.

#### Plugin System (`photopipeline-plugin`)

- **Plugin** base trait: `id()`, `name()`, `version()`, `category()`, `description()`, `tags()`, `requires_pixel_access()`, `produces_pixel_output()`, `supported_hardware()`, `parameter_schema()`, `gui_schema()`, `initialize()`, `shutdown()`, `validate()`.
- **MetadataProcessor** trait: `metadata_scope()`, `read_metadata()`, `write_metadata()`.
- **PixelProcessor** trait: `supported_input_formats()`, `supported_output_formats()`, `supported_color_spaces()`, `required_gpu_backend()`, `process_pixels()`.
- **FormatProcessor** trait: `format_id()`, `supported_extensions()`, `can_decode()`, `can_encode()`, `decode()`, `encode()`.
- **GpuProcessor** trait: `supported_backends()`, `gpu_memory_required()`, `process_gpu()`.
- **AiProcessor** trait: `model_info()`, `supported_backends()`, `load_model()`, `unload_model()`, `infer()`.
- **ExternalToolProcessor** trait: `tool_id()`, `tool_version_requirement()`, `trusted()`, `check_available()`, `execute()`.
- **ProgressSink** trait: `set_progress(fraction, message)`, `is_canceled()`.
- **Registry**: thread‑safe global plugin registry backed by `DashMap`. Supports registration of all 6 capability traits plus base Plugin, query by category/tags/keyword/pixel‑requirement, unregistration, manifest listing.
- **ParameterSchema**: schema‑driven parameter definition with `ParameterSection` and `ParameterField`.
- **18 ParameterType** variants: `String`, `Integer`, `Float`, `Boolean`, `Enum`, `Color`, `FilePath`, `Coordinate`, `Slider`, `ComboSlider`, `Expression`, `Preset`, `Array`, `MapWidget`, `BeforeAfter`, `Separator`, `Section`.
- **ParameterSet**: JSON‑value parameter container with typed getters (`get_str`, `get_i64`, `get_f64`, `get_bool`) and shallow `merge()`.
- **EnumOption**, **VariableDef**, **NamedPreset**: supporting schema types.
- **PluginLoader** trait and three implementations: `BuiltinPluginLoader`, `NativePluginLoader` (reads `.toml` manifest from `.so`/`.dll` directory), `ExternalToolPluginLoader`.
- **PluginLoaderManager**: scans search paths, probes loaders, discovers and registers plugins.
- **PluginManifest**: serializable plugin metadata structure.
- **PluginQuery**: filter struct for Registry queries.
- **PluginConfig**: plugin configuration with settings and search paths.
- **ModelInfo / ModelSource**: AI model metadata with `Bundled`, `ExternalFile`, `HuggingFace`, `Url` sources.
- **ToolAvailability**: external tool check result.
- **NodePanelDefinition**, **PanelSection**, **PanelWidget**, **DropdownOption**, **CardOption**, **ContextBarConfig**: GUI schema types for auto‑generated plugin panels.

#### Pipeline Engine (`photopipeline-engine`)

- **PipelineGraph**: DAG structure with `add_node()`, `remove_node()`, `connect()`, `disconnect()`, `topological_order()` (Kahn's algorithm with cycle detection), `has_cycle()`, `validate_graph()`, `from_template()`.
- **PipelineNode**: node with UUID id, label, plugin reference, enabled flag, input/output ports, parameter overrides.
- **PipelineTemplate**: TOML‑serializable pipeline definition with `TemplateMetadata`, `TemplateNode`, `TemplateEdge`, `ImageOverride`, `ParamGroup`, `BatchConfig`.
  - `validate()`: checks node count, edge source/target existence.
  - `into_graph()`: converts template to executable `PipelineGraph`.
- **NodeExecutor**: executes a `PipelineGraph` in topological order.
  - Separates pixel and metadata node execution (`process_pixel_node` / `process_metadata_node`).
  - Parameter validation per node; halts on validation errors.
  - Progress reporting and cancellation support.
- **ParameterResolver**: 4‑level parameter resolution system.
  - Level 0: plugin built‑in defaults.
  - Level 1: template‑level defaults.
  - Level 2: group overrides (last matching wins).
  - Level 3: per‑image overrides (highest priority).
  - `resolve()` merges all levels, evaluates inline expressions.
  - `resolve_single()` for non‑batch (single image) use.
- **GroupCondition**: condition types for group‑based overrides.
  - `ExifEq`, `ExifGte`, `ExifLte`, `GpsNear`, `Always`, `And`, `Or`, `Expression`.
  - **Haversine** formula for GPS distance calculation.
- **ExpressionEngine**: inline expression evaluator.
  - Variables: `exif.iso`, `exif.aperture`, `exif.shutter`, `exif.focal_length`, `exif.make`, `exif.model`, `exif.lens`, `image.filename`, `image.width`, `image.height`, `image.filesize`.
  - Operators: `>`, `<`, `>=`, `<=`, `==`, `!=`.
  - Ternary: `condition ? true_value : false_value` (nestable via balanced bracket matching).
  - String and numeric literal support; epsilon‑based float comparison.
- **TileEngine**: tile‑based processing for large images.
  - Default tile size: 1024, overlap: 64, parallelism: auto‑detected.
  - `process_tiled()`: copies source tiles, processes each, blits results back.
  - `copy_tile_from_source()` / `blit_tile_to_output()`: safe bounds‑checked slice copying.

#### 14 Built‑in Plugins

1. **exif_rw** — EXIF/XMP/IPTC/GPS metadata read/write via ExifTool subprocess
2. **gps_set** — GPS coordinate management: manual, GPX track interpolation, clear
3. **time_shift** — capture time adjustment and timezone conversion
4. **colorspace** — colour space conversion with ICC profile support and rendering intents
5. **lut3d** — 3D LUT colour grading (.cube, .3dl, .look, .csp)
6. **transform** — resize, rotate, crop, flip with bilinear/Lanczos3/nearest filters
7. **lens_correct** — lens distortion, TCA, vignetting correction via LensFun
8. **ai_denoise** — AI image denoising via ONNX Runtime (CUDA/TensorRT/CoreML/OpenVINO backends)
9. **raw_input** — RAW file input (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF, and more)
10. **heif_encoder** — HEIF/HEIC 10‑bit encoding (libheif + x265)
11. **jxl_encoder** — JPEG XL 16‑bit encoding (libjxl; lossless and visually lossless modes)
12. **avif_encoder** — AVIF encoding (libheif + aom)
13. **tiff_encoder** — TIFF/BigTIFF encoding (none, LZW, deflate, packbits compression)
14. **png_encoder** — PNG 16‑bit encoding (deflate; ICC and EXIF chunk support)

#### CLI Application (`photopipeline-cli`)

- **pipeline** subcommand: `run` (execute pipeline), `validate` (validate pipeline config).
- **plugin** subcommand: `list` (list all registered plugins), `info <PLUGIN_ID>` (show plugin details).
- **batch** subcommand: `run` (batch processing with glob pattern), `validate` (batch config validation).
- TOML configuration loading via `serde` + `toml`.
- Progress bar indication via `indicatif`.

#### gRPC Server (`photopipeline-server`)

- **Protobuf definitions**: `pipeline.proto`, `image.proto`, `batch.proto`.
- **PipelineService**: `CreatePipeline`, `Execute` (streaming), `Validate`, `GetNodeSchema`.
- **ImageService**: `Load`, `Decode` (streaming), `Encode` (streaming), `GetThumbnail`.
- **BatchService**: `SubmitBatch`, `GetProgress` (streaming), `Cancel`.
- **tonic** implementation with Tokio async runtime.

#### Protobuf Message Types

- **PipelineSpec**, **PipelineNode**, **PipelineEdge**, **PipelineId** — pipeline creation and execution.
- **ExecuteRequest**, **ExecuteProgress** — streaming execution progress with stage tracking.
- **ValidationResult**, **ValidationIssue** — parameter validation feedback.
- **NodeSchema** — plugin schema transfer to GUI clients.
- **ImagePath**, **ImageInfo**, **MetadataInfo** — image metadata.
- **DecodeRequest**, **PixelDataChunk** — streaming image decode.
- **EncodeRequest**, **EncodeProgress** — streaming image encode.
- **BatchSpec**, **BatchId**, **BatchProgress** — batch job submission and progress monitoring.

#### Halide Generators (C++)

- `colorspace_generator.cpp` — colour space transformation kernel.
- `resize_generator.cpp` — image resize kernel (Lanczos3).
- `tonemap_generator.cpp` — HDR tone‑mapping kernel.
- `CMakeLists.txt` — build configuration for CI compilation.

#### OIIO FFI Bindings

- `crates/oiio/`: feature‑gated OpenImageIO FFI wrappers.
- `OIIO_read_image`, `OIIO_write_image`, `OIIO_free_image` extern declarations.

#### External Crate

- `crates/external/`: external tool wrapper stubs (ExifTool, libvips, commercial API hooks).

#### GUI

- **WinUI 3** (.NET 8) — Windows desktop application project skeleton.
- **SwiftUI** — macOS desktop application project skeleton.
- **GTK4 + Rust** — Linux desktop application project skeleton.

#### CI/CD

- **build-halide.yml** — cross‑platform Halide generator compilation (Linux, Windows, macOS matrix).
- **build-rust.yml** — cross‑platform Rust workspace build (linux‑x86_64, linux‑aarch64, windows‑x86_64, macos‑arm64, macos‑x86_64 matrix).
- **release.yml** — full release packaging pipeline (AppImage, MSIX, DMG).

#### Testing

- **Core**: 67 unit tests covering PixelBuffer, AlignedBuffer, TileLayout, ColorSpace, ColorRGB, Metadata, GPS, GPX interpolation, Version, Format, Backend display, schema defaults.
- **Plugin**: 20 unit tests covering Registry registration/query/unregistration, Manifest listing, Schema defaults/field lookup, ParameterSet get/set/merge/iterate.
- **Engine**: 43 unit tests covering Graph add/remove/connect/disconnect, topological sort, cycle detection, graph validation, serialization, Template validation/construction, ParameterResolver resolution/merging, condition evaluation (ExifEq/Gte/Lte/GpsNear/And/Or/Expression), expression engine (variable resolution, comparisons, ternary, literals, quoted strings), full priority chain.
- **Plugins**: validation logic embedded in all 14 built‑in plugins.
- **CLI**: 10 integration tests covering plugin counts, categories, graph construction, serialization, validation, query, display.
- **Total**: 168+ unit tests + 10 integration tests.

### Infrastructure

- Cargo workspace structure (6 workspace members + 1 excluded crate).
- `justfile` task runner: `just build`, `just check`, `just test`, `just lint`, `just fmt`, `just clean`, `just run-pipeline`, `just run-server`.
- `tracing` structured logging with `tracing-subscriber` env‑filter.
- `tokio` async runtime for engine, server, and CLI.
- `serde` + `serde_json` for serialization of all core types.
- `clap` for CLI argument parsing.
- `tonic` + `prost` for gRPC server and protobuf code generation.
- `dashmap` for concurrent plugin registry.
- `parking_lot` for synchronisation primitives.
- `chrono` for EXIF datetime handling and GPX interpolation.
- `uuid` for node/image/batch/port identifiers.
- `regex` for expression engine tokenisation.
- `glob` for batch file pattern matching.
- `indicatif` for CLI progress bars.
- `bytemuck` for zero‑copy byte slicing.
- `strum` for enum display and string conversion.
- `derive_builder` for builder pattern structs.

---

[0.1.0]: https://github.com/zhang-hz/photopipeline/releases/tag/v0.1.0
