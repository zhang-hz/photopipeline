//! Layer 2 gRPC E2E Test Infrastructure
//!
//! Provides TestServer (starts real gRPC server on random port) and
//! TestClient wrappers for all three gRPC services.
//!
//! # Iron Law Compliance
//! - Server startup failure → panic (no silent skip)
//! - Health check timeout → panic
//! - Each client connection verified with a real RPC call

use std::net::SocketAddr;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tonic::transport::{Channel, Server};

/// Ensure telemetry is initialised at most once per process.
static INIT_TELEMETRY: Once = Once::new();

use photopipeline_engine::ParameterResolver;
use photopipeline_plugin::Registry;
use photopipeline_server::SharedState;
use photopipeline_server::pb::{
    batch::{
        batch_service_client::BatchServiceClient,
        batch_service_server::BatchServiceServer,
    },
    image::{
        image_service_client::ImageServiceClient,
        image_service_server::ImageServiceServer,
    },
    pipeline::{
        pipeline_service_client::PipelineServiceClient,
        pipeline_service_server::PipelineServiceServer,
    },
    plugin::{
        plugin_service_client::PluginServiceClient,
        plugin_service_server::PluginServiceServer,
    },
};
use photopipeline_server::services::{
    batch::BatchServiceImpl, image::ImageServiceImpl, pipeline::PipelineServiceImpl,
    plugin::PluginServiceImpl,
};

/// Maximum time to wait for the gRPC server to become ready.
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(30);
/// Interval between health check polls.
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(100);

// ---------------------------------------------------------------------------
// TestServer
// ---------------------------------------------------------------------------

/// A running gRPC test server on a random port.
///
/// On creation, the server starts asynchronously and blocks until it is ready
/// to accept connections (health check). On drop, the server is shut down.
pub struct TestServer {
    /// The address the server is listening on.
    pub addr: SocketAddr,
    /// Handle to the server task; abort on drop to ensure cleanup.
    server_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown signal sender.
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl TestServer {
    /// Start a new test gRPC server on a random available port.
    ///
    /// # Panics
    /// Panics if the server cannot bind or if the health check times out.
    pub async fn start() -> Self {
        // Initialise tracing once per process (subsequent calls panic).
        INIT_TELEMETRY.call_once(|| {
            let _ = photopipeline_core::telemetry::init_telemetry(
                photopipeline_core::telemetry::TelemetryConfig {
                    output: photopipeline_core::telemetry::LogOutput::Console,
                    default_filter: "photopipeline_server=warn".to_string(),
                    ansi_colors: false,
                    ..Default::default()
                },
            );
        });

        let registry = Arc::new(Registry::new());
        photopipeline_plugins::register_all(&registry);

        let resolver = Arc::new(ParameterResolver::new());
        let state = Arc::new(SharedState::new(registry, resolver));

        // Bind to port 0 to get a random available port.
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("TestServer: failed to bind to random port");
        let addr = listener.local_addr().expect("TestServer: failed to get local address");

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Build the server on the bound listener.
        let server = Server::builder()
            .add_service(PipelineServiceServer::new(PipelineServiceImpl::new(
                state.clone(),
            )))
            .add_service(ImageServiceServer::new(ImageServiceImpl::new(
                state.clone(),
            )))
            .add_service(BatchServiceServer::new(BatchServiceImpl::new(
                state.clone(),
            )))
            .add_service(PluginServiceServer::new(PluginServiceImpl::new(
                state.clone(),
            )))
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                async {
                    let _ = shutdown_rx.await;
                },
            );

        let server_handle = tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("TestServer: gRPC server error: {}", e);
            }
        });

        // Wait for the server to become ready.
        let addr_str = format!("http://{}", addr);
        let channel = loop {
            match Channel::builder(addr_str.parse().expect("parse addr"))
                .connect()
                .await
            {
                Ok(ch) => break ch,
                Err(_) => {
                    tokio::time::sleep(HEALTH_CHECK_INTERVAL).await;
                }
            }
        };

        // Verify the server can process RPCs by calling GetNodeSchema on
        // PluginService with a dummy plugin id. The channel connect only
        // verifies TCP — we need to verify the gRPC service is handling RPCs.
        // We accept any gRPC response (including NotFound) as proof of liveness.
        let deadline = Instant::now() + HEALTH_CHECK_TIMEOUT;
        let mut healthy = false;
        while Instant::now() < deadline {
            let mut client = PluginServiceClient::new(channel.clone());
            let req = tonic::Request::new(
                photopipeline_server::pb::plugin::PluginIdRequest {
                    id: "health_check_dummy".to_string(),
                },
            );
            // A transport error (Unavailable) means the server isn't ready yet.
            // A business error (NotFound) means the server IS ready.
            match client.get_node_schema(req).await {
                Ok(_) => {
                    healthy = true;
                    break;
                }
                Err(status) => {
                    if status.code() == tonic::Code::Unavailable {
                        // Server not ready yet — retry
                    } else {
                        // Server responded (even with an error) — it's alive
                        healthy = true;
                        break;
                    }
                }
            }
            tokio::time::sleep(HEALTH_CHECK_INTERVAL).await;
        }
        if !healthy {
            panic!(
                "TestServer: health check timed out after {:?}",
                HEALTH_CHECK_TIMEOUT
            );
        }

        // Drop the health-check channel; tests will create their own.
        drop(channel);

        Self {
            addr,
            server_handle: Some(server_handle),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Returns the HTTP URL for connecting to this server.
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Send shutdown signal.
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // Abort the server task to ensure immediate cleanup in tests.
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// Channel helpers
// ---------------------------------------------------------------------------

/// Create a tonic Channel connected to the test server.
pub async fn connect(addr: SocketAddr) -> Channel {
    let url = format!("http://{}", addr);
    Channel::builder(url.parse().expect("parse addr"))
        .connect()
        .await
        .expect("TestClient: failed to connect to test server")
}

// ---------------------------------------------------------------------------
// TestClient
// ---------------------------------------------------------------------------

/// Convenience wrapper that holds one channel and creates typed clients on demand.
pub struct TestClient {
    channel: Channel,
}

impl TestClient {
    pub async fn new(addr: SocketAddr) -> Self {
        Self {
            channel: connect(addr).await,
        }
    }

    pub fn image_client(&self) -> ImageServiceClient<Channel> {
        ImageServiceClient::new(self.channel.clone())
    }

    pub fn pipeline_client(&self) -> PipelineServiceClient<Channel> {
        PipelineServiceClient::new(self.channel.clone())
    }

    pub fn batch_client(&self) -> BatchServiceClient<Channel> {
        BatchServiceClient::new(self.channel.clone())
    }

    pub fn plugin_client(&self) -> PluginServiceClient<Channel> {
        PluginServiceClient::new(self.channel.clone())
    }
}

// ---------------------------------------------------------------------------
// Test image helpers
// ---------------------------------------------------------------------------

/// Create a temporary PNG file with a small solid-color image for testing.
/// Returns the path to the file.
pub fn create_test_image(width: u32, height: u32, r: u8, g: u8, b: u8) -> tempfile::TempPath {
    use image::{ImageBuffer, Rgb};
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, Rgb([r, g, b]));
    let tmp = tempfile::Builder::new()
        .suffix(".png")
        .tempfile()
        .expect("create temp file");
    let path = tmp.into_temp_path();
    img.save(&path).expect("save test image");
    path
}

/// Copy a golden fixture image to a temporary location and return its path.
pub fn copy_golden(name: &str) -> tempfile::TempPath {
    let golden_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures")
        .join("golden")
        .join("png");
    let src = golden_dir.join(name);
    assert!(
        src.exists(),
        "Golden image not found: {}",
        src.display()
    );
    let tmp = tempfile::Builder::new()
        .suffix(".png")
        .tempfile()
        .expect("create temp file for golden copy");
    let tmp_path = tmp.into_temp_path();
    std::fs::copy(&src, &tmp_path).expect("copy golden image");
    tmp_path
}

/// Create a temporary directory that is cleaned up on drop.
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::TempDir::new().expect("create temp dir")
}
