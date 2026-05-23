# Contributing to Photopipeline

Thank you for your interest in contributing to Photopipeline. This document outlines the
process for setting up your development environment, making changes, and submitting pull requests.

## Code of Conduct

This project adheres to a Code of Conduct adapted from the [Contributor Covenant](https://www.contributor-covenant.org/).
By participating, you are expected to uphold this code.

**TL;DR:** Be respectful, constructive, and professional. Harassment and abusive behaviour are not tolerated.

---

## Development Setup

### Prerequisites

| Tool | Minimum Version | Purpose |
|---|---|---|
| Rust | 1.90+ | Compilation |
| CMake | 3.20+ | Halide / OIIO build (CI) |
| pkg-config | — | System library detection |
| libheif-dev | 1.12+ | HEIF / AVIF codec |
| libjxl-dev | 0.8+ | JPEG XL codec |
| liblcms2-dev | 2.0+ | Colour management |
| exiftool | 12.00+ | Metadata manipulation |
| just | latest | Task runner |

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline

# Install system dependencies
# Ubuntu/Debian:
sudo apt install build-essential cmake pkg-config \
  libheif-dev libjxl-dev liblcms2-dev libimage-exiftool-perl

# macOS:
brew install cmake pkg-config libheif jpeg-xl little-cms2 exiftool

# Verify the build
just build
just test
```

### IDE Configuration

The workspace uses Rust edition 2024. Ensure your editor supports it:

- **VS Code**: `rust-analyzer` extension (enable `rust-analyzer.cargo.features: "all"`).
- **IntelliJ / CLion**: Rust plugin with workspace detection.
- **Neovim**: `rustaceanvim` or `rust-analyzer` LSP.

---

## Project Structure

```
photopipeline/
├── crates/
│   ├── core/            # Shared types (no dependencies on other crates)
│   │   └── src/
│   │       ├── lib.rs       # Re-exports all modules
│   │       ├── image.rs     # PixelBuffer, TileLayout, ChannelLayout, etc.
│   │       ├── color.rs     # ColorSpace, TransferFunction, RenderingIntent
│   │       ├── metadata.rs  # ExifData, XmpData, GpsData, GPX track
│   │       ├── types.rs     # PluginVersion, ImageFormat, GpuBackend, etc.
│   │       └── error.rs     # PluginError, ValidationIssue
│   ├── plugin/          # Plugin framework (depends on core)
│   │   └── src/
│   │       ├── lib.rs       # Re-exports
│   │       ├── trait_def.rs # Plugin, MetadataProcessor, PixelProcessor, etc.
│   │       ├── schema.rs    # ParameterSchema, ParameterType (18 variants)
│   │       ├── registry.rs  # Registry: thread-safe plugin registry
│   │       ├── loader.rs    # PluginLoader trait + implementations
│   │       └── gui_schema.rs# NodePanelDefinition, PanelWidget
│   ├── engine/          # Pipeline runtime (depends on core, plugin)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs     # PipelineGraph, PipelineTemplate
│   │       ├── executor.rs  # NodeExecutor: topological execution
│   │       ├── params.rs    # ParameterResolver (4-level), ExpressionEngine
│   │       └── tile.rs      # TileEngine: tile-based processing
│   ├── plugins/         # 14 built-in plugins (depends on core, plugin)
│   │   └── src/
│   │       ├── lib.rs       # register_all() function
│   │       ├── exif_rw.rs   # EXIF reader/writer
│   │       ├── gps_set.rs   # GPS coordinate manager
│   │       ├── time_shift.rs# Time adjustment
│   │       ├── colorspace.rs# Colour space conversion
│   │       ├── lut3d.rs     # 3D LUT colour grading
│   │       ├── transform.rs # Resize/rotate/crop/flip
│   │       ├── lens_correct.rs # Lens correction
│   │       ├── ai_denoise.rs# AI denoising
│   │       ├── heif_encoder.rs  # HEIF encoding
│   │       ├── jxl_encoder.rs   # JPEG XL encoding
│   │       ├── avif_encoder.rs  # AVIF encoding
│   │       ├── tiff_encoder.rs  # TIFF encoding
│   │       ├── png_encoder.rs   # PNG encoding
│   │       └── raw_input.rs     # RAW input
│   ├── external/        # External tool wrappers
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── exiftool.rs
│   │       ├── libvips.rs
│   │       └── commercial.rs
│   └── oiio/            # OIIO FFI bindings (feature-gated)
├── cli/                 # CLI binary
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── pipeline.rs
│   │       ├── plugin.rs
│   │       └── batch.rs
│   └── tests/
│       └── integration_test.rs
├── proto/               # Protobuf definitions
│   ├── pipeline.proto
│   ├── image.proto
│   └── batch.proto
├── halide_generators/   # Halide C++ sources (compiled on CI)
├── examples/            # Example pipeline TOML files
├── gui/                 # Platform GUI projects
│   ├── linux/           # GTK4 + Rust
│   ├── windows/         # WinUI 3 (.NET 8)
│   └── macos/           # SwiftUI
├── justfile             # Task runner
├── Cargo.toml           # Workspace root
└── README.md
```

### Crate Dependency Graph

```
photopipeline-core       (zero internal dependencies)
    ↑
photopipeline-plugin     (depends on core)
    ↑
photopipeline-engine     (depends on core, plugin)
photopipeline-plugins    (depends on core, plugin)
photopipeline-external   (depends on core, plugin)
photopipeline-cli        (depends on core, plugin, engine, plugins)
photopipeline-server     (depends on core, plugin, engine, plugins)
```

---

## Coding Standards

### Rust Style

- Follow `rustfmt` defaults. Run `just fmt` before committing.
- All code must pass `just lint` (clippy with `-D warnings`).
- Use `#![warn(missing_docs)]` on new public items.
- Prefer `&str` over `&String` in function signatures.
- Use `Option<T>` rather than sentinel values (e.g., `-1` for "none").

### Error Handling

- Use `PluginError` and `PluginResult<T>` from `photopipeline-core`.
- Never `unwrap()` or `expect()` in production code; use `?` or pattern matching.
- Validation functions return `Vec<ValidationIssue>` rather than early `Err` returns.

### Async Code

- All plugin trait methods are `async` via `#[async_trait]`.
- Use `tokio` as the runtime (configured in workspace `Cargo.toml`).
- Prefer `futures::future::join_all` for concurrent independent operations.
- Avoid `block_on` in async contexts; use `.await`.

### Naming Conventions

| Element | Convention | Example |
|---|---|---|
| Crate names | `photopipeline-{name}` | `photopipeline-core` |
| Plugin IDs | `photopipeline.plugins.{name}` | `photopipeline.plugins.ai_denoise` |
| Parameter IDs | `snake_case` | `denoise_strength` |
| Node IDs in TOML | `snake_case` | `raw_input_node` |
| Error variant names | `UpperCamelCase` | `GpuOutOfMemory` |
| Test modules | `#[cfg(test)] mod tests` | — |
| Test function names | `snake_case` | `test_resolve_priority` |

### Documentation

- Every public type, trait, and function must have a doc comment (`///`).
- Module‑level docs should explain the module's responsibility.
- Use `# Examples` sections in doc comments for non‑trivial APIs.
- Chinese documentation files follow the same structure as their English counterparts.
- Technical terms (pixel, buffer, GPU, ICC) remain in English within Chinese docs.

---

## Testing

### Running Tests

```bash
# Run all tests
just test

# Run tests for a specific crate
cargo test -p photopipeline-core
cargo test -p photopipeline-plugin
cargo test -p photopipeline-engine

# Run tests with output
cargo test -p photopipeline-engine -- --nocapture

# Run CLI integration tests
cargo test -p photopipeline-cli --test integration_test

# Run tests matching a pattern
cargo test -p photopipeline-engine -- condition_
```

### Test Requirements

1. **Unit tests** go in `#[cfg(test)] mod tests` at the bottom of each source file.
2. **Integration tests** go in `cli/tests/` (for CLI) or `tests/` directory.
3. Each public function should have at least one test covering the happy path.
4. Parameter validation tests must cover boundary values (min, max, just over, just under).
5. Async tests use `tokio::test` or `tokio_test::block_on`.
6. Graph tests must include: empty graph, single node, linear chain, diamond pattern, cycle detection.

### Test Coverage Targets

| Crate | Current | Target |
|---|---|---|
| photopipeline-core | ~90% | 90% |
| photopipeline-plugin | ~85% | 85% |
| photopipeline-engine | ~88% | 90% |
| photopipeline-plugins | ~70% | 80% |
| photopipeline-cli | ~75% | 85% |

---

## Pull Request Process

### Before Submitting

1. Run the full CI check locally:
   ```bash
   just check          # cargo check --workspace
   just lint           # cargo clippy --workspace -- -D warnings
   just fmt-check      # cargo fmt --all -- --check
   just test           # cargo test --workspace
   ```

2. Update documentation if your change affects public APIs:
   - Add/update doc comments on changed public items.
   - If you add a new plugin, update the plugin table in `README.md` and `README_zh.md`.
   - If you add a new CLI subcommand, update `USER_GUIDE.md`.

3. Add a CHANGELOG entry under `## [Unreleased]` following the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

4. Rebase onto `main` and resolve conflicts.

### Submitting

1. Create a branch from `main`:
   ```bash
   git checkout -b feature/my-feature
   ```

2. Push and open a Pull Request against `main`.

3. Title the PR in the format: `[area] Brief description`
   - Examples: `[core] Add XYZ color space`, `[cli] Add --verbose flag`, `[engine] Fix cycle detection for diamond graphs`

4. Fill in the PR template (if one exists) or include:
   - What the change does and why.
   - Which crates are affected.
   - Any breaking changes.
   - Test plan.

### Review

- At least one maintainer must approve.
- CI must pass (build, lint, fmt, test on all platforms).
- Address all review comments before merge.
- Squash merge is preferred for clean history.

---

## Release Process

Release is performed by maintainers:

1. Update `version` in `[workspace.package]` in `Cargo.toml`.
2. Move `[Unreleased]` section in `CHANGELOG.md` to a new version section with the release date.
3. Run the full CI suite: `just check && just lint && just fmt-check && just test`.
4. Tag the release:
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push --tags
   ```
5. The `release.yml` workflow triggers automatically on tag push, building and uploading platform binaries.
6. Create a GitHub Release from the tag with the relevant CHANGELOG section as release notes.

---

## Adding a New Built‑in Plugin

1. Create a new file in `crates/plugins/src/` (e.g., `my_plugin.rs`).
2. Define the plugin struct and implement `Plugin` trait.
3. Define `ParameterSchema` and `GuiSchema` using `LazyLock`.
4. Implement one or more capability traits (`MetadataProcessor`, `PixelProcessor`, `FormatProcessor`, etc.).
5. Add `pub mod my_plugin;` to `crates/plugins/src/lib.rs`.
6. Register the plugin in `register_all()` with the appropriate capability traits.
7. Add unit tests in `#[cfg(test)]` at the bottom of the plugin file.
8. Add integration test in `cli/tests/integration_test.rs`.
9. Update the plugin table in both README files.
10. Update `PLUGIN_DEV.md` if the plugin demonstrates a new pattern.

---

## Questions?

Open a [GitHub Discussion](https://github.com/zhang-hz/photopipeline/discussions) or join the community.
For bug reports, use the [Issue Tracker](https://github.com/zhang-hz/photopipeline/issues).

Thank you for contributing to Photopipeline.

---
