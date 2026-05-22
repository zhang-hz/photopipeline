use thiserror::Error;

use crate::types::{GpuBackend, PluginId, PluginVersion, VersionRequirement};

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin '{0}' not found")]
    NotFound(PluginId),

    #[error("plugin '{}' already loaded", .plugin)]
    AlreadyLoaded { plugin: PluginId },

    #[error("plugin '{plugin}' load failed: {reason}")]
    LoadFailed { plugin: PluginId, reason: String },

    #[error("plugin '{plugin}' version {actual} doesn't satisfy {required}")]
    VersionMismatch {
        plugin: PluginId,
        actual: PluginVersion,
        required: VersionRequirement,
    },

    #[error("plugin '{plugin}' parameter '{field}' invalid: {message}")]
    InvalidParameter {
        plugin: PluginId,
        field: String,
        message: String,
    },

    #[error("plugin '{plugin}' missing tool: {tool} (need {required})")]
    MissingTool {
        plugin: PluginId,
        tool: String,
        required: String,
    },

    #[error("plugin '{plugin}' requires GPU backend '{backend}', none available")]
    GpuNotAvailable {
        plugin: PluginId,
        backend: GpuBackend,
    },

    #[error("plugin '{plugin}' out of GPU memory: need {needed}MB, have {available}MB")]
    GpuOutOfMemory {
        plugin: PluginId,
        needed: u64,
        available: u64,
    },

    #[error("plugin '{plugin}' expression error in '{field}': {error}")]
    ExpressionError {
        plugin: PluginId,
        field: String,
        error: String,
    },

    #[error("plugin '{plugin}' processing timeout ({elapsed}s > {timeout}s)")]
    Timeout {
        plugin: PluginId,
        elapsed: f64,
        timeout: f64,
    },

    #[error("plugin '{plugin}' internal error: {message}")]
    Internal { plugin: PluginId, message: String },

    #[error("plugin '{plugin}' canceled by user")]
    Canceled { plugin: PluginId },

    #[error("plugin '{plugin}' IO error: {error}")]
    Io {
        plugin: PluginId,
        #[source]
        error: std::io::Error,
    },

    #[error("pipeline validation failed: {0}")]
    ValidationFailed(String),

    #[error("pipeline execution failed at node '{node}': {message}")]
    NodeExecutionFailed { node: String, message: String },

    #[error("circular dependency detected in pipeline")]
    CircularDependency,

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("encoding failed: {0}")]
    EncodingFailed(String),

    #[error("decoding failed: {0}")]
    DecodingFailed(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub enum ValidationIssue {
    Error {
        param: String,
        message: String,
    },
    Warning {
        param: String,
        message: String,
    },
    Info {
        param: String,
        message: String,
    },
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error { param, message } => write!(f, "ERROR({}): {}", param, message),
            Self::Warning { param, message } => write!(f, "WARNING({}): {}", param, message),
            Self::Info { param, message } => write!(f, "INFO({}): {}", param, message),
        }
    }
}
