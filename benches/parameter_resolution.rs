use std::collections::HashMap;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use photopipeline_core::{ColorSpace, ExifData, ImageFormat, Metadata, PixelFormat};
use photopipeline_engine::{GroupCondition, ParameterResolver};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType,
};
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

fn make_multi_field_schema(num_fields: usize) -> ParameterSchema {
    let fields: Vec<ParameterField> = (0..num_fields)
        .map(|i| ParameterField {
            id: format!("field_{i}"),
            label: format!("Field {i}"),
            description: None,
            help_url: None,
            field_type: ParameterType::Integer {
                min: 0,
                max: 255,
                step: 1,
                unit: None,
                style: Default::default(),
            },
            default: serde_json::json!(i as i64),
            required: false,
            advanced: false,
            allow_override: true,
            supports_expression: false,
        })
        .collect();

    ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "main".into(),
            label: "Main".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields,
        }],
    }
}

fn make_metadata() -> Metadata {
    Metadata {
        exif: Some(ExifData {
            iso: Some(800),
            make: Some("Canon".into()),
            model: Some("EOS R5".into()),
            lens_model: Some("24-70mm".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_image_info() -> photopipeline_core::ImageInfo {
    photopipeline_core::ImageInfo {
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

fn bench_resolve_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_4_levels");

    let schema = make_schema();
    let metadata = make_metadata();
    let image_info = make_image_info();
    let node_id = Uuid::new_v4();
    let image_id = image_info.id;

    let combinations = [
        (0usize, 0usize),
        (10, 0),
        (50, 0),
        (100, 0),
        (0, 10),
        (0, 50),
        (0, 100),
        (10, 10),
        (50, 50),
        (100, 100),
    ];

    for &(num_groups, num_images) in &combinations {
        let mut resolver = ParameterResolver::new();

        for g in 0..num_groups {
            let mut params = ParameterSet::new();
            params.insert("threshold".into(), serde_json::json!(g as i64));
            let mut node_map = HashMap::new();
            node_map.insert(node_id, params);
            resolver.add_group_override(
                GroupCondition::ExifGte {
                    tag: "iso".into(),
                    value: (g * 10) as f64,
                },
                node_map,
            );
        }

        for i in 0..num_images {
            let mut params = ParameterSet::new();
            params.insert("threshold".into(), serde_json::json!(i as i64));
            resolver.set_image_override(Uuid::new_v4(), node_id, params);
        }

        let bench_name = format!("groups_{num_groups}_images_{num_images}");
        group.bench_with_input(
            BenchmarkId::new("resolve", &bench_name),
            &(num_groups, num_images),
            |b, _| {
                b.iter(|| {
                    let result = black_box(&resolver).resolve(
                        black_box(node_id),
                        black_box(image_id),
                        black_box(&schema),
                        black_box(&metadata),
                        black_box(&image_info),
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_condition_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("condition_evaluation");

    let metadata = make_metadata();
    let image_info = make_image_info();
    let resolver = ParameterResolver::new();

    fn build_and_tree(depth: usize) -> GroupCondition {
        if depth == 0 {
            GroupCondition::ExifGte {
                tag: "iso".into(),
                value: 400.0,
            }
        } else {
            let left = build_and_tree(depth - 1);
            let right = GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Canon".into(),
            };
            GroupCondition::And(vec![left, right])
        }
    }

    fn build_or_tree(depth: usize) -> GroupCondition {
        if depth == 0 {
            GroupCondition::ExifGte {
                tag: "iso".into(),
                value: 400.0,
            }
        } else {
            let left = build_or_tree(depth - 1);
            let right = GroupCondition::ExifEq {
                tag: "model".into(),
                value: "EOS R5".into(),
            };
            GroupCondition::Or(vec![left, right])
        }
    }

    for depth in [1usize, 5, 10, 20, 50] {
        let and_tree = build_and_tree(depth);
        let bench_name = format!("and_depth_{depth}");
        group.bench_with_input(BenchmarkId::new("and_tree", &bench_name), &depth, |b, _| {
            b.iter(|| {
                let result = black_box(&resolver).evaluate_condition(
                    black_box(&and_tree),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                black_box(result)
            });
        });

        let or_tree = build_or_tree(depth);
        let bench_name = format!("or_depth_{depth}");
        group.bench_with_input(BenchmarkId::new("or_tree", &bench_name), &depth, |b, _| {
            b.iter(|| {
                let result = black_box(&resolver).evaluate_condition(
                    black_box(&or_tree),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                black_box(result)
            });
        });

        let always_cond = GroupCondition::Always;
        group.bench_with_input(BenchmarkId::new("always", &bench_name), &depth, |b, _| {
            b.iter(|| {
                let result = black_box(&resolver).evaluate_condition(
                    black_box(&always_cond),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_deep_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("parameter_set_merge");

    for num_keys in [1usize, 10, 50, 100, 500] {
        let mut base = ParameterSet::new();
        let mut other = ParameterSet::new();

        for i in 0..num_keys {
            base.insert(format!("key_{i}"), serde_json::json!(i as i64));
            other.insert(format!("key_{i}"), serde_json::json!((i + 1) as i64));
        }

        let bench_name = format!("{num_keys}_keys");
        group.bench_with_input(BenchmarkId::new("merge", &bench_name), &num_keys, |b, _| {
            b.iter_custom(|iters| {
                let mut total = std::time::Duration::ZERO;
                for _ in 0..iters {
                    let mut to_merge = black_box(base.clone());
                    let start = std::time::Instant::now();
                    to_merge.merge(black_box(&other));
                    total += start.elapsed();
                    black_box(to_merge);
                }
                total
            });
        });
    }

    group.finish();
}

fn bench_resolve_plugin_defaults(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_plugin_defaults");

    for num_fields in [1usize, 10, 50, 100] {
        let schema = make_multi_field_schema(num_fields);
        let bench_name = format!("{num_fields}_fields");
        group.bench_with_input(
            BenchmarkId::new("defaults", &bench_name),
            &num_fields,
            |b, _| {
                b.iter(|| {
                    let defaults = black_box(&schema).defaults();
                    black_box(defaults)
                });
            },
        );
    }

    group.finish();
}

fn bench_expression_condition(c: &mut Criterion) {
    let mut group = c.benchmark_group("expression_condition");

    let metadata = make_metadata();
    let image_info = make_image_info();
    let resolver = ParameterResolver::new();

    let comparisons = [
        "${exif.iso > 400}",
        "${exif.iso >= 800}",
        "${exif.make == \"Canon\"}",
        "${exif.model != \"Nikon\"}",
        "${image.width > 1000}",
        "${exif.iso > 400 && exif.make == \"Canon\"}",
    ];

    for expr in comparisons {
        let cond = GroupCondition::Expression(expr.into());
        group.bench_with_input(BenchmarkId::new("eval", expr), &expr, |b, _| {
            b.iter(|| {
                let result = black_box(&resolver).evaluate_condition(
                    black_box(&cond),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                black_box(result)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_resolve_levels,
    bench_condition_evaluation,
    bench_deep_merge,
    bench_resolve_plugin_defaults,
    bench_expression_condition,
);
criterion_main!(benches);
