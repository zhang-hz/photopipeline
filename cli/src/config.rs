use photopipeline_engine::graph::PipelineTemplate;

pub fn load_template(content: &str) -> Result<PipelineTemplate, String> {
    let toml_val: toml::Value = toml::from_str(content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;
    let json_val = serde_json::to_value(&toml_val)
        .map_err(|e| format!("Failed to convert TOML to JSON: {}", e))?;
    serde_json::from_value(json_val)
        .map_err(|e| format!("Failed to parse pipeline config: {}", e))
}
