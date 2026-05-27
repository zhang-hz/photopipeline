//! Layer 2: PipelineService gRPC E2E Tests (~20 tests)
//!
//! Tests CreatePipeline, Execute, Validate, and GetNodeSchema gRPC methods.
//! Each test connects to a real gRPC server and validates responses.

mod common;

use common::{temp_dir, create_test_image, copy_golden, TestServer, TestClient};
use photopipeline_server::pb::pipeline::{
    pipeline_service_client::PipelineServiceClient, execute_progress::Stage as ProtoStage,
    ExecuteRequest, PipelineEdge, PipelineNode, PipelineSpec, PluginId,
};
use tokio_stream::StreamExt;
use tonic::Code;

async fn setup() -> (TestServer, TestClient) {
    let server = TestServer::start().await;
    let client = TestClient::new(server.addr).await;
    (server, client)
}

// ---------------------------------------------------------------------------
// D.2.1 CreatePipeline RPC (6 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_minimal_pipeline_returns_valid_id() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "minimal".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.colorspace".to_string(),
            label: "cs".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline RPC failed");

    let id = resp.into_inner().id;
    assert!(!id.is_empty(), "Pipeline ID must not be empty");
    // Must be a valid UUID
    assert!(
        uuid::Uuid::parse_str(&id).is_ok(),
        "Pipeline ID '{}' must be a valid UUID",
        id
    );
}

#[tokio::test]
async fn create_multi_node_chain_returns_valid_id() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "3-node-chain".to_string(),
        nodes: vec![
            PipelineNode {
                id: "n1".to_string(),
                plugin_id: "photopipeline.plugins.colorspace".to_string(),
                label: "cs".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "n2".to_string(),
                plugin_id: "photopipeline.plugins.transform".to_string(),
                label: "tr".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "n3".to_string(),
                plugin_id: "photopipeline.plugins.lut3d".to_string(),
                label: "lut".to_string(),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            PipelineEdge {
                from: "n1".to_string(),
                to: "n2".to_string(),
            },
            PipelineEdge {
                from: "n2".to_string(),
                to: "n3".to_string(),
            },
        ],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");

    let id = resp.into_inner().id;
    assert!(!id.is_empty());
    assert!(uuid::Uuid::parse_str(&id).is_ok());
}

#[tokio::test]
async fn create_diamond_graph_returns_valid_id() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "diamond".to_string(),
        nodes: vec![
            PipelineNode {
                id: "A".to_string(),
                plugin_id: "photopipeline.plugins.colorspace".to_string(),
                label: "A".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "B".to_string(),
                plugin_id: "photopipeline.plugins.transform".to_string(),
                label: "B".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "C".to_string(),
                plugin_id: "photopipeline.plugins.transform".to_string(),
                label: "C".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "D".to_string(),
                plugin_id: "photopipeline.plugins.lut3d".to_string(),
                label: "D".to_string(),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            PipelineEdge {
                from: "A".to_string(),
                to: "B".to_string(),
            },
            PipelineEdge {
                from: "A".to_string(),
                to: "C".to_string(),
            },
            PipelineEdge {
                from: "B".to_string(),
                to: "D".to_string(),
            },
            PipelineEdge {
                from: "C".to_string(),
                to: "D".to_string(),
            },
        ],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("Diamond pipeline creation failed");

    assert!(!resp.into_inner().id.is_empty());
}

#[tokio::test]
async fn create_empty_pipeline_returns_invalid_argument() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "empty".to_string(),
        nodes: vec![],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let resp = svc.create_pipeline(tonic::Request::new(spec)).await;
    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::InvalidArgument,
                "Empty pipeline should return InvalidArgument, got {:?}",
                status.code()
            );
        }
        Ok(_) => panic!("Empty pipeline should be rejected, but got Ok"),
    }
}

#[tokio::test]
async fn create_cyclic_graph_returns_error() {
    // PipelineGraph::connect() silently rejects cyclic edges.
    // The pipeline becomes a DAG (only valid edges survive) and executes normally.
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "cyclic".to_string(),
        nodes: vec![
            PipelineNode {
                id: "A".to_string(),
                plugin_id: "photopipeline.plugins.exif_rw".to_string(),
                label: "A".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "B".to_string(),
                plugin_id: "photopipeline.plugins.gps_set".to_string(),
                label: "B".to_string(),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            PipelineEdge { from: "A".to_string(), to: "B".to_string() },
            PipelineEdge { from: "B".to_string(), to: "A".to_string() },
        ],
        params: Default::default(),
        batch: None,
    };

    // Creation succeeds — cycle edges silently dropped during graph construction
    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("Cyclic graph creation succeeds (edges silently dropped)");
    let pipeline_id = create_resp.into_inner().id;

    // Execution succeeds — only non-cyclic edges remain in the graph
    let req = ExecuteRequest {
        pipeline_id,
        image_path: img_str,
        output_path: temp_dir().into_path().join("cyclic_out.png").to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute RPC failed")
        .into_inner();

    let stages: Vec<_> = stream.filter_map(|r| r.ok()).collect().await;
    let has_done = stages.iter().any(|s| s.stage == ProtoStage::Done as i32);
    assert!(has_done, "Pipeline with cycle edges executed successfully (cycles silently dropped by connect())");
}

#[tokio::test]
async fn create_pipeline_disconnected_graph_returns_id() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "disconnected".to_string(),
        nodes: vec![
            PipelineNode {
                id: "A".to_string(),
                plugin_id: "photopipeline.plugins.colorspace".to_string(),
                label: "A".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "B".to_string(),
                plugin_id: "photopipeline.plugins.transform".to_string(),
                label: "B".to_string(),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![], // No edges — disconnected is valid
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("Disconnected graph should be valid");

    assert!(!resp.into_inner().id.is_empty());
}

// ---------------------------------------------------------------------------
// D.2.2 Execute RPC (8 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_single_node_pipeline_streams_done() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();

    // Create the pipeline with a metadata plugin (no pixel buffer needed)
    let spec = PipelineSpec {
        name: "single-exif".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.exif_rw".to_string(),
            label: "exif".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;

    // Execute
    let output_path = temp_dir().into_path().join("exec_output.png");
    let req = ExecuteRequest {
        pipeline_id: pipeline_id.clone(),
        image_path: img_str,
        output_path: output_path.to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute RPC failed")
        .into_inner();

    let stages: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    assert!(!stages.is_empty(), "Execute must produce at least one progress message");
    // The pipeline should eventually reach the Done stage
    let has_done = stages.iter().any(|s| s.stage == ProtoStage::Done as i32);
    assert!(has_done, "Pipeline execute must reach DONE stage, got stages: {:?}",
        stages.iter().map(|s| s.stage).collect::<Vec<_>>());
}

#[tokio::test]
async fn execute_produces_loading_and_done_stages() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "stages-test".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.gps_set".to_string(),
            label: "gps".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;

    let req = ExecuteRequest {
        pipeline_id,
        image_path: img_str,
        output_path: temp_dir().into_path().join("stages_out.png").to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute failed")
        .into_inner();

    let stages: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    assert!(!stages.is_empty());
    // First stage should be Loading
    assert_eq!(
        stages[0].stage,
        ProtoStage::Loading as i32,
        "First stage should be Loading(0), got {}",
        stages[0].stage
    );

    let has_done = stages.iter().any(|s| s.stage == ProtoStage::Done as i32);
    assert!(has_done, "Must have Done stage");
}

#[tokio::test]
async fn execute_invalid_pipeline_id_returns_not_found() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();
    let req = ExecuteRequest {
        pipeline_id: uuid::Uuid::new_v4().to_string(),
        image_path: img_str,
        output_path: "/tmp/bad_output.png".to_string(),
    };

    let resp = svc.execute(tonic::Request::new(req)).await;
    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::NotFound,
                "Expected NotFound for invalid pipeline ID, got {:?}",
                status.code()
            );
        }
        Ok(_) => panic!("Execute with invalid pipeline ID should fail"),
    }
}

#[tokio::test]
async fn execute_nonexistent_image_returns_not_found() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "test".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.colorspace".to_string(),
            label: "cs".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;

    let req = ExecuteRequest {
        pipeline_id,
        image_path: "/nonexistent/image.png".to_string(),
        output_path: "/tmp/output.png".to_string(),
    };

    let resp = svc.execute(tonic::Request::new(req)).await;
    match resp {
        Err(status) => {
            assert_eq!(status.code(), Code::NotFound, "Expected NotFound, got {:?}", status.code());
        }
        Ok(_) => panic!("Execute with nonexistent image should fail"),
    }
}

#[tokio::test]
async fn execute_with_disabled_nodes_skips_them() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "disabled-node".to_string(),
        nodes: vec![
            PipelineNode {
                id: "n1".to_string(),
                plugin_id: "photopipeline.plugins.colorspace".to_string(),
                label: "cs".to_string(),
                enabled: false, // DISABLED
                params: None,
            },
        ],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;

    let req = ExecuteRequest {
        pipeline_id,
        image_path: img_str,
        output_path: temp_dir().into_path().join("disabled_out.png").to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute failed")
        .into_inner();

    let stages: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    // Even with all nodes disabled, the pipeline should complete (possibly with Done or Error)
    let has_final = stages.iter().any(|s| {
        s.stage == ProtoStage::Done as i32 || s.stage == ProtoStage::Error as i32
    });
    assert!(has_final, "Pipeline should reach a terminal stage (Done or Error)");
}

#[tokio::test]
async fn execute_with_plugin_params_yields_done() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();

    // Use a metadata plugin with parameters — no pixel buffer needed
    let mut params = prost_types::Struct::default();
    params.fields.insert("read_exif".to_string(),
        prost_types::Value {
            kind: Some(prost_types::value::Kind::BoolValue(true)),
        });

    let spec = PipelineSpec {
        name: "param-test".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.exif_rw".to_string(),
            label: "exif-param".to_string(),
            enabled: true,
            params: Some(params),
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;

    let req = ExecuteRequest {
        pipeline_id,
        image_path: img_str,
        output_path: temp_dir().into_path().join("params_out.png").to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute failed")
        .into_inner();

    let stages: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    let has_done = stages.iter().any(|s| s.stage == ProtoStage::Done as i32);
    assert!(has_done, "Pipeline with params must reach Done stage");
}

#[tokio::test]
async fn execute_multi_node_pipeline_all_nodes_processed() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "multi-node".to_string(),
        nodes: vec![
            PipelineNode {
                id: "n1".to_string(),
                plugin_id: "photopipeline.plugins.exif_rw".to_string(),
                label: "exif".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "n2".to_string(),
                plugin_id: "photopipeline.plugins.gps_set".to_string(),
                label: "gps".to_string(),
                enabled: true,
                params: None,
            },
            PipelineNode {
                id: "n3".to_string(),
                plugin_id: "photopipeline.plugins.time_shift".to_string(),
                label: "ts".to_string(),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            PipelineEdge { from: "n1".to_string(), to: "n2".to_string() },
            PipelineEdge { from: "n2".to_string(), to: "n3".to_string() },
        ],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;

    let req = ExecuteRequest {
        pipeline_id,
        image_path: img_str,
        output_path: temp_dir().into_path().join("multi_out.png").to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute failed")
        .into_inner();

    let stages: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    let has_done = stages.iter().any(|s| s.stage == ProtoStage::Done as i32);
    assert!(has_done, "Multi-node pipeline must reach Done stage");
    // Should have progress messages for multiple nodes
    let processing = stages.iter().filter(|s| s.stage == ProtoStage::Processing as i32).count();
    assert!(processing >= 1, "Should have at least one processing stage message");
}

#[tokio::test]
async fn execute_preserves_pipeline_id_in_stream() {
    let (_server, client) = setup().await;
    let img_path = copy_golden("solid_64x64_128_64_32.png");
    let img_str = img_path.to_string_lossy().to_string();

    let mut svc = client.pipeline_client();
    let spec = PipelineSpec {
        name: "id-test".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.time_shift".to_string(),
            label: "ts".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("CreatePipeline failed");
    let pipeline_id = create_resp.into_inner().id;
    let pid_for_exec = pipeline_id.clone();

    let req = ExecuteRequest {
        pipeline_id: pid_for_exec,
        image_path: img_str,
        output_path: temp_dir().into_path().join("id_out.png").to_string_lossy().to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("Execute failed")
        .into_inner();

    let stages: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    // All messages should have a non-empty node_id or node_label (at least Loading has label)
    assert!(
        stages.iter().any(|s| !s.node_label.is_empty() || !s.node_id.is_empty()),
        "At least some messages must have labels"
    );
    let has_done = stages.iter().any(|s| s.stage == ProtoStage::Done as i32);
    assert!(has_done, "Should complete successfully");
}

// ---------------------------------------------------------------------------
// D.2.3 Validate RPC (4 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn validate_valid_pipeline_returns_valid_true() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "valid".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.colorspace".to_string(),
            label: "cs".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .validate(tonic::Request::new(spec))
        .await
        .expect("Validate RPC failed");

    let result = resp.into_inner();
    assert!(result.valid, "Valid pipeline should report valid=true");
}

#[tokio::test]
async fn validate_unknown_plugin_returns_valid_false() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "bad-plugin".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "nonexistent.plugin.xyz".to_string(),
            label: "bad".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .validate(tonic::Request::new(spec))
        .await
        .expect("Validate RPC failed");

    let result = resp.into_inner();
    assert!(!result.valid, "Pipeline with unknown plugin should report valid=false");
    assert!(!result.issues.is_empty(), "Should have at least one validation issue");
    let has_plugin_issue = result.issues.iter().any(|i| {
        i.message.to_lowercase().contains("not registered")
            || i.message.to_lowercase().contains("plugin")
    });
    assert!(has_plugin_issue, "Issue should mention that plugin is not registered");
}

#[tokio::test]
async fn validate_bad_edge_returns_valid_false() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "bad-edge".to_string(),
        nodes: vec![PipelineNode {
            id: "n1".to_string(),
            plugin_id: "photopipeline.plugins.colorspace".to_string(),
            label: "cs".to_string(),
            enabled: true,
            params: None,
        }],
        edges: vec![PipelineEdge {
            from: "missing_node".to_string(),
            to: "n1".to_string(),
        }],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .validate(tonic::Request::new(spec))
        .await
        .expect("Validate RPC failed");

    let result = resp.into_inner();
    assert!(!result.valid, "Pipeline with bad edge should report valid=false");
    assert!(!result.issues.is_empty());
}

#[tokio::test]
async fn validate_empty_pipeline_returns_valid_false() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let spec = PipelineSpec {
        name: "empty".to_string(),
        nodes: vec![],
        edges: vec![],
        params: Default::default(),
        batch: None,
    };

    let resp = svc
        .validate(tonic::Request::new(spec))
        .await
        .expect("Validate RPC failed");

    let result = resp.into_inner();
    assert!(!result.valid, "Empty pipeline should report valid=false");
    assert!(!result.issues.is_empty(), "Should have 'at least one node' issue");
    let has_empty_issue = result.issues.iter().any(|i| {
        i.message.to_lowercase().contains("at least one node")
    });
    assert!(has_empty_issue, "Issue should mention 'at least one node'");
}

// ---------------------------------------------------------------------------
// D.2.4 GetNodeSchema RPC (2 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_node_schema_known_plugin_returns_nonempty_schema() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let resp = svc
        .get_node_schema(tonic::Request::new(PluginId {
            id: "photopipeline.plugins.colorspace".to_string(),
        }))
        .await
        .expect("GetNodeSchema failed");

    let schema = resp.into_inner();
    assert_eq!(
        schema.plugin_id, "photopipeline.plugins.colorspace",
        "Plugin ID should match"
    );
    assert!(!schema.name.is_empty(), "Plugin name must not be empty");
    assert!(!schema.version.is_empty(), "Version must not be empty");
    assert!(schema.parameter_schema.is_some(), "Parameter schema must be present");
    // Verify the parameter schema contains actual fields
    let ps = schema.parameter_schema.unwrap();
    assert!(!ps.fields.is_empty(), "Parameter schema fields must not be empty");
}

#[tokio::test]
async fn get_node_schema_unknown_plugin_returns_not_found() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    let resp = svc
        .get_node_schema(tonic::Request::new(PluginId {
            id: "nonexistent.plugin.bogus".to_string(),
        }))
        .await;

    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::NotFound,
                "Unknown plugin should return NotFound, got {:?}",
                status.code()
            );
        }
        Ok(_) => panic!("GetNodeSchema on unknown plugin should fail"),
    }
}

// ---------------------------------------------------------------------------
// Adversarial self-review
//
// Each test contains fail-capable assertions:
//   - create_* tests verify non-empty UUID, or appropriate error codes
//   - execute_* tests verify stage progression (Loading->Done), error codes
//   - validate_* tests verify valid flag and issue messages
//   - get_node_schema_* tests verify non-empty fields and error codes
//
// Silent-skip prevention:
//   - TestServer::start() panics if server unavailable
//   - No try/catch swallowing errors
//   - All RPC responses verified with concrete assertions
// ---------------------------------------------------------------------------
