use photopipeline_plugin::registry::Registry;

const EXIF_ID: &str = "photopipeline.plugins.exif_rw";
const GPS_ID: &str = "photopipeline.plugins.gps_set";
const TIME_ID: &str = "photopipeline.plugins.time_shift";
const COLORSPACE_ID: &str = "photopipeline.plugins.colorspace";
const LUT_ID: &str = "photopipeline.plugins.lut3d";
const TRANSFORM_ID: &str = "photopipeline.plugins.transform";
const LENS_ID: &str = "photopipeline.plugins.lens_correct";
const AI_ID: &str = "photopipeline.plugins.ai_denoise";
const HEIF_ID: &str = "photopipeline.plugins.heif_encoder";
const JXL_ID: &str = "photopipeline.plugins.jxl_encoder";
const AVIF_ID: &str = "photopipeline.plugins.avif_encoder";
const TIFF_ID: &str = "photopipeline.plugins.tiff_encoder";
const PNG_ID: &str = "photopipeline.plugins.png_encoder";
const RAW_ID: &str = "photopipeline.plugins.raw_input";

#[test]
fn all_plugins_have_valid_schema() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let all = registry.all();
    assert_eq!(all.len(), 14);
    for plugin in &all {
        let schema = plugin.parameter_schema();
        assert!(schema.version > 0);
        assert!(
            !schema.sections.is_empty(),
            "plugin {} has empty schema",
            plugin.id()
        );
    }
}

#[test]
fn all_plugins_have_valid_guischema() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let all = registry.all();
    for plugin in &all {
        let gui = plugin.gui_schema();
        assert!(
            gui.min_panel_width > 0,
            "plugin {} has invalid min_panel_width",
            plugin.id()
        );
    }
}

#[test]
fn all_plugins_validate_their_defaults() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let all = registry.all();
    let rt = tokio::runtime::Runtime::new().unwrap();
    for plugin in &all {
        let defaults = plugin.parameter_schema().defaults();
        let result = rt.block_on(async { plugin.validate(&defaults).await });
        assert!(
            result.is_ok(),
            "plugin {} validation failed: {:?}",
            plugin.id(),
            result.err()
        );
    }
}

#[test]
fn all_plugins_have_nonempty_id() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        assert!(!plugin.id().is_empty());
    }
}

#[test]
fn all_plugins_have_nonempty_name() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        assert!(!plugin.name().is_empty());
    }
}

#[test]
fn all_plugins_have_version() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let v = plugin.version();
        assert!(v.major > 0 || v.minor > 0 || v.patch > 0);
    }
}

#[test]
fn all_plugins_have_description() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        assert!(!plugin.description().is_empty());
    }
}

#[test]
fn all_plugins_have_tags() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        assert!(
            !plugin.tags().is_empty(),
            "plugin {} has empty tags",
            plugin.id()
        );
    }
}

#[test]
fn all_plugins_have_valid_categories() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let cat_str = format!("{}", plugin.category());
        assert!(
            !cat_str.is_empty(),
            "plugin {} has invalid category",
            plugin.id()
        );
    }
}

#[test]
fn metadata_plugins_dont_require_pixel_access() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let id = plugin.id().to_string();
        if id == EXIF_ID || id == GPS_ID || id == TIME_ID {
            assert!(
                !plugin.requires_pixel_access(),
                "{} should not require pixel access",
                plugin.id()
            );
            assert!(
                !plugin.produces_pixel_output(),
                "{} should not produce pixel output",
                plugin.id()
            );
        }
    }
}

#[test]
fn pixel_plugins_require_pixel_access() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let id = plugin.id().to_string();
        if id == COLORSPACE_ID || id == LUT_ID || id == TRANSFORM_ID || id == LENS_ID || id == AI_ID
        {
            assert!(
                plugin.requires_pixel_access(),
                "{} should require pixel access",
                plugin.id()
            );
        }
    }
}

#[test]
fn format_plugins_have_manifests() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let id = plugin.id().to_string();
        if id == HEIF_ID
            || id == JXL_ID
            || id == AVIF_ID
            || id == TIFF_ID
            || id == PNG_ID
            || id == RAW_ID
        {
            let manifest = registry.manifest(plugin.id());
            assert!(manifest.is_some(), "no manifest for {}", plugin.id());
        }
    }
}

#[test]
fn heif_encoder_validates_quality_range() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry.get(&HEIF_ID.to_string());
    assert!(plugin.is_some());
    if let Some(p) = plugin {
        let mut params = p.parameter_schema().defaults();
        params.insert("quality".into(), serde_json::json!(50.0));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { p.validate(&params).await });
        assert!(result.is_ok());
    }
}

#[test]
fn jxl_encoder_validates_effort_range() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry.get(&JXL_ID.to_string());
    assert!(plugin.is_some());
    if let Some(p) = plugin {
        let mut params = p.parameter_schema().defaults();
        params.insert("effort".into(), serde_json::json!(5));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { p.validate(&params).await });
        assert!(result.is_ok());
    }
}

#[test]
fn exif_rw_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&EXIF_ID.to_string())
        .expect("exif_rw not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn gps_set_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&GPS_ID.to_string())
        .expect("gps_set not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn time_shift_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&TIME_ID.to_string())
        .expect("time_shift not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn colorspace_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&COLORSPACE_ID.to_string())
        .expect("colorspace not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn transform_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&TRANSFORM_ID.to_string())
        .expect("transform not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn ai_denoise_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&AI_ID.to_string())
        .expect("ai_denoise not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn raw_input_schema_not_empty() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let plugin = registry
        .get(&RAW_ID.to_string())
        .expect("raw_input not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty());
}

#[test]
fn all_plugins_have_valid_hardware_requirements() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let hw = plugin.supported_hardware();
        assert!(
            hw.requires_cpu || hw.requires_gpu,
            "plugin {} has no hardware requirements",
            plugin.id()
        );
    }
}
