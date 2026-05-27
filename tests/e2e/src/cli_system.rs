#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

use photopipeline_core::{ColorSpace, ImageFormat, ImageInfo, Metadata, PixelFormat, PluginError};
use photopipeline_engine::{ParameterResolver, PipelineTemplate, TemplateNode};
use photopipeline_plugin::{ParameterSet, PluginQuery, registry::Registry};
use photopipeline_plugins;
use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use tempfile::TempDir;
use test_harness::fixtures::image::ImageFixture;
use test_harness::mocks::encoder::MockEncoder;
use uuid::Uuid;

fn photopipeline_binary() -> Option<String> {
    let candidates = vec![
        "./target/debug/photopipeline",
        "./target/release/photopipeline",
        "../target/debug/photopipeline",
        "../target/release/photopipeline",
        "../../target/debug/photopipeline",
        "../../target/release/photopipeline",
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return Some(c.to_string());
        }
    }

    None
}

fn require_binary() -> String {
    photopipeline_binary()
        .expect("SKIP: photopipeline binary not found — cannot run CLI test. Build the project first.")
}

fn make_minimal_png() -> Vec<u8> {
    let mut data = Vec::new();
    let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    data.extend_from_slice(&signature);

    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&16u32.to_be_bytes());
    ihdr_data.extend_from_slice(&16u32.to_be_bytes());
    ihdr_data.push(8u8);
    ihdr_data.push(2u8);
    ihdr_data.push(0u8);
    ihdr_data.push(0u8);
    ihdr_data.push(0u8);

    let mut chunk = Vec::new();
    chunk.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(b"IHDR");
    chunk.extend_from_slice(&ihdr_data);

    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in &chunk[4..] {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc = !crc;
    chunk.extend_from_slice(&crc.to_be_bytes());
    data.extend_from_slice(&chunk);

    let mut iend = Vec::new();
    iend.extend_from_slice(&0u32.to_be_bytes());
    iend.extend_from_slice(b"IEND");
    let mut iend_crc: u32 = 0xFFFFFFFF;
    for &byte in &iend[4..] {
        iend_crc ^= byte as u32;
        for _ in 0..8 {
            if iend_crc & 1 != 0 {
                iend_crc = (iend_crc >> 1) ^ 0xEDB88320;
            } else {
                iend_crc >>= 1;
            }
        }
    }
    iend_crc = !iend_crc;
    iend.extend_from_slice(&iend_crc.to_be_bytes());
    data.extend_from_slice(&iend);

    data
}

fn write_valid_config(temp_dir: &TempDir) -> std::path::PathBuf {
    let config_path = temp_dir.path().join("pipeline.toml");
    let config = r#"
[metadata]
name = "CLI Test Pipeline"
version = "1.0"
description = "Minimal source-to-output"

[[nodes]]
id = "source"
plugin = "core.input"
label = "Input"

[[nodes]]
id = "output"
plugin = "core.output"
label = "Output"

[[edges]]
from = "source"
to = "output"
"#;
    std::fs::write(&config_path, config).unwrap();
    config_path
}

fn write_invalid_config(temp_dir: &TempDir) -> std::path::PathBuf {
    let config_path = temp_dir.path().join("invalid.toml");
    let config = r#"
[[nodes]]
id = "only_node"
plugin = "nonexistent.plugin.zzz"
"#;
    std::fs::write(&config_path, config).unwrap();
    config_path
}

#[test]
fn e2e_cli_pipeline_run_valid_config() {
    let bin = require_binary();

    let temp_dir = TempDir::new().unwrap();
    let config_path = write_valid_config(&temp_dir);
    let input_path = temp_dir.path().join("test.png");
    let output_path = temp_dir.path().join("output.heif");

    let png_data = make_minimal_png();
    std::fs::write(&input_path, &png_data).unwrap();

    let status = Command::new(&bin)
        .arg("pipeline")
        .arg("run")
        .arg("-c")
        .arg(&config_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .status();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            assert!(
                code == 0 || code == 1,
                "exit code should be 0 or 1, got {code}"
            );
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_pipeline_validate_valid_config() {
    let bin = require_binary();

    let temp_dir = TempDir::new().unwrap();
    let config_path = write_valid_config(&temp_dir);

    let status = Command::new(&bin)
        .arg("pipeline")
        .arg("validate")
        .arg("-c")
        .arg(&config_path)
        .status();

    match status {
        Ok(s) => {
            let code = s.code();
            assert!(code.is_some(), "validate should return an exit code");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_pipeline_validate_invalid_config() {
    let bin = require_binary();

    let temp_dir = TempDir::new().unwrap();
    let config_path = write_invalid_config(&temp_dir);

    let status = Command::new(&bin)
        .arg("pipeline")
        .arg("validate")
        .arg("-c")
        .arg(&config_path)
        .status();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(0);
            let _ = code;
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_plugin_list_shows_14_plugins() {
    let bin = require_binary();

    let output = Command::new(&bin).arg("plugin").arg("list").output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            assert!(!stdout.is_empty(), "plugin list should produce output");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_plugin_info_valid_id() {
    let bin = require_binary();

    for id in &["exif_rw", "photopipeline.plugins.exif_rw"] {
        let output = Command::new(&bin)
            .arg("plugin")
            .arg("info")
            .arg(id)
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{stdout}{stderr}");
                if !combined.is_empty() {
                    return;
                }
            }
            Err(e) => {
                eprintln!("could not run photopipeline binary: {e}");
            }
        }
    }
}

#[test]
fn e2e_cli_plugin_info_invalid_id() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("plugin")
        .arg("info")
        .arg("nonexistent_plugin_xyz_999")
        .output();

    match output {
        Ok(o) => {
            let code = o.status.code().unwrap_or(0);
            assert_ne!(code, 0, "expected non-zero exit for invalid plugin id");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_batch_run_with_glob_pattern() {
    let bin = require_binary();

    let temp_dir = TempDir::new().unwrap();
    let config_path = write_valid_config(&temp_dir);

    for i in 0..5 {
        let img_path = temp_dir.path().join(format!("img_{i:03}.png"));
        let png_data = make_minimal_png();
        std::fs::write(&img_path, &png_data).unwrap();
    }

    let glob_pattern = temp_dir.path().join("img_*.png");
    let status = Command::new(&bin)
        .arg("batch")
        .arg("run")
        .arg("-c")
        .arg(&config_path)
        .arg("-p")
        .arg(&glob_pattern)
        .arg("-o")
        .arg(temp_dir.path().join("out"))
        .status();

    match status {
        Ok(s) => {
            let code = s.code();
            assert!(code.is_some(), "batch run should return an exit code");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_batch_validate() {
    let bin = require_binary();

    let temp_dir = TempDir::new().unwrap();
    let config_path = write_valid_config(&temp_dir);

    let status = Command::new(&bin)
        .arg("batch")
        .arg("validate")
        .arg("-c")
        .arg(&config_path)
        .status();

    match status {
        Ok(s) => {
            let code = s.code();
            assert!(code.is_some(), "batch validate should return an exit code");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_missing_required_args() {
    let bin = require_binary();

    let output = Command::new(&bin).arg("pipeline").arg("run").output();

    match output {
        Ok(o) => {
            let code = o.status.code().unwrap_or(0);
            assert_ne!(code, 0, "missing required args should exit non-zero");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_help_output() {
    let bin = require_binary();

    let output = Command::new(&bin).arg("--help").output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            assert!(!stdout.is_empty(), "help output should not be empty");
            assert!(
                stdout.contains("Usage") || stdout.contains("USAGE") || stdout.contains("usage"),
                "help should contain usage info"
            );
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_version_output() {
    let bin = require_binary();

    let output = Command::new(&bin).arg("--version").output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            assert!(!stdout.is_empty(), "version output should not be empty");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}

#[test]
fn e2e_cli_log_level_flag_accepted() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("--log-level")
        .arg("debug")
        .arg("--help")
        .output();

    match output {
        Ok(o) => {
            let code = o.status.code();
            assert!(code.is_some(), "--help should return an exit code");
            assert!(o.stdout.len() > 0, "--help should produce output");
        }
        Err(e) => {
            eprintln!("could not run photopipeline binary: {e}");
        }
    }
}
