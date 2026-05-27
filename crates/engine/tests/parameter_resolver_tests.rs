// Engine ParameterResolver Tests (~30 test cases)
// Tests four-layer priority: default < template < group < image override.
// Also tests expressions, conditions, and resolve behavior.

use photopipeline_core::{
    ColorSpace, ExifData, GpsData, ImageFormat, ImageInfo, Metadata, PixelFormat,
};
use photopipeline_engine::{ExpressionEngine, GroupCondition, ParameterResolver};
use photopipeline_plugin::{ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType};
use std::collections::HashMap;
use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────────────

fn make_simple_schema() -> ParameterSchema {
    ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "main".into(),
            label: "Main".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "threshold".into(),
                label: "Threshold".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 255,
                    step: 1,
                    unit: None,
                    style: Default::default(),
                },
                default: serde_json::json!(128),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    }
}

fn make_multi_field_schema() -> ParameterSchema {
    ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "main".into(),
            label: "Main".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "strength".into(),
                    label: "Strength".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 100,
                        step: 1,
                        unit: Some("%".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(50),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "enabled".into(),
                    label: "Enable".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("On".into()),
                        label_false: Some("Off".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "name".into(),
                    label: "Name".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 100,
                        pattern: None,
                        placeholder: None,
                    },
                    default: serde_json::json!("default"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "factor".into(),
                    label: "Factor".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: 0.0,
                        max: 10.0,
                        step: 0.1,
                        precision: 2,
                        unit: None,
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(1.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        }],
    }
}

fn make_test_metadata(iso: u32, make: &str) -> Metadata {
    Metadata {
        exif: Some(ExifData {
            iso: Some(iso),
            make: Some(make.to_string()),
            model: Some("EOS R5".into()),
            lens_model: Some("24-70mm".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_test_gps_metadata(lat: f64, lon: f64) -> Metadata {
    Metadata {
        gps: Some(GpsData {
            latitude: Some(lat),
            longitude: Some(lon),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_test_image_info() -> ImageInfo {
    ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/test.jpg".into(),
        filename: "test.jpg".into(),
        format: ImageFormat::JPEG,
        width: 1920,
        height: 1080,
        file_size_bytes: 102400,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

// ── Layer 1: Default Layer ──────────────────────────────────────────

#[test]
fn default_layer_wins_when_no_override() {
    let resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let result = resolver.resolve_single(node_id, &schema);
    assert_eq!(result.get_i64("threshold"), Some(128));
}

#[test]
fn default_layer_multi_fields() {
    let resolver = ParameterResolver::new();
    let schema = make_multi_field_schema();
    let node_id = Uuid::new_v4();
    let result = resolver.resolve_single(node_id, &schema);
    assert_eq!(result.get_i64("strength"), Some(50));
    assert_eq!(result.get_i64("factor"), None); // factor is f64 in json, not i64
    let factor_val = result.values.get("factor").and_then(|v| v.as_f64());
    assert!((factor_val.unwrap() - 1.0).abs() < 0.001);
}

// ── Layer 2: Template Overrides Default ─────────────────────────────

#[test]
fn template_overrides_default() {
    let mut resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();

    let mut params = ParameterSet::new();
    params.insert("threshold".into(), serde_json::json!(200));
    resolver.set_template_params(node_id, params);

    let result = resolver.resolve_single(node_id, &schema);
    assert_eq!(result.get_i64("threshold"), Some(200));
}

// ── Layer 3: Group Overrides Template ───────────────────────────────

#[test]
fn group_overrides_template() {
    let mut resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), serde_json::json!(200));
    resolver.set_template_params(node_id, template_params);

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), serde_json::json!(150));
    let mut node_map = HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(GroupCondition::Always, node_map);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(150));
}

// ── Layer 4: Image Override Wins ─────────────────────────────────────

#[test]
fn image_override_wins_highest_priority() {
    let mut resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), serde_json::json!(200));
    resolver.set_template_params(node_id, template_params);

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), serde_json::json!(150));
    let mut node_map = HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(GroupCondition::Always, node_map);

    let mut image_params = ParameterSet::new();
    image_params.insert("threshold".into(), serde_json::json!(10));
    resolver.set_image_override(image_id, node_id, image_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(10));
}

#[test]
fn image_override_wins_over_all_with_group_unmatched() {
    // Image override should win even when group condition does NOT match
    let mut resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), serde_json::json!(255));
    let mut node_map = HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Nikon".into(),
        },
        node_map,
    );

    let mut image_params = ParameterSet::new();
    image_params.insert("threshold".into(), serde_json::json!(1));
    resolver.set_image_override(image_id, node_id, image_params);

    let metadata = make_test_metadata(400, "Canon");
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(1));
}

// ── Allow Override False Blocks Higher Layer ────────────────────────

#[test]
fn allow_override_false_blocks_group() {
    // Note: The ParameterResolver's priority implementation already handles
    // `allow_override` via template_snapshot. This test verifies the behavior
    // by using a schema with allow_override=false.
    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "main".into(),
            label: "Main".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "locked".into(),
                label: "Locked".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::String {
                    max_length: 100,
                    pattern: None,
                    placeholder: None,
                },
                default: serde_json::json!("default_val"),
                required: false,
                advanced: false,
                allow_override: false,
                supports_expression: false,
            }],
        }],
    };

    let mut resolver = ParameterResolver::new();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("locked".into(), serde_json::json!("template_val"));
    resolver.set_template_params(node_id, template_params);

    // image override
    let mut image_params = ParameterSet::new();
    image_params.insert("locked".into(), serde_json::json!("image_val"));
    resolver.set_image_override(image_id, node_id, image_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);

    // The allow_override=false block is implemented via template_snapshot.
    // The result should NOT be the image override value.
    let val = result.values.get("locked").and_then(|v| v.as_str());
    assert_ne!(
        val,
        Some("image_val"),
        "image override should be blocked by allow_override=false"
    );
}

// ── Expression Evaluation ───────────────────────────────────────────

#[test]
fn expression_variable_substitution_exif() {
    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let result = engine.evaluate("${exif.iso}", &metadata, &image_info).unwrap();
    // NOTE: expression evaluation currently returns String for numeric EXIF values.
    // Consider whether this should return Number(800) for type correctness.
    assert_eq!(result, serde_json::Value::String("800".into()));
}

#[test]
fn expression_variable_substitution_image() {
    let engine = ExpressionEngine::default();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = engine.evaluate("${image.width}", &metadata, &image_info).unwrap();
    assert_eq!(result, serde_json::Value::String("1920".into()));
}

#[test]
fn expression_ternary_true_branch() {
    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let result = engine
        .evaluate("${exif.iso >= 400 ? 'high' : 'low'}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, serde_json::Value::String("high".into()));
}

#[test]
fn expression_ternary_false_branch() {
    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(200, "Canon");
    let image_info = make_test_image_info();
    let result = engine
        .evaluate("${exif.iso >= 400 ? 'high' : 'low'}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, serde_json::Value::String("low".into()));
}

#[test]
fn expression_comparison_gt() {
    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    // width > 1000
    let result = engine
        .evaluate("${image.width > 1000 ? 'large' : 'small'}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, serde_json::Value::String("large".into()));

    // width > 5000
    let result = engine
        .evaluate("${image.width > 5000 ? 'large' : 'small'}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, serde_json::Value::String("small".into()));
}

#[test]
fn expression_string_concat() {
    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    // Note: the engine does simple ${} replacement, not full concat.
    // We test multiple variables together.
    let result = engine
        .evaluate("${exif.make} ${exif.model}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, serde_json::Value::String("Canon EOS R5".into()));
}

#[test]
fn expression_undefined_variable_error() {
    let engine = ExpressionEngine::default();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = engine.evaluate("${nonexistent.var}", &metadata, &image_info);
    assert!(result.is_err(), "undefined variable must produce error");
}

#[test]
fn expression_unknown_exif_field_error() {
    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(100, "Canon");
    let image_info = make_test_image_info();
    let result = engine.evaluate("${exif.unknown_field}", &metadata, &image_info);
    assert!(result.is_err(), "unknown exif field must produce error");
}

#[test]
fn expression_malformed_syntax_graceful() {
    let engine = ExpressionEngine::default();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = engine.evaluate("${exif.make", &metadata, &image_info);
    // Malformed expression must either return Ok (with unchanged output)
    // or Err — must not panic.
    assert!(result.is_ok() || result.is_err(),
        "malformed expression must not panic");
}

// ── Group Conditions ────────────────────────────────────────────────

#[test]
fn condition_exif_eq_match() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let cond = GroupCondition::ExifEq {
        tag: "make".into(),
        value: "Canon".into(),
    };
    assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_exif_eq_no_match() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let cond = GroupCondition::ExifEq {
        tag: "make".into(),
        value: "Sony".into(),
    };
    assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_exif_gte_match() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let cond = GroupCondition::ExifGte {
        tag: "iso".into(),
        value: 400.0,
    };
    assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_exif_lte_no_match() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let cond = GroupCondition::ExifLte {
        tag: "iso".into(),
        value: 400.0,
    };
    assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_gps_near_within_radius() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_gps_metadata(34.0522, -118.2437);
    let image_info = make_test_image_info();
    let cond = GroupCondition::GpsNear {
        lat: 34.05,
        lon: -118.24,
        radius_km: 10.0,
    };
    assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_gps_near_outside_radius() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_gps_metadata(34.0522, -118.2437);
    let image_info = make_test_image_info();
    let cond = GroupCondition::GpsNear {
        lat: 40.71,
        lon: -74.00,
        radius_km: 1.0,
    };
    assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_and_both_true() {
    let resolver = ParameterResolver::new();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let cond = GroupCondition::And(vec![GroupCondition::Always, GroupCondition::Always]);
    assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_and_one_false() {
    let resolver = ParameterResolver::new();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let cond = GroupCondition::And(vec![
        GroupCondition::Always,
        GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Canon".into(),
        },
    ]);
    assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_or_one_true() {
    let resolver = ParameterResolver::new();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let cond = GroupCondition::Or(vec![
        GroupCondition::Always,
        GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Nikon".into(),
        },
    ]);
    assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_or_all_false() {
    let resolver = ParameterResolver::new();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let cond = GroupCondition::Or(vec![
        GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Canon".into(),
        },
        GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Nikon".into(),
        },
    ]);
    assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_expression_evaluates_true() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_metadata(800, "Canon");
    let image_info = make_test_image_info();
    let cond = GroupCondition::Expression("${exif.iso > 400}".into());
    assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
}

#[test]
fn condition_expression_evaluates_false() {
    let resolver = ParameterResolver::new();
    let metadata = make_test_metadata(100, "Canon");
    let image_info = make_test_image_info();
    let cond = GroupCondition::Expression("${exif.iso > 400}".into());
    assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
}

// ── Resolver Fall-through Tests ─────────────────────────────────────

#[test]
fn resolve_missing_image_override_falls_through_to_group() {
    let mut resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), serde_json::json!(77));
    let mut node_map = HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(GroupCondition::Always, node_map);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(77));
}

#[test]
fn resolve_missing_group_falls_through_to_template() {
    let mut resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), serde_json::json!(42));
    resolver.set_template_params(node_id, template_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(42));
}

#[test]
fn resolve_missing_all_uses_plugin_default() {
    let resolver = ParameterResolver::new();
    let schema = make_simple_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(128));
}

// ── Expression Edge Cases ───────────────────────────────────────────

#[test]
fn expression_literal_number() {
    let engine = ExpressionEngine::default();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = engine.evaluate("${3.14}", &metadata, &image_info).unwrap();
    assert_eq!(result, serde_json::Value::String("3.14".into()));
}

#[test]
fn expression_quoted_string() {
    let engine = ExpressionEngine::default();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = engine.evaluate("${\"hello\"}", &metadata, &image_info).unwrap();
    assert_eq!(result, serde_json::Value::String("hello".into()));
}

#[test]
fn expression_plain_text_no_dollar() {
    let engine = ExpressionEngine::default();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = engine.evaluate("plain text", &metadata, &image_info).unwrap();
    assert_eq!(result, serde_json::Value::String("plain text".into()));
}

// ── Mixed Types ─────────────────────────────────────────────────────

#[test]
fn resolve_mixed_types_all_correct() {
    let schema = make_multi_field_schema();
    let mut resolver = ParameterResolver::new();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("strength".into(), serde_json::json!(75));
    template_params.insert("name".into(), serde_json::json!("custom_name"));
    template_params.insert("factor".into(), serde_json::json!(2.5));
    resolver.set_template_params(node_id, template_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);

    assert_eq!(result.get_i64("strength"), Some(75)); // Integer
    assert_eq!(result.get_str("name"), Some("custom_name")); // String
    // Boolean default
    let enabled = result.values.get("enabled").and_then(|v| v.as_bool());
    assert_eq!(enabled, Some(false));
    // Float
    let factor = result.values.get("factor").and_then(|v| v.as_f64());
    assert!((factor.unwrap() - 2.5).abs() < 0.001);
}
