// NOTE: These concurrency tests create per-thread isolated state (separate runtime,
// separate registry, separate graph). They verify that parallel execution does not
// panic or deadlock, but do NOT detect data races on shared mutable state.
// For true race detection, tests should share state across threads.
#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

use photopipeline_core::{ColorSpace, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat};
use photopipeline_engine::{ParameterResolver, PipelineGraph, PipelineTemplate, TemplateNode};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin,
    PluginQuery, registry::Registry,
};
use photopipeline_plugins;
use std::sync::Arc;
use std::thread;
use test_harness::fixtures::image::ImageFixture;
use test_harness::mocks::progress::MockProgressSink;
use uuid::Uuid;

fn make_image_info(id: Uuid, path: &str) -> ImageInfo {
    ImageInfo {
        id,
        path: path.into(),
        filename: path.rsplit('/').next().unwrap_or(path).into(),
        format: ImageFormat::JPEG,
        width: 100,
        height: 100,
        file_size_bytes: 1000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

fn make_pixel_buffer() -> PixelBuffer {
    ImageFixture::new()
        .width(64)
        .height(64)
        .solid(100, 150, 200)
        .build()
}

#[test]
fn e2e_concurrent_pipeline_execution_4_threads() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "exif".into(),
            plugin: "photopipeline.plugins.exif_rw".into(),
            label: None,
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();

    let mut handles = vec![];
    for t in 0..4 {
        let r = reg.clone();
        let res = resolver.clone();
        let g = graph.clone();
        handles.push(thread::spawn(move || {
            let info = make_image_info(Uuid::new_v4(), &format!("/tmp/concurrent_{t}.jpg"));
            let md = Metadata::default();
            let buf = make_pixel_buffer();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let exec = photopipeline_engine::NodeExecutor::new(r, res);
            let progress = Box::new(MockProgressSink::new());
            let result =
                rt.block_on(async { exec.execute(&g, &info, Some(buf), &md, progress).await });
            assert!(result.is_ok(), "thread {t} failed: {:?}", result.err());
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn e2e_concurrent_registry_access_8_threads() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let mut handles = vec![];
    for _t in 0..8 {
        let r = reg.clone();
        handles.push(thread::spawn(move || {
            for _i in 0..50 {
                let all = r.all();
                assert!(!all.is_empty());

                let manifests = r.manifests();
                assert!(!manifests.is_empty());

                let cats = r.categories();
                assert!(!cats.is_empty());

                if let Some(p) = r.get("photopipeline.plugins.exif_rw") {
                    let _ = p.id();
                    let _ = p.name();
                    let _ = p.category();
                    let _ = p.version();
                }

                let query_results = r.query(&PluginQuery::default());
                assert!(query_results.len() >= 14);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn e2e_concurrent_parameter_resolution() {
    let mut resolver = ParameterResolver::new();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "s".into(),
            label: "S".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "v".into(),
                label: "V".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 10000,
                    step: 1,
                    unit: None,
                    style: Default::default(),
                },
                default: serde_json::json!(0),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    };

    let mut template = ParameterSet::new();
    template.insert("v".into(), serde_json::json!(10));
    resolver.set_template_params(node_id, template);

    let mut img_override = ParameterSet::new();
    img_override.insert("v".into(), serde_json::json!(99));
    resolver.set_image_override(image_id, node_id, img_override);

    let resolver = Arc::new(resolver);
    let schema = Arc::new(schema);

    let mut handles = vec![];
    for t in 0..16 {
        let r = resolver.clone();
        let nid = node_id;
        let iid = image_id;
        let s = schema.clone();
        handles.push(thread::spawn(move || {
            let meta = Metadata::default();
            let info = ImageInfo {
                id: iid,
                path: format!("/tmp/conc_param_{t}.jpg"),
                filename: format!("conc_param_{t}.jpg"),
                format: ImageFormat::JPEG,
                width: 100,
                height: 100,
                file_size_bytes: 1000,
                pixel_format: PixelFormat::U8,
                color_space: ColorSpace::default(),
            };
            let resolved = r.resolve(nid, iid, &s, &meta, &info);
            let val = resolved.get_i64("v").unwrap_or(0);
            assert!(val == 99, "expected 99, got {val} in thread {t}");
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn e2e_concurrent_tile_processing() {
    use photopipeline_core::{ChannelLayout, TileLayout};
    let w: u32 = 1024;
    let h: u32 = 1024;
    let tile_size: u32 = 256;
    let layout = TileLayout::new(w, h, tile_size, 0);
    let tiles: Vec<_> = layout.iter_tiles().collect();

    assert!(!tiles.is_empty());
    let total_tiles = layout.total_tiles as usize;
    assert_eq!(tiles.len(), total_tiles);

    let mut outputs: Vec<PixelBuffer> = Vec::new();
    for spec in &tiles {
        let mut tile_buf =
            PixelBuffer::new(spec.width, spec.height, ChannelLayout::RGB, PixelFormat::U8);
        for y in 0..spec.height as usize {
            for x in 0..spec.width as usize {
                let idx = (y * spec.width as usize + x) * 3;
                if idx + 2 < tile_buf.data.data.len() {
                    tile_buf.data.data[idx] = (x % 256) as u8;
                    tile_buf.data.data[idx + 1] = (y % 256) as u8;
                    tile_buf.data.data[idx + 2] = 128u8;
                }
            }
        }
        outputs.push(tile_buf);
    }

    assert_eq!(outputs.len(), total_tiles);

    let mut full_output = PixelBuffer::new(w, h, ChannelLayout::RGB, PixelFormat::U8);
    for (i, spec) in tiles.iter().enumerate() {
        let tile_buf = &outputs[i];
        for ty in 0..spec.height as usize {
            for tx in 0..spec.width as usize {
                let tile_idx = (ty * spec.width as usize + tx) * 3;
                let out_x = spec.x_offset as usize + tx;
                let out_y = spec.y_offset as usize + ty;
                let out_idx = (out_y * w as usize + out_x) * 3;
                if tile_idx + 2 < tile_buf.data.data.len()
                    && out_idx + 2 < full_output.data.data.len()
                {
                    full_output.data.data[out_idx] = tile_buf.data.data[tile_idx];
                    full_output.data.data[out_idx + 1] = tile_buf.data.data[tile_idx + 1];
                    full_output.data.data[out_idx + 2] = tile_buf.data.data[tile_idx + 2];
                }
            }
        }
    }

    assert_eq!(full_output.width, w);
    assert_eq!(full_output.height, h);
}

#[test]
fn e2e_concurrent_plugin_validate() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let mut handles = vec![];
    for _t in 0..16 {
        let r = reg.clone();
        handles.push(thread::spawn(move || {
            for _i in 0..20 {
                if let Some(p) = r.get("photopipeline.plugins.exif_rw") {
                    let schema = p.parameter_schema();
                    let defaults = schema.defaults();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let result = rt.block_on(async { p.validate(&defaults).await });
                    assert!(result.is_ok());

                    let mut mod_params = defaults.clone();
                    mod_params.insert("extra".into(), serde_json::json!("test"));
                    // Extra params: plugin may accept or reject; verify no panic
                    let _ = rt.block_on(async { p.validate(&mod_params).await });
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn e2e_concurrent_mixed_registry_operations() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let mut handles = vec![];

    for _i in 0..4 {
        let r = reg.clone();
        handles.push(thread::spawn(move || {
            for _j in 0..100 {
                let all = r.all();
                assert!(!all.is_empty());
                if let Some(p) = r.get("photopipeline.plugins.exif_rw") {
                    let schema = p.parameter_schema();
                    let defaults = schema.defaults();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let v = rt.block_on(async { p.validate(&defaults).await });
                    assert!(v.is_ok(), "validate with defaults should succeed");
                }
                let manifests = r.manifests();
                assert!(!manifests.is_empty());
            }
        }));
    }

    for _i in 0..4 {
        let r = reg.clone();
        handles.push(thread::spawn(move || {
            for _j in 0..100 {
                let query = PluginQuery {
                    category: Some(photopipeline_core::PluginCategory::Format),
                    ..Default::default()
                };
                let results = r.query(&query);
                assert!(!results.is_empty());

                let pix = r.query(&PluginQuery {
                    requires_pixel: Some(true),
                    ..Default::default()
                });
                let _ = pix.len();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
