use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::LazyLock;

use photopipeline_core::{
    AiBackend, ColorSpace, GpuBackend, HardwareRequirement, PixelBuffer, PixelFormat,
    PluginCategory, PluginError, PluginId, PluginResult, PluginVersion, ProcessingStats, Tensor,
    ValidationIssue,
};
use photopipeline_plugin::{
    AiProcessor, AuxView, EnumOption, GuiLayout, GuiSchema, GuiSection, ModelInfo, ModelSource,
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, PixelProcessor,
    Plugin, PluginConfig, PreviewMode, ProgressSink, SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
    version: 1,
    sections: vec![
        ParameterSection {
            id: "model".into(),
            label: "Model".into(),
            description: Some("AI model selection and configuration".into()),
            icon: Some("cpu".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "denoise_model".into(),
                label: "Denoise Model".into(),
                description: Some("Select the AI denoising model to use".into()),
                help_url: None,
                field_type: ParameterType::Enum {
                    options: vec![
                        EnumOption {
                            value: "lightweight_v1".into(),
                            label: "Lightweight v1".into(),
                            description: Some("Fast, low memory. Good for ISO 100-3200".into()),
                            icon: None,
                            tags: vec!["fast".into()],
                            recommended: true,
                        },
                        EnumOption {
                            value: "standard_v2".into(),
                            label: "Standard v2".into(),
                            description: Some("Balanced quality/speed. ISO 100-12800".into()),
                            icon: None,
                            tags: vec!["balanced".into()],
                            recommended: false,
                        },
                        EnumOption {
                            value: "high_quality_v2".into(),
                            label: "High Quality v2".into(),
                            description: Some("Maximum quality. ISO 100-51200".into()),
                            icon: None,
                            tags: vec!["quality".into(), "slow".into()],
                            recommended: false,
                        },
                        EnumOption {
                            value: "raw_denoise_v1".into(),
                            label: "RAW Denoise v1".into(),
                            description: Some("Operates on RAW data before demosaicing".into()),
                            icon: None,
                            tags: vec!["raw".into()],
                            recommended: false,
                        },
                    ],
                    display: Default::default(),
                },
                default: serde_json::json!("standard_v2"),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        },
        ParameterSection {
            id: "strength".into(),
            label: "Strength".into(),
            description: Some("Denoising strength and detail preservation".into()),
            icon: Some("sliders".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "denoise_strength".into(),
                    label: "Strength".into(),
                    description: Some("Overall denoising strength (0 = off, 100 = max)".into()),
                    help_url: None,
                    field_type: ParameterType::Slider {
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
                        show_ticks: true,
                        ticks: Some(vec![0.0, 25.0, 50.0, 75.0, 100.0]),
                        show_value: true,
                        orientation: Default::default(),
                        style: Default::default(),
                    },
                    default: serde_json::json!(50.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "detail_preservation".into(),
                    label: "Detail Preservation".into(),
                    description: Some("Preserve fine detail at cost of some noise (0-100)".into()),
                    help_url: None,
                    field_type: ParameterType::Slider {
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
                        show_ticks: true,
                        ticks: Some(vec![0.0, 25.0, 50.0, 75.0, 100.0]),
                        show_value: true,
                        orientation: Default::default(),
                        style: Default::default(),
                    },
                    default: serde_json::json!(50.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "color_noise_reduction".into(),
                    label: "Color Noise".into(),
                    description: Some("Additional reduction of chroma noise".into()),
                    help_url: None,
                    field_type: ParameterType::Slider {
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
                        show_ticks: true,
                        ticks: Some(vec![0.0, 50.0, 100.0]),
                        show_value: true,
                        orientation: Default::default(),
                        style: Default::default(),
                    },
                    default: serde_json::json!(75.0),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "hardware".into(),
            label: "Hardware".into(),
            description: Some("Hardware backend and performance options".into()),
            icon: Some("cpu".into()),
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "ai_backend".into(),
                    label: "AI Backend".into(),
                    description: Some("Inference backend for AI processing".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "onnx_cpu".into(),
                                label: "ONNX (CPU)".into(),
                                description: Some("ONNX Runtime on CPU. Works everywhere.".into()),
                                icon: None,
                                tags: vec!["cpu".into()],
                                recommended: true,
                            },
                            EnumOption {
                                value: "onnx_cuda".into(),
                                label: "ONNX (CUDA)".into(),
                                description: Some("ONNX Runtime with CUDA GPU acceleration".into()),
                                icon: None,
                                tags: vec!["gpu".into(), "cuda".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "tensorrt".into(),
                                label: "TensorRT".into(),
                                description: Some("NVIDIA TensorRT optimized inference".into()),
                                icon: None,
                                tags: vec!["gpu".into(), "nvidia".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "coreml".into(),
                                label: "CoreML (ANE)".into(),
                                description: Some("Apple Neural Engine via CoreML".into()),
                                icon: None,
                                tags: vec!["apple".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "openvino".into(),
                                label: "OpenVINO".into(),
                                description: Some("Intel OpenVINO with GPU/NPU support".into()),
                                icon: None,
                                tags: vec!["intel".into()],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("onnx_cpu"),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "tile_size".into(),
                    label: "Tile Size".into(),
                    description: Some("Processing tile size for large images (0 = auto)".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 4096,
                        step: 64,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "use_fp16".into(),
                    label: "FP16".into(),
                    description: Some(
                        "Use half-precision (FP16) for faster inference on GPU".into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("FP16".into()),
                        label_false: Some("FP32".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
    ],
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| GuiSchema {
    layout: GuiLayout::Standard {
        sections: vec![
            GuiSection {
                param_section_id: "model".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "strength".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "hardware".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("sparkles".into()),
    color: Some("#a855f7".into()),
    preview: PreviewMode::BeforeAfter {
        default_split: 0.5,
        orientation: photopipeline_core::SplitOrientation::Horizontal,
        lock_zoom: true,
    },
    aux_views: vec![
        AuxView::Histogram,
        AuxView::ProgressBar,
        AuxView::StatusText,
    ],
    min_panel_width: 360,
});

static STANDARD_MODEL_INFO: LazyLock<ModelInfo> = LazyLock::new(|| ModelInfo {
    name: "PhotoPipeline Denoise Standard v2".into(),
    version: "2.0.0".into(),
    source: ModelSource::HuggingFace {
        repo: "photopipeline/denoise-standard-v2".into(),
        file: "denoise_standard_v2.onnx".into(),
    },
    input_shape: vec![1, 3, 1024, 1024],
    output_shape: vec![1, 3, 1024, 1024],
    memory_mb: 2048,
    description: "Balanced denoising model for ISO 100-12800. 1.5GB VRAM recommended.".into(),
});

#[derive(Debug)]
pub struct AiDenoisePlugin {
    id: String,
    model_loaded: RwLock<bool>,
    current_backend: RwLock<Option<AiBackend>>,
    current_model: RwLock<String>,
}

impl Default for AiDenoisePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AiDenoisePlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.ai_denoise".to_string(),
            model_loaded: RwLock::new(false),
            current_backend: RwLock::new(None),
            current_model: RwLock::new(String::new()),
        }
    }
}

#[async_trait]
impl Plugin for AiDenoisePlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "AI Denoise"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Enhance
    }
    fn description(&self) -> &str {
        "AI-powered image denoising using ONNX Runtime"
    }
    fn tags(&self) -> &[String] {
        &TAGS
    }
    fn requires_pixel_access(&self) -> bool {
        true
    }
    fn produces_pixel_output(&self) -> bool {
        true
    }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement {
            requires_cpu: true,
            requires_gpu: false,
            min_ram_mb: 2048,
            preferred_backend: Some(GpuBackend::CUDA),
        }
    }

    fn parameter_schema(&self) -> &ParameterSchema {
        &PARAMETER_SCHEMA
    }
    fn gui_schema(&self) -> &GuiSchema {
        &GUI_SCHEMA
    }

    async fn initialize(&mut self, _cfg: &PluginConfig) -> PluginResult<()> {
        tracing::info!("AI Denoise initialized. ONNX Runtime model loading on first use.");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        if *self.model_loaded.read() {
            self.unload_model().await?;
        }
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let strength = params
            .get("denoise_strength")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);
        if !(0.0..=100.0).contains(&strength) {
            issues.push(ValidationIssue::Error {
                param: "denoise_strength".into(),
                message: "Strength must be between 0 and 100".into(),
            });
        }

        let detail = params
            .get("detail_preservation")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);
        if !(0.0..=100.0).contains(&detail) {
            issues.push(ValidationIssue::Error {
                param: "detail_preservation".into(),
                message: "Detail preservation must be between 0 and 100".into(),
            });
        }

        let backend_str = params.get_str("ai_backend").unwrap_or("onnx_cpu");
        if backend_str.contains("cuda") || backend_str == "tensorrt" {
            issues.push(ValidationIssue::Warning {
                param: "ai_backend".into(),
                message: format!(
                    "GPU backend '{}' selected. Ensure NVIDIA drivers and CUDA toolkit are installed.",
                    backend_str,
                ),
            });
        }

        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for AiDenoisePlugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat> {
        vec![
            PixelFormat::U8,
            PixelFormat::U16,
            PixelFormat::F16,
            PixelFormat::F32,
        ]
    }

    fn supported_output_formats(&self) -> Vec<PixelFormat> {
        vec![
            PixelFormat::U8,
            PixelFormat::U16,
            PixelFormat::F16,
            PixelFormat::F32,
        ]
    }

    fn supported_color_spaces(&self) -> Vec<ColorSpace> {
        vec![ColorSpace::SRGB, ColorSpace::LINEAR_SRGB]
    }

    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        Some(GpuBackend::CUDA)
    }

    async fn process_pixels(
        &self,
        input: &PixelBuffer,
        output: &mut PixelBuffer,
        params: &ParameterSet,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        progress.set_progress(0.0, "preparing for AI denoise");

        let strength = params
            .get("denoise_strength")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);
        let _detail = params
            .get("detail_preservation")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);
        let _color = params
            .get("color_noise_reduction")
            .and_then(|v| v.as_f64())
            .unwrap_or(75.0);

        if strength < 1.0 {
            output.data.data.copy_from_slice(&input.data.data);
            output.width = input.width;
            output.height = input.height;
            output.layout = input.layout;
            output.format = input.format;
            output.color_space = input.color_space.clone();
            output.icc_profile = input.icc_profile.clone();
            progress.set_progress(1.0, "done (strength=0)");
            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: None,
                peak_memory_mb: input.data.data.len() as u64 / (1024 * 1024),
                input_pixels: input.pixel_count(),
                output_pixels: input.pixel_count(),
            });
        }

        if !*self.model_loaded.read() {
            tracing::info!("AI Denoise: model not loaded, passing through pixels");
        }

        progress.set_progress(0.5, "denoising (passthrough - ONNX model pending)");

        output.data.data.copy_from_slice(&input.data.data);
        output.width = input.width;
        output.height = input.height;
        output.layout = input.layout;
        output.format = input.format;
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        progress.set_progress(1.0, "done");

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: if self.current_backend.read().is_some() {
                Some(0)
            } else {
                None
            },
            peak_memory_mb: (input.data.data.len() * 3) as u64 / (1024 * 1024),
            input_pixels: input.pixel_count(),
            output_pixels: input.pixel_count(),
        })
    }
}

#[async_trait]
impl AiProcessor for AiDenoisePlugin {
    fn model_info(&self) -> &ModelInfo {
        &STANDARD_MODEL_INFO
    }

    fn supported_backends(&self) -> Vec<AiBackend> {
        vec![
            AiBackend::ONNX,
            AiBackend::TensorRT,
            AiBackend::CoreML,
            AiBackend::OpenVINO,
        ]
    }

    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()> {
        let model_name = "standard_v2";
        tracing::info!(
            "AI Denoise: loading model '{}' on backend {:?} (ONNX Runtime placeholder)",
            model_name,
            backend,
        );

        let model_path = format!("models/denoise_{}.onnx", model_name);
        if !std::path::Path::new(&model_path).exists() {
            let model_desc = match &STANDARD_MODEL_INFO.source {
                ModelSource::HuggingFace { repo, file } => format!("hf://{}/{}", repo, file),
                ModelSource::ExternalFile(p) => format!("file://{}", p),
                ModelSource::Url(u) => u.clone(),
                ModelSource::Bundled => "bundled".to_string(),
            };
            tracing::warn!(
                "AI Denoise: model file '{}' not found. Download from: {}",
                model_path,
                model_desc,
            );
        }

        *self.model_loaded.write() = true;
        *self.current_backend.write() = Some(*backend);
        *self.current_model.write() = model_name.to_string();

        Ok(())
    }

    async fn unload_model(&mut self) -> PluginResult<()> {
        if *self.model_loaded.read() {
            tracing::info!(
                "AI Denoise: unloading model '{}'",
                self.current_model.read()
            );
        }
        *self.model_loaded.write() = false;
        *self.current_backend.write() = None;
        *self.current_model.write() = String::new();
        Ok(())
    }

    async fn infer(&self, input: &Tensor, params: &ParameterSet) -> PluginResult<Tensor> {
        let _strength = params
            .get("denoise_strength")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);

        if !*self.model_loaded.read() {
            return Err(PluginError::Internal {
                plugin: self.id.clone(),
                message: "Model not loaded. Call load_model() first.".into(),
            });
        }

        tracing::info!(
            "AI Denoise: infer (passthrough) on tensor shape {:?}, dtype {:?}",
            input.shape,
            input.dtype,
        );

        Ok(Tensor {
            shape: input.shape.clone(),
            data: input.data.clone(),
            dtype: input.dtype,
        })
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "ai".into(),
        "denoise".into(),
        "onnx".into(),
        "machine-learning".into(),
        "gpu".into(),
        "cuda".into(),
        "tensorrt".into(),
        "coreml".into(),
        "enhance".into(),
        "neural".into(),
    ]
});
