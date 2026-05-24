use indicatif::{ProgressBar, ProgressStyle};
use photopipeline_core::{EncodeOptions, FormatProbe, ImageInfo, PerfTimer, PixelBuffer};
use photopipeline_engine::executor::NodeExecutor;
use photopipeline_engine::params::ParameterResolver;
use photopipeline_plugin::{ProgressSink, Registry};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use crate::config;

struct CliProgress {
    pb: ProgressBar,
    canceled: Arc<AtomicBool>,
}

impl ProgressSink for CliProgress {
    fn set_progress(&self, fraction: f32, message: &str) {
        self.pb.set_message(message.to_string());
        self.pb.set_position((fraction * 100.0) as u64);
    }

    fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::Relaxed)
    }
}

fn probe_format(path: &str) -> FormatProbe {
    let p = Path::new(path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    FormatProbe {
        path: Some(p.to_path_buf()),
        extension: ext,
        magic_bytes: None,
        mime_type: None,
    }
}

fn find_decoder(
    registry: &Registry,
    probe: &FormatProbe,
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
    for fp in registry.all() {
        if let Some(fmt_proc) = registry.get_format_processor(fp.id()) {
            if fmt_proc.can_encode(&ext_to_format(ext)) {
                return Some(fmt_proc);
            }
        }
    }
    None
}

fn ext_to_format(ext: &str) -> photopipeline_core::ImageFormat {
    match ext.to_lowercase().as_str() {
        "png" => photopipeline_core::ImageFormat::PNG,
        "tiff" | "tif" => photopipeline_core::ImageFormat::TIFF,
        "heif" | "heic" => photopipeline_core::ImageFormat::HEIF,
        "avif" => photopipeline_core::ImageFormat::AVIF,
        "jxl" => photopipeline_core::ImageFormat::JXL,
        "jpg" | "jpeg" => photopipeline_core::ImageFormat::JPEG,
        _ => photopipeline_core::ImageFormat::PNG,
    }
}

pub async fn run(registry: &Arc<Registry>, config_path: &str, input: &str, output: &str) {
    let _timer = PerfTimer::with_target("pipeline_cli_run", "cli");
    tracing::info!(
        config_path = config_path,
        input = input,
        output = output,
        "Running pipeline"
    );

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error reading config '{}': {}", config_path, e);
            std::process::exit(1);
        }
    };

    let template = match config::load_template(&content) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = template.validate() {
        tracing::error!(config_path = config_path, error = %e, "Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    if !Path::new(input).exists() {
        tracing::error!(input = input, "Error: input file '{}' not found", input);
        std::process::exit(1);
    }

    // --- Decode input ---
    let input_probe = probe_format(input);
    let decoder = find_decoder(registry, &input_probe).unwrap_or_else(|| {
        tracing::error!(input = input, "No decoder found for input file");
        std::process::exit(1);
    });

    let input_bytes = match std::fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(input = input, error = %e, "Error reading input file: {}", e);
            std::process::exit(1);
        }
    };

    let decode_opts = photopipeline_core::DecodeOptions::default();
    let decoded = match decoder.decode(&input_bytes, &decode_opts).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(input = input, error = %e, "Error decoding input: {}", e);
            std::process::exit(1);
        }
    };

    let input_filename = Path::new(input)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "input".to_string());

    let image_info = ImageInfo {
        id: Uuid::new_v4(),
        path: input.to_string(),
        filename: input_filename,
        format: decoded.format,
        width: decoded.buffer.width,
        height: decoded.buffer.height,
        file_size_bytes: input_bytes.len() as u64,
        pixel_format: decoded.buffer.format,
        color_space: decoded.buffer.color_space.clone(),
    };

    let graph = template.into_graph();
    let node_count = graph.nodes.len();
    let resolver = Arc::new(ParameterResolver::default());
    let executor = NodeExecutor::new(registry.clone(), resolver);

    println!("Running pipeline: {} nodes", node_count);
    println!(
        "  Input:  {} ({}x{})",
        input, image_info.width, image_info.height
    );
    println!("  Output: {}", output);

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% ({msg})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let canceled = Arc::new(AtomicBool::new(false));
    let progress = Box::new(CliProgress {
        pb: pb.clone(),
        canceled: canceled.clone(),
    });

    let result = executor
        .execute(
            &graph,
            &image_info,
            Some(decoded.buffer),
            &decoded.metadata,
            progress,
        )
        .await;

    match result {
        Ok(exec_result) => {
            pb.finish_with_message("encoding output");

            // --- Encode output ---
            let out_ext = Path::new(output)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");

            let encoder = find_encoder(registry, out_ext).unwrap_or_else(|| {
                tracing::error!(ext = out_ext, "No encoder found for output format");
                std::process::exit(1);
            });

            let out_buffer = exec_result.buffer.unwrap_or_else(|| {
                // No pixel buffer produced — create empty placeholder (shouldn't happen normally)
                PixelBuffer::new(
                    image_info.width,
                    image_info.height,
                    photopipeline_core::ChannelLayout::RGB,
                    photopipeline_core::PixelFormat::U8,
                )
            });

            let encode_opts = EncodeOptions {
                format: ext_to_format(out_ext),
                ..Default::default()
            };

            let encoded = match encoder
                .encode(&out_buffer, &exec_result.metadata, &encode_opts)
                .await
            {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(output = output, error = %e, "Error encoding output: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = std::fs::write(output, &encoded) {
                tracing::error!(output = output, error = %e, "Error writing output file: {}", e);
                std::process::exit(1);
            }

            pb.finish_with_message("complete");
            println!("Pipeline completed successfully.");
            println!("  Output: {} ({} bytes)", output, encoded.len());
        }
        Err(e) => {
            pb.finish_with_message("failed");
            tracing::error!(error = %e, "Pipeline execution failed: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn validate(config_path: &str) {
    tracing::info!(config_path = config_path, "Validating pipeline config");
    tracing::debug!("Loading pipeline config from '{}'", config_path);

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error reading config '{}': {}", config_path, e);
            std::process::exit(1);
        }
    };

    let template = match config::load_template(&content) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = template.validate() {
        tracing::error!(config_path = config_path, error = %e, "Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    let graph = template.into_graph();

    println!("Pipeline config is valid.");
    println!("  Nodes: {}", graph.nodes.len());
    println!("  Edges: {}", graph.edges.len());
}
