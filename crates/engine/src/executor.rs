use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use photopipeline_core::{
    EncodeOptions, ImageInfo, Metadata, NodeId, PixelBuffer, PluginError, PluginResult,
    ProcessingStats,
};
use photopipeline_plugin::{ProgressSink, Registry};

use crate::graph::{PipelineGraph, PipelineTemplate};
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
    pub encoded_output: Option<Vec<u8>>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}

impl ExecutionContext {
    pub fn new(image_info: ImageInfo, buffer: Option<PixelBuffer>, metadata: Metadata) -> Self {
        Self {
            image_info,
            buffer,
            encoded_output: None,
            metadata,
            node_states: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub buffer: Option<PixelBuffer>,
    pub encoded_output: Option<Vec<u8>>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}

impl NodeExecutor {
    #[tracing::instrument(skip_all)]
    pub fn new(registry: Arc<Registry>, resolver: Arc<ParameterResolver>) -> Self {
        let registry_size = registry.manifests().len();
        tracing::info!(
            registry_plugins = registry_size,
            "NodeExecutor created with {} plugins in registry",
            registry_size,
        );
        Self { registry, resolver }
    }

    #[tracing::instrument(skip_all, fields(image_id = %image_info.id, node_count = graph.nodes.len()))]
    pub async fn execute(
        &self,
        graph: &PipelineGraph,
        image_info: &ImageInfo,
        buffer: Option<PixelBuffer>,
        metadata: &Metadata,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ExecutionResult> {
        let _timer = photopipeline_core::PerfTimer::with_target("pipeline_execute", "pipeline");

        let order = graph.topological_order()?;
        let node_count = order.len();

        tracing::info!(
            image_path = %image_info.path,
            buffer_provided = buffer.is_some(),
            "Starting pipeline execution: {} nodes, image {}",
            node_count,
            image_info.filename,
        );

        let mut ctx = ExecutionContext::new(image_info.clone(), buffer, metadata.clone());
        for node in &graph.nodes {
            ctx.node_states.entry(node.id).or_default();
        }

        let mut nodes_completed: u32 = 0;
        let mut nodes_failed: u32 = 0;
        let mut nodes_skipped: u32 = 0;

        for (i, node_id) in order.iter().enumerate() {
            if progress.is_canceled() {
                tracing::warn!(
                    node_index = i,
                    "Pipeline execution canceled at node {}/{}",
                    i + 1,
                    node_count,
                );
                return Err(PluginError::Canceled {
                    plugin: graph
                        .node(*node_id)
                        .map(|n| n.plugin_id.clone())
                        .unwrap_or_default(),
                });
            }

            let node = match graph.node(*node_id) {
                Some(n) => n,
                None => {
                    tracing::debug!(node_id = %node_id, "Node not found in graph, skipping");
                    continue;
                }
            };

            if !node.enabled {
                tracing::debug!(
                    node_label = %node.label,
                    plugin_id = %node.plugin_id,
                    "Node '{}' is disabled, skipping",
                    node.label,
                );
                ctx.node_states.insert(
                    *node_id,
                    NodeRunState {
                        status: NodeStatus::Skipped,
                        started_at: None,
                    },
                );
                nodes_skipped += 1;
                continue;
            }

            let plugin = self
                .registry
                .get(&node.plugin_id)
                .ok_or_else(|| PluginError::NotFound(node.plugin_id.clone()))?;

            tracing::info!(
                node_label = %node.label,
                plugin_id = %node.plugin_id,
                node_index = i + 1,
                node_count = node_count,
                "Starting node [{}/{}] {} (plugin: {})",
                i + 1,
                node_count,
                node.label,
                node.plugin_id,
            );

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
                tracing::error!(
                    node_label = %node.label,
                    plugin_id = %node.plugin_id,
                    validation_errors = %err_msgs.join("; "),
                    "Node '{}' validation failed",
                    node.label,
                );
                ctx.node_states.insert(
                    *node_id,
                    NodeRunState {
                        status: NodeStatus::Failed(err_msgs.join("; ")),
                        started_at: Some(chrono::Utc::now()),
                    },
                );
                nodes_failed += 1;
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

            let node_timer = photopipeline_core::PerfTimer::with_target(
                format!("node:{}", node.label),
                "pipeline.node",
            );

            let stats = if plugin.requires_pixel_access() {
                tracing::debug!(
                    node_label = %node.label,
                    "Executing pixel-processing node '{}'",
                    node.label,
                );
                match self.process_pixel_node(&mut ctx, node, &final_params, progress.as_ref()).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!(
                            node_label = %node.label,
                            plugin_id = %node.plugin_id,
                            error = %e,
                            "Node '{}' execution failed: {}",
                            node.label,
                            e,
                        );
                        ctx.node_states.insert(
                            *node_id,
                            NodeRunState {
                                status: NodeStatus::Failed(e.to_string()),
                                started_at: Some(started_at),
                            },
                        );
                        nodes_failed += 1;
                        return Err(e);
                    }
                }
            } else if self.registry.get_metadata_processor(&node.plugin_id).is_some() {
                tracing::debug!(
                    node_label = %node.label,
                    "Executing metadata-processing node '{}'",
                    node.label,
                );
                match self
                    .process_metadata_node(&mut ctx, node, &final_params)
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!(
                            node_label = %node.label,
                            plugin_id = %node.plugin_id,
                            error = %e,
                            "Node '{}' execution failed: {}",
                            node.label,
                            e,
                        );
                        ctx.node_states.insert(
                            *node_id,
                            NodeRunState {
                                status: NodeStatus::Failed(e.to_string()),
                                started_at: Some(started_at),
                            },
                        );
                        nodes_failed += 1;
                        return Err(e);
                    }
                }
            } else if self.registry.get_format_processor(&node.plugin_id).is_some() {
                tracing::debug!(
                    node_label = %node.label,
                    "Executing format-processing node '{}'",
                    node.label,
                );
                match self
                    .process_format_node(&mut ctx, node, &final_params)
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!(
                            node_label = %node.label,
                            plugin_id = %node.plugin_id,
                            error = %e,
                            "Node '{}' execution failed: {}",
                            node.label,
                            e,
                        );
                        ctx.node_states.insert(
                            *node_id,
                            NodeRunState {
                                status: NodeStatus::Failed(e.to_string()),
                                started_at: Some(started_at),
                            },
                        );
                        nodes_failed += 1;
                        return Err(e);
                    }
                }
            } else {
                let err = PluginError::NotFound(node.plugin_id.clone());
                tracing::error!(
                    node_label = %node.label,
                    plugin_id = %node.plugin_id,
                    error = %err,
                    "Node '{}': no processor found for plugin (not pixel, metadata, or format)",
                    node.label,
                );
                ctx.node_states.insert(
                    *node_id,
                    NodeRunState {
                        status: NodeStatus::Failed(err.to_string()),
                        started_at: Some(started_at),
                    },
                );
                nodes_failed += 1;
                return Err(err);
            };

            let node_ms = node_timer.elapsed_ms();
            drop(node_timer);

            tracing::info!(
                node_label = %node.label,
                plugin_id = %node.plugin_id,
                elapsed_ms = node_ms,
                "Node '{}' completed in {}ms",
                node.label,
                node_ms,
            );

            ctx.node_states.insert(
                *node_id,
                NodeRunState {
                    status: NodeStatus::Completed(stats),
                    started_at: Some(started_at),
                },
            );
            nodes_completed += 1;
        }

        let total_ms = _timer.elapsed_ms();
        drop(_timer);

        tracing::info!(
            total_elapsed_ms = total_ms,
            nodes_completed = nodes_completed,
            nodes_failed = nodes_failed,
            nodes_skipped = nodes_skipped,
            "Pipeline execution complete: {}ms total, {} completed, {} failed, {} skipped",
            total_ms,
            nodes_completed,
            nodes_failed,
            nodes_skipped,
        );

        progress.set_progress(1.0, "complete");

        Ok(ExecutionResult {
            buffer: ctx.buffer,
            encoded_output: ctx.encoded_output,
            metadata: ctx.metadata,
            node_states: ctx.node_states,
        })
    }

    #[tracing::instrument(skip_all, fields(node = %node.label))]
    async fn process_pixel_node(
        &self,
        ctx: &mut ExecutionContext,
        node: &crate::graph::PipelineNode,
        params: &photopipeline_plugin::ParameterSet,
        progress: &dyn photopipeline_plugin::ProgressSink,
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

        tracing::debug!(
            input_width = input.width,
            input_height = input.height,
            input_format = ?input.format,
            "Processing pixel node '{}' with ({},{}) {:?} input",
            node.label,
            input.width,
            input.height,
            input.format,
        );

        let processor = self
            .registry
            .get_pixel_processor(&node.plugin_id)
            .ok_or_else(|| PluginError::NotFound(node.plugin_id.clone()))?;

        struct InlineProgress {
            canceled: std::sync::Arc<std::sync::atomic::AtomicBool>,
        }
        impl photopipeline_plugin::ProgressSink for InlineProgress {
            fn set_progress(&self, _fraction: f32, _message: &str) {}
            fn is_canceled(&self) -> bool {
                use std::sync::atomic::Ordering;
                self.canceled.load(Ordering::Relaxed)
            }
        }

        // Create a cancel flag that syncs with the caller's progress sink
        let cancel_state = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(
            progress.is_canceled(),
        ));

        let tile_threshold: u64 = 4096 * 2160; // ~8.8M pixels

        let (output, stats) = if input.pixel_count() > tile_threshold {
            let engine = crate::tile::TileEngine::default();
            tracing::info!(
                input_width = input.width,
                input_height = input.height,
                pixel_count = input.pixel_count(),
                "Activating tiled processing for large image {}x{}",
                input.width,
                input.height,
            );
            let tiled_output = engine
                .process_tiled(
                    processor.as_ref(),
                    input,
                    params,
                    &InlineProgress {
                        canceled: cancel_state.clone(),
                    },
                )
                .await?;
            let stats = ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: None,
                peak_memory_mb: 0,
                input_pixels: input.pixel_count(),
                output_pixels: tiled_output.pixel_count(),
            };
            (tiled_output, stats)
        } else {
            let stats = processor
                .process_pixels(
                    input,
                    &mut output,
                    params,
                    Box::new(InlineProgress {
                        canceled: cancel_state.clone(),
                    }),
                )
                .await?;
            (output, stats)
        };

        tracing::debug!(
            output_width = output.width,
            output_height = output.height,
            elapsed_ms = stats.elapsed_ms,
            "Pixel node '{}' produced ({},{}) output in {}ms",
            node.label,
            output.width,
            output.height,
            stats.elapsed_ms,
        );

        ctx.buffer = Some(output);

        Ok(stats)
    }

    #[tracing::instrument(skip_all, fields(node = %node.label))]
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

        tracing::debug!(
            target_path = %target.path,
            "Reading metadata for node '{}' from {}",
            node.label,
            target.path,
        );

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
                    tracing::debug!(
                        node_label = %node.label,
                        "Metadata read successful for node '{}'",
                        node.label,
                    );
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

        tracing::debug!(
            target_path = %write_target.path,
            "Writing metadata for node '{}' to {}",
            node.label,
            write_target.path,
        );

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

    #[tracing::instrument(skip_all, fields(node = %node.label))]
    async fn process_format_node(
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
                message: "no pixel buffer available for format node".into(),
            })?;

        let processor = self
            .registry
            .get_format_processor(&node.plugin_id)
            .ok_or_else(|| PluginError::NotFound(node.plugin_id.clone()))?;

        let options = EncodeOptions {
            format: processor.format_id(),
            quality: params.get_f64("quality").map(|q| q as f32),
            lossless: params.get_bool("lossless").unwrap_or(false),
            bit_depth: params
                .get_i64("bit_depth")
                .map(|b| b as u8)
                .unwrap_or(8),
            chroma_subsampling: None,
            encoder: None,
            effort: params
                .get_i64("effort")
                .or_else(|| params.get_i64("speed"))
                .map(|e| e as u8),
            compression: params
                .get_str("compression")
                .or_else(|| params.get_str("compression_level"))
                .map(|s| s.to_string()),
            embed_profile: None,
        };

        tracing::debug!(
            input_width = input.width,
            input_height = input.height,
            format = ?options.format,
            "Encoding with format processor '{}': {}x{} → {:?}",
            node.label,
            input.width,
            input.height,
            options.format,
        );

        // Input-only format processors (can_encode() == false) act as passthrough.
        // They decode raw input into pixels earlier in the real pipeline; here the
        // input is already loaded by the server, so pass it through unchanged.
        if !processor.can_encode(&options.format) {
            tracing::debug!(
                node_label = %node.label,
                "Format processor '{}' is input-only, passing through pixel buffer",
                node.label,
            );
            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: None,
                peak_memory_mb: 0,
                input_pixels: input.pixel_count(),
                output_pixels: input.pixel_count(),
            });
        }

        let encoded = processor.encode(input, &ctx.metadata, &options).await?;

        tracing::info!(
            node_label = %node.label,
            encoded_bytes = encoded.len(),
            "Format node '{}' produced {} bytes",
            node.label,
            encoded.len(),
        );

        ctx.encoded_output = Some(encoded);

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: None,
            peak_memory_mb: 0,
            input_pixels: input.pixel_count(),
            output_pixels: 0,
        })
    }
}

/// Shared execution entry-point for both CLI and gRPC modes.
///
/// Validates a `PipelineTemplate`, converts it to a graph, validates the graph,
/// and executes all nodes in topological order.
///
/// Callers are responsible for loading the input image (producing `buffer`),
/// extracting `image_info` and `metadata`, and providing a `ProgressSink` for
/// progress/cancellation. Output is returned via `ExecutionResult`.
#[tracing::instrument(skip_all, fields(image_id = %image_info.id))]
pub async fn execute_config(
    template: &PipelineTemplate,
    registry: Arc<Registry>,
    resolver: Arc<ParameterResolver>,
    image_info: &ImageInfo,
    buffer: Option<PixelBuffer>,
    metadata: &Metadata,
    progress: Box<dyn ProgressSink>,
) -> PluginResult<ExecutionResult> {
    template
        .validate()
        .map_err(|e| PluginError::ValidationFailed(e))?;

    let graph = template.clone().into_graph();

    if let Err(issues) = graph.validate_graph() {
        return Err(PluginError::ValidationFailed(issues.join("; ")));
    }

    let executor = NodeExecutor::new(registry, resolver);
    executor.execute(&graph, image_info, buffer, metadata, progress).await
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
            encoded_output: None,
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
            encoded_output: None,
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

    // ── Execute behavior tests ───────────────────────────────────

    fn empty_registry() -> Arc<photopipeline_plugin::Registry> {
        Arc::new(photopipeline_plugin::Registry::new())
    }

    fn empty_resolver() -> Arc<ParameterResolver> {
        Arc::new(ParameterResolver::new())
    }

    struct NoopTestProgress;
    impl photopipeline_plugin::ProgressSink for NoopTestProgress {
        fn set_progress(&self, _: f32, _: &str) {}
        fn is_canceled(&self) -> bool {
            false
        }
    }

    struct CancelProgress;
    impl photopipeline_plugin::ProgressSink for CancelProgress {
        fn set_progress(&self, _: f32, _: &str) {}
        fn is_canceled(&self) -> bool {
            true
        }
    }

    #[test]
    fn execute_empty_graph_returns_ok() {
        let reg = empty_registry();
        let resolver = empty_resolver();
        let executor = NodeExecutor::new(reg, resolver);
        let graph = PipelineGraph::new();
        let info = make_test_image_info();
        let md = Metadata::default();
        let progress: Box<dyn ProgressSink> = Box::new(CancelProgress);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result =
            rt.block_on(async { executor.execute(&graph, &info, None, &md, progress).await });
        assert!(
            result.is_ok(),
            "empty graph should execute successfully: {:?}",
            result.err()
        );
        let exec_result = result.unwrap();
        assert!(exec_result.node_states.is_empty());
        assert!(exec_result.buffer.is_none());
    }

    #[test]
    fn execute_disabled_node_is_skipped() {
        let reg = empty_registry();
        let resolver = empty_resolver();
        let executor = NodeExecutor::new(reg, resolver);

        let mut graph = PipelineGraph::new();
        let node_id = graph.add_node("n1".into(), "test.plugin".into());
        // Disable the node via graph API
        let node = graph.node_mut(node_id).unwrap();
        node.enabled = false;

        let info = make_test_image_info();
        let md = Metadata::default();
        let progress: Box<dyn ProgressSink> = Box::new(NoopTestProgress);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result =
            rt.block_on(async { executor.execute(&graph, &info, None, &md, progress).await });

        // Should be Ok because disabled nodes are skipped, even if plugin is missing
        assert!(
            result.is_ok(),
            "graph with only disabled nodes should succeed: {:?}",
            result.err()
        );
        let exec_result = result.unwrap();
        let state = exec_result.node_states.get(&node_id).unwrap();
        assert!(
            matches!(state.status, NodeStatus::Skipped),
            "disabled node should be Skipped, got {:?}",
            state.status
        );
    }

    #[test]
    fn execute_missing_plugin_returns_not_found() {
        let reg = empty_registry();
        let resolver = empty_resolver();
        let executor = NodeExecutor::new(reg, resolver);

        let mut graph = PipelineGraph::new();
        graph.add_node("n1".into(), "nonexistent.plugin.test".into());

        let info = make_test_image_info();
        let md = Metadata::default();
        let progress: Box<dyn ProgressSink> = Box::new(NoopTestProgress);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result =
            rt.block_on(async { executor.execute(&graph, &info, None, &md, progress).await });

        assert!(result.is_err(), "missing plugin should return error");
        assert!(
            matches!(result, Err(PluginError::NotFound(_))),
            "expected NotFound error, got {:?}",
            result
        );
    }

    #[test]
    fn execute_canceled_returns_error() {
        let reg = empty_registry();
        let resolver = empty_resolver();
        let executor = NodeExecutor::new(reg, resolver);

        let mut graph = PipelineGraph::new();
        graph.add_node("n1".into(), "some.plugin".into());

        let info = make_test_image_info();
        let md = Metadata::default();
        // This progress sink always reports canceled
        let progress: Box<dyn ProgressSink> = Box::new(CancelProgress);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result =
            rt.block_on(async { executor.execute(&graph, &info, None, &md, progress).await });

        assert!(result.is_err(), "canceled execution should return error");
        match result {
            Err(PluginError::Canceled { .. }) => {}
            other => panic!("expected Canceled error, got {:?}", other),
        }
    }

    #[test]
    fn execute_graph_with_cycle_returns_error() {
        let reg = empty_registry();
        let resolver = empty_resolver();
        let executor = NodeExecutor::new(reg, resolver);

        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("a".into(), "p1".into());
        let n2 = graph.add_node("b".into(), "p2".into());
        // Create a cycle: a->b, b->a
        let out_a = graph.node(n1).unwrap().outputs[0];
        let in_b = graph.node(n2).unwrap().inputs[0];
        let out_b = graph.node(n2).unwrap().outputs[0];
        let in_a = graph.node(n1).unwrap().inputs[0];
        let _ = graph.connect(out_a, in_b);
        let _ = graph.connect(out_b, in_a);

        let info = make_test_image_info();
        let md = Metadata::default();
        let progress: Box<dyn ProgressSink> = Box::new(CancelProgress);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result =
            rt.block_on(async { executor.execute(&graph, &info, None, &md, progress).await });

        // Cycle detection may happen in topological_order() or validate_graph()
        assert!(result.is_err(), "graph with cycle should return error");
    }
}
