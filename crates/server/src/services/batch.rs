use photopipeline_core::perf::PerfTimer;
use std::sync::Arc;
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

        self.state.batch_jobs.write().insert(
            batch_id,
            crate::BatchJobState {
                spec,
                total_files: total,
                completed_files: 0,
                failed_files: 0,
                current_file: String::new(),
                status: ProtoStatus::Pending as i32,
            },
        );

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

                let _ = tx.send(Ok(progress));
            }
            None => {
                let _ = tx.send(Err(Status::not_found(format!(
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
                tracing::info!("Batch {} canceled", bid);
                Ok(Response::new(()))
            }
            None => Err(Status::not_found(format!("batch job not found: {}", bid))),
        }
    }
}
