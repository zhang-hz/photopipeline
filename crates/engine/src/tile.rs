use photopipeline_core::{PixelBuffer, PluginError, PluginResult, TileLayout};
use photopipeline_plugin::{ParameterSet, PixelProcessor, ProgressSink};

#[derive(Debug, Clone)]
pub struct TileEngine {
    pub default_tile_size: u32,
    pub overlap: u32,
    pub max_parallel: usize,
}

struct TileResult {
    spec: photopipeline_core::TileSpec,
    data: Vec<u8>,
}

impl TileEngine {
    pub fn new(default_tile_size: u32, overlap: u32, max_parallel: usize) -> Self {
        Self {
            default_tile_size,
            overlap,
            max_parallel,
        }
    }

    pub async fn process_tiled(
        &self,
        processor: &dyn PixelProcessor,
        input: &PixelBuffer,
        params: &ParameterSet,
        progress: &dyn ProgressSink,
    ) -> PluginResult<PixelBuffer> {
        let layout = TileLayout::new(
            input.width,
            input.height,
            self.default_tile_size,
            self.overlap,
        );

        let tile_count = layout.total_tiles;

        let mut output = PixelBuffer::new(
            input.width,
            input.height,
            input.layout,
            input.format,
        );
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        let tile_specs: Vec<_> = layout.iter_tiles().collect();
        let mut results: Vec<TileResult> = Vec::new();

        for (i, spec) in tile_specs.iter().enumerate() {
            if progress.is_canceled() {
                return Err(PluginError::Canceled {
                    plugin: processor.id().clone(),
                });
            }

            let fraction = (i as f32) / (tile_count.max(1) as f32);
            progress.set_progress(fraction, &format!("processing tile {}/{}", i + 1, tile_count));

            let mut tile_input = PixelBuffer::new(
                spec.width,
                spec.height,
                input.layout,
                input.format,
            );
            tile_input.color_space = input.color_space.clone();
            tile_input.icc_profile = input.icc_profile.clone();

            self.copy_tile_from_source(input, &mut tile_input, spec);

            let mut tile_output = PixelBuffer::new(
                spec.width,
                spec.height,
                input.layout,
                input.format,
            );
            tile_output.color_space = input.color_space.clone();
            tile_output.icc_profile = input.icc_profile.clone();

            let boxed_progress: Box<dyn ProgressSink> =
                Box::new(TileProgressSink {
                    _current: i,
                    _total: tile_count as usize,
                    _inner: tile_count as usize,
                });

            let _stats = processor
                .process_pixels(&tile_input, &mut tile_output, params, boxed_progress)
                .await?;

            results.push(TileResult {
                spec: spec.clone(),
                data: tile_output.data.data.clone(),
            });
        }

        for result in &results {
            self.blit_tile_to_output(&mut output, result);
        }

        progress.set_progress(1.0, "tiles complete");
        Ok(output)
    }

    fn copy_tile_from_source(
        &self,
        source: &PixelBuffer,
        dest: &mut PixelBuffer,
        spec: &photopipeline_core::TileSpec,
    ) {
        let channels = source.layout.channel_count() as usize;
        let bpc = source.format.bytes_per_channel();
        let src_stride = source.width as usize * channels * bpc;
        let dst_stride = spec.width as usize * channels * bpc;

        for row in 0..spec.height as usize {
            let src_offset = ((spec.y_offset as usize + row) * src_stride)
                + (spec.x_offset as usize * channels * bpc);
            let dst_offset = row * dst_stride;

            if src_offset + dst_stride <= source.data.data.len()
                && dst_offset + dst_stride <= dest.data.data.len()
            {
                dest.data.data[dst_offset..dst_offset + dst_stride]
                    .copy_from_slice(
                        &source.data.data[src_offset..src_offset + dst_stride],
                    );
            }
        }
    }

    fn blit_tile_to_output(&self, output: &mut PixelBuffer, result: &TileResult) {
        let channels = output.layout.channel_count() as usize;
        let bpc = output.format.bytes_per_channel();
        let out_stride = output.width as usize * channels * bpc;
        let tile_stride = result.spec.width as usize * channels * bpc;

        for row in 0..result.spec.height as usize {
            let dst_offset = ((result.spec.y_offset as usize + row) * out_stride)
                + (result.spec.x_offset as usize * channels * bpc);
            let src_offset = row * tile_stride;

            if dst_offset + tile_stride <= output.data.data.len()
                && src_offset + tile_stride <= result.data.len()
            {
                output.data.data[dst_offset..dst_offset + tile_stride]
                    .copy_from_slice(&result.data[src_offset..src_offset + tile_stride]);
            }
        }
    }
}

impl Default for TileEngine {
    fn default() -> Self {
        Self {
            default_tile_size: 1024,
            overlap: 64,
            max_parallel: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
        }
    }
}

struct TileProgressSink {
    _current: usize,
    _total: usize,
    _inner: usize,
}

impl ProgressSink for TileProgressSink {
    fn set_progress(&self, _fraction: f32, _message: &str) {}

    fn is_canceled(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_engine_construction() {
        let engine = TileEngine::new(512, 32, 4);
        assert_eq!(engine.default_tile_size, 512);
        assert_eq!(engine.overlap, 32);
        assert_eq!(engine.max_parallel, 4);
    }

    #[test]
    fn tile_engine_default() {
        let engine = TileEngine::default();
        assert_eq!(engine.default_tile_size, 1024);
        assert_eq!(engine.overlap, 64);
        assert!(engine.max_parallel >= 1);
    }

    #[test]
    fn tile_engine_clone() {
        let engine = TileEngine::new(1024, 64, 2);
        let cloned = engine.clone();
        assert_eq!(cloned.default_tile_size, engine.default_tile_size);
        assert_eq!(cloned.overlap, engine.overlap);
        assert_eq!(cloned.max_parallel, engine.max_parallel);
    }

    #[test]
    fn tile_layout_for_tile_engine_params() {
        let layout = photopipeline_core::TileLayout::new(1920, 1080, 512, 64);
        let tiles: Vec<_> = layout.iter_tiles().collect();
        assert!(!tiles.is_empty());
        assert_eq!(tiles.len() as u32, layout.total_tiles);
    }

    #[test]
    fn tile_counts_various_sizes() {
        let layout = photopipeline_core::TileLayout::new(256, 256, 256, 0);
        assert_eq!(layout.total_tiles, 1);

        let layout = photopipeline_core::TileLayout::new(512, 512, 256, 0);
        assert_eq!(layout.total_tiles, 4);

        let layout = photopipeline_core::TileLayout::new(800, 600, 256, 0);
        assert_eq!(layout.total_tiles, 12);
    }

    #[test]
    fn tile_boundaries_within_image() {
        let layout = photopipeline_core::TileLayout::new(1920, 1080, 512, 0);
        for spec in layout.iter_tiles() {
            assert!(spec.x_offset < 1920);
            assert!(spec.y_offset < 1080);
            assert!(spec.x_offset + spec.width <= 1920);
            assert!(spec.y_offset + spec.height <= 1080);
        }
    }
}
