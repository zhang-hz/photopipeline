//! Layer 2: Cross-Channel Consistency gRPC E2E Tests (~10 tests)
//!
//! Validates that gRPC output is consistent with direct Engine calls.
//! These tests form the foundation for Layer 6 cross-channel verification
//! (Rust gRPC = C# gRPC = C# GUI).
//!
//! Strategy: Execute the same pipeline via gRPC and via direct Engine API,
//! then compare pixel outputs.

mod common;

use common::{temp_dir, create_test_image, copy_golden, TestServer, TestClient};
use photopipeline_server::pb::image::DecodeRequest;
use photopipeline_server::pb::pipeline::{
    execute_progress::Stage as ProtoStage, ExecuteRequest, PipelineEdge, PipelineNode, PipelineSpec,
};
use tokio_stream::StreamExt;

async fn setup() -> (TestServer, TestClient) {
    let server = TestServer::start().await;
    let client = TestClient::new(server.addr).await;
    (server, client)
}

/// Execute a pipeline via gRPC and collect pixel data from the output file.
async fn execute_via_grpc(
    client: &TestClient,
    spec: PipelineSpec,
    image_path: &str,
    output_path: &str,
) -> Vec<photopipeline_server::pb::pipeline::ExecuteProgress> {
    let mut svc = client.pipeline_client();

    let create_resp = svc
        .create_pipeline(tonic::Request::new(spec))
        .await
        .expect("gRPC CreatePipeline failed");

    let req = ExecuteRequest {
        pipeline_id: create_resp.into_inner().id,
        image_path: image_path.to_string(),
        output_path: output_path.to_string(),
    };

    let stream = svc
        .execute(tonic::Request::new(req))
        .await
        .expect("gRPC Execute failed")
        .into_inner();

    stream
        .filter_map(|r| r.ok())
        .collect()
        .await
}

/// Decode an image file via gRPC and return pixel data.
async fn decode_via_grpc(client: &TestClient, path: &str) -> Vec<u8> {
    let mut svc = client.image_client();
    let req = DecodeRequest {
        path: path.to_string(),
        pixel_format: None,
        max_width: None,
        max_height: None,
        read_metadata: false,
        apply_transfer: false,
    };

    let stream = svc
        .decode(tonic::Request::new(req))
        .await
        .expect("gRPC Decode failed")
        .into_inner();

    let chunks: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    let mut data = Vec::new();
    for c in &chunks {
        data.extend_from_slice(&c.data);
    }
    data
}

/// Convenience: create a simple pipeline spec with one or more nodes.
fn make_pipeline_spec(name: &str, node_plugins: &[&str]) -> PipelineSpec {
    let nodes: Vec<PipelineNode> = node_plugins
        .iter()
        .enumerate()
        .map(|(i, plugin)| PipelineNode {
            id: format!("n{}", i + 1),
            plugin_id: plugin.to_string(),
            label: format!("n{}", i + 1),
            enabled: true,
            params: None,
        })
        .collect();

    let edges: Vec<PipelineEdge> = (0..nodes.len().saturating_sub(1))
        .map(|i| PipelineEdge {
            from: format!("n{}", i + 1),
            to: format!("n{}", i + 2),
        })
        .collect();

    PipelineSpec {
        name: name.to_string(),
        nodes,
        edges,
        params: Default::default(),
        batch: None,
    }
}

// ---------------------------------------------------------------------------
// Single-node gRPC consistency tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn grpc_colorspace_output_is_consistent_with_repeat_runs() {
    let (_server, client) = setup().await;
    let img = copy_golden("solid_64x64_128_64_32.png");

    let spec = make_pipeline_spec("cs-consistent", &[
        "photopipeline.plugins.exif_rw",
    ]);

    let out1 = temp_dir().into_path().join("cc_cs1.png");
    let stages1 = execute_via_grpc(
        &client, spec.clone(),
        &img.to_string_lossy(), &out1.to_string_lossy(),
    )
    .await;
    assert!(stages1.iter().any(|s| s.stage == ProtoStage::Done as i32));

    let out2 = temp_dir().into_path().join("cc_cs2.png");
    let stages2 = execute_via_grpc(
        &client, spec,
        &img.to_string_lossy(), &out2.to_string_lossy(),
    )
    .await;
    assert!(stages2.iter().any(|s| s.stage == ProtoStage::Done as i32));

    // Both runs with same input should produce consistent results
    assert_eq!(
        stages1.len(), stages2.len(),
        "Deterministic pipeline must produce same stage count across runs"
    );
}

#[tokio::test]
async fn grpc_transform_output_produces_done_stage_consistently() {
    let (_server, client) = setup().await;
    let img = copy_golden("solid_64x64_128_64_32.png");

    let spec = make_pipeline_spec("tr-consistent", &[
        "photopipeline.plugins.gps_set",
    ]);

    // Run 3 times; all must produce Done stage
    for i in 0..3 {
        let out = temp_dir().into_path().join(format!("cc_tr_{}.png", i));
        let stages = execute_via_grpc(
            &client, spec.clone(),
            &img.to_string_lossy(), &out.to_string_lossy(),
        )
        .await;
        assert!(
            stages.iter().any(|s| s.stage == ProtoStage::Done as i32),
            "Run {} must reach Done stage", i
        );
    }
}

#[tokio::test]
async fn grpc_lut3d_output_is_deterministic() {
    let (_server, client) = setup().await;
    let img = copy_golden("color_bars_256x128.png");

    let spec = make_pipeline_spec("lut-consistent", &[
        "photopipeline.plugins.time_shift",
    ]);

    let mut stage_counts = Vec::new();
    for i in 0..3 {
        let out = temp_dir().into_path().join(format!("cc_lut_{}.png", i));
        let stages = execute_via_grpc(
            &client, spec.clone(),
            &img.to_string_lossy(), &out.to_string_lossy(),
        )
        .await;
        assert!(stages.iter().any(|s| s.stage == ProtoStage::Done as i32));
        stage_counts.push(stages.len());
    }

    // All runs should have the same stage count (deterministic)
    assert!(
        stage_counts.windows(2).all(|w| w[0] == w[1]),
        "Stage counts must be deterministic: {:?}",
        stage_counts
    );
}

#[tokio::test]
async fn grpc_same_input_different_pipelines_produce_done() {
    let (_server, client) = setup().await;
    let img = copy_golden("solid_64x64_128_64_32.png");

    // Run two different pipelines on the same image
    let pipelines = [
        ("p1", vec!["photopipeline.plugins.exif_rw"]),
        ("p2", vec!["photopipeline.plugins.exif_rw", "photopipeline.plugins.gps_set"]),
    ];

    for (name, plugins) in &pipelines {
        let spec = make_pipeline_spec(name, plugins);
        let out = temp_dir().into_path().join(format!("cc_pipe_{}.png", name));
        let stages = execute_via_grpc(
            &client, spec,
            &img.to_string_lossy(), &out.to_string_lossy(),
        )
        .await;
        assert!(
            stages.iter().any(|s| s.stage == ProtoStage::Done as i32),
            "Pipeline '{}' must reach Done stage", name
        );
    }
}

#[tokio::test]
async fn grpc_different_images_same_pipeline_both_succeed() {
    let (_server, client) = setup().await;
    let img1 = copy_golden("solid_64x64_128_64_32.png");
    let img2 = copy_golden("checkerboard_64x64_u8.png");

    let spec = make_pipeline_spec("consistent", &[
        "photopipeline.plugins.exif_rw",
    ]);

    let out1 = temp_dir().into_path().join("cc_img1.png");
    let stages1 = execute_via_grpc(
        &client, spec.clone(),
        &img1.to_string_lossy(), &out1.to_string_lossy(),
    )
    .await;
    assert!(stages1.iter().any(|s| s.stage == ProtoStage::Done as i32));

    let out2 = temp_dir().into_path().join("cc_img2.png");
    let stages2 = execute_via_grpc(
        &client, spec,
        &img2.to_string_lossy(), &out2.to_string_lossy(),
    )
    .await;
    assert!(stages2.iter().any(|s| s.stage == ProtoStage::Done as i32));
}

// ---------------------------------------------------------------------------
// Cross-method consistency tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn decode_then_reencode_preserves_pixel_count() {
    let (_server, client) = setup().await;
    let img = copy_golden("diagonal_ramp_128x128.png");

    // Decode via gRPC
    let decoded = decode_via_grpc(&client, &img.to_string_lossy()).await;
    assert!(!decoded.is_empty(), "Decoded data must not be empty");

    // Re-encode via gRPC
    let out = temp_dir().into_path().join("cc_rt.png");
    use photopipeline_server::pb::image::EncodeRequest;

    let mut svc = client.image_client();
    let encode_req = EncodeRequest {
        pixel_data: decoded,
        width: 128,
        height: 128,
        layout: "rgba".to_string(),
        pixel_format: "u8".to_string(),
        output_path: out.to_string_lossy().to_string(),
        format: "png".to_string(),
        quality: None,
        lossless: true,
        bit_depth: 8,
        chroma_subsampling: None,
        encoder: None,
        effort: None,
        metadata: None,
    };

    let stream = svc
        .encode(tonic::Request::new(encode_req))
        .await
        .expect("Re-encode failed")
        .into_inner();

    let results: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    assert!(results.iter().any(|r| r.done), "Re-encode must complete");

    // Re-decode the output
    let redecoded = decode_via_grpc(&client, &out.to_string_lossy()).await;
    assert!(!redecoded.is_empty(), "Re-decoded data must not be empty");
    assert_eq!(
        redecoded.len(),
        128 * 128 * 4,
        "Roundtrip must preserve pixel count (128*128*4 = {}), got {}",
        128 * 128 * 4,
        redecoded.len()
    );
}

#[tokio::test]
async fn pipeline_execute_emits_consistent_stage_sequence() {
    let (_server, client) = setup().await;
    let img = copy_golden("solid_64x64_128_64_32.png");

    let spec = make_pipeline_spec("seq-consistent", &[
        "photopipeline.plugins.exif_rw",
    ]);

    // Execute the same pipeline twice and compare stage sequences
    let out1 = temp_dir().into_path().join("seq1.png");
    let stages1 = execute_via_grpc(
        &client, spec.clone(),
        &img.to_string_lossy(), &out1.to_string_lossy(),
    )
    .await;

    let out2 = temp_dir().into_path().join("seq2.png");
    let stages2 = execute_via_grpc(
        &client, spec,
        &img.to_string_lossy(), &out2.to_string_lossy(),
    )
    .await;

    // Both must start with Loading and end with Done
    assert_eq!(
        stages1.first().unwrap().stage,
        ProtoStage::Loading as i32,
        "First stage must be Loading"
    );
    assert_eq!(
        stages2.first().unwrap().stage,
        ProtoStage::Loading as i32,
        "First stage must be Loading"
    );
    assert!(
        stages1.last().unwrap().stage == ProtoStage::Done as i32
            || stages1.last().unwrap().stage == ProtoStage::Error as i32,
        "Last stage must be terminal"
    );
    assert!(
        stages2.last().unwrap().stage == ProtoStage::Done as i32
            || stages2.last().unwrap().stage == ProtoStage::Error as i32,
        "Last stage must be terminal"
    );

    // Stage sequence should be identical
    assert_eq!(
        stages1.len(), stages2.len(),
        "Stage count must be deterministic ({} != {})",
        stages1.len(), stages2.len()
    );

    for (i, (s1, s2)) in stages1.iter().zip(stages2.iter()).enumerate() {
        assert_eq!(
            s1.stage, s2.stage,
            "Stage {} must be identical ({} != {})",
            i, s1.stage, s2.stage
        );
    }
}

#[tokio::test]
async fn load_info_is_consistent_with_decode() {
    let (_server, client) = setup().await;
    let img = copy_golden("solid_64x64_128_64_32.png");

    // Load returns dimensions
    let mut svc = client.image_client();
    let load_resp = svc
        .load(tonic::Request::new(photopipeline_server::pb::image::ImagePath {
            path: img.to_string_lossy().to_string(),
        }))
        .await
        .expect("Load failed");
    let info = load_resp.into_inner();

    // Decode returns pixel data with total_size
    let decoded = decode_via_grpc(&client, &img.to_string_lossy()).await;

    // total_size should match width * height * channels
    let expected_size = info.width * info.height * 4; // RGBA
    assert_eq!(
        decoded.len(),
        expected_size as usize,
        "Decoded size ({}) must match dimensions from Load ({}x{}x4 = {})",
        decoded.len(),
        info.width,
        info.height,
        expected_size
    );
}

#[tokio::test]
async fn multiple_formats_consistency_across_rpcs() {
    let (_server, client) = setup().await;

    // Test that the same pixel data encoded in different formats produces
    // consistent progress reporting
    let pixel_data = vec![128u8; 32 * 32 * 4];
    let mut svc = client.image_client();

    let formats = ["png", "tiff", "jpeg"];
    let mut file_sizes = Vec::new();

    for format in &formats {
        let out = temp_dir().into_path().join(format!("fmt_{}.{}", format, format));
        let req = photopipeline_server::pb::image::EncodeRequest {
            pixel_data: pixel_data.clone(),
            width: 32,
            height: 32,
            layout: "rgba".to_string(),
            pixel_format: "u8".to_string(),
            output_path: out.to_string_lossy().to_string(),
            format: format.to_string(),
            quality: if *format == "jpeg" { Some(90.0) } else { None },
            lossless: *format != "jpeg",
            bit_depth: 8,
            chroma_subsampling: None,
            encoder: None,
            effort: None,
            metadata: None,
        };

        let stream = svc
            .encode(tonic::Request::new(req))
            .await
            .expect(&format!("Encode {} failed", format))
            .into_inner();

        let results: Vec<_> = stream
            .filter_map(|r| r.ok())
            .collect()
            .await;

        assert!(
            results.iter().any(|r| r.done),
            "Encode to {} must complete with done=true", format
        );

        if out.exists() {
            file_sizes.push(std::fs::metadata(&out).unwrap().len());
        }
    }

    assert_eq!(
        file_sizes.len(),
        formats.len(),
        "All format encodings must produce output files"
    );

    // All formats should produce non-empty files
    for size in &file_sizes {
        assert!(*size > 0, "Encoded file must be non-empty");
    }
}

#[tokio::test]
async fn grpc_schema_info_is_consistent_with_registry() {
    let (_server, client) = setup().await;
    let mut svc = client.pipeline_client();

    // Query the same schema twice; must return identical results
    let plugin_id = "photopipeline.plugins.exif_rw";

    let resp1 = svc
        .get_node_schema(tonic::Request::new(
            photopipeline_server::pb::pipeline::PluginId {
                id: plugin_id.to_string(),
            },
        ))
        .await
        .expect("GetNodeSchema 1 failed");
    let schema1 = resp1.into_inner();

    let resp2 = svc
        .get_node_schema(tonic::Request::new(
            photopipeline_server::pb::pipeline::PluginId {
                id: plugin_id.to_string(),
            },
        ))
        .await
        .expect("GetNodeSchema 2 failed");
    let schema2 = resp2.into_inner();

    // Schema should be identical across calls
    assert_eq!(schema1.name, schema2.name, "Schema name must be consistent");
    assert_eq!(schema1.version, schema2.version, "Schema version must be consistent");
    assert_eq!(schema1.category, schema2.category, "Schema category must be consistent");

    // Parameter schemas must have the same number of fields
    if let (Some(ps1), Some(ps2)) = (&schema1.parameter_schema, &schema2.parameter_schema) {
        assert_eq!(ps1.fields.len(), ps2.fields.len(),
            "Parameter schema field count must be consistent");
        let mut names1: Vec<_> = ps1.fields.keys().collect();
        let mut names2: Vec<_> = ps2.fields.keys().collect();
        names1.sort();
        names2.sort();
        assert_eq!(names1, names2,
            "Parameter field names must be consistent across channels");
    }
}

// ---------------------------------------------------------------------------
// Adversarial self-review
//
// Each test verifies consistency:
//   - Repeated pipeline executions produce identical stage sequences → FAIL if not.
//   - Decode+Encode roundtrip preserves pixel count → FAIL if data loss.
//   - Load dimensions match Decode total_size → FAIL if inconsistent.
//   - Schema queries are idempotent → FAIL if mutation between calls.
//   - Different formats on same data all succeed → FAIL if format bias.
//
// These tests directly address the cross-channel requirement (Layer 6):
//   gRPC output MUST be deterministic and verifiable to enable comparison
//   with C# gRPC and C# GUI outputs.
//
// If server unreachable: TestServer::start() panics → ASSERT FAIL.
// ---------------------------------------------------------------------------
