/// Build a single-node pipeline JSON config
pub fn single_node_config(plugin_id: &str, params: Option<Vec<(&str, serde_json::Value)>>) -> String {
    let mut node = serde_json::json!({
        "id": "n1",
        "plugin": plugin_id,
        "enabled": true
    });
    if let Some(p) = params {
        let params_map: serde_json::Map<String, serde_json::Value> = p.into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        node["params"] = serde_json::json!(params_map);
    }
    build_json(vec![node], vec![])
}

/// Build a two-node chain pipeline JSON config
pub fn two_node_config(
    p1_id: &str, p1_params: Option<Vec<(&str, serde_json::Value)>>,
    p2_id: &str, p2_params: Option<Vec<(&str, serde_json::Value)>>,
) -> String {
    let mut n1 = serde_json::json!({"id": "n1", "plugin": p1_id, "enabled": true});
    let mut n2 = serde_json::json!({"id": "n2", "plugin": p2_id, "enabled": true});
    if let Some(ref p) = p1_params {
        n1["params"] = params_to_json(p);
    }
    if let Some(ref p) = p2_params {
        n2["params"] = params_to_json(p);
    }
    let edges = vec![serde_json::json!({"from": "n1", "to": "n2"})];
    build_json(vec![n1, n2], edges)
}

/// Build a three-node chain pipeline JSON config
pub fn three_node_config(
    p1_id: &str, p1_params: Option<Vec<(&str, serde_json::Value)>>,
    p2_id: &str, p2_params: Option<Vec<(&str, serde_json::Value)>>,
    p3_id: &str, p3_params: Option<Vec<(&str, serde_json::Value)>>,
) -> String {
    let mut n1 = serde_json::json!({"id": "n1", "plugin": p1_id, "enabled": true});
    let mut n2 = serde_json::json!({"id": "n2", "plugin": p2_id, "enabled": true});
    let mut n3 = serde_json::json!({"id": "n3", "plugin": p3_id, "enabled": true});
    if let Some(ref p) = p1_params { n1["params"] = params_to_json(p); }
    if let Some(ref p) = p2_params { n2["params"] = params_to_json(p); }
    if let Some(ref p) = p3_params { n3["params"] = params_to_json(p); }
    let edges = vec![
        serde_json::json!({"from": "n1", "to": "n2"}),
        serde_json::json!({"from": "n2", "to": "n3"}),
    ];
    build_json(vec![n1, n2, n3], edges)
}

/// Build a four-node chain pipeline JSON config
pub fn four_node_config(
    p1_id: &str, p1_params: Option<Vec<(&str, serde_json::Value)>>,
    p2_id: &str, p2_params: Option<Vec<(&str, serde_json::Value)>>,
    p3_id: &str, p3_params: Option<Vec<(&str, serde_json::Value)>>,
    p4_id: &str, p4_params: Option<Vec<(&str, serde_json::Value)>>,
) -> String {
    let mut n1 = serde_json::json!({"id": "n1", "plugin": p1_id, "enabled": true});
    let mut n2 = serde_json::json!({"id": "n2", "plugin": p2_id, "enabled": true});
    let mut n3 = serde_json::json!({"id": "n3", "plugin": p3_id, "enabled": true});
    let mut n4 = serde_json::json!({"id": "n4", "plugin": p4_id, "enabled": true});
    if let Some(ref p) = p1_params { n1["params"] = params_to_json(p); }
    if let Some(ref p) = p2_params { n2["params"] = params_to_json(p); }
    if let Some(ref p) = p3_params { n3["params"] = params_to_json(p); }
    if let Some(ref p) = p4_params { n4["params"] = params_to_json(p); }
    let edges = vec![
        serde_json::json!({"from": "n1", "to": "n2"}),
        serde_json::json!({"from": "n2", "to": "n3"}),
        serde_json::json!({"from": "n3", "to": "n4"}),
    ];
    build_json(vec![n1, n2, n3, n4], edges)
}

/// Build a diamond topology config (A→B, A→C, B→D, C→D)
pub fn diamond_config(p1: &str, p2: &str, p3: &str, p4: &str) -> String {
    build_json(
        vec![
            serde_json::json!({"id": "A", "plugin": p1, "enabled": true}),
            serde_json::json!({"id": "B", "plugin": p2, "enabled": true}),
            serde_json::json!({"id": "C", "plugin": p3, "enabled": true}),
            serde_json::json!({"id": "D", "plugin": p4, "enabled": true}),
        ],
        vec![
            serde_json::json!({"from": "A", "to": "B"}),
            serde_json::json!({"from": "A", "to": "C"}),
            serde_json::json!({"from": "B", "to": "D"}),
            serde_json::json!({"from": "C", "to": "D"}),
        ],
    )
}

/// Build an N-node linear chain config
pub fn linear_chain_config(n: usize, plugin_id: &str) -> String {
    let nodes: Vec<_> = (0..n).map(|i| {
        serde_json::json!({"id": format!("n{}", i), "plugin": plugin_id, "enabled": true})
    }).collect();
    let edges: Vec<_> = (0..n-1).map(|i| {
        serde_json::json!({"from": format!("n{}", i), "to": format!("n{}", i+1)})
    }).collect();
    build_json(nodes, edges)
}

/// Build a config with a disabled node
pub fn disabled_node_config(plugin_id: &str, disabled_pos: usize) -> String {
    let nodes: Vec<_> = (0..3).map(|i| {
        serde_json::json!({"id": format!("n{}", i), "plugin": plugin_id, "enabled": i != disabled_pos})
    }).collect();
    let edges: Vec<_> = vec![
        serde_json::json!({"from": "n0", "to": "n1"}),
        serde_json::json!({"from": "n1", "to": "n2"}),
    ];
    build_json(nodes, edges)
}

/// Build bare JSON pipeline config from nodes and edges
pub fn build_json(nodes: Vec<serde_json::Value>, edges: Vec<serde_json::Value>) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "metadata": { "name": "E2E Test Pipeline" },
        "nodes": nodes,
        "edges": edges
    })).unwrap()
}

fn params_to_json(params: &[(&str, serde_json::Value)]) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = params.iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    serde_json::json!(map)
}
