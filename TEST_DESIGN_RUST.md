# Photopipeline Rust 端测试系统详细设计

**版本**: 1.0  
**日期**: 2026-05-26  
**范围**: Layer 0 (单元测试) + Layer 1 (管线集成测试) + Layer 2 (gRPC E2E 测试)  
**总量**: ~670 条测试用例  

---

## 目录

- [A. 目录结构设计](#a-目录结构设计)
- [B. Layer 0: Rust 单元测试详细设计](#b-layer-0-rust-单元测试详细设计)
- [C. Layer 1: Rust Pipeline 集成测试详细设计](#c-layer-1-rust-pipeline-集成测试详细设计)
- [D. Layer 2: Rust gRPC E2E 测试详细设计](#d-layer-2-rust-grpc-e2e-测试详细设计)
- [E. Shared JSON 格式定义](#e-shared-json-格式定义)
- [F. 实施步骤](#f-实施步骤)

---

## A. 目录结构设计

### A.1 现有结构

```
crates/
  plugins/src/              # 14 个插件的实现
  plugins/tests/            # 现有 plugin_tests.rs (11 个通用全插件测试)
  engine/src/               # graph.rs, params.rs, tile.rs, executor.rs
  engine/tests/             # (空目录, 需创建)
  server/src/               # lib.rs + services/{pipeline,image,batch}.rs
  server/tests/             # (不存在, 需创建)
tests/
  test_harness/src/         # 现有 assertions/fixtures/mocks
  e2e/src/                  # 现有 13 个 e2e 测试文件
shared/
  test_cases/               # (需创建) Layer 2/4/6 的 JSON 定义
```

### A.2 新增目录结构

```
crates/
  plugins/tests/
    plugin_tests.rs            # 保留现有的 11 个全插件通用测试
    schema_validation.rs       # NEW: 140 条 ParameterSchema 验证测试
    process_boundary.rs        # NEW: 70 条 process/tile 边界测试
  engine/tests/                # NEW 目录
    graph_tests.rs             # NEW: 30 条 DAG 拓扑排序测试
    parameter_resolver_tests.rs # NEW: 30 条 ParameterResolver 测试
    tile_engine_tests.rs       # NEW: 25 条 TileEngine 测试
    node_executor_tests.rs     # NEW: 20 条 NodeExecutor 测试
  server/tests/                # NEW 目录
    grpc_service_tests.rs      # NEW: 35 条 gRPC Service 验证测试
  test-defs/                   # NEW crate: JSON 解析
    Cargo.toml
    src/
      lib.rs                   # CrossTestCase struct + JSON 反序列化
      schema.rs                # JSON Schema 验证
tests/
  pipeline_integration/        # NEW 目录 (替代现有 e2e/)
    Cargo.toml
    tests/
      single_plugin.rs         # NEW: 56 条单插件 x 多图像
      param_variation.rs       # NEW: 28 条单插件参数变异
      multi_plugin_2chain.rs   # NEW: 40 条两插件链
      multi_plugin_3chain.rs   # NEW: 30 条三插件链
      multi_plugin_4plus.rs    # NEW: 20 条四+插件链
      format_roundtrip.rs      # NEW: 10 条格式往返
      boundary_conditions.rs   # NEW: 16 条边界条件
  grpc_e2e/                    # NEW 目录
    Cargo.toml
    tests/
      single_plugin.rs         # NEW: 40 条单插件 gRPC
      multi_plugin.rs          # NEW: 30 条多插件 gRPC
      format_roundtrip.rs      # NEW: 15 条格式 gRPC
      batch_operations.rs      # NEW: 15 条批处理 gRPC
      error_paths.rs           # NEW: 10 条错误路径 gRPC
      concurrency.rs           # NEW: 10 条性能/并发 gRPC
shared/
  test_cases/
    grpc_cases.json            # NEW: Layer 2/4 共享 (120 条)
    cross_chain_cases.json     # NEW: Layer 6 (60 条)
    schema.json                # NEW: JSON Schema
tests/
  fixtures/
    input/                     # NEW: 20 张测试输入图像
```

### A.3 目录布局理由

| 存放位置 | 用途 | 理由 |
|---------|------|------|
| `crates/plugins/tests/` | 插件单元测试 | 遵循 Rust 惯例，测试私有 API，`cargo test -p photopipeline-plugins` 即可 |
| `crates/engine/tests/` | Engine 单元测试 | 与实现同 crate，可访问非 pub 类型 |
| `crates/server/tests/` | gRPC service 单元测试 | 与实现同 crate，可 mock 内部状态 |
| `tests/pipeline_integration/` | Layer 1 集成测试 | 独立测试 binary，依赖 `test-harness` + 所有 crate |
| `tests/grpc_e2e/` | Layer 2 E2E 测试 | 独立的测试二进制，需要 `tonic` 的 test server 依赖 |
| `crates/test-defs/` | JSON 解析 | 被 Rust 和 C# 两端共享，独立 crate 方便序列化策略 |
| `shared/test_cases/` | JSON 定义 | 跨语言共享，与具体实现解耦 |


## B. Layer 0: Rust 单元测试详细设计

### B.1 14 插件 ParameterSchema 验证测试 (140 条)

#### B.1.1 通用验证 (56 条)

**测试文件**: `crates/plugins/tests/schema_validation.rs`

**14 插件的 plugin_id 常量**:
```rust
const PLUGIN_IDS: [&str; 14] = [
    "photopipeline.plugins.raw_input",
    "photopipeline.plugins.transform",
    "photopipeline.plugins.colorspace",
    "photopipeline.plugins.lut3d",
    "photopipeline.plugins.lens_correct",
    "photopipeline.plugins.ai_denoise",
    "photopipeline.plugins.exif_rw",
    "photopipeline.plugins.gps_set",
    "photopipeline.plugins.time_shift",
    "photopipeline.plugins.avif_encoder",
    "photopipeline.plugins.jxl_encoder",
    "photopipeline.plugins.heif_encoder",
    "photopipeline.plugins.tiff_encoder",
    "photopipeline.plugins.png_encoder",
];
```

**通用验证 (4 per plugin = 56)**:
| 测试名模式 | 实现逻辑 | 断言 |
|-----------|---------|------|
| {Plugin}_Schema_Is_NonEmpty | plugin.parameter_schema() sections | `assert!(!sections.is_empty())` |
| {Plugin}_Schema_All_Fields_Have_Defaults | 遍历每个 field | field.default != Value::Null |
| {Plugin}_Schema_Defaults_Match_Field_Types | default JSON 类型匹配 field_type | Integer->is_number, String->is_string |
| {Plugin}_Schema_Enum_Options_Valid | 遍历 Enum options | `!options.is_empty()`; value/label 非空 |

#### B.1.2 参数类型专项验证 (84 条)

| 测试函数 | 核心逻辑 |
|---------|---------|
| raw_input_RawMode_Enum_HasAllOptions | raw_mode 含 auto/dcraw/libraw/rawtherapee (4) |
| raw_input_OutputFormat_Defaults_U16 | default == json("u16") |
| transform_ScalePercent_Range | scale_percent Integer { min: 1, max: 400 } |
| transform_Angle_Accepts_Negative | angle Integer { min: -360, max: 360 } |
| transform_FilterType_Enum_Complete | filter_type 含 bilinear/lanczos3/nearest |
| colorspace_SourceSpace_8Options | source_color_space 8 options |
| colorspace_RenderingIntent_4Options | rendering_intent 4 options |
| lut3d_LutFormat_4Formats | lut_format 4 options |
| lut3d_Intensity_Range | intensity Float { min: 0.0, max: 100.0 } |
| lens_correct_Mode_3Options | correction_mode 含 auto/manual/off |
| ai_denoise_Strength_Range | denoise_strength { min: 0, max: 100 } |
| exif_rw_WriteExif_Enum | write_exif 含 preserve/custom/clear |
| gps_set_Mode_3Options | gps_mode 含 manual/gpx_track/clear |
| gps_set_Latitude_Range | latitude Float { min: -90.0, max: 90.0 } |
| time_shift_Hours_Range | shift_hours { min: -23, max: 23 } |
| avif_Quality_Range | quality { min: 0, max: 100 } |
| avif_Chroma_Enum_3Options | chroma_subsampling 含 444/422/420 |
| jxl_Quality_Range | quality { min: -1, max: 100 } |
| jxl_Effort_Range | effort { min: 1, max: 9 } |
| heif_Quality_Range | quality { min: 0, max: 100 } |
| tiff_Compression_Enum | compression 含 none/lzw/deflate/packbits |
| png_ColorType_Enum | color_type 含 rgb/rgba/gray/graya |
| png_Compression_Range | compression_level { min: 0, max: 9 } |

#### 额外参数测试 (建议加入)
| 测试函数 | 内容 |
|---------|------|
| raw_input_WhiteBalance_Boolean | apply_white_balance 是 Boolean 类型 |
| transform_RotateMode_Enum | rotate_mode Enum 含 5 个值 |
| colorspace_BpComp_Boolean | bp_comp 是 Boolean 类型 |
| ai_denoise_DetailPreservation_Range | detail_preservation { min: 0, max: 100 } |
| exif_rw_ReadXmp_Boolean | read_xmp 是 Boolean 类型 |
| time_shift_SourceTz_Default | source_tz default = UTC |
| tiff_Encoder_PixelFormat_Enum | pixel_format Enum 含 u8/u16 |

### B.2 14 插件 process/tile 边界测试 (70 条)

**测试文件**: crates/plugins/tests/process_boundary.rs

**PixelProcessor 组** (5 processors x 5 = 25):
| 测试 | 输入 | 预期 |
|------|------|------|
| {Plugin}_Process_EmptyInput_ReturnsError | 0x0 PixelBuffer | Err(PluginError) |
| {Plugin}_Process_NullMetadata_NoCrash | Metadata::default() | 不 panic |
| {Plugin}_Tile_SingleTile_SameAsProcess | TileEngine 单瓦片 vs process | 逐像素等同 |
| {Plugin}_Tile_OutOfBounds_Clamped | 越界坐标 | clamp 有效范围 |
| {Plugin}_Process_Cancelled_MidExecution | MockProgressSink.cancel() | Err(PluginError::Canceled) |

**FormatProcessor 组** (6 processors x 5 = 30):
| 测试 | 输入 | 预期 |
|------|------|------|
| {Plugin}_Process_EmptyInput_ReturnsError | encode 0x0 buffer | Err(PluginError) |
| {Plugin}_Process_NullMetadata_NoCrash | encode 空 Metadata | 不 panic |
| {Plugin}_Tile_SingleTile_SameAsProcess | decode 单次 vs 分片 | 相同 |
| {Plugin}_Tile_OutOfBounds_Clamped | N/A | 跳过 |
| {Plugin}_Process_Cancelled_MidExecution | progress cancel | Err(PluginError::Canceled) |

**MetadataProcessor 组** (3 processors x 5 = 15):
| 测试 | 输入 | 预期 |
|------|------|------|
| {Plugin}_Process_EmptyInput_ReturnsError | write_metadata 空 target | Err(PluginError) |
| {Plugin}_Process_NullMetadata_NoCrash | 空 Metadata | 不 panic |
| {Plugin}_Tile_SingleTile_SameAsProcess | N/A | 跳过 |
| {Plugin}_Tile_OutOfBounds_Clamped | N/A | 跳过 |
| {Plugin}_Process_Cancelled_MidExecution | progress cancel | 优雅处理 |

### B.3 Engine Graph/DAG 拓扑排序测试 (30 条)

**文件**: crates/engine/tests/graph_tests.rs

**假设**: 存在 topological_sort(graph: &PipelineGraph) -> Result<Vec<NodeId>, GraphError>

| # | 函数名 | 构造 | 断言 |
|---|--------|------|------|
| 1 | linear_3nodes_correct_order | A->B->C | Ok([A,B,C]) |
| 2 | diamond_4nodes | A->B, A->C, B->D, C->D | A 在 B/C 前, B/C 在 D 前 |
| 3 | branch_then_merge | A->B->C, A->D->C | 二合法序之一 |
| 4 | single_node_trivial | 仅 A | Ok([A]) |
| 5 | empty_graph | 空 | Ok([]) |
| 6 | disconnected_components | A->B, C->D | Ok 含 4 节点 |
| 7 | simple_cycle | A->B->C->A | Err(CircularDependency) |
| 8 | self_loop | A->A | Err(CircularDependency) |
| 9 | complex_cycle_5nodes | A->B->C->D->E->B | Err(CircularDependency) |
| 10 | cycle_with_diamond | 钻石反向边 | Err(CircularDependency) |
| 11 | edge_missing_source | 边缺 from | Err(InvalidEdge) |
| 12 | edge_missing_target | 边缺 to | Err(InvalidEdge) |
| 13 | duplicate_edge | A->B 重复 | Err(DuplicateEdge) |
| 14 | max_nodes_1000_linear | 1000 节点链 | 排序 < 50ms |
| 15 | deep_diamond_20_levels | 20 层钻石 | 合法 |
| 16 | wide_fan_out_100 | A->B1..B100 | 所有 B 在 A 后 |
| 17 | wide_fan_in_100 | A1..A100->B | 所有 A 在 B 前 |
| 18 | disabled_node_skipped | A->B(disabled)->C | [A,C] |
| 19 | all_disabled_empty | 全 disabled | Ok([]) |
| 20 | mixed_enabled_disabled | diamond 有 disabled | 只含 enabled |
| 21 | parallel_branch_independence | A->(B,C,D)->E | B/C/D 无序 A后E前 |
| 22 | multiple_entry_points | A->C, B->C | A,B 在 C 前 |
| 23 | multiple_exit_points | A->B, A->C | B,C 在 A 后 |
| 24 | repeated_single_edge | edges 两次 A->B | Err(DuplicateEdge) |
| 25 | consecutive_disabled | A->B(d)->C(d)->D | [A,D] |
| 26 | first_node_disabled | A(d)->B | [B] |
| 27 | last_node_disabled | A->B(d) | [A] |
| 28 | sort_stability | 同一图 10 次 | 每次相同 |
| 29 | node_id_special_chars | id 含 -, _, 数字 | 正常 |
| 30 | large_dense_50_200 | 50 nodes, 200 edges | < 200ms |

### B.4 Engine ParameterResolver 测试 (30 条)

**文件**: crates/engine/tests/parameter_resolver_tests.rs

| # | 函数名 | 场景 | 断言 |
|---|--------|------|------|
| 1 | default_layer_wins | 只设 template_params | 输出=默认值 |
| 2 | template_overrides_default | template x=2 > default x=1 | x=2 |
| 3 | group_overrides_template | group x=3 > template x=2 | x=3 |
| 4 | image_override_wins | 四层都设值 | output=image override |
| 5 | allow_override_false | allow_override=false | 高层被忽略 |
| 6 | expr_variable_substitution | ${exif.iso} | 替换为 800 |
| 7 | expr_ternary_true | iso>=400 ? high : low, iso=800 | high |
| 8 | expr_ternary_false | iso>=400 ? high : low, iso=200 | low |
| 9 | expr_nested_ternary | a?(b?1:2):3 | 正确求值 |
| 10 | expr_comparison_gt | width>1000 ? large : small | large |
| 11 | expr_arithmetic | exif.iso / 100 | 8 |
| 12 | expr_undefined_variable | ${nonexistent} | Err |
| 13 | expr_divide_by_zero | ${1/0} | Err |
| 14 | expr_malformed_syntax | ${???} | Err |
| 15 | expr_string_concat | IMG_ + exif.iso | IMG_800 |
| 16 | cond_exifeq_match | ExifEq(make, SONY) | 应用 |
| 17 | cond_exifeq_nomatch | ExifEq(make, SONY) 实际 Canon | 跳过 |
| 18 | cond_gpsnear_match | GpsNear(39.9,116.4,100km) | 应用 |
| 19 | cond_gpsnear_nomatch | GpsNear(39.9,116.4,1km) 在100km外 | 跳过 |
| 20 | cond_and_bothtrue | And(ExifEq, GpsNear) | 应用 |
| 21 | cond_or_onetrue | Or(ExifEq, GpsNear) 仅 ExifEq | 应用 |
| 22 | cond_exifgte_match | ExifGte(iso, 800) | 应用 |
| 23 | resolve_all_params | 部分有值 | 所有有输出 |
| 24 | resolve_required_missing | required 无值 | Err(MissingRequiredField) |
| 25 | resolve_type_mismatch | Int 字段 String 值 | 类型错误 |
| 26 | expr_metadata_chain | exif.make_model 链式 | 正确解析 |
| 27 | cond_exiflt_nomatch | ExifLte(iso,400) iso=800 | 跳过 |
| 28 | cond_expression | Expression(iso>500) | 应用 |
| 29 | resolve_empty | 无参数定义 | 空 ParameterSet |
| 30 | resolve_mixed_types | Str+Int+Float+Bool | 全部正确 |

### B.5 Engine TileEngine 测试 (25 条)

**文件**: crates/engine/tests/tile_engine_tests.rs

| # | 函数 | 图像尺寸 | tile_size | 断言 |
|---|------|---------|-----------|------|
| 1 | single_tile_equals_full | 512x512 | 1024 | 单瓦片==全图 |
| 2 | two_tiles_horizontal | 2048x1024 | 1024 | 左+右==原图 |
| 3 | two_tiles_vertical | 1024x2048 | 1024 | 上+下==原图 |
| 4 | four_tiles_2x2 | 2048x2048 | 1024 | 4块==原图 |
| 5 | overlap_blended | 2048x1024 | 1024, overlap=64 | 重叠正确混合 |
| 6 | tile_larger_than_image | 512x512 | 1024 | 单瓦片处理 |
| 7 | non_divisible_tile | 1500x1500 | 1024 | 最后行列正确裁剪 |
| 8 | boundary_no_artifacts | 2049x2049 | 1024 | 无接缝伪影 |
| 9 | empty_0x0 | 0x0 | 1024 | 空或 Err |
| 10 | single_row | 1x1024 | 1024 | 正确处理 |
| 11 | single_column | 1024x1 | 1024 | 正确处理 |
| 12 | tile_count_max | 8000x4000 | 1024 | 32 tiles |
| 13 | cancel_mid_tile | 4 tiles 第3个cancel | Canceled, 仅2完成 |
| 14 | progress_correct | 4 tiles | 0.25/0.50/0.75/1.00 |
| 15 | huge_overlap_small | overlap > image | 优雅处理 |
| 16 | zero_overlap | overlap=0 | 无混合 |
| 17 | non_square_tiles | 1000x2000, tile=512 | 非正方形瓦片 |
| 18 | odd_boundary | 1001x1001, tile=128 | 边界正确 |
| 19 | progress_01 | 1 tile | 0.0, 1.0 |
| 20 | multiple_cancels | 多次 cancel | 不 panic |
| 21 | error_propagates | FailProcessor | Err 传播 |
| 22 | max_parallel | max_parallel>0 | 不 panic |
| 23 | default_creation | TileEngine::new(1024,64,4) | 字段正确 |
| 24 | format_preserved | U16 RGB 输入 | 同格式输出 |
| 25 | colorspace_preserved | Linear sRGB | 同色域 |

### B.6 Engine NodeExecutor 测试 (20 条)

**文件**: crates/engine/tests/node_executor_tests.rs

| # | 函数名 | 图 | 断言 |
|---|--------|-----|------|
| 1 | single_node | A(colorspace) | 输出正确, Completed |
| 2 | linear_2nodes | A->B | 都执行, 顺序正确 |
| 3 | failure_propagates | A(失败)->B | A=Failed, B=NotStarted |
| 4 | cancel_propagates | A->B(取消)->C | B=Canceled, C=NotStarted |
| 5 | disabled_node | A->B(disabled)->C | B=Skipped, A->C |
| 6 | all_disabled | 全 disabled | 无错误无输出 |
| 7 | concurrent_branches | A->B, A->C | B,C 可并行 |
| 8 | diamond_merge | A->B, A->C, B->D, C->D | D 在 B,C 后 |
| 9 | progress_aggregation | 多节点 | 总进度跨所有节点 |
| 10 | metadata_passthrough | 处理->编码 | 元数据传递 |
| 11 | empty_graph | 空图 | Err |
| 12 | single_node_fail | process 返回 Err | Failed |
| 13 | multiple_processors_chain | 3 个串联 | 全部处理 |
| 14 | metadata_in_chain | metadata + pixel | 元数据+像素 |
| 15 | disabled_middle_3 | A->B(d)->C | A->C |
| 16 | single_disabled | 1 disabled | 空输出 |
| 17 | progress_aggregation_3 | 3 节点各 33% | sum=100% |
| 18 | resolver_overrides | override 参数 | 覆盖生效 |
| 19 | idempotent | 同管线 2 次 | 结果相同 |
| 20 | no_processor_skipped | 无匹配 processor | Skipped |

### B.7 gRPC Service 验证测试 (35 条)

**文件**: crates/server/tests/grpc_service_tests.rs

**Mock SharedState**: make_state() 创建 Arc<SharedState> 含已注册插件

**Protobuf 序列化往返 (10 条)**:
1-10: pipeline_spec / execute_request / pipeline_id / image_path / decode_request
    / encode_request / batch_spec / batch_id / progress_proto / validation_result
    步骤: 构建 -> 序列化到 bytes -> 反序列化 -> assert_eq 字段

**PipelineService 请求验证 (11-20)**:
| # | 函数 | 输入 | 预期 |
|---|------|------|------|
| 11 | create_empty_nodes | PipelineSpec { nodes: [] } | Status::invalid_argument |
| 12 | create_invalid_plugin | plugin_id = nonexistent | Status::not_found |
| 13 | execute_invalid_pipeline_id | 不存在 UUID | Status::not_found |
| 14 | execute_empty_input | input_path = empty | Status::invalid_argument |
| 15 | execute_empty_output | output_path = empty | Status::invalid_argument |
| 16 | validate_empty_spec | 空 spec | valid=false |
| 17 | validate_cycle | A->B->C->A | valid=false (cycle issue) |
| 18 | validate_valid | 标准管线 | valid=true |
| 19 | get_node_schema_valid | 存在的 plugin_id | schema 非空 |
| 20 | get_node_schema_invalid | 不存在的 plugin_id | Status::not_found |

**ImageService 请求验证 (21-30)**:
| # | 函数 | 输入 | 预期 |
|---|------|------|------|
| 21 | load_nonexistent | 不存在路径 | Status::not_found |
| 22 | decode_unsupported_format | .xyz 后缀 | Status::invalid_argument |
| 23 | encode_invalid_options | 负 quality | Status::invalid_argument |
| 24 | encode_empty_data | 空 pixel_data | Status::invalid_argument |
| 25 | encode_zero_dimensions | 0x0 | Status::invalid_argument |
| 26 | encode_unsupported_format | format=xyz | Status::invalid_argument |
| 27 | thumbnail_nonexistent | 不存在路径 | Status::not_found |
| 28 | load_empty_path | path=empty | Status::invalid_argument |
| 29 | decode_empty_path | path=empty | Status::invalid_argument |
| 30 | encode_output_empty | output_path=empty | Status::invalid_argument |

**BatchService 请求验证 (31-35)**:
| # | 函数 | 输入 | 预期 |
|---|------|------|------|
| 31 | create_no_files | file_pattern=empty | Status::invalid_argument |
| 32 | create_missing_pipeline | pipeline_config_path=empty | Status::invalid_argument |
| 33 | get_progress_invalid_id | 不存在 UUID | Status::not_found |
| 34 | cancel_invalid_id | 不存在 UUID | Status::not_found |
| 35 | create_invalid_pattern | 无匹配文件 | Status::invalid_argument |

## C. Layer 1: Rust Pipeline 集成测试详细设计

### C.0 概述

- **总量**: ~200 条
- **测试框架**: Rust #[test] + tokio runtime
- **运行**: cargo test -p photopipeline-integration-tests
- **复用**: test_harness 的 assertions/ 和 fixtures/
- **核心原则**: 不使用 I/O, 所有图像在内存中构建, 每条测试必须包含可 FAIL 的像素级断言

### C.1 测试输入图像生成 (20 张)

**生成器**: 所有图像由 ImageFixture builder 生成, 签入 tests/fixtures/input/

| ID | 文件 | 生成方式 |
|---|------|---------|
| I01 | solid_color_1920.png | ImageFixture 1920x1080 solid(120,150,200) |
| I02 | adobergb_wide_1920.tiff | ImageFixture + AdobeRGB -> TIFF |
| I03 | web_photo_800.jpg | ImageFixture 800x600 -> JPEG Q=92 |
| I04 | 4k_highres_3840.png | ImageFixture 3840x2160 gradient -> PNG |
| I05 | displayp3_wide_1920.png | ImageFixture + DisplayP3 -> PNG |
| I06 | noisy_texture_1920.png | ImageFixture + 人工噪声 -> PNG |
| I07 | barrel_distortion_1920.png | ImageFixture + 桶形畸变模拟 -> PNG |
| I08 | pincushion_vignette_1920.png | ImageFixture + 枕形畸变+暗角 -> PNG |
| I09 | grayscale_1024.png | ImageFixture Gray 256-step -> PNG |
| I10 | high_bitdepth_1920.tiff | ImageFixture U16 gradient -> TIFF |
| I11 | camera_jpeg_exif.jpg | ImageFixture + exif_sony_a7r5 -> JPEG |
| I12 | alpha_transparent_1024.png | ImageFixture RGBA 棋盘格 -> PNG |
| I13 | icon_tiny_256.png | ImageFixture 256x256 solid -> PNG |
| I14 | panorama_wide_8000.png | ImageFixture 8000x4000 gradient -> PNG |
| I15 | cmyk_print_1920.tiff | 特殊 CMYK 构造 -> TIFF |
| I16 | zone_plate_test_1920.png | 正弦波带片 -> PNG |
| I17 | color_checker_1920.png | 24 色 ColorChecker -> PNG |
| I18 | gradient_all_1920.png | 复合渐变 -> PNG |
| I19 | single_pixel_1x1.png | 1x1 solid white -> PNG |
| I20 | extreme_aspect_100x65535.png | 100x65535 gradient -> PNG |

### C.2 单插件集成测试 (84 条)

#### C.2.1 单插件 x 多图像 (56 条)

**文件**: tests/pipeline_integration/tests/single_plugin.rs

**通用模式**: 
1. 从 input_path 解码图像 -> PixelBuffer
2. 构建 PipelineTemplate 含 {source, plugin_node, encoder}
3. NodeExecutor 执行
4. 解码输出 buffer
5. 执行 assertion (assert_pixels_eq / assert_buffer_dimensions / compute_psnr)

**14 插件 x 4 图像分配**:
| 插件 | 图像组合 |
|------|---------|
| raw_input | I01(auto), I02(dcraw+5500K), I10(u16), I11(libraw) |
| transform | I01(crop 50%), I03(resize 200%), I04(rotate 90), I13(flip HV) |
| colorspace | I01(sRGB->AdobeRGB), I05(sRGB->P3), I03(sRGB->Gray), I09(Gray->sRGB) |
| lut3d | I01(warm.cube), I03(film.cube), I06(extreme.cube), I18(cool.cube) |
| lens_correct | I07(auto), I08(full), I07(TCA), I08(manual) |
| ai_denoise | I06(strength=20), I06(strength=50), I06(strength=90), I04(strength=30) |
| exif_rw | I11(preserve), I01(write custom), I11(clear), I11(read_xmp) |
| gps_set | I01(manual), I11(clear), I11(gpx_track), I01(altitude=100) |
| time_shift | I11(+1h), I11(-24h), I11(timezone), I11(+30m) |
| avif_encoder | I01(Q=50), I01(Q=100,10bit), I01(lossless), I05(Q=75) |
| jxl_encoder | I01(Q=50), I01(lossless), I10(Q=100,16bit), I14(Q=80) |
| heif_encoder | I01(Q=80), I01(Q=100,10bit), I01(chroma=444) |
| tiff_encoder | I01(deflate), I10(zip+u16), I04(lzw+4K) |
| png_encoder | I01(rgb), I10(16bit), I12(rgba) |

#### C.2.2 单插件参数变异 (28 条)

**文件**: tests/pipeline_integration/tests/param_variation.rs

| # | 测试 | 参数 A | 参数 B | 预期 |
|---|------|--------|--------|------|
| 1 | transform_resize_50_vs_200 | scale=50% | scale=200% | 输出大小比例 1:4 |
| 2 | transform_rotate_45_vs_180 | angle=45 | angle=180 | 输出像素不同 |
| 3 | colorspace_6_rendering_intents | 6 种 intent 遍历 | - | 每种产生不同 pixel |
| 4 | colorspace_bpcomp_on_vs_off | bp_comp=true | bp_comp=false | 不同输出 |
| 5 | lut3d_intensity_0_50_100 | intensity=0 | intensity=50,100 | 强度递增 |
| 6 | lut3d_trilinear_vs_tetrahedral | interp=trilinear | interp=tetrahedral | SSIM 不同 |
| 7 | lens_correct_auto_vs_manual_vs_off | 三种 mode | - | off 无校正 |
| 8 | ai_denoise_strength_0_50_100 | strength=0 | strength=50,100 | PSNR 递增 |
| 9 | avif_encoder_444_vs_420_vs_422 | 3 种 chroma | - | 文件大小不同 |
| 10 | jxl_encoder_modular_true_vs_false | modular=true | modular=false | 输出不同 |
| 11 | tiff_encoder_4_compression | 4 种 compression | - | 文件大小不同 |
| 12 | png_encoder_compression_0_vs_9 | level=0 | level=9 | 像素相同, 大小不同 |
| 13 | png_encoder_4_color_types | 4 种 color_type | - | 通道数正确 |
| 14 | heif_encoder_quality_10_50_90 | q=10 | q=50, q=90 | 文件大小递增 |

### C.3 多插件链集成测试 (90 条)

#### C.3.1 两插件链 (40 条)

**文件**: tests/pipeline_integration/tests/multi_plugin_2chain.rs

**20 组合 x 2 图像 = 40 条**

| # | 管线 | 图像 | 验证点 |
|---|------|------|--------|
| 1 | raw_input->colorspace | I01, I03 | 中间+最终 |
| 2 | raw_input->transform(crop) | I01, I03 | 裁剪后尺寸 |
| 3 | raw_input->lut3d | I01, I03 | LUT 色彩变化 |
| 4 | raw_input->tiff_encoder | I10, I04 | TIFF 格式+像素 |
| 5 | transform(rotate)->colorspace | I01, I04 | 旋转后色彩转换 |
| 6 | transform(resize)->png_encoder | I01, I13 | 缩放后 PNG |
| 7 | colorspace->tiff_encoder(ICC) | I01, I05 | ICC 嵌入 |
| 8 | colorspace->avif_encoder | I01, I05 | AVIF 含色彩空间 |
| 9 | lut3d->png_encoder | I01, I03 | LUT 效果保留 |
| 10 | lens_correct->tiff_encoder | I07, I08 | TIFF 校正后 |
| 11 | ai_denoise->jxl_encoder | I06, I04 | JXL 降噪后 |
| 12 | exif_rw->tiff_encoder | I11, I01 | TIFF 元数据 |
| 13 | gps_set->png_encoder | I01, I11 | PNG GPS 数据 |
| 14 | time_shift->tiff_encoder | I11, I01 | TIFF EXIF 偏移 |
| 15 | colorspace->jxl_encoder(lossless) | I01, I02 | lossless 像素完整 |
| 16 | ai_denoise->colorspace | I06 | 降噪后色彩转换 |
| 17 | transform(crop+resize)->avif_encoder | I01, I04 | 复合变换后 AVIF |
| 18 | lens_correct->colorspace | I07, I08 | 校正后色彩转换 |
| 19 | colorspace->lut3d | I01, I03 | 先转色彩再套 LUT |
| 20 | raw_input->heif_encoder | I10, I11 | RAW->HEIF 直出 |

#### C.3.2 三插件链 (30 条)

**文件**: tests/pipeline_integration/tests/multi_plugin_3chain.rs

| # | 管线(15组合x2图像=30) | 验证策略 |
|---|----------------------|---------|
| 1 | transform->colorspace->tiff_encoder | 全链像素验证 |
| 2 | raw_input->colorspace->png_encoder | RAW 显影到 PNG |
| 3 | ai_denoise->colorspace->jxl_encoder | 降噪+色彩+JXL |
| 4 | lens_correct->colorspace->tiff_encoder | 校正+色彩+TIFF |
| 5 | raw_input->transform->avif_encoder | 裁剪后 AVIF |
| 6 | exif_rw->colorspace->tiff_encoder | 元数据全链传递 |
| 7 | colorspace->lut3d->png_encoder | LUT 效果持久化 |
| 8 | raw_input->ai_denoise->tiff_encoder | 降噪后归档 |
| 9 | transform->lut3d->jxl_encoder | 缩放+LUT+JXL |
| 10 | gps_set->time_shift->tiff_encoder | 元数据链加工 |
| 11 | colorspace->transform(crop)->avif_encoder | 色彩+裁剪+导出 |
| 12 | lens_correct->ai_denoise->png_encoder | 光学+降噪+PNG |
| 13 | raw_input->lens_correct->heif_encoder | RAW 校正+HEIF |
| 14 | transform(resize)->colorspace->jxl_encoder | 缩放+色彩+无损 |
| 15 | ai_denoise->lut3d->tiff_encoder | 降噪+LUT+归档 |

#### C.3.3 四+插件链 (20 条)

**文件**: tests/pipeline_integration/tests/multi_plugin_4plus.rs

| # | 管线(10组合x2图像=20) | 节点数 |
|---|----------------------|--------|
| 1 | raw->lens_correct->colorspace->tiff | 4 |
| 2 | raw->ai_denoise->colorspace->lut3d->png | 5 |
| 3 | transform->colorspace->lut3d->jxl(lossless) | 4 |
| 4 | raw->colorspace->transform(crop)->avif | 4 |
| 5 | ai_denoise->lens_correct->colorspace->tiff | 4 |
| 6 | exif_rw->gps_set->time_shift->colorspace->tiff | 5 (元数据链) |
| 7 | raw->lens_correct->colorspace->lut3d->jxl | 5 |
| 8 | transform(rotate->crop->resize)->colorspace->png | 3 (复合变换) |
| 9 | ai_denoise->colorspace->transform->lut3d->avif | 5 |
| 10 | raw->lens_correct->ai_denoise->colorspace->tiff(16bit) | 5 |

### C.4 格式往返测试 (10 条)

**文件**: tests/pipeline_integration/tests/format_roundtrip.rs

| # | 流程 | 验证 |
|---|------|------|
| 1 | PNG->decode->PNG(encode) | assert_pixels_eq(tolerance=0) |
| 2 | PNG->decode->TIFF(encode)->decode->PNG(encode) | 像素完全相同 |
| 3 | TIFF->decode->TIFF(encode) | 像素完全相同 |
| 4 | JPEG->decode->PNG(encode) | PSNR>40dB (跳过有损) |
| 5 | PNG->decode->AVIF(lossless)->decode->PNG(encode) | 像素完全相同 |
| 6 | PNG->decode->JXL(lossless)->decode->PNG(encode) | 像素完全相同 |
| 7 | 16bit TIFF->decode->16bit PNG(encode) | assert_pixel_format(U16) |
| 8 | RGBA PNG->decode->TIFF(encode)->decode | assert_channel_count(4) |
| 9 | Gray PNG->decode->RGB->decode->Gray PNG | 灰度数值保真 |
| 10 | CMYK TIFF->decode->sRGB PNG(encode) | compute_delta_e < 2 |

### C.5 边界条件测试 (16 条)

**文件**: tests/pipeline_integration/tests/boundary_conditions.rs

| # | 测试 | 场景 | 断言 |
|---|------|------|------|
| 1 | empty_pipeline | nodes=[] | Err 或无输出 |
| 2 | single_pixel_image | I19 -> transform(resize 200%) | 输出 2x2 |
| 3 | extreme_aspect_ratio | I20 -> colorspace | 输出 100x65535 |
| 4 | max_pipeline_100_nodes | A1->A2->...->A100 | 全部完成 |
| 5 | giant_lut_64cube | 大型 Cube LUT 文件 | 不 OOM |
| 6 | parallel_4_pipelines | 4 条管线同时 | 全部正确 |
| 7 | parallel_8_pipelines | 8 条管线 | 无数据竞争 |
| 8 | large_image_8000x4000 | I14 完整处理 | 瓦片正确拼接 |
| 9 | all_disabled_pipeline | 全 disabled | 输入直通 |
| 10 | no_encoder_pipeline | 只有 transform | 输出 = 最后 buffer |
| 11 | mixed_pixel_metadata | raw->exif_rw->colorspace | 两类共存 |
| 12 | cancel_mid_pipeline | 5 节点链第 3 个 cancel | 前 2 完成, 后 2 未启动 |
| 13 | diamond_disabled | A->B(d)->C, A->D->C | D 正常, B 跳过 |
| 14 | deterministic | 同管线 10 次 | 结果相同 |
| 15 | different_images | I01 和 I03 同管线 | 各自正确 |
| 16 | param_boundary_min_max | 每 Int/Float 参数 min/max | 正确执行 |

## D. Layer 2: Rust gRPC E2E 测试详细设计

### D.0 概述

- **总量**: ~120 条
- **测试框架**: Rust #[tokio::test] + tonic client
- **运行**: cargo test -p photopipeline-e2e-tests --features e2e-grpc
- **依赖**: 测试期间启动真实 gRPC 服务器 (127.0.0.1:0 随机端口)
- **复用**: test_harness 的 assertions/, fixtures/, mocks/
- **铁律遵守**: 所有测试必须连接到真实服务器并验证像素值, 不可跳过或仅检查 File.Exists

#### D.0.1 服务器启动模式

所有 E2E 测试共享一个服务器启动/关闭生命周期:

```rust
// tests/e2e/src/grpc_server.rs (测试辅助)
pub struct TestServer {
    addr: SocketAddr,
    shutdown: oneshot::Sender<()>,
}

impl TestServer {
    /// 启动真实服务器, 返回 gRPC 地址
    pub async fn start() -> Self {
        let state = SharedState::new(Registry::new(), PathBuf::from("test_plugins"));
        let svc = PipelineApp::new(state);
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        // ... tonic transport 启动
    }
}
```

#### D.0.2 客户端工厂

```rust
// tests/e2e/src/grpc_client.rs (测试辅助)
pub fn create_image_client(addr: &SocketAddr) -> ImageServiceClient<Channel> {
    ImageServiceClient::new(Channel::from_shared(format!("http://{}", addr)).unwrap())
}
pub fn create_pipeline_client(addr: &SocketAddr) -> PipelineServiceClient<Channel> { ... }
pub fn create_batch_client(addr: &SocketAddr) -> BatchServiceClient<Channel> { ... }
```

#### D.0.3 验证流水线

所有 E2E 测试必须执行以下验证: 
1. **RPC 状态码**: 确认 tonic Status 为 Ok 或预期的 Error 类型
2. **图像存在**: 输出文件存在于磁盘 (仅基本检查)
3. **像素验证**: 使用 assert_pixels_eq() 或 compute_psnr() 对比预期图像 (铁律 1)
4. **维度正确**: 输出宽度/高度符合预期 (铁律 5)
5. **格式正确**: 输出文件扩展名和内部格式一致

### D.1 ImageService E2E 测试 (30 条)

**文件**: tests/e2e/tests/image_service_e2e.rs

#### D.1.1 Load RPC (8 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | load_valid_png | I01.png (64x64 RGB) | ImageInfo.width=64, height=64, format=PNG |
| 2 | load_valid_tiff | I11.tiff (1024x1024 RGBA) | ImageInfo.width=1024, format=TIFF |
| 3 | load_valid_jpeg | I06.jpg (1920x1080) | ImageInfo.width=1920, format=JPEG |
| 4 | load_16bit_tiff | I15.tiff (4096x4096 U16) | ImageInfo.pixel_format=U16 |
| 5 | load_raw_file | I09.raw (6000x4000) | ImageInfo.width=6000, format=RAW |
| 6 | load_nonexistent | /nonexistent.png | Status::not_found |
| 7 | load_empty_path | "" | Status::invalid_argument |
| 8 | load_directory_path | /tmp | Status::invalid_argument |

#### D.1.2 Decode RPC (10 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | decode_png_stream | I01.png | 收到 PixelDataChunk, final chunk 含完整数据 |
| 2 | decode_tiff_stream | I11.tiff | chunk 总数 > 1 (大文件分块) |
| 3 | decode_raw_stream | I09.raw | 解码成功, 像素数据非空 |
| 4 | decode_with_opts_rgb | I01.png -> DecodeOptions{rgb=true} | 3 通道 |
| 5 | decode_with_opts_no_alpha | I11.tiff -> DecodeOptions{alpha=false} | 3 通道 (移除 alpha) |
| 6 | decode_thumbnail_flag | I14.png -> DecodeOptions{thumbnail=true} | 宽高 < 原图 |
| 7 | decode_cancel_mid_stream | Cancel 之后 | stream 终止 (Err 或空) |
| 8 | decode_invalid_format | I01.png -> DecodeOptions{format=JPEG} | Err (格式不匹配) |
| 9 | decode_corrupted_file | I08.bin (非图像) | Err |
| 10 | decode_zero_size | 空文件 | Err |

#### D.1.3 Encode RPC (8 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | encode_png | PixelData(64x64 RGB) -> PNG | 输出文件是有效 PNG |
| 2 | encode_tiff_lossless | PixelData(1024x1024 RGBA U16) -> TIFF | 像素无损 |
| 3 | encode_jpeg_quality | PixelData(1920x1080 RGB) -> JPEG q=90 | 文件生成, compute_psnr > 35dB |
| 4 | encode_jxl_lossless | PixelData(64x64) -> JXL lossless | 文件生成 |
| 5 | encode_avif_lossless | PixelData(64x64) -> AVIF lossless | 文件生成 |
| 6 | encode_invalid_format | PixelData -> enum UNKNOWN | Err |
| 7 | encode_empty_buffer | 0x0 PixelData | Err |
| 8 | encode_progress_stream | 大文件 -> TIFF | stream 包含进度消息 |

#### D.1.4 GetThumbnail RPC (4 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | thumbnail_png_256 | I01.png -> 256x256 | 输出 = 256x256 |
| 2 | thumbnail_tiff_512 | I11.tiff -> 512x512 | 输出 = 512x512 |
| 3 | thumbnail_no_larger_than_input | I19(1x1) -> 256x256 | 输出保留 1x1 (不放大) |
| 4 | thumbnail_16bit_quantize | I15(U16) -> thumbnail | assert_pixel_format(U8 或 U16) |

### D.2 PipelineService E2E 测试 (20 条)

**文件**: tests/e2e/tests/pipeline_service_e2e.rs

#### D.2.1 CreatePipeline RPC (6 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | create_minimal_pipeline | 单个 colorspace 节点 | 返回 PipelineId, id 非空 |
| 2 | create_5_node_chain | colorspace->transform->lut3d->exif_rw->png_out | 返回 PipelineId |
| 3 | create_diamond_graph | A->B, A->C, B->D, C->D | 返回 PipelineId |
| 4 | create_empty_pipeline | nodes=[] | Status::invalid_argument |
| 5 | create_disconnected_graph | A, B (无 edges) | 通过 (图合法) |
| 6 | create_cyclic_graph | A->B, B->A | Status::invalid_argument |

#### D.2.2 Execute RPC (8 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | execute_colorspace_only | I01.png -> colorspace(srgb->linear) | stream 包含 Done 阶段, 输出像素线性化 |
| 2 | execute_full_chain | I01.png -> colorspace->transform->lut3d->png | 各阶段消息按序到达, 最终 Done |
| 3 | execute_progress_stream | I14(大图) -> colorspace | stream 包含 Loading/Processing/Done |
| 4 | execute_invalid_pipeline_id | uuid 不存在 | Status::not_found |
| 5 | execute_nonexistent_image | valid id + /bad.png | Status::not_found |
| 6 | execute_cancel_midway | 5 节点链, 收到第 1 个进度后 cancel | stream 终止, 无 Done |
| 7 | execute_16bit_preserved | I15(U16) -> colorspace | 输出像素格式 U16 |
| 8 | execute_metadata_propagation | I09(raw) -> exif_rw | 输出图像的 EXIF 与输入一致 |

#### D.2.3 Validate RPC (4 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | validate_valid_pipeline | colorspace->transform | ValidationResult.valid=true |
| 2 | validate_unknown_plugin | node.plugin_id = "nonexistent" | valid=false, issue 包含 "not registered" |
| 3 | validate_bad_edge | edge.from = "missing_node" | valid=false, issue 包含 "unknown source node" |
| 4 | validate_empty | nodes=[] | valid=false, issue 包含 "at least one node" |

#### D.2.4 GetNodeSchema RPC (2 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | get_known_plugin_schema | "colorspace" | NodeSchema.parameter_schema 非空 |
| 2 | get_unknown_plugin_schema | "bogus" | Status::not_found |

### D.3 多插件 gRPC 链测试 (30 条)

**文件**: tests/e2e/tests/multi_plugin_grpc.rs

这些测试通过 gRPC CreatePipeline + Execute 运行真实管线链, 并从磁盘读取输出图像进行像素级验证。

| # | 管线链 | 输入 | 验证方法 |
|---|--------|------|----------|
| 1 | colorspace(srgb->linear) | I01.png | assert_pixels_eq 与已知线性图像 |
| 2 | colorspace(linear->srgb) | I02.png | assert_pixels_eq 与已知 sRGB 图像 |
| 3 | transform(rotate 90) | I03.png | compute_ssim > 0.99 |
| 4 | transform(rotate 180) | I03.png | compute_ssim > 0.99 |
| 5 | transform(resize 200%) | I01.png | 输出 128x128, assert_pixels_eq 插值 |
| 6 | transform(crop 10,10,50,50) | I04.png | 输出 40x40, 左上角像素匹配 |
| 7 | transform(flip h) | I03.png | compute_ssim > 0.99 |
| 8 | transform(flip v) | I03.png | compute_ssim > 0.99 |
| 9 | lut3d(identity.cube) | I05.png | assert_pixels_eq(tolerance=1) |
| 10 | lut3d(contrast.cube) | I05.png | compute_psnr > 35dB (视觉变化) |
| 11 | exif_rw(read->write) | I09.raw | gRPC 返回的 Metadata 非空 |
| 12 | gps_set(lat,lng) | I09.raw | 输出 EXIF 含 GPS 坐标 |
| 13 | time_shift(+2h) | I09.raw | 输出 EXIF DateTime 偏移 2h |
| 14 | ai_denoise(model=v1) | I12.png | compute_psnr > 30dB (降噪后) |
| 15 | raw->colorspace(srgb) | I09.raw | 输出 TIFF, assert_pixels_eq |
| 16 | colorspace->transform(rotate) | I03.png | 输出旋转后, assert_pixels_eq |
| 17 | transform->lut3d | I05.png | 复合变换后 compute_psnr > 35dB |
| 18 | raw->lens_correct | I09.raw | 输出畸变校正, 棋盘格直线度 |
| 19 | raw->ai_denoise->colorspace | I12.png | 降噪+校色, compute_ssim > 0.95 |
| 20 | colorspace->lut3d->transform->png | I01.png | 全链通过 gRPC Execute, stream 含 4 个阶段 |
| 21 | exif_rw->gps_set->time_shift | I09.raw | 元数据链, 3 个阶段验证 |
| 22 | colorspace(prophoto->srgb) | I10.tiff | compute_delta_e < 1 (宽色域) |
| 23 | colorspace(rec2020->srgb) | I10.tiff | compute_delta_e < 1.5 |
| 24 | transform(resize 10%) | I14.png | 输出 800x400, 缩略图生成 |
| 25 | colorspace->transform(rotate+crop) | I04.png | 旋转后裁剪, 输出尺寸匹配 |
| 26 | ai_denoise->colorspace->lut3d->tiff | I12.png | 全链 E2E 像素验证 |
| 27 | raw->colorspace->transform->lut3d->jxl | I09.raw | RAW 到 JXL 全链 |
| 28 | lens_correct->colorspace->transform->png | I13.jpg | 畸变校正链 |
| 29 | colorspace->lut3d(exposure+1ev) | I05.png | 提亮 1EV 后 compute_mae 约 2x |
| 30 | colorspace(acescg->srgb) | I16.exr | ACEScg 到 sRGB, compute_delta_e < 2 |

### D.4 格式 gRPC E2E 测试 (15 条)

**文件**: tests/e2e/tests/format_grpc_e2e.rs

| # | 测试 | 流程 | 断言 |
|---|------|------|------|
| 1 | png_load_decode_encode_grpc | Load->Decode->Encode(PNG) | 输出有效 PNG, assert_pixels_eq |
| 2 | tiff_load_decode_encode_grpc | Load->Decode->Encode(TIFF LZW) | 无损, assert_pixels_eq(tolerance=0) |
| 3 | jpeg_decode_encode_psnr | Decode->Encode(JPEG q=95) | compute_psnr > 40dB |
| 4 | jxl_lossless_roundtrip | Decode->Encode(JXL lossless) | assert_pixels_eq(tolerance=0) |
| 5 | avif_lossless_roundtrip | Decode->Encode(AVIF lossless) | assert_pixels_eq(tolerance=0) |
| 6 | exr_roundtrip_grpc | Load(I16.exr)->Decode->Encode(EXR) | assert_pixels_eq(tolerance=1) |
| 7 | bmp_roundtrip_grpc | Load->Decode->Encode(BMP) | assert_pixels_eq(tolerance=0) |
| 8 | webp_lossless_roundtrip | Decode->Encode(WEBP lossless) | assert_pixels_eq(tolerance=0) |
| 9 | raw_to_tiff_grpc | Load(I09.raw)->Decode->Encode(TIFF) | 输出 TIFF, assert_pixels_eq |
| 10 | heif_to_tiff_grpc | Load(I18.heic)->Decode->Encode(TIFF) | 输出 TIFF, assert_pixels_eq |
| 11 | tiff_16bit_preserve_grpc | Load(I15.tiff)->Decode->Encode(TIFF) | assert_pixel_format(U16) |
| 12 | png_rgba_alpha_grpc | Load(I17.png)->Decode->Encode(PNG) | assert_channel_count(4), alpha 保留 |
| 13 | format_conversion_png_to_tiff | Load->Decode(I01)->Encode(TIFF) | 输出 .tiff, 像素相同 |
| 14 | format_conversion_tiff_to_jxl | Load->Decode(I11)->Encode(JXL) | 输出 .jxl, compute_psnr > 50dB |
| 15 | format_conversion_raw_to_tiff | Load->Decode(I09)->Encode(TIFF) | 输出 .tiff, 有效图像 |

### D.5 Batch gRPC E2E 测试 (15 条)

**文件**: tests/e2e/tests/batch_grpc_e2e.rs

#### D.5.1 SubmitBatch (8 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | submit_3_png_files | file_pattern="I01.png|I02.png|I03.png" | BatchId 非空, status=Pending |
| 2 | submit_glob_pattern | file_pattern="I*.png" | 匹配多个文件 |
| 3 | submit_single_raw | file_pattern="I09.raw" | BatchId 返回 |
| 4 | submit_no_match | file_pattern="nonexistent*" | Status::not_found (文件数为 0) |
| 5 | submit_invalid_glob | file_pattern="[" | Status::invalid_argument |
| 6 | submit_invalid_pipeline_config | pipeline_config_path="/bad/path" | Status::not_found |
| 7 | submit_output_dir_creation | output_dir="/tmp/new_batch_output" | 目录自动创建 |
| 8 | submit_mixed_formats | I01.png + I09.raw + I11.tiff | 三种格式各处理成功 |

#### D.5.2 GetProgress (4 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | progress_pending_to_done | submit + poll until done | fraction=1.0, status=Done |
| 2 | progress_with_failures | 包含损坏文件 | status=Done, failed_files>0 |
| 3 | progress_unknown_batch | 随机 uuid | Status::not_found |
| 4 | progress_canceled_batch | submit + cancel + poll | status=Canceled |

#### D.5.3 Cancel (3 条)

| # | 测试 | 输入 | 断言 |
|---|------|------|------|
| 1 | cancel_running_batch | 大图批量处理中途取消 | status=Canceled, 部分文件完成 |
| 2 | cancel_completed_batch | 完成后再取消 | status=Done (不改变) |
| 3 | cancel_nonexistent | 随机 uuid | Status::not_found |

### D.6 错误路径测试 (10 条)

**文件**: tests/e2e/tests/error_paths_grpc.rs

| # | 测试 | 场景 | 预期行为 |
|---|------|------|----------|
| 1 | unauthenticated_request | 客户端无认证 | Status::unauthenticated (如启用) |
| 2 | malformed_protobuf | 发送随机字节 | Status::internal 或通道关闭 |
| 3 | pipeline_timeout | 设 deadline=1ms | DeadlineExceeded |
| 4 | server_shutdown_mid_execute | 执行中关闭服务器 | stream 中断 (Err) |
| 5 | concurrent_invalid_requests | 10 个并行非法请求 | 各返回正确 Status |
| 6 | oversized_request | 超大 PipelineSpec | Status::resource_exhausted |
| 7 | pipeline_execute_no_permission | 只读文件 | Status::permission_denied |
| 8 | decode_unsupported_format | .xyz 格式 | Status::invalid_argument |
| 9 | encode_unsupported_format | 请求编码为 XYZ | Status::invalid_argument |
| 10 | batch_submit_readonly_output | 只读输出目录 | Status::internal (写入失败) |

### D.7 并发测试 (10 条)

**文件**: tests/e2e/tests/concurrency_grpc.rs

| # | 测试 | 并发量 | 验证 |
|---|------|--------|------|
| 1 | parallel_10_execute | 10 个 Execute 同时 | 全部返回 Done, 无数据竞争 |
| 2 | parallel_mixed_rpcs | 5 Create + 5 Execute + 5 Validate | 各自正确 |
| 3 | serial_100_pipelines | 顺序创建 100 个 | 全部通过, 无泄漏 |
| 4 | concurrent_batch_submit | 5 个批处理同时 | 全部完成 |
| 5 | image_service_concurrent | 10 个 Load | 全部返回 ImageInfo |
| 6 | rapid_cancel_restart | 取消后立即提交 | 新批处理正常 |
| 7 | many_simultaneous_streams | 20 个 Execute stream | 全部正常流式传输 |
| 8 | mixed_traffic | 所有服务的混合请求 | 无死锁 |
| 9 | batch_with_concurrent_progress | 处理中 10 个 GetProgress | 全部返回有效进度 |
| 10 | stress_1000_pipelines | 创建 1000 个管线 | 无 OOM, 全部可执行 |

### D.8 跨通道一致性验证 (10 条)

**文件**: tests/e2e/tests/cross_channel_consistency.rs

#### D.8.1 原理

跨通道一致性保证 Rust gRPC (Layer 2) 和 C# gRPC (Layer 4) 和 C# GUI (Layer 5) 对同一管线产生相同输出。
验证方法: 读取 shared/test_cases/ 目录下的 JSON 测试用例定义, 连接到 Rust gRPC 服务器执行, 将输出与 C# 端预先生成的黄金参考图像对比。

| # | 测试 | JSON 用例 | Rust 输出 vs C# 黄金参考 |
|---|------|-----------|------------------------|
| 1 | cs_srgb_to_linear | cross_001.json | assert_pixels_eq(tolerancePerChannel=0) |
| 2 | cs_linear_to_srgb | cross_002.json | assert_pixels_eq(tolerance=0) |
| 3 | rotate_90 | cross_003.json | assert_pixels_eq(tolerance=0) |
| 4 | resize_200 | cross_004.json | assert_pixels_eq(tolerance=0) |
| 5 | identity_lut | cross_005.json | assert_pixels_eq(tolerance=0) |
| 6 | raw_development | cross_006.json | compute_ssim > 0.999 |
| 7 | full_chain | cross_007.json | assert_pixels_eq(tolerance=1) |
| 8 | batch_single_file | cross_008.json | assert_pixels_eq(tolerance=0) |
| 9 | format_roundtrip | cross_009.json | assert_pixels_eq(tolerance=0) |
| 10 | metadata_chain | cross_010.json | EXIF 字段逐项比对 |

#### D.8.2 一致性基础设施

```rust
// tests/e2e/src/cross_channel.rs
pub struct CrossChannelVerifier {
    golden_dir: PathBuf,      // C# 端生成的黄金参考图像目录
    tolerance: u8,            // 容差 (默认 0)
}

impl CrossChannelVerifier {
    /// 从 JSON 测试用例加载, 连接到 Rust gRPC 执行, 对比 C# 黄金参考
    pub async fn verify(json_path: &Path) -> Result<()> {
        let tc: CrossTestCase = serde_json::from_slice(&std::fs::read(json_path)?)?;
        let addr = TestServer::start().await;
        let mut client = create_pipeline_client(&addr);
        // ... 执行管线并对比
    }
}
```

## E. Shared JSON 格式定义

### E.1 目标

Rust 端 (Layer 0-2) 和 C# 端 (Layer 3-5) 共享同一套测试用例定义, 存储在 `shared/test_cases/` 目录下。
这样确保: 
1. 同一测试用例在 Rust 和 C# 端执行相同的管线配置和验证标准
2. 新加测试用例只需添加一个 JSON 文件, 两端自动可用
3. CrossTestCase (D.8) 可直接引用 JSON 定义, 保证跨通道一致性

### E.2 目录结构

```
shared/test_cases/
  cross/                  # 跨通道验证用例 (Layer 2 vs Layer 4 vs Layer 5)
    cross_001.json
    cross_002.json
    ... (10 个)
  unit/                   # 单元测试用例 (Layer 0)
    param_schema/         # ParameterSchema 验证 (14 x 5 = 70 个)
    process_boundary/     # process/tile 边界 (14 x 5 = 70 个)
    executor/             # NodeExecutor 测试 (20 个)
    ...
  integration/            # 集成测试用例 (Layer 1)
    single_plugin/        # 单插件 (84 个)
    multi_plugin/         # 多插件链 (90 个)
    format_roundtrip/     # 格式往返 (10 个)
    boundary/             # 边界条件 (16 个)
  e2e/                    # gRPC E2E 用例 (Layer 2)
    image_service/        # (30 个)
    pipeline_service/     # (20 个)
    batch_service/        # (15 个)
    error_paths/          # (10 个)
  schema.json             # JSON Schema 定义文件
```

### E.3 CrossTestCase JSON Schema

每个 JSON 文件定义一个完整的测试用例:

```json
// shared/test_cases/schema.json (JSON Schema 定义)
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CrossTestCase",
  "type": "object",
  "required": ["id", "layer", "category", "name", "pipeline", "inputs", "assertions"],
  "properties": {
    "id":                { "type": "string", "pattern": "^[A-Z][A-Z0-9_]{3,63}$" },
    "layer":             { "type": "integer", "enum": [0, 1, 2, 3, 4, 5] },
    "category":          { "type": "string" },
    "name":              { "type": "string", "description": "人类可读的测试名" },
    "description":       { "type": "string" },
    "skip":              { "type": "boolean", "default": false },
    "skip_reason":       { "type": "string" },
    "pipeline":          { "$ref": "#/definitions/PipelineDef" },
    "inputs":            { "type": "array", "items": { "$ref": "#/definitions/InputDef" } },
    "assertions":        { "$ref": "#/definitions/AssertionSet" },
    "metadata":          { "$ref": "#/definitions/MetadataAssert" },
    "tags":              { "type": "array", "items": { "type": "string" } }
  },
  "definitions": {
    "PipelineDef": {
      "type": "object",
      "required": ["nodes"],
      "properties": {
        "nodes": { "type": "array", "items": { "$ref": "#/definitions/NodeDef" } },
        "edges": { "type": "array", "items": { "$ref": "#/definitions/EdgeDef" } },
        "overrides": { "type": "object" }
      }
    },
    "NodeDef": {
      "type": "object",
      "required": ["id", "plugin"],
      "properties": {
        "id":        { "type": "string" },
        "plugin":    { "type": "string" },
        "label":     { "type": "string" },
        "enabled":   { "type": "boolean", "default": true },
        "params":    { "type": "object", "additionalProperties": true }
      }
    },
    "EdgeDef": {
      "type": "object",
      "required": ["from", "to"],
      "properties": {
        "from": { "type": "string" },
        "to":   { "type": "string" }
      }
    },
    "InputDef": {
      "type": "object",
      "required": ["path", "format"],
      "properties": {
        "path":     { "type": "string" },
        "format":   { "type": "string", "enum": ["PNG", "TIFF", "JPEG", "RAW", "HEIF", "JXL", "AVIF", "EXR", "BMP", "WEBP"] },
        "metadata": { "type": "object" }
      }
    },
    "AssertionSet": {
      "type": "object",
      "properties": {
        "type": { "type": "string", "enum": ["pixel_exact", "psnr", "ssim", "delta_e", "mae", "file_exists", "dimensions", "format"] },
        "tolerance_per_channel": { "type": "integer", "default": 0 },
        "min_psnr":              { "type": "number" },
        "min_ssim":              { "type": "number" },
        "max_mae":               { "type": "number" },
        "max_delta_e":           { "type": "number" },
        "expected_width":        { "type": "integer" },
        "expected_height":       { "type": "integer" },
        "expected_format":       { "type": "string" },
        "golden_reference":      { "type": "string", "description": "黄金参考图像路径" }
      }
    },
    "MetadataAssert": {
      "type": "object",
      "properties": {
        "exif_fields":   { "type": "object" },
        "xmp_fields":    { "type": "object" },
        "gps_latitude":  { "type": "number" },
        "gps_longitude": { "type": "number" },
        "datetime":      { "type": "string" }
      }
    }
  }
}
```


### E.4 示例 JSON 测试用例

```json
// shared/test_cases/cross/cross_001.json
{
  "id": "CROSS_001",
  "layer": 2,
  "category": "cross_channel",
  "name": "Color space sRGB to linear via gRPC",
  "description": "Verify that Rust gRPC and C# gRPC produce identical sRGB->linear conversion",
  "pipeline": {
    "nodes": [
      { "id": "cs", "plugin": "colorspace", "params": { "from": "srgb", "to": "linear" } }
    ]
  },
  "inputs": [
    { "path": "images/I01.png", "format": "PNG" }
  ],
  "assertions": {
    "type": "pixel_exact",
    "tolerance_per_channel": 0,
    "golden_reference": "golden/I01_srgb_to_linear.png"
  }
}
```

### E.5 Rust 端 Serde 结构体

在 tests/test_harness 或独立 crate 中定义:

```rust
// tests/shared/src/cross_test_case.rs (新建 shared crate)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrossTestCase {
    pub id: String,
    pub layer: u32,
    pub category: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub skip: bool,
    #[serde(default)]
    pub skip_reason: Option<String>,
    pub pipeline: PipelineDef,
    pub inputs: Vec<InputDef>,
    pub assertions: AssertionSet,
    #[serde(default)]
    pub metadata: Option<MetadataAssert>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDef {
    pub nodes: Vec<NodeDef>,
    #[serde(default)]
    pub edges: Vec<EdgeDef>,
    #[serde(default)]
    pub overrides: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    pub id: String,
    pub plugin: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub params: serde_json::Value,
}

fn default_enabled() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDef {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDef {
    pub path: String,
    pub format: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssertionType {
    PixelExact(AssertionParams),
    Psnr(AssertionParams),
    Ssim(AssertionParams),
    DeltaE(AssertionParams),
    Mae(AssertionParams),
    FileExists(AssertionParams),
    Dimensions(AssertionParams),
    Format(AssertionParams),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssertionParams {
    #[serde(default)]
    pub tolerance_per_channel: u8,
    #[serde(default)]
    pub min_psnr: Option<f64>,
    #[serde(default)]
    pub min_ssim: Option<f64>,
    #[serde(default)]
    pub max_mae: Option<f64>,
    #[serde(default)]
    pub max_delta_e: Option<f64>,
    #[serde(default)]
    pub expected_width: Option<u32>,
    #[serde(default)]
    pub expected_height: Option<u32>,
    #[serde(default)]
    pub expected_format: Option<String>,
    #[serde(default)]
    pub golden_reference: Option<String>,
}
```

### E.6 C# 端兼容性

C# 端使用 System.Text.Json 反序列化相同的 JSON 格式:

```csharp
// Photopipeline.Tests.Shared/Models/CrossTestCase.cs
public class CrossTestCase
{
    public string Id { get; set; } = string.Empty;
    public int Layer { get; set; }
    public string Category { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public bool Skip { get; set; }
    public string? SkipReason { get; set; }
    public PipelineDef Pipeline { get; set; } = new();
    public List<InputDef> Inputs { get; set; } = new();
    public AssertionSet Assertions { get; set; } = new();
    public MetadataAssert? Metadata { get; set; }
    public List<string> Tags { get; set; } = new();
}
```

两端采用相同的字段命名、类型结构、序列化标签, 确保 JSON 文件可互读。
golden_reference 路径在 CI 中通过环境变量 GOLDEN_DIR 定位, 默认为 shared/golden/。

### E.7 JSON 生成方案

JSON 测试用例文件不作为手写维护, 而是通过以下方式生成:

1. **设计阶段**: 在本设计文档中定义每个测试的结构和预期断言
2. **代码生成**: 使用 scripts/generate_test_cases.py 从本文档的 Markdown 表格中提取并生成 JSON
3. **手动添加**: 特殊跨通道测试用例手动编写 JSON
4. **CI 验证**: CI 中运行 schema 验证, 确保所有 JSON 文件符合 schema.json

## F. 实施步骤

### F.0 优先级定义

| 优先级 | 含义 | 时间要求 |
|--------|------|----------|
| P0 | 阻塞所有其他测试的基础设施 | 必须先完成 |
| P1 | 必须实现的核心测试 | Phase 1 |
| P2 | 重要测试 | Phase 2 |
| P3 | 补充测试 | Phase 3 |
| P4 | 完善/压力测试 | Phase 4 |

### F.1 实施阶段 A0: 基础设施 (P0)

这些任务必须在任何测试编写之前完成。

| # | 任务 | 文件 | 说明 |
|---|------|------|------|
| 1 | 创建 tests/shared crate | tests/shared/Cargo.toml | serde, serde_json, uuid 依赖 |
| 2 | 定义 CrossTestCase 结构体 | tests/shared/src/cross_test_case.rs | 所有 Serde 结构体 (E.5) |
| 3 | 加载器函数 | tests/shared/src/loader.rs | load_test_case(dir, id) -> CrossTestCase |
| 4 | 生成 20 张测试输入图像 | scripts/generate_test_images.py | I01-I20, seed=42 |
| 5 | 创建 shared/schema.json | shared/test_cases/schema.json | JSON Schema 定义 (E.3) |
| 6 | 完善 test_harness 基础设施 | tests/test_harness/ | golden/quality/structural assertions |
| 7 | 创建 TestServer 启动器 | tests/e2e/src/grpc_server.rs | 随机端口服务器生命期管理 |
| 8 | 创建 gRPC 客户端工厂 | tests/e2e/src/grpc_client.rs | Image/Pipeline/Batch 客户端构建 |
| 9 | CI gRPC 服务可用性脚本 | scripts/ci/wait_for_grpc.sh | 等待服务器就绪 |

### F.2 实施阶段 A1: Layer 0 单元测试 (P1, ~350 条)

| # | 任务 | 文件 | 测试数 | 优先级 |
|---|------|------|--------|--------|
| 1 | 14 插件 ParameterSchema 验证 | crates/plugins/tests/param_schema.rs | 140 | P1 |
| 2 | 14 插件 process/tile 边界测试 | crates/plugins/tests/process_boundary.rs | 70 | P1 |
| 3 | Engine Graph/DAG 拓扑排序 | crates/engine/tests/graph.rs | 30 | P1 |
| 4 | Engine ParameterResolver | crates/engine/tests/params.rs | 30 | P1 |
| 5 | Engine TileEngine | crates/engine/tests/tile.rs | 25 | P1 |
| 6 | Engine NodeExecutor | crates/engine/tests/executor.rs | 20 | P1 |
| 7 | gRPC Service 验证 | crates/server/tests/service_validation.rs | 35 | P1 |

### F.3 实施阶段 A2: Layer 1 管线集成测试 (P2, ~200 条)

| # | 任务 | 文件 | 测试数 | 优先级 |
|---|------|------|--------|--------|
| 1 | 单插件集成测试 | tests/pipeline_integration/tests/single_plugin.rs | 84 | P2 |
| 2 | 多插件链集成测试 | tests/pipeline_integration/tests/multi_plugin_chains.rs | 90 | P2 |
| 3 | 格式往返测试 | tests/pipeline_integration/tests/format_roundtrip.rs | 10 | P2 |
| 4 | 边界条件测试 | tests/pipeline_integration/tests/boundary_conditions.rs | 16 | P2 |

### F.4 实施阶段 A3: Layer 2 gRPC E2E 测试 (P3, ~120 条)

| # | 任务 | 文件 | 测试数 | 优先级 |
|---|------|------|--------|--------|
| 1 | ImageService E2E | tests/e2e/tests/image_service_e2e.rs | 30 | P3 |
| 2 | PipelineService E2E | tests/e2e/tests/pipeline_service_e2e.rs | 20 | P3 |
| 3 | 多插件 gRPC 链 | tests/e2e/tests/multi_plugin_grpc.rs | 30 | P3 |
| 4 | 格式 gRPC E2E | tests/e2e/tests/format_grpc_e2e.rs | 15 | P3 |
| 5 | Batch gRPC E2E | tests/e2e/tests/batch_grpc_e2e.rs | 15 | P3 |
| 6 | 错误路径 | tests/e2e/tests/error_paths_grpc.rs | 10 | P3 |

### F.5 实施阶段 A4: 进阶测试 (P4, ~60 条)

| # | 任务 | 文件 | 测试数 | 优先级 |
|---|------|------|--------|--------|
| 1 | 并发测试 | tests/e2e/tests/concurrency_grpc.rs | 10 | P4 |
| 2 | 跨通道一致性 | tests/e2e/tests/cross_channel_consistency.rs | 10 | P4 |
| 3 | 生成 JSON 测试用例 | scripts/generate_test_cases.py | ~670 | P4 |
| 4 | CI 集成 | .github/workflows/test.yml | - | P4 |
| 5 | 压力测试 | tests/stress/ | 待定 | P4 |

### F.6 实施依赖图

```
A0 (基础设施)
  |
  +---> A1 (Layer 0 单元测试)
  |       |
  |       +---> A2 (Layer 1 集成测试)
  |               |
  |               +---> A3 (Layer 2 gRPC E2E 测试)
  |                       |
  |                       +---> A4 (进阶测试)
  |
  +---> C# 端 (Layer 3-5, 独立进行)
          |
          +---> 跨通道验证 (A4.2)
```

### F.7 文件创建清单 (完整)

以下为完整的新增文件清单, 按实施顺序排列:

```
Phase A0 (基础设施, P0):
  [ ] tests/shared/Cargo.toml
  [ ] tests/shared/src/lib.rs
  [ ] tests/shared/src/cross_test_case.rs
  [ ] tests/shared/src/loader.rs
  [ ] tests/e2e/src/grpc_server.rs
  [ ] tests/e2e/src/grpc_client.rs
  [ ] shared/test_cases/schema.json
  [ ] scripts/generate_test_images.py
  [ ] scripts/ci/wait_for_grpc.sh

Phase A1 (Layer 0 单元测试, P1):
  [ ] crates/plugins/tests/param_schema.rs
  [ ] crates/plugins/tests/process_boundary.rs
  [ ] crates/engine/tests/graph.rs
  [ ] crates/engine/tests/params.rs
  [ ] crates/engine/tests/tile.rs
  [ ] crates/engine/tests/executor.rs
  [ ] crates/server/tests/service_validation.rs

Phase A2 (Layer 1 集成测试, P2):
  [ ] tests/pipeline_integration/Cargo.toml
  [ ] tests/pipeline_integration/tests/single_plugin.rs
  [ ] tests/pipeline_integration/tests/multi_plugin_chains.rs
  [ ] tests/pipeline_integration/tests/format_roundtrip.rs
  [ ] tests/pipeline_integration/tests/boundary_conditions.rs

Phase A3 (Layer 2 gRPC E2E, P3):
  [ ] tests/e2e/tests/image_service_e2e.rs
  [ ] tests/e2e/tests/pipeline_service_e2e.rs
  [ ] tests/e2e/tests/multi_plugin_grpc.rs
  [ ] tests/e2e/tests/format_grpc_e2e.rs
  [ ] tests/e2e/tests/batch_grpc_e2e.rs
  [ ] tests/e2e/tests/error_paths_grpc.rs

Phase A4 (进阶测试, P4):
  [ ] tests/e2e/tests/concurrency_grpc.rs
  [ ] tests/e2e/tests/cross_channel_consistency.rs
  [ ] tests/e2e/src/cross_channel.rs
  [ ] scripts/generate_test_cases.py
  [ ] .github/workflows/test.yml (更新)
```

### F.8 铁律遵守清单

将六个铁律映射到本设计中的具体机制:

| 铁律 | 设计中的保障机制 |
|------|------------------|
| 铁律 1: 每个测试必须有 FAIL-able 断言 | 所有测试表格中明确列出断言列; 禁止 File.Exists 作为唯一断言 |
| 铁律 2: 禁止静默跳过 | test_harness 不提供静默 catch 模式; 后端不可用时使用 Assert::Fail |
| 铁律 3: 基础设施必须有消费者 | 所有 test_harness 工具方法在 B/C/D 中有对应使用者; 新增工具必须同时添加调用测试 |
| 铁律 4: UI 测试必须启动真实进程 | Layer 2 使用真实 TestServer 进程 (D.0.1); Layer 5 使用 WinAppDriver |
| 铁律 5: 对抗性自查 | 每个测试设计时问: 如果后端错误是否 FAIL? 如果全黑图像是否 FAIL? (已内建至设计模板) |
| 铁律 6: 回归测试必须有黄金参考 | D.8 跨通道测试使用 C# 生成的 golden_reference; 质量断言使用 golden 图像 |

### F.9 测试计数汇总

| 层 | 分类 | 计划数 | 状态 |
|-----|------|--------|------|
| Layer 0 | 14 插件 ParameterSchema | 140 | 设计完成 |
| Layer 0 | 14 插件 process/tile 边界 | 70 | 设计完成 |
| Layer 0 | Engine Graph/DAG | 30 | 设计完成 |
| Layer 0 | Engine ParameterResolver | 30 | 设计完成 |
| Layer 0 | Engine TileEngine | 25 | 设计完成 |
| Layer 0 | Engine NodeExecutor | 20 | 设计完成 |
| Layer 0 | gRPC Service 验证 | 35 | 设计完成 |
| Layer 1 | 单插件集成 | 84 | 设计完成 |
| Layer 1 | 多插件链 | 90 | 设计完成 |
| Layer 1 | 格式往返 | 10 | 设计完成 |
| Layer 1 | 边界条件 | 16 | 设计完成 |
| Layer 2 | ImageService E2E | 30 | 设计完成 |
| Layer 2 | PipelineService E2E | 20 | 设计完成 |
| Layer 2 | 多插件 gRPC 链 | 30 | 设计完成 |
| Layer 2 | 格式 gRPC E2E | 15 | 设计完成 |
| Layer 2 | Batch gRPC E2E | 15 | 设计完成 |
| Layer 2 | 错误路径 | 10 | 设计完成 |
| Layer 2 | 并发测试 | 10 | 设计完成 |
| Layer 2 | 跨通道一致性 | 10 | 设计完成 |
| **合计** | | **690** | **设计完成** |

---

*本文档是 Rust 端测试系统的完整设计蓝图, 覆盖 Layer 0 (单元测试)、Layer 1 (管线集成测试)、Layer 2 (gRPC E2E 测试), 共约 690 条测试用例。实现应严格按照 F 节的阶段顺序进行, 确保障碍 (P0) 优先解决, 核心测试 (P1) 紧随其后。*

