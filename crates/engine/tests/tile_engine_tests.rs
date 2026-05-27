// Engine TileEngine Tests (~25 test cases)
// Tests tile splitting, overlap, boundary handling, counting,
// tile spec correctness, large images, and edge cases.

use photopipeline_core::TileLayout;

// ── Single Tile / Exact Fit ─────────────────────────────────────────

#[test]
fn single_tile_equals_full_image() {
    // Image fits within one tile
    let layout = TileLayout::new(512, 512, 1024, 0);
    assert_eq!(layout.total_tiles, 1);
    assert_eq!(layout.tiles_x, 1);
    assert_eq!(layout.tiles_y, 1);
}

#[test]
fn tile_smaller_than_image_exact_factor_2() {
    let layout = TileLayout::new(2048, 1024, 1024, 0);
    assert_eq!(layout.tiles_x, 2);
    assert_eq!(layout.tiles_y, 1);
    assert_eq!(layout.total_tiles, 2);
}

#[test]
fn two_tiles_vertical() {
    let layout = TileLayout::new(1024, 2048, 1024, 0);
    assert_eq!(layout.tiles_x, 1);
    assert_eq!(layout.tiles_y, 2);
    assert_eq!(layout.total_tiles, 2);
}

#[test]
fn four_tiles_2x2_grid() {
    let layout = TileLayout::new(2048, 2048, 1024, 0);
    assert_eq!(layout.tiles_x, 2);
    assert_eq!(layout.tiles_y, 2);
    assert_eq!(layout.total_tiles, 4);
}

// ── Tile Spec Tests ─────────────────────────────────────────────────

#[test]
fn tile_spec_first_tile_starts_at_origin() {
    let layout = TileLayout::new(1920, 1080, 512, 0);
    let spec = layout.tile_spec(0, 0);
    assert_eq!(spec.x_offset, 0);
    assert_eq!(spec.y_offset, 0);
    assert_eq!(spec.width, 512);
    assert_eq!(spec.height, 512);
}

#[test]
fn tile_spec_last_tile_may_be_smaller() {
    let layout = TileLayout::new(1500, 1500, 1024, 0);
    let last_x = layout.tiles_x - 1;
    let last_y = layout.tiles_y - 1;
    let spec = layout.tile_spec(last_x, last_y);
    // Last tile should be clamped to image boundaries
    assert!(spec.x_offset + spec.width <= 1500);
    assert!(spec.y_offset + spec.height <= 1500);
    assert!(spec.width <= 1024);
    assert!(spec.height <= 1024);
    // For 1500 with tile 1024: tiles_x = ceil(1500/1024) = 2
    // Last tile x_offset = 1024, width = 1500-1024 = 476
    assert_eq!(spec.x_offset, 1024);
    assert_eq!(spec.width, 476);
    assert_eq!(spec.y_offset, 1024);
    assert_eq!(spec.height, 476);
}

#[test]
fn all_tiles_within_bounds() {
    let layout = TileLayout::new(1920, 1080, 512, 0);
    for ty in 0..layout.tiles_y {
        for tx in 0..layout.tiles_x {
            let spec = layout.tile_spec(tx, ty);
            assert!(spec.x_offset < 1920, "x_offset out of range");
            assert!(spec.y_offset < 1080, "y_offset out of range");
            assert!(
                spec.x_offset + spec.width <= 1920,
                "tile extends past image width"
            );
            assert!(
                spec.y_offset + spec.height <= 1080,
                "tile extends past image height"
            );
            assert!(spec.width > 0, "tile width must be positive");
            assert!(spec.height > 0, "tile height must be positive");
        }
    }
}

// ── Overlap Tests ───────────────────────────────────────────────────

#[test]
fn overlap_tiles_share_content() {
    let layout = TileLayout::new(500, 500, 256, 128);
    let stride = 256 - 128; // tile_size - overlap
    assert_eq!(stride, 128);
    let spec0 = layout.tile_spec(0, 0);
    let spec1 = layout.tile_spec(1, 0);
    // With overlap 128, tile stride is 128, so tile1 starts at x=128
    // tile0 covers 0..256, tile1 covers 128..384 (overlapping 128..256)
    assert!(spec0.x_offset + spec0.width > spec1.x_offset,
        "overlap region must exist between tiles");
}

#[test]
fn zero_overlap_tiles_abut() {
    let layout = TileLayout::new(512, 256, 256, 0);
    let spec0 = layout.tile_spec(0, 0);
    let spec1 = layout.tile_spec(1, 0);
    assert_eq!(spec0.x_offset + spec0.width, spec1.x_offset,
        "with zero overlap, tiles should abut exactly");
}

// ── Tile Count Tests ────────────────────────────────────────────────

#[test]
fn tile_count_max_resolution_8000x4000() {
    let layout = TileLayout::new(8000, 4000, 1024, 0);
    assert_eq!(layout.tiles_x, (8000u32 + 1023) / 1024);
    assert_eq!(layout.tiles_y, (4000u32 + 1023) / 1024);
    assert_eq!(layout.total_tiles, layout.tiles_x * layout.tiles_y);
}

#[test]
fn tile_count_odd_dimensions() {
    let layout = TileLayout::new(300, 300, 256, 0);
    assert_eq!(layout.tiles_x, 2);
    assert_eq!(layout.tiles_y, 2);
    assert_eq!(layout.total_tiles, 4);
}

#[test]
fn tile_count_large_8192x8192() {
    let layout = TileLayout::new(8192, 8192, 256, 0);
    assert_eq!(layout.tiles_x, 32);
    assert_eq!(layout.tiles_y, 32);
    assert_eq!(layout.total_tiles, 1024);
}

#[test]
fn tile_count_medium_1024x800() {
    let layout = TileLayout::new(1024, 800, 256, 0);
    assert_eq!(layout.tiles_x, 4);
    assert_eq!(layout.tiles_y, 4);
    assert_eq!(layout.total_tiles, 16);
}

// ── Edge Cases ──────────────────────────────────────────────────────

#[test]
fn empty_image_1x1() {
    let layout = TileLayout::new(1, 1, 256, 0);
    assert_eq!(layout.total_tiles, 1);
    let spec = layout.tile_spec(0, 0);
    assert_eq!(spec.width, 1);
    assert_eq!(spec.height, 1);
}

#[test]
fn single_row_image() {
    let layout = TileLayout::new(1024, 1, 256, 0);
    assert_eq!(layout.tiles_y, 1);
    assert_eq!(layout.tiles_x, 4);
    for tx in 0..layout.tiles_x {
        let spec = layout.tile_spec(tx, 0);
        assert_eq!(spec.height, 1);
    }
}

#[test]
fn single_column_image() {
    let layout = TileLayout::new(1, 1024, 256, 0);
    assert_eq!(layout.tiles_x, 1);
    assert_eq!(layout.tiles_y, 4);
    for ty in 0..layout.tiles_y {
        let spec = layout.tile_spec(0, ty);
        assert_eq!(spec.width, 1);
    }
}

#[test]
fn tile_larger_than_image() {
    let layout = TileLayout::new(512, 512, 1024, 0);
    assert_eq!(layout.total_tiles, 1);
    let spec = layout.tile_spec(0, 0);
    assert_eq!(spec.width, 512);
    assert_eq!(spec.height, 512);
}

#[test]
fn non_square_tiles_on_non_square_image() {
    let layout = TileLayout::new(1000, 2000, 512, 0);
    assert_eq!(layout.tiles_x, 2);
    assert_eq!(layout.tiles_y, 4);
    assert_eq!(layout.total_tiles, 8);
}

#[test]
fn odd_boundary_1001x1001_tile_128() {
    let layout = TileLayout::new(1001, 1001, 128, 0);
    let tiles_x = (1001u32 + 127) / 128;
    let tiles_y = (1001u32 + 127) / 128;
    assert_eq!(layout.tiles_x, tiles_x);
    assert_eq!(layout.tiles_y, tiles_y);
    assert_eq!(layout.total_tiles, tiles_x * tiles_y);
    // Verify all tiles have positive dimensions
    for ty in 0..layout.tiles_y {
        for tx in 0..layout.tiles_x {
            let spec = layout.tile_spec(tx, ty);
            assert!(spec.width > 0);
            assert!(spec.height > 0);
        }
    }
}

// ── Overlap Boundary Tests ──────────────────────────────────────────

#[test]
fn overlap_last_tile_clamped() {
    let layout = TileLayout::new(600, 600, 256, 64);
    let max_x = layout.tiles_x - 1;
    let max_y = layout.tiles_y - 1;
    let spec = layout.tile_spec(max_x, max_y);
    // Last tile should not exceed image bounds
    assert!(spec.x_offset + spec.width <= 600);
    assert!(spec.y_offset + spec.height <= 600);
    assert!(spec.width > 0, "last tile width must be positive");
    assert!(spec.height > 0, "last tile height must be positive");
}

// ── Tile Layout Field Correctness ───────────────────────────────────

#[test]
fn tile_layout_fields_match() {
    let layout = TileLayout::new(1920, 1080, 512, 64);
    assert_eq!(layout.image_width, 1920);
    assert_eq!(layout.image_height, 1080);
    assert_eq!(layout.tile_size, 512);
    assert_eq!(layout.overlap, 64);
}

#[test]
fn tile_layout_total_tiles_equals_x_times_y() {
    for (w, h, ts) in [
        (1920, 1080, 512),
        (800, 600, 256),
        (4096, 4096, 512),
        (100, 100, 256),
        (1, 1, 256),
    ] {
        let layout = TileLayout::new(w, h, ts, 0);
        assert_eq!(
            layout.total_tiles,
            layout.tiles_x * layout.tiles_y,
            "total tiles must equal tiles_x * tiles_y for {}x{}",
            w,
            h
        );
    }
}

// ── TileEngine Construction ─────────────────────────────────────────

#[test]
fn tile_engine_construction_values() {
    let engine = photopipeline_engine::TileEngine::new(512, 32, 4);
    assert_eq!(engine.default_tile_size, 512);
    assert_eq!(engine.overlap, 32);
    assert_eq!(engine.max_parallel, 4);
}

#[test]
fn tile_engine_default_values() {
    let engine = photopipeline_engine::TileEngine::default();
    assert_eq!(engine.default_tile_size, 1024);
    assert_eq!(engine.overlap, 64);
    assert!(engine.max_parallel >= 1);
}

#[test]
fn tile_engine_max_parallel_zero() {
    // max_parallel=0 means no parallel tile execution.
    // The engine still works — it just runs tiles sequentially.
    let engine = photopipeline_engine::TileEngine::new(512, 32, 0);
    assert_eq!(engine.max_parallel, 0);
    // Building a layout with the engine's parameters should still work
    let layout = TileLayout::new(1024, 1024, engine.default_tile_size, engine.overlap);
    assert!(layout.tiles_x > 0);
    assert!(layout.tiles_y > 0);
    assert!(layout.total_tiles > 0);
    assert_eq!(layout.tile_size, 512);
    assert_eq!(layout.overlap, 32);
}

#[test]
fn tile_engine_clone_is_equal() {
    let engine = photopipeline_engine::TileEngine::new(1024, 64, 2);
    let cloned = engine.clone();
    assert_eq!(cloned.default_tile_size, engine.default_tile_size);
    assert_eq!(cloned.overlap, engine.overlap);
    assert_eq!(cloned.max_parallel, engine.max_parallel);
}

// ── Iterate Tiles ───────────────────────────────────────────────────

#[test]
fn iterate_all_tiles_count_match() {
    let layout = TileLayout::new(1920, 1080, 512, 0);
    let tiles: Vec<_> = layout.iter_tiles().collect();
    assert_eq!(tiles.len(), layout.total_tiles as usize);
}

#[test]
fn iterate_tiles_unique_coords() {
    let layout = TileLayout::new(1920, 1080, 512, 0);
    let tiles: Vec<_> = layout.iter_tiles().collect();
    let mut coords: Vec<(u32, u32)> = tiles.iter().map(|t| (t.coord.x, t.coord.y)).collect();
    coords.sort();
    coords.dedup();
    assert_eq!(coords.len(), tiles.len(), "all tile coordinates must be unique");
}

// ── Tile Coordinate Iteration ──────────────────────────────────────

// ── Overlap Boundary with Iterate ───────────────────────────────────

#[test]
fn iterate_with_overlap_all_tiles_positive() {
    let layout = TileLayout::new(500, 500, 256, 128);
    for spec in layout.iter_tiles() {
        assert!(spec.width > 0, "every tile must have positive width");
        assert!(spec.height > 0, "every tile must have positive height");
        assert!(spec.x_offset < 500, "x_offset within image");
        assert!(spec.y_offset < 500, "y_offset within image");
    }
}
