# Photopipeline Test Suite

580 tests across 15 files. All run on every push via GitHub Actions CI.
[![CI](https://github.com/zhang-hz/photopipeline/actions/workflows/build-rust.yml/badge.svg)](https://github.com/zhang-hz/photopipeline/actions/workflows/build-rust.yml)

## Test File Locations

| File | Tests | Category |
|------|:---:|------|
| `crates/core/src/types.rs` | 81 | Type system, version comparison, serde roundtrip |
| `crates/core/src/image.rs` | 73 | PixelBuffer, AlignedBuffer, TileLayout, format conversion |
| `crates/core/src/metadata.rs` | 53 | GPX interpolation, EXIF/XMP/IPTC types |
| `crates/core/src/color.rs` | 49 | ColorSpace presets, color math, transfer functions |
| `crates/core/src/error.rs` | 28 | Error variant construction, display, source chain |
| **Core subtotal** | **284** | |
| |
| `crates/engine/src/params.rs` | 65 | 4-level parameter priority, expression engine, group conditions |
| `crates/engine/src/graph.rs` | 50 | DAG construction, topological sort, cycle detection, serialization |
| `crates/engine/src/tile.rs` | 19 | Tile splitting, parallel processing, boundary conditions |
| `crates/engine/src/executor.rs` | 14 | Node execution, progress reporting, metadata-only fast path |
| **Engine subtotal** | **148** | |
| |
| `crates/plugin/src/schema.rs` | 46 | ParameterSchema, ParameterType (18 variants), merge, serde |
| `crates/plugin/src/registry.rs` | 29 | Plugin register/query/unregister, manifest, categories |
| `crates/plugin/src/loader.rs` | 14 | PluginLoader traits, discovery, search paths |
| **Plugin subtotal** | **89** | |
| |
| `cli/tests/integration_test.rs` | 27 | CLI commands, pipeline config, batch processing |
| `crates/plugins/tests/plugin_tests.rs` | 22 | All 14 plugins: schema validation, capabilities, metadata |
| `tests/stress/stress_tests.rs` | 10 | 1000-node pipeline, 16-thread concurrency, fuzzing, memory |
| **Integration subtotal** | **59** | |
| |
| **TOTAL** | **580** | |

## Test Categories

### 1. Unit Tests — Inline (`#[cfg(test)]`)
Embedded in each source file. Tests live module members:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_name() { ... }
}
```

### 2. Integration Tests — Separate crates
- `cli/tests/integration_test.rs` — Tests the CLI binary as an external consumer
- `crates/plugins/tests/plugin_tests.rs` — Tests all 14 plugins through public API
- `tests/stress/stress_tests.rs` — Stress, concurrency, memory tests

### 3. Property Tests — Boundary coverage
Every type tested at boundaries:
- Empty / zero / MAX values
- Single element / many elements
- Negative inputs (where applicable)
- NaN / infinity (for float operations)

## Running Tests

### All tests
```bash
cargo test --workspace
```

### Specific crate
```bash
cargo test -p photopipeline-core
cargo test -p photopipeline-engine
cargo test -p photopipeline-plugin
cargo test -p photopipeline-plugins
```

### Single test
```bash
cargo test -p photopipeline-core -- pixel_format_is_high_precision
```

### Stress tests only
```bash
cargo test -p photopipeline-stress-tests
```

## CI Integration

Every push to `master` and every pull request triggers:

| Job | Platform | What |
|-----|----------|------|
| `build-rust.yml` | ubuntu, windows, macos | Build + 580 tests + fmt + clippy |
| `coverage.yml` | ubuntu | Single-threaded tests + 5x stress iteration |
| `build-gui-windows.yml` | windows | .NET 8 GUI build + tests |
| `security-audit.yml` | ubuntu (weekly) | cargo audit |

Test results are uploaded as GitHub Actions artifacts (30-day retention).
