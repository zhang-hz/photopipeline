use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use photopipeline_core::{
    ChannelLayout, EncodeOptions, ImageFormat, Metadata, PixelBuffer, PixelFormat,
};
use photopipeline_plugin::FormatProcessor;
use photopipeline_plugins::png_encoder::PngEncoderPlugin;
use photopipeline_plugins::tiff_encoder::TiffEncoderPlugin;

fn make_buffer(width: u32, height: u32, layout: ChannelLayout, format: PixelFormat) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, layout, format);
    for (i, byte) in buf.data.data.iter_mut().enumerate() {
        *byte = (i % 251) as u8;
    }
    buf
}

fn bench_png_encode(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let encoder = PngEncoderPlugin::new();

    let mut group = c.benchmark_group("png_encode");

    let sizes_4320p: [(u32, u32); 3] = [(256, 256), (1920, 1080), (3840, 2160)];

    for &(w, h) in &sizes_4320p {
        let buffer_u8 = make_buffer(w, h, ChannelLayout::RGBA, PixelFormat::U8);
        let buffer_u16 = make_buffer(w, h, ChannelLayout::RGBA, PixelFormat::U16);

        let name = format!("{w}x{h}");
        let options_u8 = EncodeOptions {
            format: ImageFormat::PNG,
            bit_depth: 8,
            lossless: true,
            ..Default::default()
        };
        let options_u16 = EncodeOptions {
            format: ImageFormat::PNG,
            bit_depth: 16,
            lossless: true,
            ..Default::default()
        };

        group.bench_with_input(BenchmarkId::new("u8", &name), &(w, h), |b, _| {
            b.iter(|| {
                let result = rt.block_on(encoder.encode(
                    black_box(&buffer_u8),
                    black_box(&Metadata::default()),
                    black_box(&options_u8),
                ));
                black_box(result)
            });
        });

        group.bench_with_input(BenchmarkId::new("u16", &name), &(w, h), |b, _| {
            b.iter(|| {
                let result = rt.block_on(encoder.encode(
                    black_box(&buffer_u16),
                    black_box(&Metadata::default()),
                    black_box(&options_u16),
                ));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_tiff_encode(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let encoder = TiffEncoderPlugin::new();

    let mut group = c.benchmark_group("tiff_encode");

    let sizes_4k: [(u32, u32); 3] = [(256, 256), (1920, 1080), (4096, 2160)];

    for &(w, h) in &sizes_4k {
        let buffer_u8 = make_buffer(w, h, ChannelLayout::RGBA, PixelFormat::U8);
        let buffer_u16 = make_buffer(w, h, ChannelLayout::RGBA, PixelFormat::U16);
        let buffer_f32 = make_buffer(w, h, ChannelLayout::RGBA, PixelFormat::F32);

        let name = format!("{w}x{h}");

        for &(pixel_format_name, buffer) in &[
            ("u8", &buffer_u8),
            ("u16", &buffer_u16),
            ("f32", &buffer_f32),
        ] {
            let options = EncodeOptions {
                format: ImageFormat::TIFF,
                lossless: true,
                ..Default::default()
            };
            group.bench_with_input(
                BenchmarkId::new(pixel_format_name, &name),
                &(w, h),
                |b, _| {
                    b.iter(|| {
                        let result = rt.block_on(encoder.encode(
                            black_box(buffer),
                            black_box(&Metadata::default()),
                            black_box(&options),
                        ));
                        black_box(result)
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_heif_encode_prepare(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let encoder = photopipeline_plugins::heif_encoder::HeifEncoderPlugin::new();

    let mut group = c.benchmark_group("heif_encode_prepare");

    let sizes: [(u32, u32); 3] = [(256, 256), (1920, 1080), (3840, 2160)];

    for &(w, h) in &sizes {
        let buffer = make_buffer(w, h, ChannelLayout::RGB, PixelFormat::U8);
        let name = format!("{w}x{h}");
        let options = EncodeOptions {
            format: ImageFormat::HEIF,
            quality: Some(95.0),
            bit_depth: 10,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::new("encode_prepare", &name),
            &(w, h),
            |b, _| {
                b.iter(|| {
                    let result = rt.block_on(encoder.encode(
                        black_box(&buffer),
                        black_box(&Metadata::default()),
                        black_box(&options),
                    ));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_png_encode,
    bench_tiff_encode,
    bench_heif_encode_prepare
);
criterion_main!(benches);
