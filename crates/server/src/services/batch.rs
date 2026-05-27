use photopipeline_core::{
    DecodeOptions, EncodeOptions, FormatProbe, ImageInfo, PerfTimer, PluginError,
};
use photopipeline_engine::executor::NodeExecutor;
use photopipeline_engine::graph::PipelineTemplate;
use photopipeline_plugin::{FormatProcessor, ProgressSink};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::SharedState;
use crate::pb::batch::{
    BatchId, BatchProgress, BatchSpec, batch_progress::Status as ProtoStatus,
    batch_service_server::BatchService,
};

pub struct BatchServiceImpl {
    state: Arc<SharedState>,
}

impl BatchServiceImpl {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self { state }
    }
}

struct NoopProgress {
    canceled: Arc<AtomicBool>,
}

impl ProgressSink for NoopProgress {
    fn set_progress(&self, _fraction: f32, _message: &str) {}
    fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::Relaxed)
    }
}

fn find_decoder<'a>(
    registry: &'a photopipeline_plugin::Registry,
    probe: &FormatProbe,
) -> Option<Arc<dyn FormatProcessor>> {
    for plugin in registry.iter_format_processors() {
        if plugin.can_decode(probe) {
            return Some(plugin);
        }
    }
    None
}

fn find_encoder<'a>(
    registry: &'a photopipeline_plugin::Registry,
    format: &photopipeline_core::ImageFormat,
) -> Option<Arc<dyn FormatProcessor>> {
    for plugin in registry.iter_format_processors() {
        if plugin.can_encode(format) {
            return Some(plugin);
        }
    }
    None
}

async fn process_batch_files(
    state: Arc<SharedState>,
    batch_id: Uuid,
    files: Vec<String>,
    output_dir: String,
    template_str: String,
    cancel_flag: Arc<AtomicBool>,
) {
    let template: PipelineTemplate = match toml::from_str(&template_str) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("batch {}: failed to parse pipeline config: {}", batch_id, e);
            if let Some(job) = state.batch_jobs.write().get_mut(&batch_id) {
                job.status = ProtoStatus::Error as i32;
            }
            return;
        }
    };

    if let Err(e) = template.validate() {
        tracing::error!("batch {}: invalid pipeline: {}", batch_id, e);
        if let Some(job) = state.batch_jobs.write().get_mut(&batch_id) {
            job.status = ProtoStatus::Error as i32;
        }
        return;
    }

    let graph = template.into_graph();
    let resolver = state.resolver.clone();
    let executor = NodeExecutor::new(state.registry.clone(), resolver);

    // Update status to running
    {
        let mut jobs = state.batch_jobs.write();
        if let Some(job) = jobs.get_mut(&batch_id) {
            job.status = ProtoStatus::Running as i32;
        }
    }

    let total = files.len();
    tracing::info!("batch {}: processing {} files", batch_id, total);

    for (i, file_path) in files.iter().enumerate() {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            tracing::warn!("batch {}: canceled at file {}/{}", batch_id, i + 1, total);
            let mut jobs = state.batch_jobs.write();
            if let Some(job) = jobs.get_mut(&batch_id) {
                job.status = ProtoStatus::Canceled as i32;
            }
            return;
        }

        // Update current file
        {
            let mut jobs = state.batch_jobs.write();
            if let Some(job) = jobs.get_mut(&batch_id) {
                job.current_file = file_path.clone();
            }
        }

        tracing::info!(
            "batch {}: [{}/{}] processing {}",
            batch_id,
            i + 1,
            total,
            file_path
        );

        match process_single_file(&executor, &state.registry, file_path, &output_dir, &graph).await
        {
            Ok(()) => {
                let mut jobs = state.batch_jobs.write();
                if let Some(job) = jobs.get_mut(&batch_id) {
                    job.completed_files = (i + 1) as i32;
                }
            }
            Err(e) => {
                tracing::error!("batch {}: failed {}: {}", batch_id, file_path, e);
                let mut jobs = state.batch_jobs.write();
                if let Some(job) = jobs.get_mut(&batch_id) {
                    job.failed_files += 1;
                    job.completed_files = (i + 1) as i32;
                }
            }
        }
    }

    // Mark as completed
    {
        let mut jobs = state.batch_jobs.write();
        if let Some(job) = jobs.get_mut(&batch_id) {
            job.status = ProtoStatus::Done as i32;
            tracing::info!(
                "batch {}: completed. {}/{} succeeded, {} failed",
                batch_id,
                job.completed_files - job.failed_files,
                job.total_files,
                job.failed_files,
            );
        }
    }
}

fn output_format_from_input(input_format: &photopipeline_core::ImageFormat) -> photopipeline_core::ImageFormat {
    use photopipeline_core::ImageFormat;
    match input_format {
        ImageFormat::RAW | ImageFormat::Unknown(_) => ImageFormat::TIFF,
        ImageFormat::HEIF | ImageFormat::AVIF => ImageFormat::TIFF,
        other => other.clone(),
    }
}

fn extension_for_format(format: &photopipeline_core::ImageFormat) -> &'static str {
    use photopipeline_core::ImageFormat;
    match format {
        ImageFormat::TIFF => "tiff",
        ImageFormat::PNG => "png",
        ImageFormat::JPEG => "jpg",
        ImageFormat::WEBP => "webp",
        ImageFormat::JXL => "jxl",
        ImageFormat::AVIF => "avif",
        ImageFormat::HEIF => "heic",
        ImageFormat::OpenEXR => "exr",
        ImageFormat::BMP => "bmp",
        _ => "tiff",
    }
}

async fn process_single_file(
    executor: &NodeExecutor,
    registry: &photopipeline_plugin::Registry,
    input_path: &str,
    output_dir: &str,
    graph: &photopipeline_engine::graph::PipelineGraph,
) -> Result<(), String> {
    // Probe format
    let probe = photopipeline_core::FormatProbe {
        path: Some(std::path::PathBuf::from(input_path)),
        extension: std::path::Path::new(input_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string()),
        magic_bytes: None,
        mime_type: None,
    };

    let decoder =
        find_decoder(registry, &probe).ok_or_else(|| format!("No decoder for {}", input_path))?;

    let input_bytes =
        std::fs::read(input_path).map_err(|e| format!("Read {}: {}", input_path, e))?;

    let decode_opts = DecodeOptions::default();
    let decoded = decoder
        .decode(&input_bytes, &decode_opts)
        .await
        .map_err(|e| format!("Decode {}: {}", input_path, e))?;

    let filename = std::path::Path::new(input_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "input".to_string());

    let image_info = ImageInfo {
        id: Uuid::new_v4(),
        path: input_path.to_string(),
        filename,
        format: decoded.format.clone(),
        width: decoded.buffer.width,
        height: decoded.buffer.height,
        file_size_bytes: input_bytes.len() as u64,
        pixel_format: decoded.buffer.format,
        color_space: decoded.buffer.color_space.clone(),
    };

    let canceled = Arc::new(AtomicBool::new(false));
    let progress = Box::new(NoopProgress {
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
        .map_err(|e| format!("Execute {}: {}", input_path, e))?;

    // Derive output format from input format (default to TIFF for RAW/unknown)
    let output_format = output_format_from_input(&decoded.format);
    let output_ext = extension_for_format(&output_format);

    // Encode output
    let output_path = {
        let stem = std::path::Path::new(input_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        std::path::Path::new(output_dir).join(format!("{}.{}", stem, output_ext))
    };

    if let Some(exec_result) = result.buffer {
        let encoder = find_encoder(registry, &output_format)
            .ok_or_else(|| format!("No {:?} encoder found", output_format))?;

        let encode_opts = EncodeOptions {
            format: output_format,
            quality: None,
            lossless: true,
            bit_depth: 16,
            chroma_subsampling: None,
            encoder: None,
            effort: None,
            compression: Some("lzw".into()),
            embed_profile: Some(true),
        };

        let encoded = encoder
            .encode(&exec_result, &decoded.metadata, &encode_opts)
            .await
            .map_err(|e| format!("Encode {}: {}", output_path.display(), e))?;

        std::fs::write(&output_path, &encoded)
            .map_err(|e| format!("Write {}: {}", output_path.display(), e))?;
    }

    Ok(())
}

#[tonic::async_trait]
impl BatchService for BatchServiceImpl {
    async fn submit_batch(&self, request: Request<BatchSpec>) -> Result<Response<BatchId>, Status> {
        let _timer = PerfTimer::with_target("rpc_submit_batch", "rpc");
        let spec = request.into_inner();
        tracing::info!(
            "submit_batch: config={}, pattern={}, parallel={}",
            spec.pipeline_config_path,
            spec.file_pattern,
            spec.parallel
        );

        let pattern = if spec.file_pattern.is_empty() {
            "*.*".to_string()
        } else {
            spec.file_pattern.clone()
        };

        let output_dir = if spec.output_dir.is_empty() {
            ".".to_string()
        } else {
            spec.output_dir.clone()
        };

        if !std::path::Path::new(&output_dir).exists() {
            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                return Err(Status::internal(format!(
                    "failed to create output directory {}: {}",
                    output_dir, e
                )));
            }
        }

        // Read pipeline config
        let template_str = std::fs::read_to_string(&spec.pipeline_config_path).map_err(|e| {
            Status::not_found(format!(
                "pipeline config '{}' not found: {}",
                spec.pipeline_config_path, e
            ))
        })?;

        let mut files: Vec<String> = Vec::new();
        let norm_pattern = if pattern.starts_with('/') || pattern.contains(':') {
            pattern.clone()
        } else {
            format!("./{}", pattern)
        };

        match glob::glob(&norm_pattern) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.is_file() {
                        files.push(entry.to_string_lossy().to_string());
                    }
                }
            }
            Err(e) => {
                return Err(Status::invalid_argument(format!(
                    "invalid glob pattern '{}': {}",
                    pattern, e
                )));
            }
        }

        if files.is_empty() {
            return Err(Status::not_found(format!(
                "no files matched pattern: {}",
                pattern
            )));
        }

        let batch_id = Uuid::new_v4();
        let total = files.len() as i32;

        tracing::info!("submit_batch: id={}, found {} files", batch_id, total);

        let cancel_flag = Arc::new(AtomicBool::new(false));

        self.state.batch_jobs.write().insert(
            batch_id,
            crate::BatchJobState {
                spec,
                total_files: total,
                completed_files: 0,
                failed_files: 0,
                current_file: String::new(),
                status: ProtoStatus::Pending as i32,
                cancel_flag: Some(cancel_flag.clone()),
            },
        );

        // Spawn background processing
        let state = self.state.clone();
        let output_dir_clone = output_dir.clone();
        tokio::spawn(async move {
            process_batch_files(state, batch_id, files, output_dir_clone, template_str, cancel_flag).await;
        });

        tracing::info!("submit_batch: batch {} started in background", batch_id);

        Ok(Response::new(BatchId {
            id: batch_id.to_string(),
        }))
    }

    type GetProgressStream = ReceiverStream<Result<BatchProgress, Status>>;

    async fn get_progress(
        &self,
        request: Request<BatchId>,
    ) -> Result<Response<Self::GetProgressStream>, Status> {
        let bid = request.into_inner().id;
        tracing::info!("get_progress: batch_id={}", bid);

        let batch_id = Uuid::parse_str(&bid)
            .map_err(|e| Status::invalid_argument(format!("invalid batch id: {}", e)))?;

        let (tx, rx) = mpsc::channel(256);

        let job_state = {
            let jobs = self.state.batch_jobs.read();
            jobs.get(&batch_id).cloned()
        };

        match job_state {
            Some(state) => {
                let total = state.total_files;
                let completed = state.completed_files;
                let failed = state.failed_files;
                let current = state.current_file.clone();
                let fraction = if total > 0 {
                    completed as f32 / total as f32
                } else {
                    0.0
                };
                let status = state.status;

                let progress = BatchProgress {
                    status,
                    total_files: total,
                    completed_files: completed,
                    failed_files: failed,
                    current_file: current,
                    fraction,
                    progress_details: format!(
                        "{}/{} completed, {} failed",
                        completed, total, failed
                    ),
                };

                let _ = tx.try_send(Ok(progress));
            }
            None => {
                let _ = tx.try_send(Err(Status::not_found(format!(
                    "batch job not found: {}",
                    bid
                ))));
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn cancel(&self, request: Request<BatchId>) -> Result<Response<()>, Status> {
        let bid = request.into_inner().id;
        tracing::info!("cancel: batch_id={}", bid);

        let batch_id = Uuid::parse_str(&bid)
            .map_err(|e| Status::invalid_argument(format!("invalid batch id: {}", e)))?;

        let mut jobs = self.state.batch_jobs.write();
        match jobs.get_mut(&batch_id) {
            Some(state) => {
                state.status = ProtoStatus::Canceled as i32;
                if let Some(ref flag) = state.cancel_flag {
                    flag.store(true, Ordering::Relaxed);
                }
                tracing::info!("Batch {} canceled", bid);
                Ok(Response::new(()))
            }
            None => Err(Status::not_found(format!("batch job not found: {}", bid))),
        }
    }
}
