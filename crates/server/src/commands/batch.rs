use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use photopipeline_core::{EncodeOptions, ImageInfo, PerfTimer, PixelBuffer};
use photopipeline_engine::executor::NodeExecutor;
use photopipeline_engine::params::ParameterResolver;
use photopipeline_plugin::{ProgressSink, Registry};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use crate::config;

struct BatchProgress {
    pb: ProgressBar,
    canceled: Arc<AtomicBool>,
}

impl ProgressSink for BatchProgress {
    fn set_progress(&self, fraction: f32, message: &str) {
        self.pb.set_message(message.to_string());
        self.pb.set_position((fraction * 100.0) as u64);
    }

    fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::Relaxed)
    }
}

fn probe_format(path: &std::path::Path) -> photopipeline_core::FormatProbe {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    photopipeline_core::FormatProbe {
        path: Some(path.to_path_buf()),
        extension: ext,
        magic_bytes: None,
        mime_type: None,
    }
}

fn find_decoder(
    registry: &Registry,
    probe: &photopipeline_core::FormatProbe,
) -> Option<Arc<dyn photopipeline_plugin::FormatProcessor>> {
    for fp in registry.all() {
        if let Some(fmt_proc) = registry.get_format_processor(fp.id()) {
            if fmt_proc.can_decode(probe) {
                return Some(fmt_proc);
            }
        }
    }
    None
}

fn find_encoder(
    registry: &Registry,
    ext: &str,
) -> Option<Arc<dyn photopipeline_plugin::FormatProcessor>> {
    let format = match ext.to_lowercase().as_str() {
        "png" => photopipeline_core::ImageFormat::PNG,
        "tiff" | "tif" => photopipeline_core::ImageFormat::TIFF,
        "heif" | "heic" => photopipeline_core::ImageFormat::HEIF,
        "avif" => photopipeline_core::ImageFormat::AVIF,
        "jxl" => photopipeline_core::ImageFormat::JXL,
        _ => photopipeline_core::ImageFormat::PNG,
    };
    for fp in registry.all() {
        if let Some(fmt_proc) = registry.get_format_processor(fp.id()) {
            if fmt_proc.can_encode(&format) {
                return Some(fmt_proc);
            }
        }
    }
    None
}

async fn process_single_file(
    registry: &Arc<Registry>,
    executor: &NodeExecutor,
    graph: &photopipeline_engine::graph::PipelineGraph,
    input_path: &Path,
    output_path: &Path,
    file_index: usize,
    total: usize,
) -> Result<(), String> {
    let probe = probe_format(input_path);
    let decoder = find_decoder(registry, &probe)
        .ok_or_else(|| format!("No decoder for {}", input_path.display()))?;

    let input_bytes =
        std::fs::read(input_path).map_err(|e| format!("Read {}: {}", input_path.display(), e))?;

    let decode_opts = photopipeline_core::DecodeOptions::default();
    let decoded = decoder
        .decode(&input_bytes, &decode_opts)
        .await
        .map_err(|e| format!("Decode {}: {}", input_path.display(), e))?;

    let filename = input_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "input".to_string());

    let image_info = ImageInfo {
        id: Uuid::new_v4(),
        path: input_path.to_string_lossy().to_string(),
        filename,
        format: decoded.format,
        width: decoded.buffer.width,
        height: decoded.buffer.height,
        file_size_bytes: input_bytes.len() as u64,
        pixel_format: decoded.buffer.format,
        color_space: decoded.buffer.color_space.clone(),
    };

    let canceled = Arc::new(AtomicBool::new(false));
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::with_template("  [{bar:30.green}] {msg}").unwrap());

    let progress = Box::new(BatchProgress {
        pb: pb.clone(),
        canceled: canceled.clone(),
    });

    let result = executor
        .execute(
            graph,
            &image_info,
            Some(decoded.buffer),
            &decoded.metadata,
            progress,
        )
        .await
        .map_err(|e| format!("Execute {}: {}", input_path.display(), e))?;

    pb.finish_and_clear();

    let out_ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let encoder =
        find_encoder(registry, out_ext).ok_or_else(|| format!("No encoder for .{}", out_ext))?;

    let out_buffer = result.buffer.unwrap_or_else(|| {
        PixelBuffer::new(
            image_info.width,
            image_info.height,
            photopipeline_core::ChannelLayout::RGB,
            photopipeline_core::PixelFormat::U8,
        )
    });

    let encode_opts = EncodeOptions {
        format: match out_ext.to_lowercase().as_str() {
            "png" => photopipeline_core::ImageFormat::PNG,
            "tiff" | "tif" => photopipeline_core::ImageFormat::TIFF,
            "heif" | "heic" => photopipeline_core::ImageFormat::HEIF,
            "avif" => photopipeline_core::ImageFormat::AVIF,
            "jxl" => photopipeline_core::ImageFormat::JXL,
            _ => photopipeline_core::ImageFormat::PNG,
        },
        ..Default::default()
    };

    let encoded = encoder
        .encode(&out_buffer, &result.metadata, &encode_opts)
        .await
        .map_err(|e| format!("Encode {}: {}", output_path.display(), e))?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Create dir {}: {}", parent.display(), e))?;
    }
    std::fs::write(output_path, &encoded)
        .map_err(|e| format!("Write {}: {}", output_path.display(), e))?;

    tracing::info!(
        index = file_index + 1,
        total = total,
        input = %input_path.display(),
        output = %output_path.display(),
        output_bytes = encoded.len(),
        "[{}/{}] {} -> {} ({} bytes)",
        file_index + 1,
        total,
        input_path.display(),
        output_path.display(),
        encoded.len(),
    );

    Ok(())
}

pub async fn run(registry: &Arc<Registry>, config_path: &str, pattern: &str, output_dir: &str) {
    let _timer = PerfTimer::with_target("batch_cli_run", "cli");
    tracing::info!(
        config_path = config_path,
        pattern = pattern,
        output_dir = output_dir,
        "Running batch"
    );

    let content = std::fs::read_to_string(config_path).unwrap_or_else(|e| {
        tracing::error!(config_path = config_path, error = %e, "Error reading config: {}", e);
        std::process::exit(1);
    });

    let template = config::load_template(&content).unwrap_or_else(|e| {
        tracing::error!(config_path = config_path, error = %e, "Error parsing config: {}", e);
        std::process::exit(1);
    });

    if let Err(e) = template.validate() {
        tracing::error!(config_path = config_path, error = %e, "Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    let graph = template.into_graph();
    let node_count = graph.nodes.len();

    let out_dir = Path::new(output_dir);
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).unwrap_or_else(|e| {
            tracing::error!(output_dir = output_dir, error = %e, "Error creating output directory: {}", e);
            std::process::exit(1);
        });
    }

    let entries: Vec<_> = match glob(pattern) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(e) => {
            tracing::error!(pattern = pattern, error = %e, "Error with glob pattern: {}", e);
            std::process::exit(1);
        }
    };

    if entries.is_empty() {
        println!("No files matched pattern '{}'", pattern);
        return;
    }

    println!(
        "Batch processing {} files ({} nodes each)",
        entries.len(),
        node_count
    );

    let resolver = Arc::new(ParameterResolver::default());
    let executor = NodeExecutor::new(registry.clone(), resolver);

    let total = entries.len();
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;

    for (i, entry) in entries.iter().enumerate() {
        let filename = entry.file_name().unwrap_or_default().to_string_lossy();
        let out_path = out_dir.join(format!("processed_{}", filename));

        println!(
            "[{}/{}] {} -> {}",
            i + 1,
            total,
            entry.display(),
            out_path.display(),
        );

        match process_single_file(registry, &executor, &graph, entry, &out_path, i, total).await {
            Ok(()) => succeeded += 1,
            Err(e) => {
                tracing::error!(file = %entry.display(), error = %e, "Batch file failed");
                eprintln!("  FAILED: {}", e);
                failed += 1;
            }
        }
    }

    println!(
        "Batch complete. {} succeeded, {} failed, {} total.",
        succeeded, failed, total
    );
    tracing::info!(
        succeeded = succeeded,
        failed = failed,
        total = total,
        "Batch complete: {}/{} succeeded",
        succeeded,
        total,
    );
}

pub async fn validate(config_path: &str, pattern: &str) {
    tracing::info!(
        config_path = config_path,
        pattern = pattern,
        "Validating batch config"
    );

    let content = std::fs::read_to_string(config_path).unwrap_or_else(|e| {
        tracing::error!(config_path = config_path, error = %e, "Error reading config: {}", e);
        std::process::exit(1);
    });

    let template = config::load_template(&content).unwrap_or_else(|e| {
        tracing::error!(config_path = config_path, error = %e, "Error parsing config: {}", e);
        std::process::exit(1);
    });

    if let Err(e) = template.validate() {
        tracing::error!(config_path = config_path, error = %e, "Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    let graph = template.into_graph();

    let entries: Vec<_> = match glob(pattern) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(e) => {
            tracing::error!(pattern = pattern, error = %e, "Error with glob pattern: {}", e);
            std::process::exit(1);
        }
    };

    println!("Validation passed.");
    println!("  Pipeline config: {}", config_path);
    println!("  Nodes: {}", graph.nodes.len());
    println!("  Edges: {}", graph.edges.len());
    println!("  Files to process: {}", entries.len());
}
