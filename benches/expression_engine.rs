use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use photopipeline_core::{ColorSpace, ExifData, ImageFormat, Metadata, PixelFormat};
use photopipeline_engine::ExpressionEngine;
use uuid::Uuid;

fn make_metadata(iso: u32, make: &str, model: &str) -> Metadata {
    Metadata {
        exif: Some(ExifData {
            iso: Some(iso),
            make: Some(make.into()),
            model: Some(model.into()),
            lens_model: Some("24-70mm".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_image_info(width: u32, height: u32) -> photopipeline_core::ImageInfo {
    photopipeline_core::ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/test.jpg".into(),
        filename: "test.jpg".into(),
        format: ImageFormat::JPEG,
        width,
        height,
        file_size_bytes: 102400,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

fn bench_simple_variable(c: &mut Criterion) {
    let engine = ExpressionEngine::default();
    let metadata = make_metadata(400, "Canon", "EOS R5");
    let image_info = make_image_info(1920, 1080);

    let mut group = c.benchmark_group("expression_simple_variable");

    let expressions = [
        ("exif_iso", "${exif.iso}"),
        ("exif_make", "${exif.make}"),
        ("exif_model", "${exif.model}"),
        ("exif_lens", "${exif.lens}"),
        ("image_filename", "${image.filename}"),
        ("image_width", "${image.width}"),
        ("image_height", "${image.height}"),
    ];

    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::new("evaluate", name), &expr, |b, _| {
            b.iter(|| {
                let result = engine.evaluate(
                    black_box(expr),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                black_box(result)
            });
        });
    }

    group.finish();

    let mut group = c.benchmark_group("expression_literal");
    let literals = ["${3.14}", "${\"hello\"}", "${'world'}"];
    for (i, lit) in literals.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("evaluate", format!("literal_{i}")),
            &lit,
            |b, _| {
                b.iter(|| {
                    let result = engine.evaluate(
                        black_box(lit),
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

fn bench_comparison(c: &mut Criterion) {
    let engine = ExpressionEngine::default();
    let metadata = make_metadata(800, "Canon", "EOS R5");
    let image_info = make_image_info(1920, 1080);

    let mut group = c.benchmark_group("expression_comparison");

    let comparisons = [
        ("gt_true", "${exif.iso > 400}", true),
        ("gte", "${exif.iso >= 800}", true),
        ("lt_false", "${exif.iso < 400}", false),
        ("lte", "${exif.iso <= 800}", true),
        ("eq_num", "${exif.iso == 800}", true),
        ("neq_num", "${exif.iso != 400}", true),
        ("eq_str", "${exif.make == \"Canon\"}", true),
        ("neq_str", "${exif.make != \"Nikon\"}", true),
    ];

    for (name, expr, _) in comparisons {
        group.bench_with_input(BenchmarkId::new("evaluate", name), &expr, |b, _| {
            b.iter(|| {
                let result = engine.evaluate(
                    black_box(expr),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_ternary(c: &mut Criterion) {
    let engine = ExpressionEngine::default();
    let metadata_high = make_metadata(1600, "Canon", "EOS R5");
    let metadata_low = make_metadata(100, "Canon", "EOS R5");
    let image_info = make_image_info(1920, 1080);

    let mut group = c.benchmark_group("expression_ternary");

    let ternaries = [
        (
            "simple_true",
            "${exif.iso > 400 ? \"high\" : \"low\"}",
            true,
        ),
        (
            "simple_false",
            "${exif.iso > 400 ? \"high\" : \"low\"}",
            false,
        ),
        ("numeric_true", "${exif.iso > 400 ? 100 : 200}", true),
        ("numeric_false", "${exif.iso > 400 ? 100 : 200}", false),
    ];

    for (name, expr, use_high) in ternaries {
        let md = if use_high {
            &metadata_high
        } else {
            &metadata_low
        };
        group.bench_with_input(BenchmarkId::new("evaluate", name), &expr, |b, _| {
            b.iter(|| {
                let result =
                    engine.evaluate(black_box(expr), black_box(md), black_box(&image_info));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_complex_expression(c: &mut Criterion) {
    let engine = ExpressionEngine::default();
    let metadata = make_metadata(2000, "Canon", "EOS R5");
    let image_info = make_image_info(1920, 1080);

    let mut group = c.benchmark_group("expression_complex");

    let nested_expr = "${exif.iso > 1600 ? 0.9 : exif.iso > 800 ? 0.7 : 0.4}";
    group.bench_function("nested_ternary", |b| {
        b.iter(|| {
            let result = engine.evaluate(
                black_box(nested_expr),
                black_box(&metadata),
                black_box(&image_info),
            );
            black_box(result)
        });
    });

    let multi_var_expr = "${exif.make} ${exif.model}";
    group.bench_function("multi_variable", |b| {
        b.iter(|| {
            let result = engine.evaluate(
                black_box(multi_var_expr),
                black_box(&metadata),
                black_box(&image_info),
            );
            black_box(result)
        });
    });

    let plain_text = "plain static text with no expressions";
    group.bench_function("plain_text_no_substitution", |b| {
        b.iter(|| {
            let result = engine.evaluate(
                black_box(plain_text),
                black_box(&metadata),
                black_box(&image_info),
            );
            black_box(result)
        });
    });

    let string_cmp_ternary = "${exif.make == \"Canon\" ? exif.model : \"unknown\"}";
    group.bench_function("string_comparison_ternary", |b| {
        b.iter(|| {
            let result = engine.evaluate(
                black_box(string_cmp_ternary),
                black_box(&metadata),
                black_box(&image_info),
            );
            black_box(result)
        });
    });

    let triple_var = "${exif.make} ${exif.model} ${exif.lens}";
    group.bench_function("triple_variable_substitution", |b| {
        b.iter(|| {
            let result = engine.evaluate(
                black_box(triple_var),
                black_box(&metadata),
                black_box(&image_info),
            );
            black_box(result)
        });
    });

    group.finish();
}

fn bench_throughput_1000_iterations(c: &mut Criterion) {
    let engine = ExpressionEngine::default();
    let metadata = make_metadata(800, "Canon", "EOS R5");
    let image_info = make_image_info(1920, 1080);

    let mut group = c.benchmark_group("expression_throughput");

    group.bench_function("simple_variable_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = engine.evaluate(
                    black_box("${exif.iso}"),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                let _ = black_box(result);
            }
            black_box(())
        });
    });

    group.bench_function("comparison_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = engine.evaluate(
                    black_box("${exif.iso > 400}"),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                let _ = black_box(result);
            }
            black_box(())
        });
    });

    group.bench_function("ternary_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = engine.evaluate(
                    black_box("${exif.iso > 400 ? \"high\" : \"low\"}"),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                let _ = black_box(result);
            }
            black_box(())
        });
    });

    group.bench_function("nested_ternary_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = engine.evaluate(
                    black_box("${exif.iso > 1600 ? 0.9 : exif.iso > 800 ? 0.7 : 0.4}"),
                    black_box(&metadata),
                    black_box(&image_info),
                );
                let _ = black_box(result);
            }
            black_box(())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_variable,
    bench_comparison,
    bench_ternary,
    bench_complex_expression,
    bench_throughput_1000_iterations,
);
criterion_main!(benches);
