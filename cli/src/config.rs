use photopipeline_engine::graph::PipelineTemplate;

pub fn load_template(content: &str) -> Result<PipelineTemplate, String> {
    tracing::debug!(
        content_len = content.len(),
        "Loading pipeline template from config ({} bytes)",
        content.len()
    );
    toml::from_str::<PipelineTemplate>(content).map_err(|e| {
        tracing::error!(error = %e, "Failed to parse pipeline config");
        format!("Failed to parse pipeline config: {}", e)
    })
}
