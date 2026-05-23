use std::sync::Arc;
use tokio::signal;
use tonic::transport::Server;

use photopipeline_engine::ParameterResolver;
use photopipeline_plugin::Registry;

use photopipeline_server::SharedState;
use photopipeline_server::pb::{
    batch::batch_service_server::BatchServiceServer,
    image::image_service_server::ImageServiceServer,
    pipeline::pipeline_service_server::PipelineServiceServer,
};
use photopipeline_server::services::{
    batch::BatchServiceImpl, image::ImageServiceImpl, pipeline::PipelineServiceImpl,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    photopipeline_core::telemetry::init_telemetry(photopipeline_core::telemetry::TelemetryConfig {
        output: photopipeline_core::telemetry::LogOutput::Console,
        default_filter: "photopipeline_server=info".to_string(),
        ansi_colors: true,
        ..Default::default()
    });

    photopipeline_core::panic_hook::install_panic_hook();

    let registry = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&registry);
    let plugin_count = registry.all().len();
    tracing::info!("Registered {} plugins", plugin_count);
    tracing::debug!(plugin_count = plugin_count, "Plugin registration complete");

    let resolver = Arc::new(ParameterResolver::new());

    let state = Arc::new(SharedState::new(registry, resolver));

    let addr = "0.0.0.0:50051".parse()?;

    tracing::info!(
        "Photopipeline gRPC server v{} starting on {}",
        env!("CARGO_PKG_VERSION"),
        addr
    );

    Server::builder()
        .add_service(PipelineServiceServer::new(PipelineServiceImpl::new(
            state.clone(),
        )))
        .add_service(ImageServiceServer::new(ImageServiceImpl::new(
            state.clone(),
        )))
        .add_service(BatchServiceServer::new(BatchServiceImpl::new(
            state.clone(),
        )))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    tracing::info!("gRPC server stopped");
    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    tracing::info!("Shutting down gRPC server");
}
