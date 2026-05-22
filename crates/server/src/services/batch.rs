use std::sync::Arc;
use parking_lot::RwLock;
use tonic::{Request, Response, Status};
use futures::stream;

use crate::pb::batch::{
    batch_service_server::BatchService,
    BatchId, BatchProgress, BatchSpec,
};
use crate::SharedState;

pub struct BatchServiceImpl {
    state: Arc<RwLock<SharedState>>,
}

impl BatchServiceImpl {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl BatchService for BatchServiceImpl {
    async fn submit_batch(
        &self,
        request: Request<BatchSpec>,
    ) -> Result<Response<BatchId>, Status> {
        let _spec = request.into_inner();
        let id = uuid::Uuid::new_v4().to_string();
        Ok(Response::new(BatchId { id }))
    }

    type GetProgressStream = stream::Iter<std::vec::IntoIter<Result<BatchProgress, Status>>>;

    async fn get_progress(
        &self,
        request: Request<BatchId>,
    ) -> Result<Response<Self::GetProgressStream>, Status> {
        let _bid = request.into_inner().id;

        let progress = vec![
            BatchProgress {
                status: 0,
                total_files: 10,
                completed_files: 0,
                failed_files: 0,
                current_file: String::new(),
                fraction: 0.0,
                progress_details: String::new(),
            },
            BatchProgress {
                status: 1,
                total_files: 10,
                completed_files: 5,
                failed_files: 0,
                current_file: "DSC0003.ARW".into(),
                fraction: 0.5,
                progress_details: "Processing files...".into(),
            },
            BatchProgress {
                status: 2,
                total_files: 10,
                completed_files: 10,
                failed_files: 0,
                current_file: String::new(),
                fraction: 1.0,
                progress_details: "All done".into(),
            },
        ];

        let iter = progress.into_iter().map(Ok).collect::<Vec<_>>().into_iter();
        Ok(Response::new(stream::iter(iter)))
    }

    async fn cancel(
        &self,
        request: Request<BatchId>,
    ) -> Result<Response<()>, Status> {
        let _bid = request.into_inner().id;
        Ok(Response::new(()))
    }
}
