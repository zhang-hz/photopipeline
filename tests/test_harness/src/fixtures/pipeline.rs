use photopipeline_engine::*;

pub fn minimal_pipeline() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("Minimal Pipeline".into()),
            version: Some("1.0".into()),
            description: Some("Minimal source to output pipeline".into()),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![TemplateEdge {
            from: "source".into(),
            to: "output".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

pub fn metadata_pipeline() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("Metadata Pipeline".into()),
            version: Some("1.0".into()),
            description: Some("Pipeline that reads/writes EXIF/GPS/time metadata".into()),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "exif_rw".into(),
                plugin: "photopipeline.plugins.exif_rw".into(),
                label: Some("EXIF Read/Write".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "gps".into(),
                plugin: "photopipeline.plugins.gps_set".into(),
                label: Some("GPS Tagger".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "time".into(),
                plugin: "photopipeline.plugins.time_shift".into(),
                label: Some("Time Adjuster".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "source".into(),
                to: "exif_rw".into(),
            },
            TemplateEdge {
                from: "exif_rw".into(),
                to: "gps".into(),
            },
            TemplateEdge {
                from: "gps".into(),
                to: "time".into(),
            },
            TemplateEdge {
                from: "time".into(),
                to: "output".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

pub fn pixel_processing_pipeline() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("Pixel Processing Pipeline".into()),
            version: Some("1.0".into()),
            description: Some("Pipeline with colorspace conversion and transforms".into()),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "colorspace".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("Colorspace".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "transform".into(),
                plugin: "photopipeline.plugins.transform".into(),
                label: Some("Resize".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "source".into(),
                to: "colorspace".into(),
            },
            TemplateEdge {
                from: "colorspace".into(),
                to: "transform".into(),
            },
            TemplateEdge {
                from: "transform".into(),
                to: "output".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

pub fn hdr_pipeline() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("HDR Pipeline".into()),
            version: Some("1.0".into()),
            description: Some(
                "HDR processing with lens correction, denoise, and HEIF output".into(),
            ),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "colorspace".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("HDR Colorspace".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "lens".into(),
                plugin: "photopipeline.plugins.lens_correct".into(),
                label: Some("Lens Correction".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "denoise".into(),
                plugin: "photopipeline.plugins.ai_denoise".into(),
                label: Some("Denoise".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.heif_encoder".into(),
                label: Some("HEIF Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "source".into(),
                to: "colorspace".into(),
            },
            TemplateEdge {
                from: "colorspace".into(),
                to: "lens".into(),
            },
            TemplateEdge {
                from: "lens".into(),
                to: "denoise".into(),
            },
            TemplateEdge {
                from: "denoise".into(),
                to: "output".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

pub fn pipeline_with_overrides() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("Pipeline with Image Overrides".into()),
            version: Some("1.0".into()),
            description: Some("Pipeline with per-image parameter overrides".into()),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "denoise".into(),
                plugin: "photopipeline.plugins.ai_denoise".into(),
                label: Some("Denoise".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "source".into(),
                to: "denoise".into(),
            },
            TemplateEdge {
                from: "denoise".into(),
                to: "output".into(),
            },
        ],
        overrides: vec![ImageOverride {
            image: "img_001.jpg".into(),
            params: {
                let mut map = std::collections::HashMap::new();
                let mut ps = photopipeline_plugin::ParameterSet::new();
                ps.insert("strength".into(), serde_json::json!(0.8));
                map.insert("denoise".into(), ps);
                map
            },
        }],
        groups: vec![],
        batch: None,
    }
}

pub fn pipeline_with_groups() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("Pipeline with Groups".into()),
            version: Some("1.0".into()),
            description: Some("Pipeline with group-based param overrides".into()),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "denoise".into(),
                plugin: "photopipeline.plugins.ai_denoise".into(),
                label: Some("Denoise".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "source".into(),
                to: "denoise".into(),
            },
            TemplateEdge {
                from: "denoise".into(),
                to: "output".into(),
            },
        ],
        overrides: vec![],
        groups: vec![ParamGroup {
            name: "High ISO".into(),
            condition: "exif.iso > 800".into(),
            params: {
                let mut map = std::collections::HashMap::new();
                let mut ps = photopipeline_plugin::ParameterSet::new();
                ps.insert("strength".into(), serde_json::json!(1.0));
                map.insert("denoise".into(), ps);
                map
            },
        }],
        batch: None,
    }
}

pub fn pipeline_with_batch() -> PipelineTemplate {
    PipelineTemplate {
        metadata: TemplateMetadata {
            name: Some("Pipeline with Batch Config".into()),
            version: Some("1.0".into()),
            description: Some("Pipeline with batch processing config".into()),
        },
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: Some("Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![TemplateEdge {
            from: "source".into(),
            to: "output".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: Some(BatchConfig {
            parallel: 4,
            output_pattern: Some("{name}_out".into()),
            on_conflict: Some("skip".into()),
            resume: true,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_pipeline_validates() {
        let t = minimal_pipeline();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn metadata_pipeline_validates() {
        let t = metadata_pipeline();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn pixel_processing_pipeline_validates() {
        let t = pixel_processing_pipeline();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn hdr_pipeline_validates() {
        let t = hdr_pipeline();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn pipeline_with_overrides_validates() {
        let t = pipeline_with_overrides();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn pipeline_with_groups_validates() {
        let t = pipeline_with_groups();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn pipeline_with_batch_validates() {
        let t = pipeline_with_batch();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn pipeline_with_batch_has_config() {
        let t = pipeline_with_batch();
        let batch = t.batch.unwrap();
        assert_eq!(batch.parallel, 4);
        assert!(batch.resume);
    }

    #[test]
    fn pipeline_with_overrides_has_image() {
        let t = pipeline_with_overrides();
        assert_eq!(t.overrides.len(), 1);
        assert_eq!(t.overrides[0].image, "img_001.jpg");
    }

    #[test]
    fn pipeline_with_groups_has_group() {
        let t = pipeline_with_groups();
        assert_eq!(t.groups.len(), 1);
        assert_eq!(t.groups[0].name, "High ISO");
    }

    #[test]
    fn all_templates_into_graph() {
        let templates: Vec<(&str, PipelineTemplate)> = vec![
            ("minimal", minimal_pipeline()),
            ("metadata", metadata_pipeline()),
            ("pixel", pixel_processing_pipeline()),
            ("hdr", hdr_pipeline()),
            ("overrides", pipeline_with_overrides()),
            ("groups", pipeline_with_groups()),
            ("batch", pipeline_with_batch()),
        ];
        for (name, t) in templates {
            let graph = t.into_graph();
            assert!(graph.nodes.len() > 0, "{} pipeline has no nodes", name);
        }
    }
}
