use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use photopipeline_engine::{PipelineGraph, PipelineTemplate, TemplateEdge, TemplateNode};

fn make_template(num_nodes: usize) -> PipelineTemplate {
    let mut nodes = Vec::with_capacity(num_nodes);
    let mut edges = Vec::with_capacity(num_nodes.saturating_sub(1));
    for i in 0..num_nodes {
        nodes.push(TemplateNode {
            id: format!("n{i}"),
            plugin: "test.plugin".into(),
            label: Some(format!("Node {i}")),
            enabled: true,
            params: None,
        });
        if i > 0 {
            edges.push(TemplateEdge {
                from: format!("n{}", i - 1),
                to: format!("n{i}"),
            });
        }
    }
    PipelineTemplate {
        metadata: Default::default(),
        nodes,
        edges,
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

fn bench_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_construction");

    for size in [1usize, 10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("add_node_linear_chain", size),
            &size,
            |b, &n| {
                b.iter(|| {
                    let mut graph = PipelineGraph::new();
                    let mut prev_out = None;
                    for i in 0..n {
                        let id = graph.add_node(
                            black_box("test.plugin".into()),
                            black_box(format!("node_{i}")),
                        );
                        let out = graph.node(id).unwrap().outputs[0];
                        if let Some(prev) = prev_out {
                            graph.connect(black_box(prev), black_box(out)).ok();
                        }
                        prev_out = Some(out);
                    }
                    black_box(graph)
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("from_template", size), &size, |b, &n| {
            let template = make_template(n);
            b.iter(|| {
                let graph = PipelineGraph::from_template(black_box(&template));
                black_box(graph)
            });
        });
    }

    group.finish();
}

fn bench_pipeline_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_metadata_nodes");

    for node_count in [1usize, 5, 10, 50] {
        let template = make_template(node_count);
        let graph = template.into_graph();

        group.bench_with_input(
            BenchmarkId::new("topological_sort", node_count),
            &node_count,
            |b, _| {
                b.iter(|| {
                    let order = black_box(&graph).topological_order().unwrap();
                    black_box(order)
                });
            },
        );
    }

    group.finish();

    let mut group = c.benchmark_group("graph_validation");
    for node_count in [10usize, 50, 100, 500] {
        let template = make_template(node_count);
        let graph = template.into_graph();
        group.bench_with_input(
            BenchmarkId::new("validate_graph", node_count),
            &node_count,
            |b, _| {
                b.iter(|| {
                    let result = black_box(&graph).validate_graph();
                    black_box(result)
                });
            },
        );
    }
    group.finish();
}

fn bench_graph_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    for size in [10usize, 100, 500, 1000] {
        let template = make_template(size);
        let template_for_serde = template.clone();
        let graph = template.into_graph();

        group.bench_with_input(
            BenchmarkId::new("serialize_to_json", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(&graph)).unwrap();
                    black_box(json)
                });
            },
        );

        let json = serde_json::to_string(&graph).unwrap();

        group.bench_with_input(
            BenchmarkId::new("deserialize_from_json", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let parsed: PipelineGraph = serde_json::from_str(black_box(&json)).unwrap();
                    black_box(parsed)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("template_serialize_to_toml", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let toml_str = toml::to_string(black_box(&template_for_serde)).unwrap();
                    black_box(toml_str)
                });
            },
        );

        let toml_str = toml::to_string(&template_for_serde).unwrap();

        group.bench_with_input(
            BenchmarkId::new("template_deserialize_from_toml", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let parsed: PipelineTemplate = toml::from_str(black_box(&toml_str)).unwrap();
                    black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

fn bench_template_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_validate");
    for size in [1usize, 10, 50, 100, 500] {
        let template = make_template(size);
        group.bench_with_input(BenchmarkId::new("validate", size), &size, |b, _| {
            b.iter(|| {
                let result = black_box(&template).validate();
                black_box(result)
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_graph_construction,
    bench_pipeline_execution,
    bench_graph_serialization,
    bench_template_validate,
);
criterion_main!(benches);
