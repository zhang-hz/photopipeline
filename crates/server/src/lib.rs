pub mod commands;
pub mod config;
pub mod services;

use parking_lot::RwLock;
use photopipeline_engine::PipelineGraph;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod pb {
    #[allow(ambiguous_associated_items)]
    pub mod pipeline {
        tonic::include_proto!("photopipeline.pipeline");
    }
    #[allow(ambiguous_associated_items)]
    pub mod image {
        tonic::include_proto!("photopipeline.image");
    }
    #[allow(ambiguous_associated_items)]
    pub mod batch {
        tonic::include_proto!("photopipeline.batch");
    }
    #[allow(ambiguous_associated_items)]
    pub mod plugin {
        tonic::include_proto!("photopipeline.plugin");
    }
    #[allow(ambiguous_associated_items)]
    pub mod execution {
        tonic::include_proto!("photopipeline.execution");
    }
}

pub struct SharedState {
    pub registry: Arc<photopipeline_plugin::Registry>,
    pub resolver: Arc<photopipeline_engine::ParameterResolver>,
    pub graphs: RwLock<HashMap<Uuid, PipelineGraph>>,
    pub batch_jobs: RwLock<HashMap<Uuid, BatchJobState>>,
}

#[derive(Debug, Clone)]
pub struct BatchJobState {
    pub spec: pb::batch::BatchSpec,
    pub total_files: i32,
    pub completed_files: i32,
    pub failed_files: i32,
    pub current_file: String,
    pub status: i32,
    #[allow(clippy::type_complexity)]
    pub cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl SharedState {
    pub fn new(
        registry: Arc<photopipeline_plugin::Registry>,
        resolver: Arc<photopipeline_engine::ParameterResolver>,
    ) -> Self {
        tracing::debug!("Creating new shared server state");
        Self {
            registry,
            resolver,
            graphs: RwLock::new(HashMap::new()),
            batch_jobs: RwLock::new(HashMap::new()),
        }
    }
}

pub fn prost_struct_to_json(struct_val: &prost_types::Struct) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in &struct_val.fields {
        map.insert(k.clone(), prost_value_to_json(v));
    }
    serde_json::Value::Object(map)
}

pub fn prost_value_to_json(val: &prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;
    match &val.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::NumberValue(n)) => {
            // Preserve integer types: protobuf NumberValue is always f64,
            // but if the value is a lossless integer, store it as i64 so
            // plugins using get_i64() can read it correctly.
            let n_int = *n as i64;
            if n_int as f64 == *n {
                serde_json::json!(n_int)
            } else {
                serde_json::json!(n)
            }
        }
        Some(Kind::StringValue(s)) => serde_json::Value::String(s.clone()),
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(Kind::StructValue(s)) => prost_struct_to_json(s),
        Some(Kind::ListValue(list)) => {
            serde_json::Value::Array(list.values.iter().map(prost_value_to_json).collect())
        }
        None => serde_json::Value::Null,
    }
}

pub fn prost_struct_to_params(
    struct_val: &prost_types::Struct,
) -> HashMap<String, serde_json::Value> {
    struct_val
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect()
}

pub fn json_to_prost_value(val: &serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match val {
        serde_json::Value::Null => Kind::NullValue(prost_types::NullValue::NullValue as i32),
        serde_json::Value::Bool(b) => Kind::BoolValue(*b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s.clone()),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.iter().map(json_to_prost_value).collect(),
        }),
        serde_json::Value::Object(obj) => Kind::StructValue(prost_types::Struct {
            fields: obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect(),
        }),
    };
    prost_types::Value { kind: Some(kind) }
}

pub fn schema_to_prost_struct(
    schema: &photopipeline_plugin::ParameterSchema,
) -> prost_types::Struct {
    let json =
        serde_json::to_value(schema).unwrap_or(serde_json::Value::Object(Default::default()));
    match json_to_prost_value(&json).kind {
        Some(Kind::StructValue(s)) => s,
        _ => prost_types::Struct::default(),
    }
}

pub fn detect_format_from_ext(path: &str) -> photopipeline_core::ImageFormat {
    use photopipeline_core::ImageFormat;
    let lower = path.to_lowercase();
    if lower.ends_with(".arw")
        || lower.ends_with(".cr2")
        || lower.ends_with(".nef")
        || lower.ends_with(".dng")
    {
        ImageFormat::RAW
    } else if lower.ends_with(".heif") || lower.ends_with(".heic") {
        ImageFormat::HEIF
    } else if lower.ends_with(".avif") {
        ImageFormat::AVIF
    } else if lower.ends_with(".jxl") {
        ImageFormat::JXL
    } else if lower.ends_with(".png") {
        ImageFormat::PNG
    } else if lower.ends_with(".tiff") || lower.ends_with(".tif") {
        ImageFormat::TIFF
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        ImageFormat::JPEG
    } else if lower.ends_with(".webp") {
        ImageFormat::WEBP
    } else if lower.ends_with(".exr") {
        ImageFormat::OpenEXR
    } else if lower.ends_with(".bmp") {
        ImageFormat::BMP
    } else {
        ImageFormat::Unknown("unknown".to_string())
    }
}

use prost_types::value::Kind;
