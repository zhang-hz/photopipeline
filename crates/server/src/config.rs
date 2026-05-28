use photopipeline_engine::graph::PipelineTemplate;

/// Load a pipeline config from a file path. Auto-detects JSON vs TOML by file content.
pub fn load_config(path: &str) -> Result<PipelineTemplate, String> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        tracing::error!(path = path, error = %e, "Failed to read config file");
        format!("Failed to read config file '{}': {}", path, e)
    })?;
    load_config_from_str(&content)
}

/// Load a pipeline config from a string. Auto-detects format:
/// - Starts with `{` → JSON
/// - Starts with `[` → try TOML first (TOML section headers like `[metadata]` are
///   more common than JSON arrays in pipeline configs), fall back to JSON
/// - Otherwise → TOML
pub fn load_config_from_str(content: &str) -> Result<PipelineTemplate, String> {
    let trimmed = content.trim();
    tracing::debug!(
        content_len = content.len(),
        "Loading pipeline config ({} bytes)",
        content.len()
    );

    if trimmed.starts_with('{') {
        return serde_json::from_str::<PipelineTemplate>(content).map_err(|e| {
            tracing::error!(error = %e, "Failed to parse JSON config");
            format!("Failed to parse JSON config: {}", e)
        });
    }

    if trimmed.starts_with('[') {
        // TOML section headers (e.g. `[metadata]`) start with `[` too.
        // Try TOML first; if it fails, fall back to JSON for JSON arrays.
        if let Ok(t) = toml::from_str::<PipelineTemplate>(content) {
            return Ok(t);
        }
        return serde_json::from_str::<PipelineTemplate>(content).map_err(|e| {
            tracing::error!(error = %e, "Failed to parse config as TOML and JSON");
            format!("Failed to parse config: {}", e)
        });
    }

    toml::from_str::<PipelineTemplate>(content).map_err(|e| {
        tracing::error!(error = %e, "Failed to parse TOML config");
        format!("Failed to parse TOML config: {}", e)
    })
}

/// Legacy alias kept for compatibility. Prefer [`load_config_from_str`].
pub fn load_template(content: &str) -> Result<PipelineTemplate, String> {
    load_config_from_str(content)
}
