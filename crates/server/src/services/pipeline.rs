use std::sync::Arc;
use parking_lot::RwLock;
use tonic::{Request, Response, Status};
use futures::stream;

use crate::pb::pipeline::{
    pipeline_service_server::PipelineService,
    ExecuteProgress, ExecuteRequest, NodeSchema, PipelineId,
    PipelineSpec, PluginId, ValidationIssue,
    ValidationResult,
};
use crate::SharedState;

pub struct PipelineServiceImpl {
    state: Arc<RwLock<SharedState>>,
}

impl PipelineServiceImpl {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl PipelineService for PipelineServiceImpl {
    async fn create_pipeline(
        &self,
        request: Request<PipelineSpec>,
    ) -> Result<Response<PipelineId>, Status> {
        let _spec = request.into_inner();
        let id = uuid::Uuid::new_v4().to_string();
        Ok(Response::new(PipelineId { id }))
    }

    type ExecuteStream = stream::Iter<std::vec::IntoIter<Result<ExecuteProgress, Status>>>;

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let req = request.into_inner();
        let pipeline_id = req.pipeline_id.clone();
        let image_path = req.image_path.clone();

        let stages = vec![
            ExecuteProgress {
                stage: 0,
                node_id: String::new(),
                node_label: format!("Loading {}", image_path),
                fraction: 0.0,
                message: "Starting pipeline...".into(),
                elapsed_ms: 0,
            },
            ExecuteProgress {
                stage: 1,
                node_id: "decode".into(),
                node_label: "Decoding".into(),
                fraction: 0.25,
                message: "Decoding image...".into(),
                elapsed_ms: 100,
            },
            ExecuteProgress {
                stage: 2,
                node_id: "process".into(),
                node_label: "Processing".into(),
                fraction: 0.5,
                message: "Applying pipeline nodes...".into(),
                elapsed_ms: 250,
            },
            ExecuteProgress {
                stage: 2,
                node_id: "process".into(),
                node_label: "Processing".into(),
                fraction: 0.75,
                message: "Finishing processing...".into(),
                elapsed_ms: 500,
            },
            ExecuteProgress {
                stage: 3,
                node_id: "encode".into(),
                node_label: "Encoding".into(),
                fraction: 0.9,
                message: "Encoding output...".into(),
                elapsed_ms: 750,
            },
            ExecuteProgress {
                stage: 4,
                node_id: String::new(),
                node_label: "Complete".into(),
                fraction: 1.0,
                message: format!("Pipeline {} done", pipeline_id),
                elapsed_ms: 1000,
            },
        ];

        let iter = stages.into_iter().map(Ok).collect::<Vec<_>>().into_iter();
        Ok(Response::new(stream::iter(iter)))
    }

    async fn validate(
        &self,
        request: Request<PipelineSpec>,
    ) -> Result<Response<ValidationResult>, Status> {
        let spec = request.into_inner();

        let mut issues = Vec::new();
        if spec.nodes.is_empty() {
            issues.push(ValidationIssue {
                severity: 2,
                param: "nodes".into(),
                message: "Pipeline must have at least one node".into(),
            });
        }

        let node_ids: Vec<&str> = spec.nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &spec.edges {
            if !node_ids.contains(&edge.from.as_str()) {
                issues.push(ValidationIssue {
                    severity: 2,
                    param: "edges".into(),
                    message: format!("Edge references unknown source node '{}'", edge.from),
                });
            }
            if !node_ids.contains(&edge.to.as_str()) {
                issues.push(ValidationIssue {
                    severity: 2,
                    param: "edges".into(),
                    message: format!("Edge references unknown target node '{}'", edge.to),
                });
            }
        }

        Ok(Response::new(ValidationResult {
            valid: issues.iter().all(|i| i.severity < 2),
            issues,
        }))
    }

    async fn get_node_schema(
        &self,
        request: Request<PluginId>,
    ) -> Result<Response<NodeSchema>, Status> {
        let pid = request.into_inner();
        Ok(Response::new(NodeSchema {
            plugin_id: pid.id.clone(),
            name: format!("Plugin {}", pid.id),
            version: "1.0.0".into(),
            category: "transform".into(),
            description: format!("Schema for plugin {}", pid.id),
            parameter_schema: None,
            gui_schema: None,
        }))
    }
}
