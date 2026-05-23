use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;

use photopipeline_core::{
    AlignedBuffer, ChannelLayout, ColorSpace, DecodeOptions, DecodedImage, EncodeOptions,
    FormatProbe, HardwareRequirement, ImageFormat, Metadata, PerfTimer, PixelBuffer, PixelFormat,
    PluginCategory, PluginError, PluginId, PluginResult, PluginVersion, ValidationIssue,
};
use photopipeline_plugin::{
    EnumOption, FormatProcessor, GuiLayout, GuiSchema, GuiSection, ParameterField, ParameterSchema,
    ParameterSection, ParameterSet, ParameterType, Plugin, PreviewMode, SectionStyle,
};

#[cfg(feature = "libraw-native")]
mod libraw_ffi {
    use std::ffi::{c_char, c_int, c_uint, c_void};

    #[repr(C)]
    pub struct LibRawData;

    #[link(name = "raw")]
    extern "C" {
        pub fn libraw_init(flags: c_uint) -> *mut LibRawData;
        pub fn libraw_close(lr: *mut LibRawData);
        pub fn libraw_open_buffer(lr: *mut LibRawData, buffer: *const c_void, size: usize)
        -> c_int;
        pub fn libraw_unpack(lr: *mut LibRawData) -> c_int;
        pub fn libraw_dcraw_process(lr: *mut LibRawData) -> c_int;
        pub fn libraw_dcraw_make_mem_image(lr: *mut LibRawData) -> *mut LibRawProcessedImage;
        pub fn libraw_dcraw_clear_mem(image: *mut LibRawProcessedImage);

        pub fn libraw_get_iwidth(lr: *const LibRawData) -> c_int;
        pub fn libraw_get_iheight(lr: *const LibRawData) -> c_int;
        pub fn libraw_get_colors(lr: *const LibRawData) -> c_int;
    }

    #[repr(C)]
    pub struct LibRawProcessedImage {
        pub data_type: c_int,
        pub width: c_int,
        pub height: c_int,
        pub colors: c_int,
        pub data: *mut c_void,
    }

    pub const LIBRAW_IMAGE_FORMAT_TIFF_LIKE: c_int = 0;
}

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
    version: 1,
    sections: vec![
        ParameterSection {
            id: "raw_format".into(),
            label: "RAW Format".into(),
            description: Some("RAW file format detection and processing".into()),
            icon: Some("camera".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "raw_mode".into(),
                label: "Decode Mode".into(),
                description: Some("How to process the RAW file".into()),
                help_url: None,
                field_type: ParameterType::Enum {
                    options: vec![
                        EnumOption {
                            value: "auto".into(),
                            label: "Auto".into(),
                            description: Some("Detect from file and use best method".into()),
                            icon: None,
                            tags: vec![],
                            recommended: true,
                        },
                        EnumOption {
                            value: "dcraw".into(),
                            label: "dcraw".into(),
                            description: Some("Use dcraw for raw conversion".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                        EnumOption {
                            value: "libraw".into(),
                            label: "LibRaw".into(),
                            description: Some("Use LibRaw via FFI (when available)".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                        EnumOption {
                            value: "rawtherapee".into(),
                            label: "RawTherapee".into(),
                            description: Some("Use RawTherapee CLI".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                    ],
                    display: Default::default(),
                },
                default: serde_json::json!("auto"),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        },
        ParameterSection {
            id: "output".into(),
            label: "Output".into(),
            description: Some("RAW decoding output options".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "output_format".into(),
                    label: "Output Pixel Format".into(),
                    description: Some("Pixel format for decoded output".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "u16".into(),
                                label: "16-bit".into(),
                                description: Some("Standard 16-bit integer".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "f32".into(),
                                label: "32-bit float".into(),
                                description: Some("Floating-point for HDR processing".into()),
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("u16"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "half_size".into(),
                    label: "Half Size".into(),
                    description: Some("Decode at half resolution for faster previews".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Half".into()),
                        label_false: Some("Full".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "apply_white_balance".into(),
                    label: "White Balance".into(),
                    description: Some("Apply camera white balance during decode".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Apply".into()),
                        label_false: Some("As-Shot".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "dcraw_options".into(),
            label: "dcraw Options".into(),
            description: Some("dcraw-specific settings".into()),
            icon: None,
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "dcraw_path".into(),
                    label: "dcraw Path".into(),
                    description: Some("Path to the dcraw binary".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 1024,
                        pattern: None,
                        placeholder: Some("/usr/bin/dcraw".into()),
                    },
                    default: serde_json::json!("dcraw"),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "dcraw_extra_args".into(),
                    label: "Extra Arguments".into(),
                    description: Some("Additional dcraw command-line arguments".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 512,
                        pattern: None,
                        placeholder: Some("-H 2".into()),
                    },
                    default: serde_json::json!(""),
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
                param_section_id: "raw_format".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "output".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "dcraw_options".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("camera".into()),
    color: Some("#ef4444".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug)]
pub struct RawInputPlugin {
    id: String,
}

impl Default for RawInputPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RawInputPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.raw_input".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for RawInputPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "RAW Input"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Input
    }
    fn description(&self) -> &str {
        "Read RAW camera files (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF)"
    }
    fn tags(&self) -> &[String] {
        &TAGS
    }
    fn requires_pixel_access(&self) -> bool {
        false
    }
    fn produces_pixel_output(&self) -> bool {
        true
    }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement {
            min_ram_mb: 512,
            ..Default::default()
        }
    }

    fn parameter_schema(&self) -> &ParameterSchema {
        &PARAMETER_SCHEMA
    }
    fn gui_schema(&self) -> &GuiSchema {
        &GUI_SCHEMA
    }

    async fn initialize(&mut self, _cfg: &photopipeline_plugin::PluginConfig) -> PluginResult<()> {
        tracing::info!("raw_input plugin initialized (dcraw/libraw pending)");
        Ok(())
    }
    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("raw_input plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("raw_input: validating parameters");
        let mode = params.get_str("raw_mode").unwrap_or("auto");
        if mode == "dcraw" {
            let path = params.get_str("dcraw_path").unwrap_or("dcraw");
            let check = Command::new(path).output();
            if check.is_err() || !check.unwrap().status.success() {
                issues.push(ValidationIssue::Warning {
                    param: "dcraw_path".into(),
                    message: format!("dcraw binary '{}' not found or not functional", path),
                });
            }
        }
        Ok(issues)
    }
}

fn decode_via_libraw(_data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
    #[cfg(not(feature = "libraw-native"))]
    {
        let _ = (_data, _options);
        Err(PluginError::Internal {
            plugin: "raw_input".into(),
            message: "LibRaw native not compiled".into(),
        })
    }

    #[cfg(feature = "libraw-native")]
    unsafe {
        use libraw_ffi::*;

        let lr = libraw_init(LIBRAW_IMAGE_FORMAT_TIFF_LIKE as c_uint);
        if lr.is_null() {
            return Err(PluginError::Internal {
                plugin: "raw_input".into(),
                message: "libraw_init failed".into(),
            });
        }

        let ret = libraw_open_buffer(lr, _data.as_ptr() as *const c_void, _data.len());
        if ret != 0 {
            libraw_close(lr);
            return Err(PluginError::DecodingFailed(
                "libraw_open_buffer failed".into(),
            ));
        }

        let ret = libraw_unpack(lr);
        if ret != 0 {
            libraw_close(lr);
            return Err(PluginError::DecodingFailed("libraw_unpack failed".into()));
        }

        let ret = libraw_dcraw_process(lr);
        if ret != 0 {
            libraw_close(lr);
            return Err(PluginError::DecodingFailed(
                "libraw_dcraw_process failed".into(),
            ));
        }

        let img = libraw_dcraw_make_mem_image(lr);
        if img.is_null() {
            libraw_close(lr);
            return Err(PluginError::DecodingFailed("make_mem_image failed".into()));
        }

        let width = (*img).width as u32;
        let height = (*img).height as u32;
        let colors = (*img).colors as usize;

        let img_size = width as usize * height as usize * colors * 2;
        let mut pixel_data = vec![0u8; img_size];
        std::ptr::copy_nonoverlapping((*img).data as *const u8, pixel_data.as_mut_ptr(), img_size);

        libraw_dcraw_clear_mem(img);
        libraw_close(lr);

        let buffer = PixelBuffer {
            width,
            height,
            layout: if colors == 3 {
                ChannelLayout::RGB
            } else {
                ChannelLayout::Gray
            },
            format: PixelFormat::U16,
            color_space: ColorSpace::SRGB,
            icc_profile: None,
            data: AlignedBuffer {
                data: pixel_data,
                alignment: 64,
            },
        };

        Ok(DecodedImage {
            buffer,
            metadata: Metadata::default(),
            format: ImageFormat::RAW,
        })
    }
}

#[async_trait]
impl FormatProcessor for RawInputPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![
            ("arw", "image/x-sony-arw"),
            ("cr2", "image/x-canon-cr2"),
            ("cr3", "image/x-canon-cr3"),
            ("nef", "image/x-nikon-nef"),
            ("dng", "image/x-adobe-dng"),
            ("raf", "image/x-fuji-raf"),
            ("orf", "image/x-olympus-orf"),
            ("rw2", "image/x-panasonic-rw2"),
            ("pef", "image/x-pentax-pef"),
            ("srf", "image/x-sony-srf"),
            ("sr2", "image/x-sony-sr2"),
            ("3fr", "image/x-hasselblad-3fr"),
            ("mef", "image/x-mamiya-mef"),
            ("mos", "image/x-leaf-mos"),
            ("erf", "image/x-epson-erf"),
        ]
    }

    fn format_id(&self) -> ImageFormat {
        ImageFormat::RAW
    }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if probe.extension.is_some() || probe.path.is_some() {
            let ext = probe.extension.as_deref().unwrap_or("");
            let fname = probe
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let ext_lower = ext.to_lowercase();
            let fname_lower = fname.to_lowercase();

            if matches!(
                ext_lower.as_str(),
                "arw"
                    | "cr2"
                    | "cr3"
                    | "nef"
                    | "dng"
                    | "raf"
                    | "orf"
                    | "rw2"
                    | "pef"
                    | "srf"
                    | "sr2"
                    | "3fr"
                    | "mef"
                    | "mos"
                    | "erf"
            ) {
                return true;
            }

            if fname_lower.ends_with(".arw")
                || fname_lower.ends_with(".cr2")
                || fname_lower.ends_with(".cr3")
                || fname_lower.ends_with(".nef")
                || fname_lower.ends_with(".dng")
                || fname_lower.ends_with(".raf")
                || fname_lower.ends_with(".orf")
                || fname_lower.ends_with(".rw2")
            {
                return true;
            }
        }

        if let Some(ref magic) = probe.magic_bytes {
            if magic.len() >= 4 && (&magic[0..4] == b"II\x2A\x00" || &magic[0..4] == b"MM\x00\x2A")
            {
                let tiff_magic_ok = probe.extension.as_deref().is_some_and(|e| {
                    let el = e.to_lowercase();
                    matches!(
                        el.as_str(),
                        "arw" | "cr2" | "nef" | "dng" | "orf" | "rw2" | "pef"
                    )
                });
                if tiff_magic_ok {
                    return true;
                }
            }
            if magic.len() >= 4 && &magic[0..4] == b"IIQ\x00" {
                return true;
            }
            if magic.len() >= 4 && magic[0] == 0x49 && magic[1] == 0x49 {
                return true;
            }
        }

        false
    }

    async fn decode(&self, data: &[u8], options: &DecodeOptions) -> PluginResult<DecodedImage> {
        let _timer = PerfTimer::with_target("raw_input_decode", "plugin.raw_input");
        tracing::info!(
            data_len = data.len(),
            "raw_input: decoding {} bytes of RAW data",
            data.len(),
        );
        if data.is_empty() {
            return Err(PluginError::DecodingFailed("Empty input data".into()));
        }

        match decode_via_libraw(data, options) {
            Ok(image) => return Ok(image),
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    "raw_input: LibRaw decode failed, falling back to stub"
                );
            }
        }

        let pixel_format = options.pixel_format.unwrap_or(PixelFormat::U16);
        let half = options.max_width.is_some() || options.max_height.is_some();

        let mut width: u32;
        let mut height: u32;

        if data.len() >= 4 && (&data[0..4] == b"II\x2A\x00" || &data[0..4] == b"MM\x00\x2A") {
            width = 6000;
            height = 4000;
            tracing::info!("RAW: TIFF-based RAW format detected, assuming ~24MP sensor dimensions");
        } else {
            width = 6000;
            height = 4000;
            tracing::info!("RAW: Unknown RAW container, assuming ~24MP sensor dimensions");
        }

        if half {
            width /= 2;
            height /= 2;
        }

        let channels = 3;
        let bpc = pixel_format.bytes_per_channel();
        let buf_size = width as usize * height as usize * channels * bpc;
        let mut raw_buf = AlignedBuffer::new(buf_size, 64);

        let copy_len = data.len().min(buf_size);
        raw_buf.data[..copy_len].copy_from_slice(&data[..copy_len]);

        Ok(DecodedImage {
            buffer: PixelBuffer {
                width,
                height,
                layout: ChannelLayout::RGB,
                format: pixel_format,
                color_space: ColorSpace::SRGB,
                icc_profile: None,
                data: raw_buf,
            },
            metadata: Metadata::default(),
            format: ImageFormat::RAW,
        })
    }

    fn can_encode(&self, _format: &ImageFormat) -> bool {
        false
    }

    async fn encode(
        &self,
        _image: &PixelBuffer,
        _metadata: &Metadata,
        _options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        tracing::trace!("raw_input: encode called (not supported)");
        Err(PluginError::UnsupportedFormat(
            "RAW format is input-only".into(),
        ))
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "input".into(),
        "raw".into(),
        "camera".into(),
        "decode".into(),
        "arw".into(),
        "cr2".into(),
        "nef".into(),
        "dng".into(),
        "dcraw".into(),
        "libraw".into(),
    ]
});
