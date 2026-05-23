use photopipeline_core::{
    ColorSpace, ExifData, GpsData, ImageFormat, ImageInfo, IntegerWidget, Metadata, PixelFormat,
};
use photopipeline_engine::ParameterResolver;
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType,
};
use serde_json::json;
use uuid::Uuid;

fn make_schema() -> ParameterSchema {
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
                    style: IntegerWidget::default(),
                },
                default: json!(128),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    }
}

fn make_schema_with_expression_support() -> ParameterSchema {
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
                    id: "label".into(),
                    label: "Label".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 256,
                        pattern: None,
                        placeholder: None,
                    },
                    default: json!("default"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: true,
                },
                ParameterField {
                    id: "quality".into(),
                    label: "Quality".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 100,
                        step: 1,
                        unit: None,
                        style: IntegerWidget::default(),
                    },
                    default: json!(50),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: true,
                },
            ],
        }],
    }
}

fn make_schema_with_locked_field() -> ParameterSchema {
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
                    id: "locked_val".into(),
                    label: "Locked".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 255,
                        step: 1,
                        unit: None,
                        style: IntegerWidget::default(),
                    },
                    default: json!(42),
                    required: false,
                    advanced: false,
                    allow_override: false,
                    supports_expression: false,
                },
                ParameterField {
                    id: "threshold".into(),
                    label: "Threshold".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 255,
                        step: 1,
                        unit: None,
                        style: IntegerWidget::default(),
                    },
                    default: json!(128),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        }],
    }
}

fn make_test_metadata(iso: u32) -> Metadata {
    Metadata {
        exif: Some(ExifData {
            iso: Some(iso),
            make: Some("Canon".into()),
            model: Some("EOS R5".into()),
            lens_model: Some("24-70mm".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_test_metadata_with_make(make: &str) -> Metadata {
    Metadata {
        exif: Some(ExifData {
            iso: Some(400),
            make: Some(make.into()),
            model: Some("EOS R5".into()),
            lens_model: Some("24-70mm".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_test_metadata_with_gps(lat: f64, lon: f64) -> Metadata {
    Metadata {
        exif: Some(ExifData {
            iso: Some(400),
            ..Default::default()
        }),
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

#[test]
fn e2e_plugin_default_only() {
    let resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();
    let metadata = Metadata::default();
    let image_info = make_test_image_info();

    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(128));
}

#[test]
fn e2e_template_overrides_default() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), json!(200));
    resolver.set_template_params(node_id, template_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(200));
}

#[test]
fn e2e_group_overrides_template() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), json!(200));
    resolver.set_template_params(node_id, template_params);

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(150));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::Always,
        node_map,
    );

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(150));
}

#[test]
fn e2e_image_overrides_group() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), json!(200));
    resolver.set_template_params(node_id, template_params);

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(150));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::Always,
        node_map,
    );

    let mut image_params = ParameterSet::new();
    image_params.insert("threshold".into(), json!(99));
    resolver.set_image_override(image_id, node_id, image_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(99));
}

#[test]
fn e2e_image_overrides_all() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), json!(10));
    resolver.set_template_params(node_id, template_params);

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(20));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::Always,
        node_map,
    );

    let mut image_params = ParameterSet::new();
    image_params.insert("threshold".into(), json!(30));
    resolver.set_image_override(image_id, node_id, image_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(30));
}

#[test]
fn e2e_group_condition_no_match_falls_through() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("threshold".into(), json!(77));
    resolver.set_template_params(node_id, template_params);

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(255));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Nikon".into(),
        },
        node_map,
    );

    let metadata = make_test_metadata_with_make("Canon");
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(77));
}

#[test]
fn e2e_allow_override_false_blocks_override() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema_with_locked_field();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("locked_val".into(), json!(100));
    resolver.set_template_params(node_id, template_params);

    let mut image_params = ParameterSet::new();
    image_params.insert("locked_val".into(), json!(200));
    resolver.set_image_override(image_id, node_id, image_params);

    let metadata = Metadata::default();
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("locked_val"), Some(100));
}

#[test]
fn e2e_expression_in_params_resolved() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema_with_expression_support();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut template_params = ParameterSet::new();
    template_params.insert("label".into(), json!("${exif.make} ${exif.model}"));
    resolver.set_template_params(node_id, template_params);

    let metadata = make_test_metadata(400);
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_str("label"), Some("Canon EOS R5"));
}

#[test]
fn e2e_group_exif_eq_matches() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(255));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Canon".into(),
        },
        node_map,
    );

    let metadata = make_test_metadata_with_make("Canon");
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(255));
}

#[test]
fn e2e_group_exif_eq_no_match() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(255));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Nikon".into(),
        },
        node_map,
    );

    let metadata = make_test_metadata_with_make("Canon");
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(128));
}

#[test]
fn e2e_group_exif_gte_matches() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(255));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::ExifGte {
            tag: "iso".into(),
            value: 400.0,
        },
        node_map,
    );

    let metadata = make_test_metadata(800);
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(255));
}

#[test]
fn e2e_group_gps_near_within_radius() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(255));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::GpsNear {
            lat: 39.90,
            lon: 116.40,
            radius_km: 10.0,
        },
        node_map,
    );

    let metadata = make_test_metadata_with_gps(39.9042, 116.4074);
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(255));
}

#[test]
fn e2e_group_gps_near_outside_radius() {
    let mut resolver = ParameterResolver::new();
    let schema = make_schema();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut group_params = ParameterSet::new();
    group_params.insert("threshold".into(), json!(255));
    let mut node_map = std::collections::HashMap::new();
    node_map.insert(node_id, group_params);
    resolver.add_group_override(
        photopipeline_engine::GroupCondition::GpsNear {
            lat: 39.90,
            lon: 116.40,
            radius_km: 0.001,
        },
        node_map,
    );

    let metadata = make_test_metadata_with_gps(39.9042, 116.4074);
    let image_info = make_test_image_info();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
    assert_eq!(result.get_i64("threshold"), Some(128));
}

#[test]
fn e2e_expression_simple_variable_substitution() {
    use photopipeline_engine::ExpressionEngine;

    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(400);
    let image_info = make_test_image_info();
    let result = engine
        .evaluate("${exif.make}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, json!("Canon"));
}

#[test]
fn e2e_expression_ternary_evaluation() {
    use photopipeline_engine::ExpressionEngine;

    let engine = ExpressionEngine::default();

    let metadata = make_test_metadata(800);
    let image_info = make_test_image_info();
    let result = engine
        .evaluate("${exif.iso >= 400 ? 'high' : 'low'}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result, json!("high"));

    let metadata_low = make_test_metadata(100);
    let result_low = engine
        .evaluate(
            "${exif.iso >= 400 ? 'high' : 'low'}",
            &metadata_low,
            &image_info,
        )
        .unwrap();
    assert_eq!(result_low, json!("low"));
}

#[test]
fn e2e_expression_comparison_operators() {
    use photopipeline_engine::ExpressionEngine;

    let engine = ExpressionEngine::default();
    let metadata = make_test_metadata(400);
    let image_info = make_test_image_info();

    let result_eq = engine
        .evaluate("${exif.iso == 400}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result_eq, json!("true"));

    let result_ne = engine
        .evaluate("${exif.iso != 800}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result_ne, json!("true"));

    let result_lt = engine
        .evaluate("${exif.iso < 800}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result_lt, json!("true"));

    let result_gt = engine
        .evaluate("${exif.iso > 100}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result_gt, json!("true"));

    let result_ge = engine
        .evaluate("${exif.iso >= 400}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result_ge, json!("true"));

    let result_le = engine
        .evaluate("${exif.iso <= 400}", &metadata, &image_info)
        .unwrap();
    assert_eq!(result_le, json!("true"));
}
