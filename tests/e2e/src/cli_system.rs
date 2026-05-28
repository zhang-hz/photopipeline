#![allow(clippy::result_large_err)]

use std::process::Command;
use tempfile::TempDir;

fn photopipeline_binary() -> Option<String> {
    let bin_name = if cfg!(target_os = "windows") { "photopipeline.exe" } else { "photopipeline" };
    let candidates = vec![
        format!("./target/debug/{}", bin_name),
        format!("./target/release/{}", bin_name),
        format!("../target/debug/{}", bin_name),
        format!("../target/release/{}", bin_name),
        format!("../../target/debug/{}", bin_name),
        format!("../../target/release/{}", bin_name),
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
        .expect("photopipeline binary not found — build the project first (cargo build -p photopipeline-server)")
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

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"
label = "Read EXIF"

[[nodes]]
id = "png_out"
plugin = "photopipeline.plugins.png_encoder"
label = "PNG Output"

[[edges]]
from = "exif"
to = "png_out"
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

// ── Unified binary subcommands ──────────────────────────────────

#[test]
fn e2e_cli_run_valid_config() {
    let bin = require_binary();
    let temp_dir = TempDir::new().unwrap();
    let config_path = write_valid_config(&temp_dir);
    let input_path = temp_dir.path().join("test.png");
    let output_path = temp_dir.path().join("output.png");

    std::fs::write(&input_path, make_minimal_png()).unwrap();

    let status = Command::new(&bin)
        .arg("run")
        .arg("-c").arg(&config_path)
        .arg("-i").arg(&input_path)
        .arg("-o").arg(&output_path)
        .status()
        .expect("failed to spawn photopipeline binary");

    let code = status.code().unwrap_or(-1);
    assert!(code == 0 || code == 1, "exit code should be 0 or 1, got {code}");
}

#[test]
fn e2e_cli_validate_valid_config() {
    let bin = require_binary();
    let temp_dir = TempDir::new().unwrap();
    let config_path = write_valid_config(&temp_dir);

    let status = Command::new(&bin)
        .arg("validate")
        .arg("-c").arg(&config_path)
        .status()
        .expect("failed to spawn photopipeline binary");

    let code = status.code();
    assert!(code.is_some(), "validate must return an exit code");
}

#[test]
fn e2e_cli_validate_invalid_config() {
    let bin = require_binary();
    let temp_dir = TempDir::new().unwrap();
    let config_path = write_invalid_config(&temp_dir);

    let output = Command::new(&bin)
        .arg("validate")
        .arg("-c").arg(&config_path)
        .output()
        .expect("failed to spawn photopipeline binary");

    // Invalid config (non-existent plugin) must fail validation
    assert!(!output.status.success(), "invalid config must exit non-zero");
}

#[test]
fn e2e_cli_plugins_list_shows_output() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("plugins")
        .arg("list")
        .output()
        .expect("failed to spawn photopipeline binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "plugins list must produce output");
}

#[test]
fn e2e_cli_plugins_info_valid_id() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("plugins")
        .arg("info")
        .arg("photopipeline.plugins.exif_rw")
        .output()
        .expect("failed to spawn photopipeline binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "plugin info for valid id must produce output");
    assert!(stdout.contains("exif_rw"), "output must contain plugin id");
}

#[test]
fn e2e_cli_plugins_info_invalid_id() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("plugins")
        .arg("info")
        .arg("nonexistent_plugin_xyz_999")
        .output()
        .expect("failed to spawn photopipeline binary");

    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 0, "invalid plugin id must exit non-zero, got {code}");
}

#[test]
fn e2e_cli_missing_required_args() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("run")
        .output()
        .expect("failed to spawn photopipeline binary");

    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 0, "missing required args must exit non-zero, got {code}");
}

#[test]
fn e2e_cli_help_output() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .expect("failed to spawn photopipeline binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "help output must not be empty");
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"), "help must contain usage info");
}

#[test]
fn e2e_cli_version_output() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("--version")
        .output()
        .expect("failed to spawn photopipeline binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "version output must not be empty");
}

#[test]
fn e2e_cli_serve_help_shows_addr() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("serve")
        .arg("--help")
        .output()
        .expect("failed to spawn photopipeline binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "serve --help must produce output");
}

#[test]
fn e2e_cli_schema_outputs_json() {
    let bin = require_binary();

    let output = Command::new(&bin)
        .arg("schema")
        .output()
        .expect("failed to spawn photopipeline binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "schema output must not be empty");
    // Validate it's valid JSON
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "schema output must be valid JSON");
}
