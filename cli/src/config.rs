use photopipeline_engine::graph::PipelineTemplate;

pub fn load_template(content: &str) -> Result<PipelineTemplate, String> {
    toml::from_str::<PipelineTemplate>(content)
        .map_err(|e| format!("Failed to parse pipeline config: {}", e))
}
