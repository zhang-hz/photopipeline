// Engine Graph/DAG Topological Sort Tests (~30 test cases)
// Tests cover: linear chains, diamond deps, cycle detection, self-loops,
// edge validation, disconnected graphs, disabled nodes, performance, stability.

use photopipeline_engine::{PipelineGraph, PipelineTemplate, TemplateEdge, TemplateNode};

// ── Helpers ──────────────────────────────────────────────────────────

fn build_linear_graph(n_nodes: usize) -> (PipelineGraph, Vec<uuid::Uuid>) {
    let mut graph = PipelineGraph::new();
    let mut ids = Vec::new();
    for i in 0..n_nodes {
        ids.push(graph.add_node(format!("p{}", i), format!("n{}", i)));
    }
    for w in ids.windows(2) {
        let out = graph.node(w[0]).unwrap().outputs[0];
        let inp = graph.node(w[1]).unwrap().inputs[0];
        graph.connect(out, inp).unwrap();
    }
    (graph, ids)
}

fn build_diamond() -> (PipelineGraph, uuid::Uuid, uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let c = graph.add_node("c".into(), "C".into());
    let d = graph.add_node("d".into(), "D".into());

    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    let in_c = graph.node(c).unwrap().inputs[0];
    let out_b = graph.node(b).unwrap().outputs[0];
    let out_c = graph.node(c).unwrap().outputs[0];
    let in_d = graph.node(d).unwrap().inputs[0];

    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_a, in_c).unwrap();
    graph.connect(out_b, in_d).unwrap();
    graph.connect(out_c, in_d).unwrap();

    (graph, a, b, c, d)
}

// ── Linear Chain Tests ──────────────────────────────────────────────

#[test]
fn linear_3nodes_correct_order() {
    let (graph, ids) = build_linear_graph(3);
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 3);
    let pos0 = order.iter().position(|&id| id == ids[0]).unwrap();
    let pos1 = order.iter().position(|&id| id == ids[1]).unwrap();
    let pos2 = order.iter().position(|&id| id == ids[2]).unwrap();
    assert!(pos0 < pos1, "node 0 must precede node 1");
    assert!(pos1 < pos2, "node 1 must precede node 2");
    assert_eq!(order, vec![ids[0], ids[1], ids[2]]);
}

#[test]
fn linear_10nodes_correct_order() {
    let (graph, ids) = build_linear_graph(10);
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 10);
    for i in 0..9 {
        let pos_a = order.iter().position(|&id| id == ids[i]).unwrap();
        let pos_b = order.iter().position(|&id| id == ids[i + 1]).unwrap();
        assert!(pos_a < pos_b, "node {} must precede node {}", i, i + 1);
    }
}

#[test]
fn linear_5nodes_exact_order() {
    let (graph, ids) = build_linear_graph(5);
    let order = graph.topological_order().unwrap();
    assert_eq!(order, ids);
}

// ── Diamond Dependency Tests ────────────────────────────────────────

#[test]
fn diamond_4nodes_a_before_bc() {
    let (graph, a, b, c, _d) = build_diamond();
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 4);
    let pos_a = order.iter().position(|&id| id == a).unwrap();
    let pos_b = order.iter().position(|&id| id == b).unwrap();
    let pos_c = order.iter().position(|&id| id == c).unwrap();
    assert!(pos_a < pos_b, "A must precede B");
    assert!(pos_a < pos_c, "A must precede C");
}

#[test]
fn diamond_4nodes_bc_before_d() {
    let (graph, _a, b, c, d) = build_diamond();
    let order = graph.topological_order().unwrap();
    let pos_b = order.iter().position(|&id| id == b).unwrap();
    let pos_c = order.iter().position(|&id| id == c).unwrap();
    let pos_d = order.iter().position(|&id| id == d).unwrap();
    assert!(pos_b < pos_d, "B must precede D");
    assert!(pos_c < pos_d, "C must precede D");
}

#[test]
fn diamond_4nodes_all_present() {
    let (graph, a, b, c, d) = build_diamond();
    let order = graph.topological_order().unwrap();
    assert!(order.contains(&a));
    assert!(order.contains(&b));
    assert!(order.contains(&c));
    assert!(order.contains(&d));
}

#[test]
fn branch_then_merge_valid_order() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let c = graph.add_node("c".into(), "C".into());
    let d = graph.add_node("d".into(), "D".into());

    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    let out_b = graph.node(b).unwrap().outputs[0];
    let in_c = graph.node(c).unwrap().inputs[0];
    let out_d = graph.node(d).unwrap().outputs[0];

    // A->B, A->D, B->C, D->C  (diamond merge at C)
    graph.connect(out_a, in_b).unwrap();
    let in_d = graph.node(d).unwrap().inputs[0];
    graph.connect(out_a, in_d).unwrap();
    graph.connect(out_b, in_c).unwrap();
    graph.connect(out_d, in_c).unwrap();

    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 4);
    let pos_a = order.iter().position(|&id| id == a).unwrap();
    let pos_b = order.iter().position(|&id| id == b).unwrap();
    let pos_d = order.iter().position(|&id| id == d).unwrap();
    let pos_c = order.iter().position(|&id| id == c).unwrap();
    assert!(pos_a < pos_b, "A must precede B");
    assert!(pos_a < pos_d, "A must precede D");
    assert!(pos_b < pos_c, "B must precede C");
    assert!(pos_d < pos_c, "D must precede C");
}

// ── Single Node / Empty Graph ───────────────────────────────────────

#[test]
fn single_node_trivial() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], a);
}

#[test]
fn empty_graph_empty_sort() {
    let graph = PipelineGraph::new();
    let order = graph.topological_order().unwrap();
    assert!(order.is_empty(), "empty graph must produce empty sort");
}

#[test]
fn two_nodes_no_edges() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 2);
    assert!(order.contains(&a));
    assert!(order.contains(&b));
}

// ── Disconnected Components ─────────────────────────────────────────

#[test]
fn disconnected_components_all_included() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let c = graph.add_node("c".into(), "C".into());
    let d = graph.add_node("d".into(), "D".into());
    // A->B disconnected from C->D
    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    let out_c = graph.node(c).unwrap().outputs[0];
    let in_d = graph.node(d).unwrap().inputs[0];
    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_c, in_d).unwrap();

    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 4);
    assert!(order.contains(&a));
    assert!(order.contains(&b));
    assert!(order.contains(&c));
    assert!(order.contains(&d));
}

#[test]
fn disconnected_all_isolated() {
    let mut graph = PipelineGraph::new();
    let ids: Vec<_> = (0..5)
        .map(|i| graph.add_node(format!("p{i}"), format!("n{i}")))
        .collect();
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 5);
    for id in &ids {
        assert!(order.contains(id));
    }
}

// ── Cycle Detection ─────────────────────────────────────────────────

#[test]
fn simple_cycle_detected() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let c = graph.add_node("c".into(), "C".into());

    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    let out_b = graph.node(b).unwrap().outputs[0];
    let in_c = graph.node(c).unwrap().inputs[0];
    let out_c = graph.node(c).unwrap().outputs[0];
    let in_a = graph.node(a).unwrap().inputs[0];

    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_b, in_c).unwrap();
    // A->B->C->A creates a cycle
    // connect() validates so we push the cycle edge directly
    graph.edges.push((out_c, in_a));
    assert!(graph.has_cycle(), "A->B->C->A cycle must be detected");
    assert!(graph.topological_order().is_err(), "topo sort must fail on cycle");
}

#[test]
fn self_loop_detected() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let port = graph.node(a).unwrap().outputs[0];
    let result = graph.connect(port, port);
    assert!(result.is_err(), "self-loop must be rejected");
}

#[test]
fn complex_cycle_5nodes_detected() {
    let mut graph = PipelineGraph::new();
    let ids: Vec<_> = (0..5)
        .map(|i| graph.add_node(format!("p{i}"), format!("n{i}")))
        .collect();

    // A->B->C->D->E->B creates a cycle
    let out_a = graph.node(ids[0]).unwrap().outputs[0];
    let in_b = graph.node(ids[1]).unwrap().inputs[0];
    graph.connect(out_a, in_b).unwrap();

    let out_b = graph.node(ids[1]).unwrap().outputs[0];
    let in_c = graph.node(ids[2]).unwrap().inputs[0];
    graph.connect(out_b, in_c).unwrap();

    let out_c = graph.node(ids[2]).unwrap().outputs[0];
    let in_d = graph.node(ids[3]).unwrap().inputs[0];
    graph.connect(out_c, in_d).unwrap();

    let out_d = graph.node(ids[3]).unwrap().outputs[0];
    let in_e = graph.node(ids[4]).unwrap().inputs[0];
    graph.connect(out_d, in_e).unwrap();

    let out_e = graph.node(ids[4]).unwrap().outputs[0];
    let in_b2 = graph.node(ids[1]).unwrap().inputs[0];
    let result = graph.connect(out_e, in_b2);
    assert!(result.is_err(), "E->B back edge must create a cycle");
}

#[test]
fn cycle_with_diamond_reverse_edge() {
    let (mut graph, a, _b, _c, d) = build_diamond();
    // Add reverse edge D->A to create cycle
    let out_d = graph.node(d).unwrap().outputs[0];
    let in_a = graph.node(a).unwrap().inputs[0];
    let result = graph.connect(out_d, in_a);
    assert!(result.is_err(), "D->A reverse edge must create a cycle");
}

#[test]
fn no_cycle_dag_valid() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    graph.connect(out_a, in_b).unwrap();
    assert!(!graph.has_cycle());
    assert!(graph.topological_order().is_ok());
}

// ── Edge Validation ─────────────────────────────────────────────────

#[test]
fn edge_missing_source_invalid() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "p1".into(),
            label: None,
            enabled: true,
            params: None,
        }],
        edges: vec![TemplateEdge {
            from: "n2".into(), // does not exist
            to: "n1".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let result = template.validate();
    assert!(result.is_err());
    // Must mention "n2" in the error
    let msg = result.unwrap_err().to_lowercase();
    assert!(msg.contains("n2"), "error must reference missing source node n2");
}

#[test]
fn edge_missing_target_invalid() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "p1".into(),
            label: None,
            enabled: true,
            params: None,
        }],
        edges: vec![TemplateEdge {
            from: "n1".into(),
            to: "n3".into(), // does not exist
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let result = template.validate();
    assert!(result.is_err());
    let msg = result.unwrap_err().to_lowercase();
    assert!(msg.contains("n3"), "error must reference missing target node n3");
}

#[test]
fn duplicate_edge_rejected() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    graph.connect(out_a, in_b).unwrap();
    let result = graph.connect(out_a, in_b);
    assert!(result.is_err(), "duplicate edge must be rejected");
}

#[test]
fn connect_nonexistent_port_fails() {
    let mut graph = PipelineGraph::new();
    graph.add_node("a".into(), "A".into());
    let fake1 = uuid::Uuid::new_v4();
    let fake2 = uuid::Uuid::new_v4();
    let result = graph.connect(fake1, fake2);
    assert!(result.is_err());
}

// ── Disabled Node Tests ─────────────────────────────────────────────

#[test]
fn disabled_node_still_present_in_topo_sort() {
    let mut graph = PipelineGraph::new();
    let a = graph.add_node("a".into(), "A".into());
    let b = graph.add_node("b".into(), "B".into());
    let c = graph.add_node("c".into(), "C".into());

    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    let out_b = graph.node(b).unwrap().outputs[0];
    let in_c = graph.node(c).unwrap().inputs[0];
    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_b, in_c).unwrap();

    // Disable B
    graph.node_mut(b).unwrap().enabled = false;

    // topo_sort does not consider enabled/disabled; it's the executor's job.
    // So this test verifies that topo_sort still returns all nodes.
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 3);
    assert!(order.contains(&a));
    assert!(order.contains(&b));
    assert!(order.contains(&c));
}

#[test]
fn all_nodes_disabled_still_in_topo_sort() {
    let mut graph = PipelineGraph::new();
    let ids: Vec<_> = (0..3)
        .map(|i| {
            let id = graph.add_node(format!("p{i}"), format!("n{i}"));
            graph.node_mut(id).unwrap().enabled = false;
            id
        })
        .collect();
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 3);
    for id in &ids {
        assert!(order.contains(id));
    }
}

// ── Fan-out / Fan-in Tests ──────────────────────────────────────────

#[test]
fn wide_fan_out_all_children_after_parent() {
    let mut graph = PipelineGraph::new();
    let root = graph.add_node("root".into(), "R".into());
    let children: Vec<_> = (0..10)
        .map(|i| graph.add_node(format!("c{i}"), format!("C{i}")))
        .collect();

    let out_root = graph.node(root).unwrap().outputs[0];
    for &child in &children {
        let in_c = graph.node(child).unwrap().inputs[0];
        graph.connect(out_root, in_c).unwrap();
    }

    let order = graph.topological_order().unwrap();
    let pos_root = order.iter().position(|&id| id == root).unwrap();
    for &child in &children {
        let pos_c = order.iter().position(|&id| id == child).unwrap();
        assert!(pos_root < pos_c, "child must come after root");
    }
}

#[test]
fn wide_fan_in_all_parents_before_child() {
    let mut graph = PipelineGraph::new();
    let parents: Vec<_> = (0..10)
        .map(|i| graph.add_node(format!("p{i}"), format!("P{i}")))
        .collect();
    let child = graph.add_node("child".into(), "C".into());

    let in_child = graph.node(child).unwrap().inputs[0];
    for &parent in &parents {
        let out_p = graph.node(parent).unwrap().outputs[0];
        graph.connect(out_p, in_child).unwrap();
    }

    let order = graph.topological_order().unwrap();
    let pos_child = order.iter().position(|&id| id == child).unwrap();
    for &parent in &parents {
        let pos_p = order.iter().position(|&id| id == parent).unwrap();
        assert!(pos_p < pos_child, "parent must come before child");
    }
}

// ── Template Tests ──────────────────────────────────────────────────

#[test]
fn template_empty_nodes_validate_fails() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let result = template.validate();
    assert!(result.is_err(), "empty nodes must fail validation");
    let msg = result.unwrap_err().to_lowercase();
    assert!(msg.contains("node") || msg.contains("one"));
}

#[test]
fn template_single_node_valid() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
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
    assert!(template.validate().is_ok());
}

#[test]
fn template_into_graph_preserves_structure() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "in".into(),
                plugin: "in.p".into(),
                label: None,
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "out".into(),
                plugin: "out.p".into(),
                label: Some("Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![TemplateEdge {
            from: "in".into(),
            to: "out".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn template_with_disabled_node_into_graph() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "n1".into(),
                plugin: "p1".into(),
                label: None,
                enabled: false,
                params: None,
            },
            TemplateNode {
                id: "n2".into(),
                plugin: "p2".into(),
                label: None,
                enabled: true,
                params: None,
            },
        ],
        edges: vec![TemplateEdge {
            from: "n1".into(),
            to: "n2".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let n1 = graph.nodes.iter().find(|n| n.plugin_id == "p1").unwrap();
    let n2 = graph.nodes.iter().find(|n| n.plugin_id == "p2").unwrap();
    assert!(!n1.enabled, "n1 must be disabled");
    assert!(n2.enabled, "n2 must be enabled");
    // Edge should still exist in graph
    assert_eq!(graph.edges.len(), 1);
}

// ── Max Nodes Performance Test ──────────────────────────────────────

#[test]
fn max_nodes_100_linear() {
    let (graph, _ids) = build_linear_graph(100);
    let start = std::time::Instant::now();
    let order = graph.topological_order().unwrap();
    let elapsed = start.elapsed();
    assert_eq!(order.len(), 100, "must return all 100 nodes");
    assert!(elapsed.as_millis() < 500, "must finish within 500ms");
}

// ── Sort Stability ──────────────────────────────────────────────────

#[test]
fn sort_stability_same_result_5_runs() {
    let mut graph = PipelineGraph::new();
    let ids: Vec<_> = (0..10)
        .map(|i| graph.add_node(format!("p{i}"), format!("n{i}")))
        .collect();
    for w in ids.windows(2) {
        let out = graph.node(w[0]).unwrap().outputs[0];
        let inp = graph.node(w[1]).unwrap().inputs[0];
        graph.connect(out, inp).unwrap();
    }

    let first = graph.topological_order().unwrap();
    for _ in 0..5 {
        let order = graph.topological_order().unwrap();
        assert_eq!(order, first, "topological order must be stable");
    }
}

// ── Graph Serialization ─────────────────────────────────────────────

#[test]
fn serialization_roundtrip_preserves_nodes_and_edges() {
    let (graph, _ids) = build_linear_graph(3);
    let json = serde_json::to_string(&graph).unwrap();
    let restored: PipelineGraph = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.nodes.len(), 3);
    assert_eq!(restored.edges.len(), 2);
}

// ── Validate Graph Tests ───────────────────────────────────────────

#[test]
fn test_validate_duplicate_node_ids_error() {
    let mut graph = PipelineGraph::new();
    // Create two nodes with the same UUID to test duplicate detection
    let id = graph.add_node("plugin_a".into(), "Node A".into());
    let id2 = graph.add_node("plugin_b".into(), "Node B".into());
    // Override the second node's ID to duplicate the first
    graph.node_mut(id2).unwrap().id = id;
    let result = graph.validate_graph();
    assert!(result.is_err(), "validate_graph must detect duplicate node UUIDs");
    let issues = result.unwrap_err();
    assert!(!issues.is_empty(), "validation issues should not be empty");
    assert!(
        issues.iter().any(|s| s.contains("duplicate")),
        "issues must mention 'duplicate': {:?}", issues
    );
}

// ── Port Owner Tests ────────────────────────────────────────────────

#[test]
fn port_owner_correct_node() {
    let mut graph = PipelineGraph::new();
    let id = graph.add_node("p1".into(), "N1".into());
    let node = graph.node(id).unwrap();
    assert_eq!(graph.port_owner(node.outputs[0]), Some(id));
    assert_eq!(graph.port_owner(node.inputs[0]), Some(id));
}

#[test]
fn port_owner_nonexistent_port() {
    let graph = PipelineGraph::new();
    assert_eq!(graph.port_owner(uuid::Uuid::new_v4()), None);
}
