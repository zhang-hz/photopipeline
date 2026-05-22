use std::sync::Arc;
use parking_lot::RwLock;
use tokio::signal;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use photopipeline_server::services::{
    pipeline::PipelineServiceImpl,
    image::ImageServiceImpl,
    batch::BatchServiceImpl,
};
use photopipeline_server::pb::{
    pipeline::pipeline_service_server::PipelineServiceServer,
    image::image_service_server::ImageServiceServer,
    batch::batch_service_server::BatchServiceServer,
};
use photopipeline_server::SharedState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let state = Arc::new(RwLock::new(SharedState::default()));
    let addr = "0.0.0.0:50051".parse()?;

    tracing::info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(PipelineServiceServer::new(PipelineServiceImpl::new(state.clone())))
        .add_service(ImageServiceServer::new(ImageServiceImpl::new(state.clone())))
        .add_service(BatchServiceServer::new(BatchServiceImpl::new(state.clone())))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    tracing::info!("Shutting down gRPC server");
}
