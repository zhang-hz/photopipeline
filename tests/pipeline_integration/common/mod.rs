// Common test utilities for pipeline integration tests.
// Shared across all test_*.rs files via #[path = "common/mod.rs"] mod common;
// NOTE: Each test binary compiles its own copy, so some items may appear unused
// in certain test files. #[allow(dead_code)] suppresses these false positives.

#![allow(dead_code)]

pub mod image_fixtures;

use photopipeline_core::{
    ColorSpace, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat,
};
use photopipeline_engine::{NodeExecutor, ParameterResolver, PipelineTemplate};
use photopipeline_plugin::registry::Registry;
use photopipeline_plugin::ProgressSink;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Create a registry with all 14 plugins registered.
pub fn make_registry() -> Arc<Registry> {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    reg
}

/// Create a default ParameterResolver.
pub fn make_resolver() -> Arc<ParameterResolver> {
    Arc::new(ParameterResolver::new())
}

/// Create a fresh ImageInfo for a test image.
pub fn make_image_info(width: u32, height: u32, format: ImageFormat) -> ImageInfo {
    let id = Uuid::new_v4();
    ImageInfo {
        id,
        path: format!("/tmp/test_{}.png", id),
        filename: format!("test_{}.png", id),
        format,
        width,
        height,
        file_size_bytes: 0,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

/// Create default metadata.
pub fn make_metadata() -> Metadata {
    Metadata::default()
}

/// A no-op progress sink for tests.
pub struct NoopProgress;

impl ProgressSink for NoopProgress {
    fn set_progress(&self, _fraction: f32, _message: &str) {}
    fn is_canceled(&self) -> bool {
        false
    }
}

/// Build a HashMap of string parameters for TemplateNode.
pub fn params(pairs: &[(&str, serde_json::Value)]) -> HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

/// Execute a pipeline template on a given input buffer and return the output buffer.
/// Panics if execution fails.
pub fn execute_pipeline(
    template: &PipelineTemplate,
    input: &PixelBuffer,
) -> PixelBuffer {
    let reg = make_registry();
    let resolver = make_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let graph = template.clone().into_graph();
    let image_info = make_image_info(input.width, input.height, ImageFormat::PNG);
    let metadata = make_metadata();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(input.clone()),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    match result {
        Ok(exec_result) => {
            exec_result
                .buffer
                .expect("pipeline produced no output buffer")
        }
        Err(e) => {
            panic!("Pipeline execution failed: {:?}", e);
        }
    }
}

/// Execute a pipeline and expect it to return an error.
pub fn execute_pipeline_expect_err(
    template: &PipelineTemplate,
    input: &PixelBuffer,
) -> String {
    let reg = make_registry();
    let resolver = make_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let graph = template.clone().into_graph();
    let image_info = make_image_info(input.width, input.height, ImageFormat::PNG);
    let metadata = make_metadata();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(input.clone()),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    match result {
        Ok(_) => panic!("Expected pipeline execution to fail, but it succeeded"),
        Err(e) => format!("{:?}", e),
    }
}

/// Execute a pipeline and return the raw execution result (may contain no buffer).
pub fn execute_pipeline_raw(
    template: &PipelineTemplate,
    input: &PixelBuffer,
) -> photopipeline_core::PluginResult<photopipeline_engine::ExecutionResult> {
    let reg = make_registry();
    let resolver = make_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let graph = template.clone().into_graph();
    let image_info = make_image_info(input.width, input.height, ImageFormat::PNG);
    let metadata = make_metadata();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(input.clone()),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    })
}
