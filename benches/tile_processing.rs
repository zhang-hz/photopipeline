use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use photopipeline_core::{ChannelLayout, PixelBuffer, PixelFormat, TileLayout};
use photopipeline_engine::TileEngine;

fn make_filled_buffer(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGBA, PixelFormat::U8);
    for (i, byte) in buf.data.data.iter_mut().enumerate() {
        *byte = (i % 251) as u8;
    }
    buf
}

fn bench_tile_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("tile_splitting");

    let image_sizes: [(u32, u32); 3] = [(1920, 1080), (4096, 2160), (8192, 4320)];

    for &(img_w, img_h) in &image_sizes {
        for &tile_size in &[64u32, 128, 256, 512, 1024] {
            let name = format!("img_{img_w}x{img_h}_tile_{tile_size}");

            group.bench_with_input(
                BenchmarkId::new("layout_new", &name),
                &(img_w, img_h, tile_size),
                |b, &(w, h, ts)| {
                    b.iter(|| {
                        let layout = TileLayout::new(black_box(w), black_box(h), black_box(ts), 64);
                        black_box(layout)
                    });
                },
            );

            let layout = TileLayout::new(img_w, img_h, tile_size, 64);

            group.bench_with_input(
                BenchmarkId::new("iter_tiles", &name),
                &(img_w, img_h, tile_size),
                |b, _| {
                    b.iter(|| {
                        let spec_count = black_box(&layout).iter_tiles().count();
                        black_box(spec_count)
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_tile_processing_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("tile_processing_throughput");

    let image_sizes: [(u32, u32); 3] = [(1920, 1080), (3840, 2160), (7680, 4320)];

    for &(img_w, img_h) in &image_sizes {
        let _buffer = make_filled_buffer(img_w, img_h);
        let name = format!("{img_w}x{img_h}");

        for &tile_size in &[256u32, 512, 1024] {
            for &overlap in &[0u32, 64] {
                let bench_name = format!("{name}_tile_{tile_size}_ov_{overlap}");

                group.bench_with_input(
                    BenchmarkId::new("sequential", &bench_name),
                    &(img_w, img_h, tile_size, overlap),
                    |b, _| {
                        b.iter(|| {
                            let engine =
                                TileEngine::new(black_box(tile_size), black_box(overlap), 1);
                            let layout = TileLayout::new(
                                black_box(img_w),
                                black_box(img_h),
                                black_box(tile_size),
                                black_box(overlap),
                            );
                            let total = layout.total_tiles;
                            let tiles: Vec<_> = layout.iter_tiles().collect();
                            black_box((engine, tiles, total))
                        });
                    },
                );

                group.bench_with_input(
                    BenchmarkId::new("parallel_tile_layout", &bench_name),
                    &(img_w, img_h, tile_size, overlap),
                    |b, _| {
                        b.iter(|| {
                            let layout = TileLayout::new(
                                black_box(img_w),
                                black_box(img_h),
                                black_box(tile_size),
                                black_box(overlap),
                            );
                            let specs: Vec<_> = layout.iter_tiles().collect();
                            black_box(specs)
                        });
                    },
                );
            }
        }
    }

    group.finish();

    let mut group = c.benchmark_group("pixelbuffer_allocation");

    for &(w, h) in &[(256, 256), (1920, 1080), (4096, 2160)] {
        let name = format!("{w}x{h}_rgba");
        group.bench_with_input(BenchmarkId::new("u8", &name), &(w, h), |b, &(w, h)| {
            b.iter(|| {
                let buf = PixelBuffer::new(
                    black_box(w),
                    black_box(h),
                    ChannelLayout::RGBA,
                    PixelFormat::U8,
                );
                black_box(buf)
            });
        });

        group.bench_with_input(BenchmarkId::new("u16", &name), &(w, h), |b, &(w, h)| {
            b.iter(|| {
                let buf = PixelBuffer::new(
                    black_box(w),
                    black_box(h),
                    ChannelLayout::RGBA,
                    PixelFormat::U16,
                );
                black_box(buf)
            });
        });
    }

    group.finish();
}

fn bench_tile_engine_construction(c: &mut Criterion) {
    c.bench_function("tile_engine_default", |b| {
        b.iter(|| {
            let engine = TileEngine::default();
            black_box(engine)
        });
    });

    c.bench_function("tile_engine_custom_1024_64_4", |b| {
        b.iter(|| {
            let engine = TileEngine::new(black_box(1024), black_box(64), black_box(4));
            black_box(engine)
        });
    });
}

criterion_group!(
    benches,
    bench_tile_splitting,
    bench_tile_processing_throughput,
    bench_tile_engine_construction,
);
criterion_main!(benches);
