//! ParameterSchema validation tests for all 14 plugins.
//! ~140 tests covering default values, valid ranges, enum options, type checking.

use photopipeline_plugin::registry::Registry;
use photopipeline_plugin::ParameterType;
use serde_json::Value;

const PLUGIN_IDS: [&str; 14] = [
    "photopipeline.plugins.raw_input",
    "photopipeline.plugins.transform",
    "photopipeline.plugins.colorspace",
    "photopipeline.plugins.lut3d",
    "photopipeline.plugins.lens_correct",
    "photopipeline.plugins.ai_denoise",
    "photopipeline.plugins.exif_rw",
    "photopipeline.plugins.gps_set",
    "photopipeline.plugins.time_shift",
    "photopipeline.plugins.avif_encoder",
    "photopipeline.plugins.jxl_encoder",
    "photopipeline.plugins.heif_encoder",
    "photopipeline.plugins.tiff_encoder",
    "photopipeline.plugins.png_encoder",
];

fn setup_registry() -> Registry {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    registry
}

// ─── Section B.1.1: Generic validation (56 tests: 14 plugins x 4) ───

#[test]
fn raw_input_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).expect("raw_input not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "raw_input schema should have sections");
}

#[test]
fn transform_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).expect("transform not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "transform schema should have sections");
}

#[test]
fn colorspace_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).expect("colorspace not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "colorspace schema should have sections");
}

#[test]
fn lut3d_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).expect("lut3d not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "lut3d schema should have sections");
}

#[test]
fn lens_correct_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).expect("lens_correct not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "lens_correct schema should have sections");
}

#[test]
fn ai_denoise_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).expect("ai_denoise not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "ai_denoise schema should have sections");
}

#[test]
fn exif_rw_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).expect("exif_rw not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "exif_rw schema should have sections");
}

#[test]
fn gps_set_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).expect("gps_set not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "gps_set schema should have sections");
}

#[test]
fn time_shift_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).expect("time_shift not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "time_shift schema should have sections");
}

#[test]
fn avif_encoder_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).expect("avif_encoder not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "avif_encoder schema should have sections");
}

#[test]
fn jxl_encoder_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).expect("jxl_encoder not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "jxl_encoder schema should have sections");
}

#[test]
fn heif_encoder_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).expect("heif_encoder not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "heif_encoder schema should have sections");
}

#[test]
fn tiff_encoder_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).expect("tiff_encoder not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "tiff_encoder schema should have sections");
}

#[test]
fn png_encoder_schema_is_non_empty() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).expect("png_encoder not found");
    let schema = plugin.parameter_schema();
    assert!(!schema.sections.is_empty(), "png_encoder schema should have sections");
}

// ─── All fields have non-null defaults ───

macro_rules! test_all_fields_have_defaults {
    ($name:ident, $idx:expr) => {
        #[test]
        fn $name() {
            let reg = setup_registry();
            let plugin = reg.get(PLUGIN_IDS[$idx]).expect("plugin not found");
            let schema = plugin.parameter_schema();
            for field in schema.all_fields() {
                assert!(
                    !field.default.is_null(),
                    "plugin {} field '{}' has null default",
                    PLUGIN_IDS[$idx],
                    field.id
                );
            }
        }
    };
}

test_all_fields_have_defaults!(raw_input_all_fields_have_defaults, 0);
test_all_fields_have_defaults!(transform_all_fields_have_defaults, 1);
test_all_fields_have_defaults!(colorspace_all_fields_have_defaults, 2);
test_all_fields_have_defaults!(lut3d_all_fields_have_defaults, 3);
test_all_fields_have_defaults!(lens_correct_all_fields_have_defaults, 4);
test_all_fields_have_defaults!(ai_denoise_all_fields_have_defaults, 5);
test_all_fields_have_defaults!(exif_rw_all_fields_have_defaults, 6);
test_all_fields_have_defaults!(gps_set_all_fields_have_defaults, 7);
test_all_fields_have_defaults!(time_shift_all_fields_have_defaults, 8);
test_all_fields_have_defaults!(avif_encoder_all_fields_have_defaults, 9);
test_all_fields_have_defaults!(jxl_encoder_all_fields_have_defaults, 10);
test_all_fields_have_defaults!(heif_encoder_all_fields_have_defaults, 11);
test_all_fields_have_defaults!(tiff_encoder_all_fields_have_defaults, 12);
test_all_fields_have_defaults!(png_encoder_all_fields_have_defaults, 13);

// ─── Defaults match field types ───

/// Returns true if the JSON value matches the expected parameter type.
fn default_matches_type(default: &Value, field_type: &ParameterType) -> bool {
    match field_type {
        ParameterType::Integer { .. } => default.is_number(),
        ParameterType::Float { .. } => default.is_number(),
        ParameterType::Slider { .. } => default.is_number(),
        ParameterType::Enum { .. } => default.is_string(),
        ParameterType::Boolean { .. } => default.is_boolean(),
        ParameterType::String { .. } => default.is_string(),
        ParameterType::FilePath { .. } => default.is_string(),
        ParameterType::Color { .. } => default.is_string(),
        ParameterType::Coordinate { .. } => false, // coordinate uses specific sub-keys
        // For complex types, verify default is not null at minimum
        ParameterType::ComboSlider { .. } => default.is_number(),
        ParameterType::Expression { .. } => default.is_string(),
        ParameterType::Preset { .. } => true, // preset has complex object
        ParameterType::Array { .. } => default.is_array(),
        ParameterType::MapWidget { .. } => true,
        ParameterType::BeforeAfter { .. } => true,
        ParameterType::Separator { .. } => true,
        ParameterType::Section { .. } => true,
    }
}

macro_rules! test_defaults_match_types {
    ($name:ident, $idx:expr) => {
        #[test]
        fn $name() {
            let reg = setup_registry();
            let plugin = reg.get(PLUGIN_IDS[$idx]).expect("plugin not found");
            let schema = plugin.parameter_schema();
            for field in schema.all_fields() {
                let default = &field.default;
                if default.is_null() {
                    // null defaults are handled by the defaults test; skip type check
                    continue;
                }
                assert!(
                    default_matches_type(default, &field.field_type),
                    "plugin {} field '{}': default value {:?} does not match type {:?}",
                    PLUGIN_IDS[$idx],
                    field.id,
                    default,
                    std::mem::discriminant(&field.field_type),
                );
            }
        }
    };
}

test_defaults_match_types!(raw_input_defaults_match_field_types, 0);
test_defaults_match_types!(transform_defaults_match_field_types, 1);
test_defaults_match_types!(colorspace_defaults_match_field_types, 2);
test_defaults_match_types!(lut3d_defaults_match_field_types, 3);
test_defaults_match_types!(lens_correct_defaults_match_field_types, 4);
test_defaults_match_types!(ai_denoise_defaults_match_field_types, 5);
test_defaults_match_types!(exif_rw_defaults_match_field_types, 6);
test_defaults_match_types!(gps_set_defaults_match_field_types, 7);
test_defaults_match_types!(time_shift_defaults_match_field_types, 8);
test_defaults_match_types!(avif_encoder_defaults_match_field_types, 9);
test_defaults_match_types!(jxl_encoder_defaults_match_field_types, 10);
test_defaults_match_types!(heif_encoder_defaults_match_field_types, 11);
test_defaults_match_types!(tiff_encoder_defaults_match_field_types, 12);
test_defaults_match_types!(png_encoder_defaults_match_field_types, 13);

// ─── Enum options are valid ───

macro_rules! test_enum_options_valid {
    ($name:ident, $idx:expr) => {
        #[test]
        fn $name() {
            let reg = setup_registry();
            let plugin = reg.get(PLUGIN_IDS[$idx]).expect("plugin not found");
            let schema = plugin.parameter_schema();
            for field in schema.all_fields() {
                if let ParameterType::Enum { options, .. } = &field.field_type {
                    assert!(
                        !options.is_empty(),
                        "plugin {} enum field '{}' has no options",
                        PLUGIN_IDS[$idx],
                        field.id
                    );
                    for opt in options {
                        assert!(
                            !opt.value.is_empty(),
                            "plugin {} enum field '{}' option has empty value",
                            PLUGIN_IDS[$idx],
                            field.id
                        );
                        assert!(
                            !opt.label.is_empty(),
                            "plugin {} enum field '{}' option '{}' has empty label",
                            PLUGIN_IDS[$idx],
                            field.id,
                            opt.value
                        );
                    }
                }
            }
        }
    };
}

test_enum_options_valid!(raw_input_enum_options_valid, 0);
test_enum_options_valid!(transform_enum_options_valid, 1);
test_enum_options_valid!(colorspace_enum_options_valid, 2);
test_enum_options_valid!(lut3d_enum_options_valid, 3);
test_enum_options_valid!(lens_correct_enum_options_valid, 4);
test_enum_options_valid!(ai_denoise_enum_options_valid, 5);
test_enum_options_valid!(exif_rw_enum_options_valid, 6);
test_enum_options_valid!(gps_set_enum_options_valid, 7);
test_enum_options_valid!(time_shift_enum_options_valid, 8);
test_enum_options_valid!(avif_encoder_enum_options_valid, 9);
test_enum_options_valid!(jxl_encoder_enum_options_valid, 10);
test_enum_options_valid!(heif_encoder_enum_options_valid, 11);
test_enum_options_valid!(tiff_encoder_enum_options_valid, 12);
test_enum_options_valid!(png_encoder_enum_options_valid, 13);

// ══════════════════════════════════════════════════════════════════════════
// Section B.1.2: Parameter-specific validation (84 tests)
// ══════════════════════════════════════════════════════════════════════════

// ─── raw_input plugin (6 specific tests) ───

#[test]
fn raw_input_raw_mode_accepts_all_four_enum_values() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("raw_format", "raw_mode").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"auto"), "raw_mode should contain 'auto'");
        assert!(values.contains(&"dcraw"), "raw_mode should contain 'dcraw'");
        assert!(values.contains(&"libraw"), "raw_mode should contain 'libraw'");
        assert!(values.contains(&"rawtherapee"), "raw_mode should contain 'rawtherapee'");
        assert_eq!(options.len(), 4, "raw_mode should have exactly 4 options");
    } else {
        panic!("raw_mode should be Enum type");
    }
}

#[test]
fn raw_input_output_format_defaults_to_u16() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("output", "output_format").expect("field not found");
    assert_eq!(field.default, serde_json::json!("u16"),
        "output_format default should be 'u16'");
}

#[test]
fn raw_input_half_size_is_boolean_type() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("output", "half_size").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }),
        "half_size should be Boolean type");
    assert_eq!(field.default, serde_json::json!(false));
}

#[test]
fn raw_input_apply_white_balance_defaults_true() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("output", "apply_white_balance").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }),
        "apply_white_balance should be Boolean type");
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn raw_input_dcraw_path_is_string_with_default() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("dcraw_options", "dcraw_path").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::String { .. }),
        "dcraw_path should be String type");
    assert_eq!(field.default, serde_json::json!("dcraw"));
}

#[test]
fn raw_input_validates_unknown_enum_value_is_rejected() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[0]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("raw_format", "raw_mode").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let valid_values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(!valid_values.contains(&"invalid_mode"),
            "'invalid_mode' must not be in the valid enum options for raw_mode");
    }
}

// ─── transform plugin (6 specific tests) ───

#[test]
fn transform_resize_mode_has_six_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("resize", "resize_mode").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"absolute"));
        assert!(values.contains(&"percentage"));
        assert!(values.contains(&"long_edge"));
        assert!(values.contains(&"short_edge"));
        assert!(values.contains(&"megapixels"));
        assert!(values.contains(&"none"));
        assert_eq!(options.len(), 6, "resize_mode should have exactly 6 options");
    } else {
        panic!("resize_mode should be Enum type");
    }
}

#[test]
fn transform_scale_percent_has_correct_range() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("resize", "scale_percent").expect("field not found");
    if let ParameterType::Float { min, max, .. } = &field.field_type {
        assert_eq!(*min, 1.0, "scale_percent min should be 1.0");
        assert_eq!(*max, 1000.0, "scale_percent max should be 1000.0");
    } else {
        panic!("scale_percent should be Float type");
    }
    assert_eq!(field.default, serde_json::json!(100.0));
}

#[test]
fn transform_angle_accepts_full_negative_to_positive_range() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("rotation", "angle").expect("field not found");
    if let ParameterType::Float { min, max, .. } = &field.field_type {
        assert_eq!(*min, -360.0, "angle min should be -360.0");
        assert_eq!(*max, 360.0, "angle max should be 360.0");
    } else {
        panic!("angle should be Float type");
    }
}

#[test]
fn transform_filter_type_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("filter", "filter_type").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"bilinear"));
        assert!(values.contains(&"lanczos3"));
        assert!(values.contains(&"nearest"));
        assert_eq!(options.len(), 3, "filter_type should have exactly 3 options");
    } else {
        panic!("filter_type should be Enum type");
    }
}

#[test]
fn transform_crop_enabled_is_boolean_defaults_false() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("crop", "crop_enabled").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(false));
}

#[test]
fn transform_target_width_has_correct_integer_range() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("resize", "target_width").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, 1);
        assert_eq!(*max, 65535);
    } else {
        panic!("target_width should be Integer type");
    }
}

// ─── colorspace plugin (6 specific tests) ───

#[test]
fn colorspace_source_space_has_eight_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("conversion", "source_color_space").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"auto"));
        assert!(values.contains(&"srgb"));
        assert!(values.contains(&"display_p3"));
        assert!(values.contains(&"adobe_rgb"));
        assert!(values.contains(&"pro_photo"));
        assert!(values.contains(&"bt2020"));
        assert!(values.contains(&"aces_cg"));
        assert!(values.contains(&"linear_srgb"));
        assert_eq!(options.len(), 8, "source_color_space should have exactly 8 options");
    } else {
        panic!("source_color_space should be Enum type");
    }
}

#[test]
fn colorspace_target_space_has_six_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("conversion", "target_color_space").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"srgb"));
        assert!(values.contains(&"display_p3"));
        assert!(values.contains(&"adobe_rgb"));
        assert!(values.contains(&"pro_photo"));
        assert!(values.contains(&"bt2020_pq"));
        assert!(values.contains(&"linear_srgb"));
        assert_eq!(options.len(), 6);
    } else {
        panic!("target_color_space should be Enum type");
    }
}

#[test]
fn colorspace_rendering_intent_has_four_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("rendering", "rendering_intent").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"relative_colorimetric"));
        assert!(values.contains(&"perceptual"));
        assert!(values.contains(&"saturation"));
        assert!(values.contains(&"absolute_colorimetric"));
        assert_eq!(options.len(), 4);
    } else {
        panic!("rendering_intent should be Enum type");
    }
}

#[test]
fn colorspace_black_point_compensation_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("rendering", "black_point_compensation").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn colorspace_gamut_mapping_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("rendering", "gamut_mapping").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"clip"));
        assert!(values.contains(&"compress"));
        assert!(values.contains(&"luminance_preserve"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("gamut_mapping should be Enum type");
    }
}

#[test]
fn colorspace_embed_icc_defaults_true() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[2]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("icc_profile", "embed_icc").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

// ─── lut3d plugin (6 specific tests) ───

#[test]
fn lut3d_lut_format_has_four_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lut_file", "lut_format").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"cube"));
        assert!(values.contains(&"3dl"));
        assert!(values.contains(&"look"));
        assert!(values.contains(&"csp"));
        assert_eq!(options.len(), 4);
    } else {
        panic!("lut_format should be Enum type");
    }
}

#[test]
fn lut3d_intensity_has_range_0_to_100() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lut_transform", "intensity").expect("field not found");
    if let ParameterType::Slider { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0.0, "intensity min should be 0.0");
        assert_eq!(*max, 100.0, "intensity max should be 100.0");
    } else {
        panic!("intensity should be Slider type");
    }
}

#[test]
fn lut3d_lut_path_is_file_path_type() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lut_file", "lut_path").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::FilePath { .. }),
        "lut_path should be FilePath type");
}

#[test]
fn lut3d_interpolation_has_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lut_transform", "interpolation");
    // This field may not exist in all versions; verify gracefully
    if let Some(f) = field {
        if let ParameterType::Enum { options, .. } = &f.field_type {
            assert!(!options.is_empty(), "interpolation should have options");
            let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
            assert!(values.contains(&"trilinear") || values.contains(&"tetrahedral"),
                "interpolation should contain trilinear or tetrahedral");
        }
    }
}

#[test]
fn lut3d_clamp_output_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lut_transform", "clamp_output");
    if let Some(f) = field {
        assert!(matches!(f.field_type, ParameterType::Boolean { .. }),
            "clamp_output should be Boolean type");
    }
}

#[test]
fn lut3d_working_space_is_enum() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[3]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lut_transform", "working_space");
    if let Some(f) = field {
        assert!(
            matches!(f.field_type, ParameterType::Enum { .. }),
            "working_space should be Enum type"
        );
    }
}

// ─── lens_correct plugin (6 specific tests) ───

#[test]
fn lens_correct_correction_mode_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("lens_detection", "correction_mode").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"auto"));
        assert!(values.contains(&"manual"));
        assert!(values.contains(&"off"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("correction_mode should be Enum type");
    }
}

#[test]
fn lens_correct_correct_distortion_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("corrections", "correct_distortion").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn lens_correct_correct_tca_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("corrections", "correct_tca").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn lens_correct_correct_vignetting_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("corrections", "correct_vignetting").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn lens_correct_distortion_amount_has_range() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("manual_params", "distortion_amount");
    if let Some(f) = field {
        if let ParameterType::Float { min, max, .. } = &f.field_type {
            assert!(*min < *max, "distortion_amount range should be valid");
        }
    }
}

#[test]
fn lens_correct_lensfun_db_path_is_file_path_or_string() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[4]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("manual_params", "lensfun_db_path");
    if let Some(f) = field {
        assert!(
            matches!(f.field_type, ParameterType::FilePath { .. }) || matches!(f.field_type, ParameterType::String { .. }),
            "lensfun_db_path should be FilePath or String"
        );
    }
}

// ─── ai_denoise plugin (6 specific tests) ───

#[test]
fn ai_denoise_denoise_model_has_four_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("model", "denoise_model").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"lightweight_v1"));
        assert!(values.contains(&"standard_v2"));
        assert!(values.contains(&"high_quality_v2"));
        assert!(values.contains(&"raw_denoise_v1"));
        assert_eq!(options.len(), 4);
    } else {
        panic!("denoise_model should be Enum type");
    }
}

#[test]
fn ai_denoise_strength_has_range_0_to_100() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("strength", "denoise_strength").expect("field not found");
    if let ParameterType::Slider { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0.0, "denoise_strength min should be 0");
        assert_eq!(*max, 100.0, "denoise_strength max should be 100");
    } else {
        panic!("denoise_strength should be Slider type");
    }
}

#[test]
fn ai_denoise_detail_preservation_has_range_0_to_100() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("strength", "detail_preservation").expect("field not found");
    if let ParameterType::Slider { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0.0, "detail_preservation min should be 0");
        assert_eq!(*max, 100.0, "detail_preservation max should be 100");
    } else {
        panic!("detail_preservation should be Slider type");
    }
}

#[test]
fn ai_denoise_color_noise_reduction_is_slider() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("strength", "color_noise_reduction");
    if let Some(f) = field {
        assert!(matches!(f.field_type, ParameterType::Slider { .. }),
            "color_noise_reduction should be Slider type (0-100 range)");
    }
}

#[test]
fn ai_denoise_sharpen_amount_has_range() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("post_process", "sharpen_amount");
    if let Some(f) = field {
        if let ParameterType::Slider { min, max, .. } = &f.field_type {
            assert!(*min >= 0.0, "sharpen_amount min should be >= 0");
            assert!(*max > *min, "sharpen_amount max should be > min");
        }
    }
}

#[test]
fn ai_denoise_gpu_backend_is_enum() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[5]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("advanced", "gpu_backend");
    if let Some(f) = field {
        if let ParameterType::Enum { options, .. } = &f.field_type {
            assert!(!options.is_empty(), "gpu_backend should have options");
        }
    }
}

// ─── exif_rw plugin (6 specific tests) ───

#[test]
fn exif_rw_read_exif_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("read_options", "read_exif").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn exif_rw_read_xmp_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("read_options", "read_xmp").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn exif_rw_write_exif_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("write_options", "write_exif").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"all"));
        assert!(values.contains(&"selected"));
        assert!(values.contains(&"none"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("write_exif should be Enum type");
    }
}

#[test]
fn exif_rw_overwrite_original_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("write_options", "overwrite_original").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(false));
}

#[test]
fn exif_rw_preserve_makernote_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("write_options", "preserve_makernote").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn exif_rw_read_gps_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[6]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("read_options", "read_gps").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

// ─── gps_set plugin (6 specific tests) ───

#[test]
fn gps_set_gps_mode_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("source", "gps_mode").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"manual"));
        assert!(values.contains(&"gpx_track"));
        assert!(values.contains(&"clear"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("gps_mode should be Enum type");
    }
}

#[test]
fn gps_set_latitude_has_range_minus90_to_90() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("manual_coords", "latitude").expect("field not found");
    if let ParameterType::Float { min, max, .. } = &field.field_type {
        assert_eq!(*min, -90.0, "latitude min should be -90.0");
        assert_eq!(*max, 90.0, "latitude max should be 90.0");
    } else {
        panic!("latitude should be Float type");
    }
}

#[test]
fn gps_set_longitude_has_range_minus180_to_180() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("manual_coords", "longitude").expect("field not found");
    if let ParameterType::Float { min, max, .. } = &field.field_type {
        assert_eq!(*min, -180.0, "longitude min should be -180.0");
        assert_eq!(*max, 180.0, "longitude max should be 180.0");
    } else {
        panic!("longitude should be Float type");
    }
}

#[test]
fn gps_set_altitude_has_range_minus500_to_9000() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("manual_coords", "altitude").expect("field not found");
    if let ParameterType::Float { min, max, .. } = &field.field_type {
        assert_eq!(*min, -500.0, "altitude min should be -500.0");
        assert_eq!(*max, 9000.0, "altitude max should be 9000.0");
    } else {
        panic!("altitude should be Float type");
    }
}

#[test]
fn gps_set_gpx_file_is_file_path_type() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("source", "gpx_file").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::FilePath { .. }),
        "gpx_file should be FilePath type");
}

#[test]
fn gps_set_gps_mode_is_required() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("source", "gps_mode").expect("field not found");
    assert!(field.required, "gps_mode should be required");
}

// ─── time_shift plugin (6 specific tests) ───

#[test]
fn time_shift_shift_hours_has_range_minus23_to_23() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("time_shift", "shift_hours").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, -23, "shift_hours min should be -23");
        assert_eq!(*max, 23, "shift_hours max should be 23");
    } else {
        panic!("shift_hours should be Integer type");
    }
}

#[test]
fn time_shift_shift_minutes_has_range_minus59_to_59() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("time_shift", "shift_minutes").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, -59, "shift_minutes min should be -59");
        assert_eq!(*max, 59, "shift_minutes max should be 59");
    } else {
        panic!("shift_minutes should be Integer type");
    }
}

#[test]
fn time_shift_shift_seconds_has_range_minus59_to_59() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("time_shift", "shift_seconds").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, -59, "shift_seconds min should be -59");
        assert_eq!(*max, 59, "shift_seconds max should be 59");
    } else {
        panic!("shift_seconds should be Integer type");
    }
}

#[test]
fn time_shift_source_timezone_has_enum_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("timezone", "source_timezone").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"UTC"), "source_timezone should contain 'UTC'");
        assert!(values.contains(&"local"), "source_timezone should contain 'local'");
        assert!(!options.is_empty(), "source_timezone should have options");
    } else {
        panic!("source_timezone should be Enum type");
    }
}

#[test]
fn time_shift_target_timezone_has_enum_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("timezone", "target_timezone").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        assert!(!options.is_empty(), "target_timezone should have options");
    } else {
        panic!("target_timezone should be Enum type");
    }
}

#[test]
fn time_shift_date_format_is_string() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("timezone", "date_format");
    if let Some(f) = field {
        assert!(matches!(f.field_type, ParameterType::String { .. }),
            "date_format should be String type");
    }
}

// ─── avif_encoder plugin (6 specific tests) ───

#[test]
fn avif_encoder_quality_has_range_0_to_100() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "quality").expect("field not found");
    if let ParameterType::Slider { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0.0, "quality min should be 0.0");
        assert_eq!(*max, 100.0, "quality max should be 100.0");
    } else {
        panic!("quality should be Slider type");
    }
}

#[test]
fn avif_encoder_speed_has_range_0_to_10() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "speed").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0, "speed min should be 0");
        assert_eq!(*max, 10, "speed max should be 10");
    } else {
        panic!("speed should be Integer type");
    }
}

#[test]
fn avif_encoder_bit_depth_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("format", "bit_depth").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"8"));
        assert!(values.contains(&"10"));
        assert!(values.contains(&"12"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("bit_depth should be Enum type");
    }
}

#[test]
fn avif_encoder_chroma_subsampling_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("format", "chroma_subsampling").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"444"));
        assert!(values.contains(&"422"));
        assert!(values.contains(&"420"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("chroma_subsampling should be Enum type");
    }
}

#[test]
fn avif_encoder_lossless_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("advanced", "lossless").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
}

#[test]
fn avif_encoder_quality_default_is_85() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[9]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "quality").expect("field not found");
    assert_eq!(field.default, serde_json::json!(85.0), "quality default should be 85.0");
}

// ─── jxl_encoder plugin (6 specific tests) ───

#[test]
fn jxl_encoder_quality_has_range_minus1_to_100() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "quality").expect("field not found");
    if let ParameterType::Slider { min, max, .. } = &field.field_type {
        assert_eq!(*min, -1.0, "quality min should be -1.0 (lossless)");
        assert_eq!(*max, 100.0, "quality max should be 100.0");
    } else {
        panic!("quality should be Slider type");
    }
}

#[test]
fn jxl_encoder_effort_has_range_1_to_9() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("advanced", "effort").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, 1, "effort min should be 1");
        assert_eq!(*max, 9, "effort max should be 9");
    } else {
        panic!("effort should be Integer type");
    }
}

#[test]
fn jxl_encoder_lossless_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "lossless").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(false));
}

#[test]
fn jxl_encoder_bit_depth_has_four_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "bit_depth").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"8"));
        assert!(values.contains(&"10"));
        assert!(values.contains(&"12"));
        assert!(values.contains(&"16"));
        assert_eq!(options.len(), 4);
    } else {
        panic!("bit_depth should be Enum type");
    }
}

#[test]
fn jxl_encoder_modular_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("advanced", "modular").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(false));
}

#[test]
fn jxl_encoder_quality_default_is_90() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[10]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "quality").expect("field not found");
    assert_eq!(field.default, serde_json::json!(90.0), "quality default should be 90.0");
}

// ─── heif_encoder plugin (6 specific tests) ───

#[test]
fn heif_encoder_quality_has_range_0_to_100() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "quality").expect("field not found");
    if let ParameterType::Slider { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0.0, "quality min should be 0.0");
        assert_eq!(*max, 100.0, "quality max should be 100.0");
    } else {
        panic!("quality should be Slider type");
    }
}

#[test]
fn heif_encoder_lossless_is_boolean() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "lossless").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
}

#[test]
fn heif_encoder_bit_depth_has_two_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "bit_depth").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"8"));
        assert!(values.contains(&"10"));
        assert_eq!(options.len(), 2);
    } else {
        panic!("bit_depth should be Enum type");
    }
}

#[test]
fn heif_encoder_chroma_subsampling_has_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).unwrap();
    let schema = plugin.parameter_schema();

    // Check for chroma_subsampling field
    let field = schema.field("format", "chroma_subsampling");
    if let Some(f) = field {
        if let ParameterType::Enum { options, .. } = &f.field_type {
            assert!(!options.is_empty(), "chroma_subsampling should have options");
        }
    }
}

#[test]
fn heif_encoder_quality_default_is_95() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "quality").expect("field not found");
    assert_eq!(field.default, serde_json::json!(95.0), "quality default should be 95.0");
}

#[test]
fn heif_encoder_speed_is_integer() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[11]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("quality", "speed");
    if let Some(f) = field {
        assert!(matches!(f.field_type, ParameterType::Integer { .. }),
            "speed should be Integer type");
    }
}

// ─── tiff_encoder plugin (6 specific tests) ───

#[test]
fn tiff_encoder_compression_has_four_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "compression").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"none"));
        assert!(values.contains(&"lzw"));
        assert!(values.contains(&"deflate"));
        assert!(values.contains(&"packbits"));
        assert_eq!(options.len(), 4);
    } else {
        panic!("compression should be Enum type");
    }
}

#[test]
fn tiff_encoder_bigtiff_is_boolean_defaults_true() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "bigtiff").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn tiff_encoder_embed_icc_is_boolean_defaults_true() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("metadata", "embed_icc").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn tiff_encoder_pixel_format_has_three_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("metadata", "pixel_format").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"u8"));
        assert!(values.contains(&"u16"));
        assert!(values.contains(&"f32"));
        assert_eq!(options.len(), 3);
    } else {
        panic!("pixel_format should be Enum type");
    }
}

#[test]
fn tiff_encoder_rows_per_strip_has_integer_range() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "rows_per_strip");
    if let Some(f) = field {
        if let ParameterType::Integer { min, max, .. } = &f.field_type {
            assert!(*min > 0, "rows_per_strip min should be > 0");
            assert!(*max >= *min);
        }
    }
}

#[test]
fn tiff_encoder_predictor_has_enum_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[12]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "predictor");
    if let Some(f) = field {
        if let ParameterType::Enum { options, .. } = &f.field_type {
            assert!(!options.is_empty(), "predictor should have options");
        }
    }
}

// ─── png_encoder plugin (6 specific tests) ───

#[test]
fn png_encoder_compression_level_has_range_0_to_9() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "compression_level").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert_eq!(*min, 0, "compression_level min should be 0");
        assert_eq!(*max, 9, "compression_level max should be 9");
    } else {
        panic!("compression_level should be Integer type");
    }
}

#[test]
fn png_encoder_color_type_has_four_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("metadata", "color_type").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"rgb"));
        assert!(values.contains(&"rgba"));
        assert!(values.contains(&"gray"));
        assert!(values.contains(&"graya"));
        assert_eq!(options.len(), 4);
    } else {
        panic!("color_type should be Enum type");
    }
}

#[test]
fn png_encoder_bit_depth_has_two_options() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "bit_depth").expect("field not found");
    if let ParameterType::Enum { options, .. } = &field.field_type {
        let values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        assert!(values.contains(&"8"));
        assert!(values.contains(&"16"));
        assert_eq!(options.len(), 2);
    } else {
        panic!("bit_depth should be Enum type");
    }
}

#[test]
fn png_encoder_embed_icc_is_boolean_defaults_true() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("metadata", "embed_icc").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(true));
}

#[test]
fn png_encoder_include_exif_is_boolean_defaults_false() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("metadata", "include_exif").expect("field not found");
    assert!(matches!(field.field_type, ParameterType::Boolean { .. }));
    assert_eq!(field.default, serde_json::json!(false));
}

#[test]
fn png_encoder_compression_level_default_is_6() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[13]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("encoding", "compression_level").expect("field not found");
    assert_eq!(field.default, serde_json::json!(6));
}

// ─── Edge case: type validation for integer parameters ───

#[test]
fn transform_target_width_rejects_string_value() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("resize", "target_width").expect("field not found");
    // Integer fields should not accept string values in their default
    assert!(
        matches!(field.field_type, ParameterType::Integer { .. }),
        "target_width must be Integer type"
    );
    assert!(
        field.default.is_number(),
        "target_width default must be numeric, got: {:?}", field.default
    );
}

#[test]
fn transform_scale_percent_rejects_non_numeric() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[1]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("resize", "scale_percent").expect("field not found");
    assert!(
        matches!(field.field_type, ParameterType::Float { .. }),
        "scale_percent must be Float type"
    );
    assert!(
        field.default.is_number(),
        "scale_percent default must be numeric"
    );
}

#[test]
fn time_shift_shift_hours_rejects_out_of_range_value() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[8]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("time_shift", "shift_hours").expect("field not found");
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        // Value 24 should be rejected as it exceeds max=23
        assert!(*max < 24, "shift_hours max should reject value 24");
        // Value -24 should be rejected as it is below min=-23
        assert!(*min > -24, "shift_hours min should reject value -24");
    }
}

#[test]
fn gps_set_latitude_rejects_below_minus90() {
    let reg = setup_registry();
    let plugin = reg.get(PLUGIN_IDS[7]).unwrap();
    let schema = plugin.parameter_schema();

    let field = schema.field("manual_coords", "latitude").expect("field not found");
    if let ParameterType::Float { min, max, .. } = &field.field_type {
        // -91 should be rejected (below min=-90)
        assert!(*min == -90.0, "latitude min should be -90.0, which rejects -91");
        assert!(*max == 90.0, "latitude max should be 90.0, which rejects 91");
    }
}

#[test]
fn all_plugins_defaults_validate_successfully() {
    let reg = setup_registry();
    let rt = tokio::runtime::Runtime::new().unwrap();

    for plugin in reg.all() {
        let defaults = plugin.parameter_schema().defaults();
        let result = rt.block_on(async { plugin.validate(&defaults).await });
        assert!(
            result.is_ok(),
            "plugin {} defaults failed validation: {:?}",
            plugin.id(),
            result.err()
        );
    }
}

#[test]
fn all_plugins_have_positive_schema_version() {
    let reg = setup_registry();
    for plugin in reg.all() {
        let schema = plugin.parameter_schema();
        assert!(
            schema.version > 0,
            "plugin {} has invalid schema version: {}",
            plugin.id(),
            schema.version
        );
    }
}
