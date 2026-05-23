use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use photopipeline_core::{
    ImageInfo, Metadata, NodeId, PixelBuffer, PluginError, PluginResult, ProcessingStats,
};
use photopipeline_plugin::{ProgressSink, Registry};

use crate::graph::PipelineGraph;
use crate::params::ParameterResolver;

pub struct NodeExecutor {
    pub registry: Arc<Registry>,
    pub resolver: Arc<ParameterResolver>,
}

impl fmt::Debug for NodeExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeExecutor")
            .field("registry", &"<Registry>")
            .field("resolver", &self.resolver)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed(ProcessingStats),
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone)]
pub struct NodeRunState {
    pub status: NodeStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl NodeRunState {
    pub fn new() -> Self {
        Self {
            status: NodeStatus::Pending,
            started_at: None,
        }
    }
}

impl Default for NodeRunState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub image_info: ImageInfo,
    pub buffer: Option<PixelBuffer>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}

impl ExecutionContext {
    pub fn new(image_info: ImageInfo, buffer: Option<PixelBuffer>, metadata: Metadata) -> Self {
        Self {
            image_info,
            buffer,
            metadata,
            node_states: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub buffer: Option<PixelBuffer>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}

impl NodeExecutor {
    pub fn new(registry: Arc<Registry>, resolver: Arc<ParameterResolver>) -> Self {
        Self { registry, resolver }
    }

    pub async fn execute(
        &self,
        graph: &PipelineGraph,
        image_info: &ImageInfo,
        buffer: Option<PixelBuffer>,
        metadata: &Metadata,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ExecutionResult> {
        let order = graph.topological_order()?;
        let node_count = order.len();

        let mut ctx = ExecutionContext::new(image_info.clone(), buffer, metadata.clone());
        for node in &graph.nodes {
            ctx.node_states.entry(node.id).or_default();
        }

        for (i, node_id) in order.iter().enumerate() {
            if progress.is_canceled() {
                return Err(PluginError::Canceled {
                    plugin: graph
                        .node(*node_id)
                        .map(|n| n.plugin_id.clone())
                        .unwrap_or_default(),
                });
            }

            let node = match graph.node(*node_id) {
                Some(n) => n,
                None => continue,
            };

            if !node.enabled {
                ctx.node_states.insert(
                    *node_id,
                    NodeRunState {
                        status: NodeStatus::Skipped,
                        started_at: None,
                    },
                );
                continue;
            }

            let plugin = self
                .registry
                .get(&node.plugin_id)
                .ok_or_else(|| PluginError::NotFound(node.plugin_id.clone()))?;

            let resolved_params = self.resolver.resolve(
                *node_id,
                ctx.image_info.id,
                plugin.parameter_schema(),
                &ctx.metadata,
                &ctx.image_info,
            );

            let mut final_params = resolved_params;
            if let Some(ref overrides) = node.parameter_overrides {
                final_params.merge(overrides);
            }

            let issues = plugin.validate(&final_params).await?;
            if issues
                .iter()
                .any(|iss| matches!(iss, photopipeline_core::ValidationIssue::Error { .. }))
            {
                let err_msgs: Vec<String> = issues.iter().map(|i| i.to_string()).collect();
                ctx.node_states.insert(
                    *node_id,
                    NodeRunState {
                        status: NodeStatus::Failed(err_msgs.join("; ")),
                        started_at: Some(chrono::Utc::now()),
                    },
                );
                return Err(PluginError::ValidationFailed(err_msgs.join("; ")));
            }

            let started_at = chrono::Utc::now();
            ctx.node_states.insert(
                *node_id,
                NodeRunState {
                    status: NodeStatus::Running,
                    started_at: Some(started_at),
                },
            );

            let fraction = (i as f32) / (node_count.max(1) as f32);
            progress.set_progress(fraction, &format!("processing node {}", node.label));

            let stats = if plugin.requires_pixel_access() {
                self.process_pixel_node(&mut ctx, node, &final_params)
                    .await?
            } else {
                self.process_metadata_node(&mut ctx, node, &final_params)
                    .await?
            };

            ctx.node_states.insert(
                *node_id,
                NodeRunState {
                    status: NodeStatus::Completed(stats),
                    started_at: Some(started_at),
                },
            );
        }

        progress.set_progress(1.0, "complete");

        Ok(ExecutionResult {
            buffer: ctx.buffer,
            metadata: ctx.metadata,
            node_states: ctx.node_states,
        })
    }

    async fn process_pixel_node(
        &self,
        ctx: &mut ExecutionContext,
        node: &crate::graph::PipelineNode,
        params: &photopipeline_plugin::ParameterSet,
    ) -> PluginResult<ProcessingStats> {
        let input = ctx
            .buffer
            .as_ref()
            .ok_or_else(|| PluginError::NodeExecutionFailed {
                node: node.label.clone(),
                message: "no pixel buffer available for pixel node".into(),
            })?;

        let mut output = PixelBuffer::new(input.width, input.height, input.layout, input.format);
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        let processor = self
            .registry
            .get_pixel_processor(&node.plugin_id)
            .ok_or_else(|| PluginError::NotFound(node.plugin_id.clone()))?;

        struct InlineProgress {
            canceled: Arc<AtomicBool>,
        }
        impl photopipeline_plugin::ProgressSink for InlineProgress {
            fn set_progress(&self, _fraction: f32, _message: &str) {}
            fn is_canceled(&self) -> bool {
                self.canceled.load(Ordering::Relaxed)
            }
        }

        let stats = processor
            .process_pixels(
                input,
                &mut output,
                params,
                Box::new(InlineProgress {
                    canceled: Arc::new(AtomicBool::new(false)),
                }),
            )
            .await?;

        ctx.buffer = Some(output);

        Ok(stats)
    }

    async fn process_metadata_node(
        &self,
        ctx: &mut ExecutionContext,
        node: &crate::graph::PipelineNode,
        params: &photopipeline_plugin::ParameterSet,
    ) -> PluginResult<ProcessingStats> {
        let processor = self
            .registry
            .get_metadata_processor(&node.plugin_id)
            .ok_or_else(|| PluginError::NotFound(node.plugin_id.clone()))?;

        let target = photopipeline_core::MetadataTarget {
            path: ctx.image_info.path.clone(),
            format: ctx.image_info.format.clone(),
        };

        match processor.read_metadata(&target, params).await {
            Ok(read_meta) => {
                if read_meta.exif.is_some()
                    || read_meta.xmp.is_some()
                    || read_meta.iptc.is_some()
                    || read_meta.gps.is_some()
                {
                    if read_meta.exif.is_some() {
                        ctx.metadata.exif = read_meta.exif;
                    }
                    if read_meta.xmp.is_some() {
                        ctx.metadata.xmp = read_meta.xmp;
                    }
                    if read_meta.iptc.is_some() {
                        ctx.metadata.iptc = read_meta.iptc;
                    }
                    if read_meta.gps.is_some() {
                        ctx.metadata.gps = read_meta.gps;
                    }
                }
            }
            Err(e) => {
                tracing::debug!("metadata processor '{}' read failed: {}", node.plugin_id, e);
            }
        }

        let mut write_target = photopipeline_core::MetadataTarget {
            path: ctx.image_info.path.clone(),
            format: ctx.image_info.format.clone(),
        };

        match processor
            .write_metadata(&mut write_target, &ctx.metadata, params)
            .await
        {
            Ok(report) => {
                tracing::debug!(
                    "metadata processor '{}' wrote {} tags ({} skipped)",
                    node.plugin_id,
                    report.tags_written,
                    report.tags_skipped
                );
            }
            Err(e) => {
                tracing::debug!(
                    "metadata processor '{}' write failed: {}",
                    node.plugin_id,
                    e
                );
            }
        }

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: None,
            peak_memory_mb: 0,
            input_pixels: 0,
            output_pixels: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photopipeline_core::{ChannelLayout, ColorSpace, ImageFormat, PixelFormat};
    use uuid::Uuid;

    fn make_test_image_info() -> ImageInfo {
        ImageInfo {
            id: Uuid::new_v4(),
            path: "/tmp/test.jpg".into(),
            filename: "test.jpg".into(),
            format: ImageFormat::JPEG,
            width: 100,
            height: 100,
            file_size_bytes: 5000,
            pixel_format: PixelFormat::U8,
            color_space: ColorSpace::default(),
        }
    }

    #[test]
    fn node_run_state_default_is_pending() {
        let state = NodeRunState::new();
        assert!(matches!(state.status, NodeStatus::Pending));
        assert!(state.started_at.is_none());
    }

    #[test]
    fn node_run_state_default_trait() {
        let state = NodeRunState::default();
        assert!(matches!(state.status, NodeStatus::Pending));
    }

    #[test]
    fn node_status_pending_display() {
        let status = NodeStatus::Pending;
        let s = format!("{:?}", status);
        assert!(s.contains("Pending"));
    }

    #[test]
    fn node_status_running_display() {
        let status = NodeStatus::Running;
        let s = format!("{:?}", status);
        assert!(s.contains("Running"));
    }

    #[test]
    fn node_status_completed_display() {
        let stats = ProcessingStats {
            elapsed_ms: 100,
            cpu_time_ms: 80,
            gpu_time_ms: None,
            peak_memory_mb: 256,
            input_pixels: 10000,
            output_pixels: 10000,
        };
        let status = NodeStatus::Completed(stats);
        let s = format!("{:?}", status);
        assert!(s.contains("Completed"));
    }

    #[test]
    fn node_status_failed_display() {
        let status = NodeStatus::Failed("error msg".into());
        let s = format!("{:?}", status);
        assert!(s.contains("Failed"));
    }

    #[test]
    fn node_status_skipped_display() {
        let status = NodeStatus::Skipped;
        let s = format!("{:?}", status);
        assert!(s.contains("Skipped"));
    }

    #[test]
    fn node_run_state_with_started_at() {
        let now = chrono::Utc::now();
        let state = NodeRunState {
            status: NodeStatus::Running,
            started_at: Some(now),
        };
        assert_eq!(state.started_at, Some(now));
    }

    #[test]
    fn execution_context_new_with_none_buffer() {
        let info = make_test_image_info();
        let metadata = Metadata::default();
        let ctx = ExecutionContext::new(info.clone(), None, metadata.clone());
        assert!(ctx.buffer.is_none());
        assert_eq!(ctx.image_info.filename, "test.jpg");
    }

    #[test]
    fn execution_context_new_with_some_buffer() {
        let info = make_test_image_info();
        let metadata = Metadata::default();
        let pb = PixelBuffer::new(10, 10, ChannelLayout::RGB, PixelFormat::U8);
        let ctx = ExecutionContext::new(info, Some(pb), metadata);
        assert!(ctx.buffer.is_some());
    }

    #[test]
    fn execution_context_node_states_empty() {
        let ctx = ExecutionContext::new(make_test_image_info(), None, Metadata::default());
        assert!(ctx.node_states.is_empty());
    }

    #[test]
    fn execution_result_default_fields() {
        let result = ExecutionResult {
            buffer: None,
            metadata: Metadata::default(),
            node_states: std::collections::HashMap::new(),
        };
        assert!(result.buffer.is_none());
    }

    #[test]
    fn execution_result_with_buffer() {
        let pb = PixelBuffer::new(10, 10, ChannelLayout::RGB, PixelFormat::U8);
        let result = ExecutionResult {
            buffer: Some(pb),
            metadata: Metadata::default(),
            node_states: Default::default(),
        };
        assert!(result.buffer.is_some());
    }

    #[test]
    fn node_executor_debug_format() {
        let registry = std::sync::Arc::new(photopipeline_plugin::Registry::new());
        let resolver = std::sync::Arc::new(ParameterResolver::new());
        let executor = NodeExecutor::new(registry, resolver);
        let debug_str = format!("{:?}", executor);
        assert!(debug_str.contains("NodeExecutor"));
    }
}
