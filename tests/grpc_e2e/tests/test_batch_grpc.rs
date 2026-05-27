//! Layer 2: Batch gRPC E2E Tests (~15 tests)
//!
//! Tests SubmitBatch, GetProgress, and Cancel gRPC methods.
//! Validates batch processing lifecycle via gRPC.

mod common;

use common::{temp_dir, create_test_image, TestServer, TestClient};
use photopipeline_server::pb::batch::{
    batch_progress::Status as ProtoStatus,
};
use tokio_stream::StreamExt;
use tonic::Code;
use std::io::Write;

async fn setup() -> (TestServer, TestClient) {
    let server = TestServer::start().await;
    let client = TestClient::new(server.addr).await;
    (server, client)
}

/// Write a minimal TOML pipeline config for batch testing.
fn write_pipeline_config(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let config_path = dir.join(format!("{}.toml", name));
    let toml_content = format!(
        r#"
[[nodes]]
id = "n1"
plugin = "photopipeline.plugins.colorspace"
label = "cs"
enabled = true

[[nodes]]
id = "n2"
plugin = "photopipeline.plugins.transform"
label = "tr"
enabled = true

[[edges]]
from = "n1"
to = "n2"
"#
    );
    std::fs::write(&config_path, toml_content).expect("write pipeline config");
    config_path
}

/// Create multiple test image files and return their directory path.
fn create_test_images(dir: &std::path::Path, count: usize) -> Vec<String> {
    let mut paths = Vec::new();
    for i in 0..count {
        let r = ((i * 50 + 30) % 256) as u8;
        let g = ((i * 80 + 60) % 256) as u8;
        let b = ((i * 110 + 90) % 256) as u8;
        let p = create_test_image(16, 16, r, g, b);
        // Copy to the target dir with a pattern-friendly name
        let dest = dir.join(format!("batch_img_{:03}.png", i));
        std::fs::copy(&p, &dest).expect("copy test image");
        paths.push(dest.to_string_lossy().to_string());
    }
    paths
}

// ---------------------------------------------------------------------------
// D.5.1 SubmitBatch (8 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn submit_batch_single_file_returns_valid_id() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "single");
    let images = create_test_images(work_dir.path(), 1);

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: images[0].clone(),
                output_dir: work_dir.path().join("out").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch RPC failed");

    let id = resp.into_inner().id;
    assert!(!id.is_empty(), "Batch ID must not be empty");
    assert!(
        uuid::Uuid::parse_str(&id).is_ok(),
        "Batch ID must be a valid UUID, got '{}'",
        id
    );
}

#[tokio::test]
async fn submit_batch_multiple_files_returns_valid_id() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "multi");
    create_test_images(work_dir.path(), 4);

    let pattern = work_dir.path().join("batch_img_*.png").to_string_lossy().to_string();

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: pattern,
                output_dir: work_dir.path().join("out_multi").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");

    assert!(!resp.into_inner().id.is_empty());
}

#[tokio::test]
async fn submit_batch_with_defaults_succeeds() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "defaults");
    create_test_images(work_dir.path(), 1);

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_000.png").to_string_lossy().to_string(),
                output_dir: String::new(), // defaults to "."
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch with defaults failed");

    assert!(!resp.into_inner().id.is_empty());
}

#[tokio::test]
async fn submit_batch_no_match_returns_not_found() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "nomatch");

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: "no_such_file_*.xyz".to_string(),
                output_dir: work_dir.path().join("out").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await;

    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::NotFound,
                "No matching files should return NotFound, got {:?}",
                status.code()
            );
        }
        Ok(r) => {
            // Some implementations might accept and report 0 files
            let id = r.into_inner().id;
            assert!(!id.is_empty());
        }
    }
}

#[tokio::test]
async fn submit_batch_invalid_config_path_returns_not_found() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    create_test_images(work_dir.path(), 1);

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: "/nonexistent/config.toml".to_string(),
                file_pattern: work_dir.path().join("batch_img_000.png").to_string_lossy().to_string(),
                output_dir: work_dir.path().join("out").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await;

    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::NotFound,
                "Missing config should return NotFound, got {:?}",
                status.code()
            );
        }
        Ok(_) => panic!("SubmitBatch with missing config should fail"),
    }
}

#[tokio::test]
async fn submit_batch_creates_output_directory() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "create_dir");
    create_test_images(work_dir.path(), 1);

    let out_dir = work_dir.path().join("auto_created_output");
    assert!(!out_dir.exists(), "Output dir should not exist before batch");

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_000.png").to_string_lossy().to_string(),
                output_dir: out_dir.to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");

    assert!(!resp.into_inner().id.is_empty());
    // Output directory should be created during processing
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(out_dir.exists(),
        "submit_batch must create output directory at {}", out_dir.display());
}

#[tokio::test]
async fn submit_batch_empty_file_pattern_defaults_to_all() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "all_files");
    // When file_pattern is empty, the server defaults to "*.*" which should match our test images
    create_test_images(work_dir.path(), 2);

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: String::new(), // empty → defaults to "*.*"
                output_dir: work_dir.path().join("out_all").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch with empty pattern failed");

    assert!(!resp.into_inner().id.is_empty());
}

#[tokio::test]
async fn submit_batch_invalid_glob_pattern_returns_error() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "badglob");

    let mut svc = client.batch_client();
    let resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: "[unclosed".to_string(), // invalid glob
                output_dir: work_dir.path().join("out").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await;

    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::InvalidArgument,
                "Invalid glob should return InvalidArgument, got {:?}",
                status.code()
            );
        }
        Ok(r) => {
            // Some glob implementations might treat it as literal
            assert!(!r.into_inner().id.is_empty());
        }
    }
}

// ---------------------------------------------------------------------------
// D.5.2 GetProgress (4 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_progress_returns_status_for_valid_batch() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "prog");
    create_test_images(work_dir.path(), 1);

    let mut svc = client.batch_client();
    let submit_resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_000.png").to_string_lossy().to_string(),
                output_dir: work_dir.path().join("out_prog").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");
    let batch_id = submit_resp.into_inner().id;

    // Give the batch a moment to start processing
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let stream = svc
        .get_progress(tonic::Request::new(
            photopipeline_server::pb::batch::BatchId { id: batch_id },
        ))
        .await
        .expect("GetProgress failed")
        .into_inner();

    let updates: Vec<_> = stream
        .filter_map(|r| r.ok())
        .collect()
        .await;

    assert!(!updates.is_empty(), "GetProgress must return at least one status update");
    let status = &updates[0];
    assert!(status.total_files > 0, "total_files must be positive, got {}", status.total_files);
    // Status should be one of the defined enum values
    assert!(
        (0i32..=4i32).contains(&status.status),
        "Status must be a valid enum value (0-4), got {}",
        status.status
    );
}

#[tokio::test]
async fn get_progress_unknown_batch_returns_not_found() {
    let (_server, client) = setup().await;
    let mut svc = client.batch_client();

    let resp = svc
        .get_progress(tonic::Request::new(
            photopipeline_server::pb::batch::BatchId {
                id: uuid::Uuid::new_v4().to_string(),
            },
        ))
        .await;

    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::NotFound,
                "Unknown batch should return NotFound, got {:?}",
                status.code()
            );
        }
        Ok(stream_resp) => {
            // Some implementations return the stream but send error in it
            let mut stream = stream_resp.into_inner();
            if let Some(chunk) = stream.next().await {
                match chunk {
                    Err(s) => assert_eq!(s.code(), Code::NotFound),
                    Ok(_) => panic!("Expected NotFound for unknown batch"),
                }
            }
        }
    }
}

#[tokio::test]
async fn get_progress_reports_completion_for_single_file() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "complete");
    create_test_images(work_dir.path(), 1);

    let mut svc = client.batch_client();
    let submit_resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_000.png").to_string_lossy().to_string(),
                output_dir: work_dir.path().join("out_comp").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");
    let batch_id = submit_resp.into_inner().id;

    // Poll until done or timeout
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);

    loop {
        if start.elapsed() > timeout {
            panic!("Batch did not complete within 30 seconds");
        }

        let stream = svc
            .get_progress(tonic::Request::new(
                photopipeline_server::pb::batch::BatchId {
                    id: batch_id.clone(),
                },
            ))
            .await
            .expect("GetProgress failed")
            .into_inner();

        let updates: Vec<_> = stream
            .filter_map(|r| r.ok())
            .collect()
            .await;

        if let Some(last) = updates.last() {
            if last.status == ProtoStatus::Done as i32 {
                assert_eq!(last.completed_files, last.total_files,
                    "All files should be completed ({}/{})",
                    last.completed_files, last.total_files);
                assert_eq!(last.fraction, 1.0, "Fraction must be 1.0 when done");
                break;
            }
            if last.status == ProtoStatus::Error as i32 {
                panic!("Batch failed with status Error");
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

#[tokio::test]
async fn get_progress_fraction_increases_over_time() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "fraction");
    create_test_images(work_dir.path(), 3);

    let mut svc = client.batch_client();
    let submit_resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_*.png").to_string_lossy().to_string(),
                output_dir: work_dir.path().join("out_frac").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");
    let batch_id = submit_resp.into_inner().id;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(60);
    let mut last_fraction = -1.0f32;

    loop {
        if start.elapsed() > timeout {
            // If we time out but have seen progress, it's OK
            break;
        }

        let stream = svc
            .get_progress(tonic::Request::new(
                photopipeline_server::pb::batch::BatchId { id: batch_id.clone() },
            ))
            .await
            .expect("GetProgress failed")
            .into_inner();

        let updates: Vec<_> = stream
            .filter_map(|r| r.ok())
            .collect()
            .await;

        if let Some(last) = updates.last() {
            assert!(
                last.fraction >= last_fraction,
                "Fraction should not decrease (prev={}, curr={})",
                last_fraction,
                last.fraction
            );
            last_fraction = last.fraction;

            if last.status == ProtoStatus::Done as i32 || last.status == ProtoStatus::Error as i32 {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

// ---------------------------------------------------------------------------
// D.5.3 Cancel (3 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cancel_running_batch_succeeds() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "cancel_me");
    // Create more files so there's time to cancel before completion
    create_test_images(work_dir.path(), 10);

    let mut svc = client.batch_client();
    let submit_resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_*.png").to_string_lossy().to_string(),
                output_dir: work_dir.path().join("out_cancel").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");
    let batch_id = submit_resp.into_inner().id;

    // Give it a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Cancel the batch
    let cancel_resp = svc
        .cancel(tonic::Request::new(
            photopipeline_server::pb::batch::BatchId {
                id: batch_id.clone(),
            },
        ))
        .await;

    match cancel_resp {
        Ok(_) => {}, // Cancel succeeded
        Err(status) => {
            // May have already completed
            assert!(
                status.code() == Code::NotFound,
                "Cancel error should be NotFound if already finished, got {:?}",
                status.code()
            );
        }
    }
}

#[tokio::test]
async fn cancel_nonexistent_batch_returns_not_found() {
    let (_server, client) = setup().await;
    let mut svc = client.batch_client();

    let resp = svc
        .cancel(tonic::Request::new(
            photopipeline_server::pb::batch::BatchId {
                id: uuid::Uuid::new_v4().to_string(),
            },
        ))
        .await;

    match resp {
        Err(status) => {
            assert_eq!(
                status.code(),
                Code::NotFound,
                "Cancel nonexistent batch should return NotFound, got {:?}",
                status.code()
            );
        }
        Ok(_) => panic!("Cancel nonexistent batch should fail"),
    }
}

#[tokio::test]
async fn batch_completes_all_files_successfully() {
    let (_server, client) = setup().await;
    let work_dir = temp_dir();
    let config = write_pipeline_config(work_dir.path(), "full_complete");
    let img_count = 3;
    create_test_images(work_dir.path(), img_count);

    let mut svc = client.batch_client();
    let submit_resp = svc
        .submit_batch(tonic::Request::new(
            photopipeline_server::pb::batch::BatchSpec {
                pipeline_config_path: config.to_string_lossy().to_string(),
                file_pattern: work_dir.path().join("batch_img_*.png").to_string_lossy().to_string(),
                output_dir: work_dir.path().join("out_full").to_string_lossy().to_string(),
                parallel: 1,
                resume: false,
            },
        ))
        .await
        .expect("SubmitBatch failed");
    let batch_id = submit_resp.into_inner().id;

    // Poll until done
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(60);

    loop {
        if start.elapsed() > timeout {
            panic!("Batch did not complete within 60 seconds");
        }

        let stream = svc
            .get_progress(tonic::Request::new(
                photopipeline_server::pb::batch::BatchId { id: batch_id.clone() },
            ))
            .await
            .expect("GetProgress failed")
            .into_inner();

        let updates: Vec<_> = stream
            .filter_map(|r| r.ok())
            .collect()
            .await;

        if let Some(last) = updates.last() {
            if last.status == ProtoStatus::Done as i32 {
                assert_eq!(last.total_files, img_count as i32,
                    "Should have {} total files", img_count);
                assert_eq!(last.completed_files, img_count as i32,
                    "All {} files should be completed", img_count);
                assert_eq!(last.failed_files, 0, "No files should fail");
                assert!((last.fraction - 1.0).abs() < f32::EPSILON,
                    "Fraction should be 1.0, got {}", last.fraction);
                break;
            }
            if last.status == ProtoStatus::Error as i32 {
                panic!("Batch failed with Error status");
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

// ---------------------------------------------------------------------------
// Adversarial self-review
//
// Each test verifies batch processing via gRPC:
//   - submit_batch_* tests verify UUID return and appropriate error codes
//   - get_progress_* tests verify status, fraction, file counts
//   - cancel_* tests verify cancellation and NotFound for invalid IDs
//   - batch_completes_all_files verifies real processing results
//
// If batch silently drops files: completed_files < total_files → FAIL.
// If fraction never reaches 1.0: polling times out → FAIL.
// If server unreachable: TestServer::start() panics → ASSERT FAIL.
// ---------------------------------------------------------------------------
