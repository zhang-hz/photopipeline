use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use photopipeline_core::{ImageInfo, Metadata};
use photopipeline_engine::{NodeExecutor, PipelineTemplate, TemplateEdge, TemplateNode};
use photopipeline_plugin::ProgressSink;

use crate::pb::pipeline::{
    ExecuteProgress, ExecuteRequest, NodeSchema, PipelineId, PipelineSpec, PluginId,
    ValidationIssue, ValidationResult, execute_progress::Stage as ProtoStage,
    pipeline_service_server::PipelineService, validation_issue::Severity as ProtoSeverity,
};
use crate::{SharedState, json_to_prost_value, prost_struct_to_params, schema_to_prost_struct};

pub struct PipelineServiceImpl {
    state: Arc<SharedState>,
}

impl PipelineServiceImpl {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self { state }
    }
}

fn build_template(spec: &PipelineSpec) -> PipelineTemplate {
    let nodes: Vec<TemplateNode> = spec
        .nodes
        .iter()
        .map(|n| {
            let params = n.params.as_ref().map(|s| prost_struct_to_params(s));
            TemplateNode {
                id: n.id.clone(),
                plugin: n.plugin_id.clone(),
                label: if n.label.is_empty() {
                    Some(n.id.clone())
                } else {
                    Some(n.label.clone())
                },
                enabled: n.enabled,
                params,
            }
        })
        .collect();

    let edges: Vec<TemplateEdge> = spec
        .edges
        .iter()
        .map(|e| TemplateEdge {
            from: e.from.clone(),
            to: e.to.clone(),
        })
        .collect();

    PipelineTemplate {
        metadata: Default::default(),
        nodes,
        edges,
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

fn build_image_info(path: &str) -> ImageInfo {
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let format = crate::detect_format_from_ext(path);

    let file_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Try to load the image to get real dimensions
    if let Ok(loaded) = load_image_from_disk(path) {
        return ImageInfo {
            id: Uuid::new_v4(),
            path: path.to_string(),
            filename,
            format,
            width: loaded.width,
            height: loaded.height,
            file_size_bytes,
            pixel_format: photopipeline_core::PixelFormat::U8,
            color_space: photopipeline_core::ColorSpace::default(),
        };
    }

    ImageInfo {
        id: Uuid::new_v4(),
        path: path.to_string(),
        filename,
        format,
        width: 0,
        height: 0,
        file_size_bytes,
        pixel_format: photopipeline_core::PixelFormat::U8,
        color_space: photopipeline_core::ColorSpace::default(),
    }
}

/// Load an image from disk and convert to PixelBuffer (RGB U8).
fn load_image_from_disk(path: &str) -> Result<photopipeline_core::PixelBuffer, String> {
    use photopipeline_core::{PixelBuffer, ChannelLayout, PixelFormat, ColorSpace, AlignedBuffer};
    let img = image::open(path).map_err(|e| format!("Failed to open image {}: {}", path, e))?;
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    let raw = rgb.into_raw();
    let len = raw.len();
    let mut aligned = AlignedBuffer::new(len, 64);
    aligned.data[..len].copy_from_slice(&raw);
    Ok(PixelBuffer {
        width,
        height,
        data: aligned,
        layout: ChannelLayout::RGB,
        format: PixelFormat::U8,
        color_space: ColorSpace::SRGB,
        icc_profile: None,
    })
}


struct ChannelProgressSink {
    sender: mpsc::Sender<Result<ExecuteProgress, Status>>,
    node_id: String,
    node_label: String,
    start: std::time::Instant,
    canceled: Option<Arc<AtomicBool>>,
}

impl ProgressSink for ChannelProgressSink {
    fn set_progress(&self, fraction: f32, message: &str) {
        let _ = self.sender.try_send(Ok(ExecuteProgress {
            stage: ProtoStage::Processing as i32,
            node_id: self.node_id.clone(),
            node_label: self.node_label.clone(),
            fraction,
            message: message.to_string(),
            elapsed_ms: self.start.elapsed().as_millis() as i64,
        }));
    }

    fn is_canceled(&self) -> bool {
        self.canceled
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }
}

#[tonic::async_trait]
impl PipelineService for PipelineServiceImpl {
    async fn create_pipeline(
        &self,
        request: Request<PipelineSpec>,
    ) -> Result<Response<PipelineId>, Status> {
        let spec = request.into_inner();
        let template = build_template(&spec);

        template
            .validate()
            .map_err(|e| Status::invalid_argument(format!("invalid pipeline: {}", e)))?;

        let graph = template.into_graph();
        let id = uuid::Uuid::new_v4();

        tracing::info!(
            pipeline_id = %id,
            node_count = spec.nodes.len(),
            "create_pipeline: stored pipeline with {} nodes",
            spec.nodes.len()
        );

        self.state.graphs.write().insert(id, graph);

        Ok(Response::new(PipelineId { id: id.to_string() }))
    }

    type ExecuteStream = ReceiverStream<Result<ExecuteProgress, Status>>;

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let req = request.into_inner();
        let pipeline_id = req.pipeline_id.clone();
        let image_path = req.image_path.clone();
        let output_path = req.output_path.clone();
        tracing::info!("execute: pipeline={}, image={}, output={}", pipeline_id, image_path, output_path);

        let graph_id = Uuid::parse_str(&pipeline_id)
            .map_err(|e| Status::invalid_argument(format!("invalid pipeline id: {}", e)))?;

        let graph = {
            let graphs = self.state.graphs.read();
            graphs.get(&graph_id).cloned()
        }
        .ok_or_else(|| Status::not_found(format!("pipeline not found: {}", pipeline_id)))?;

        if !std::path::Path::new(&image_path).exists() {
            return Err(Status::not_found(format!(
                "image file not found: {}",
                image_path
            )));
        }

        let image_info = build_image_info(&image_path);
        let metadata = Metadata::default();

        let (tx, rx) = mpsc::channel::<Result<ExecuteProgress, Status>>(256);
        let start = std::time::Instant::now();

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let progress = ChannelProgressSink {
            sender: tx.clone(),
            node_id: String::new(),
            node_label: String::new(),
            start,
            canceled: Some(cancel_flag.clone()),
        };

        let _ = tx.try_send(Ok(ExecuteProgress {
            stage: ProtoStage::Loading as i32,
            node_id: String::new(),
            node_label: format!("Loading {}", image_path),
            fraction: 0.0,
            message: "Starting pipeline...".into(),
            elapsed_ms: 0,
        }));

        // Load input image from disk
        let input_buffer = match load_image_from_disk(&image_path) {
            Ok(buf) => Some(buf),
            Err(e) => {
                let _ = tx.try_send(Ok(ExecuteProgress {
                    stage: ProtoStage::Error as i32,
                    node_id: String::new(),
                    node_label: "LoadError".into(),
                    fraction: 0.0,
                    message: format!("Failed to load input image: {}", e),
                    elapsed_ms: 0,
                }));
                drop(tx);
                return Err(Status::internal(format!("Failed to load image: {}", e)));
            }
        };

        let executor = NodeExecutor::new(self.state.registry.clone(), self.state.resolver.clone());

        tokio::spawn(async move {
            match executor
                .execute(&graph, &image_info, input_buffer, &metadata, Box::new(progress))
                .await
            {
                Ok(result) => {
                    // Save output to the requested output path.
                    // Format-processor pipelines produce encoded bytes; other pipelines produce
                    // a PixelBuffer that we save with save_buffer_to_file.
                    if !output_path.is_empty() {
                        if let Some(ref encoded) = result.encoded_output {
                            if let Err(e) = std::fs::write(&output_path, encoded) {
                                let _ = tx.try_send(Ok(ExecuteProgress {
                                    stage: ProtoStage::Error as i32,
                                    node_id: String::new(),
                                    node_label: "SaveError".into(),
                                    fraction: 1.0,
                                    message: format!("Failed to save encoded output: {}", e),
                                    elapsed_ms: start.elapsed().as_millis() as i64,
                                }));
                                return;
                            }
                            tracing::info!(
                                "Encoded output saved to {} ({} bytes)",
                                output_path,
                                encoded.len()
                            );
                        } else if let Some(buffer) = &result.buffer {
                            let save_result = save_buffer_to_file(buffer, &output_path);
                            if let Err(e) = &save_result {
                                let _ = tx.try_send(Ok(ExecuteProgress {
                                    stage: ProtoStage::Error as i32,
                                    node_id: String::new(),
                                    node_label: "SaveError".into(),
                                    fraction: 1.0,
                                    message: format!("Failed to save output: {}", e),
                                    elapsed_ms: start.elapsed().as_millis() as i64,
                                }));
                                return;
                            }
                        }
                    }
                    let _ = tx.try_send(Ok(ExecuteProgress {
                        stage: ProtoStage::Done as i32,
                        node_id: String::new(),
                        node_label: "Complete".into(),
                        fraction: 1.0,
                        message: format!(
                            "Pipeline {} done in {}ms",
                            pipeline_id,
                            start.elapsed().as_millis()
                        ),
                        elapsed_ms: start.elapsed().as_millis() as i64,
                    }));
                    tracing::info!("Pipeline {} completed successfully", pipeline_id);
                }
                Err(e) => {
                    let _ = tx.try_send(Ok(ExecuteProgress {
                        stage: ProtoStage::Error as i32,
                        node_id: String::new(),
                        node_label: "Error".into(),
                        fraction: 0.0,
                        message: e.to_string(),
                        elapsed_ms: start.elapsed().as_millis() as i64,
                    }));
                    tracing::error!("Pipeline {} failed: {}", pipeline_id, e);
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn validate(
        &self,
        request: Request<PipelineSpec>,
    ) -> Result<Response<ValidationResult>, Status> {
        let spec = request.into_inner();
        tracing::info!(
            node_count = spec.nodes.len(),
            "validate RPC called with {} nodes",
            spec.nodes.len()
        );
        tracing::info!("validate: name={}, nodes={}", spec.name, spec.nodes.len());

        let template = build_template(&spec);

        let mut issues = Vec::new();

        if spec.nodes.is_empty() {
            issues.push(ValidationIssue {
                severity: ProtoSeverity::Error as i32,
                param: "nodes".into(),
                message: "Pipeline must have at least one node".into(),
            });
        }

        let node_ids: Vec<&str> = spec.nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &spec.edges {
            if !node_ids.contains(&edge.from.as_str()) {
                issues.push(ValidationIssue {
                    severity: ProtoSeverity::Error as i32,
                    param: "edges".into(),
                    message: format!("Edge references unknown source node '{}'", edge.from),
                });
            }
            if !node_ids.contains(&edge.to.as_str()) {
                issues.push(ValidationIssue {
                    severity: ProtoSeverity::Error as i32,
                    param: "edges".into(),
                    message: format!("Edge references unknown target node '{}'", edge.to),
                });
            }
        }

        for node in &spec.nodes {
            if !self.state.registry.is_loaded(&node.plugin_id) {
                issues.push(ValidationIssue {
                    severity: ProtoSeverity::Error as i32,
                    param: format!("nodes.{}.plugin_id", node.id),
                    message: format!("Plugin '{}' is not registered", node.plugin_id),
                });
            }
        }

        match template.validate() {
            Ok(()) => {}
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: ProtoSeverity::Error as i32,
                    param: "template".into(),
                    message: e,
                });
            }
        }

        let valid = !issues
            .iter()
            .any(|i| i.severity >= ProtoSeverity::Error as i32);

        Ok(Response::new(ValidationResult { valid, issues }))
    }

    async fn get_node_schema(
        &self,
        request: Request<PluginId>,
    ) -> Result<Response<NodeSchema>, Status> {
        let pid = request.into_inner();
        tracing::info!(plugin_id = %pid.id, "get_node_schema RPC called for plugin {}", pid.id);
        tracing::info!("get_node_schema: plugin={}", pid.id);

        let plugin = self
            .state
            .registry
            .get(&pid.id)
            .ok_or_else(|| Status::not_found(format!("plugin not found: {}", pid.id)))?;

        let schema = plugin.parameter_schema();
        let gui = plugin.gui_schema();

        let parameter_schema = Some(schema_to_prost_struct(schema));
        let gui_schema = serde_json::to_value(gui).ok().map(|v| {
            let pv = json_to_prost_value(&v);
            match pv.kind {
                Some(prost_types::value::Kind::StructValue(s)) => s,
                _ => prost_types::Struct::default(),
            }
        });

        Ok(Response::new(NodeSchema {
            plugin_id: pid.id.clone(),
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            category: plugin.category().to_string(),
            description: plugin.description().to_string(),
            parameter_schema,
            gui_schema,
        }))
    }
}

/// Encode a PixelBuffer to a TIFF file and write it to the given path.
fn save_buffer_to_file(buffer: &photopipeline_core::PixelBuffer, output_path: &str) -> Result<(), String> {
    use image::{ImageBuffer, RgbImage, RgbaImage};
    use photopipeline_core::ChannelLayout;
    use std::path::Path;

    let parent = Path::new(output_path).parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create output directory {}: {}", parent.display(), e))?;

    let width = buffer.width;
    let height = buffer.height;

    match buffer.layout {
        ChannelLayout::RGB => {
            let mut img: RgbImage = ImageBuffer::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let r = buffer.data.data.get(idx * 3).copied().unwrap_or(0);
                    let g = buffer.data.data.get(idx * 3 + 1).copied().unwrap_or(0);
                    let b = buffer.data.data.get(idx * 3 + 2).copied().unwrap_or(0);
                    img.put_pixel(x, y, image::Rgb([r, g, b]));
                }
            }
            img.save(output_path)
                .map_err(|e| format!("Failed to save TIFF: {}", e))?;
        }
        ChannelLayout::RGBA => {
            let mut img: RgbaImage = ImageBuffer::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let r = buffer.data.data.get(idx * 4).copied().unwrap_or(0);
                    let g = buffer.data.data.get(idx * 4 + 1).copied().unwrap_or(0);
                    let b = buffer.data.data.get(idx * 4 + 2).copied().unwrap_or(0);
                    let a = buffer.data.data.get(idx * 4 + 3).copied().unwrap_or(255);
                    img.put_pixel(x, y, image::Rgba([r, g, b, a]));
                }
            }
            img.save(output_path)
                .map_err(|e| format!("Failed to save TIFF: {}", e))?;
        }
        ChannelLayout::Gray | ChannelLayout::GrayAlpha
        | ChannelLayout::Planar(_) | ChannelLayout::Custom(_) => {
            // Simplify: save as RGB
            let mut img: RgbImage = ImageBuffer::new(width, height);
            let channels = buffer.layout.channel_count() as usize;
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let v = buffer.data.data.get(idx * channels).copied().unwrap_or(0);
                    img.put_pixel(x, y, image::Rgb([v, v, v]));
                }
            }
            img.save(output_path)
                .map_err(|e| format!("Failed to save TIFF: {}", e))?;
        }
    }

    tracing::info!("Output saved to {}", output_path);
    Ok(())
}
