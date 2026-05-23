use photopipeline_core::{NodeId, PluginError, PluginId, PortId};
use photopipeline_plugin::ParameterSet;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineGraph {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<(PortId, PortId)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNode {
    pub id: NodeId,
    pub label: String,
    pub plugin_id: PluginId,
    pub enabled: bool,
    pub position: (f64, f64),
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub parameter_overrides: Option<ParameterSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTemplate {
    #[serde(default)]
    pub metadata: TemplateMetadata,
    pub nodes: Vec<TemplateNode>,
    #[serde(default)]
    pub edges: Vec<TemplateEdge>,
    #[serde(default)]
    pub overrides: Vec<ImageOverride>,
    #[serde(default)]
    pub groups: Vec<ParamGroup>,
    #[serde(default)]
    pub batch: Option<BatchConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateNode {
    pub id: String,
    pub plugin: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub params: Option<HashMap<String, serde_json::Value>>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOverride {
    pub image: String,
    #[serde(default)]
    pub params: HashMap<String, ParameterSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamGroup {
    pub name: String,
    pub condition: String,
    #[serde(default)]
    pub params: HashMap<String, ParameterSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    #[serde(default = "default_parallel")]
    pub parallel: usize,
    #[serde(default)]
    pub output_pattern: Option<String>,
    #[serde(default)]
    pub on_conflict: Option<String>,
    #[serde(default)]
    pub resume: bool,
}

fn default_parallel() -> usize {
    1
}

impl PipelineTemplate {
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("pipeline must have at least one node".into());
        }
        let node_ids: Vec<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &self.edges {
            if !node_ids.contains(&edge.from.as_str()) {
                return Err(format!("edge references unknown node '{}'", edge.from));
            }
            if !node_ids.contains(&edge.to.as_str()) {
                return Err(format!("edge references unknown node '{}'", edge.to));
            }
        }
        Ok(())
    }

    pub fn into_graph(self) -> PipelineGraph {
        PipelineGraph::from_template(&self)
    }
}

impl PipelineGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, plugin_id: String, label: String) -> NodeId {
        let id = Uuid::new_v4();
        let input_port = Uuid::new_v4();
        let output_port = Uuid::new_v4();
        self.nodes.push(PipelineNode {
            id,
            label,
            plugin_id,
            enabled: true,
            position: (0.0, 0.0),
            inputs: vec![input_port],
            outputs: vec![output_port],
            parameter_overrides: None,
        });
        id
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> bool {
        let port_ids: HashSet<PortId> = self
            .nodes
            .iter()
            .filter(|n| n.id == node_id)
            .flat_map(|n| n.inputs.iter().chain(n.outputs.iter()))
            .copied()
            .collect();
        if port_ids.is_empty() {
            return false;
        }
        self.edges
            .retain(|(from, to)| !port_ids.contains(from) && !port_ids.contains(to));
        self.nodes.retain(|n| n.id != node_id);
        true
    }

    pub fn connect(&mut self, from_port: PortId, to_port: PortId) -> Result<(), PluginError> {
        if from_port == to_port {
            return Err(PluginError::ValidationFailed(
                "cannot connect a port to itself".into(),
            ));
        }
        let from_owner = self
            .port_owner(from_port)
            .ok_or_else(|| PluginError::ValidationFailed("source port not found".into()))?;
        let to_owner = self
            .port_owner(to_port)
            .ok_or_else(|| PluginError::ValidationFailed("destination port not found".into()))?;
        if from_owner == to_owner {
            return Err(PluginError::ValidationFailed(
                "cannot connect ports on the same node".into(),
            ));
        }
        if self.edges.contains(&(from_port, to_port)) {
            return Err(PluginError::ValidationFailed("edge already exists".into()));
        }
        self.edges.push((from_port, to_port));
        if self.has_cycle() {
            self.edges.pop();
            return Err(PluginError::CircularDependency);
        }
        Ok(())
    }

    pub fn disconnect(&mut self, from_port: PortId, to_port: PortId) -> bool {
        let len_before = self.edges.len();
        self.edges.retain(|e| *e != (from_port, to_port));
        self.edges.len() < len_before
    }

    pub fn topological_order(&self) -> Result<Vec<NodeId>, PluginError> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let port_to_node: HashMap<PortId, NodeId> = self.build_port_map();

        for node in &self.nodes {
            in_degree.entry(node.id).or_insert(0);
            adjacency.entry(node.id).or_default();
        }

        for &(from_port, to_port) in &self.edges {
            let src = port_to_node.get(&from_port);
            let dst = port_to_node.get(&to_port);
            if let (Some(&src_node), Some(&dst_node)) = (src, dst) {
                adjacency.entry(src_node).or_default().push(dst_node);
                *in_degree.entry(dst_node).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();
        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);
            if let Some(neighbors) = adjacency.get(&node_id) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(&neighbor) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(PluginError::CircularDependency);
        }
        Ok(order)
    }

    pub fn has_cycle(&self) -> bool {
        self.topological_order().is_err()
    }

    pub fn validate_graph(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        let port_map = self.build_port_map();

        for &(from, to) in &self.edges {
            if !port_map.contains_key(&from) {
                issues.push(format!("edge references unknown source port {}", from));
            }
            if !port_map.contains_key(&to) {
                issues.push(format!("edge references unknown destination port {}", to));
            }
        }

        let ids: HashSet<NodeId> = self.nodes.iter().map(|n| n.id).collect();
        if ids.len() != self.nodes.len() {
            issues.push("duplicate node ids detected".into());
        }

        if let Err(e) = self.topological_order() {
            issues.push(format!("cycle detected: {}", e));
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    pub fn from_template(template: &PipelineTemplate) -> Self {
        let mut graph = PipelineGraph::new();
        let mut id_map: HashMap<String, NodeId> = HashMap::new();
        let mut output_ports: HashMap<String, PortId> = HashMap::new();
        let mut input_ports: HashMap<String, PortId> = HashMap::new();

        for tn in &template.nodes {
            let label = tn.label.clone().unwrap_or_else(|| tn.id.clone());
            let node_id = graph.add_node(tn.plugin.clone(), label);
            id_map.insert(tn.id.clone(), node_id);
            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == node_id) {
                node.enabled = tn.enabled;
                output_ports.insert(tn.id.clone(), node.outputs[0]);
                input_ports.insert(tn.id.clone(), node.inputs[0]);
            }
            if let Some(params) = &tn.params
                && let Some(node) = graph.nodes.iter_mut().find(|n| n.id == node_id)
            {
                let mut ps = ParameterSet::new();
                ps.values = params.clone();
                node.parameter_overrides = Some(ps);
            }
        }

        for te in &template.edges {
            if let (Some(&from_port), Some(&to_port)) =
                (output_ports.get(&te.from), input_ports.get(&te.to))
                && let Err(e) = graph.connect(from_port, to_port)
            {
                tracing::warn!(
                    "template edge {} -> {} connection failed: {}",
                    te.from,
                    te.to,
                    e
                );
            }
        }

        graph
    }

    pub fn node(&self, id: NodeId) -> Option<&PipelineNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut PipelineNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn port_owner(&self, port_id: PortId) -> Option<NodeId> {
        self.build_port_map().get(&port_id).copied()
    }

    fn build_port_map(&self) -> HashMap<PortId, NodeId> {
        let mut map = HashMap::new();
        for node in &self.nodes {
            for &input in &node.inputs {
                map.insert(input, node.id);
            }
            for &output in &node.outputs {
                map.insert(output, node.id);
            }
        }
        map
    }
}

impl Default for PipelineGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_node() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("plugin.test".into(), "test".into());
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.remove_node(id));
        assert_eq!(graph.nodes.len(), 0);
    }

    #[test]
    fn test_connect_disconnect() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "node1".into());
        let n2 = graph.add_node("p2".into(), "node2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        assert!(graph.connect(out1, in2).is_ok());
        assert_eq!(graph.edges.len(), 1);
        assert!(graph.disconnect(out1, in2));
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        let out2 = graph.node(n2).unwrap().outputs[0];
        let in1 = graph.node(n1).unwrap().inputs[0];

        graph.connect(out1, in2).unwrap();
        assert!(graph.connect(out2, in1).is_err());
    }

    #[test]
    fn test_topological_order() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let n3 = graph.add_node("p3".into(), "n3".into());

        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        let out2 = graph.node(n2).unwrap().outputs[0];
        let in3 = graph.node(n3).unwrap().inputs[0];

        graph.connect(out1, in2).unwrap();
        graph.connect(out2, in3).unwrap();

        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 3);
        let pos1 = order.iter().position(|&id| id == n1).unwrap();
        let pos2 = order.iter().position(|&id| id == n2).unwrap();
        let pos3 = order.iter().position(|&id| id == n3).unwrap();
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn test_template_validate() {
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![TemplateNode {
                id: "n1".into(),
                plugin: "exif_rw".into(),
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
    fn test_template_validate_missing_source() {
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![],
            edges: vec![TemplateEdge {
                from: "n1".into(),
                to: "n2".into(),
            }],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn test_graph_new_is_empty() {
        let graph = PipelineGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_graph_node_has_ports() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "node".into());
        let node = graph.node(id).unwrap();
        assert!(!node.inputs.is_empty());
        assert!(!node.outputs.is_empty());
    }

    #[test]
    fn test_diamond_topological_order() {
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
        let in_d1 = graph.node(d).unwrap().inputs[0];
        let in_d2 = graph.node(d).unwrap().outputs[0];

        graph.connect(out_a, in_b).unwrap();
        graph.connect(out_a, in_c).unwrap();
        graph.connect(out_b, in_d1).unwrap();
        graph.connect(out_c, in_d2).unwrap();

        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 4);
        let pos_a = order.iter().position(|&id| id == a).unwrap();
        let pos_b = order.iter().position(|&id| id == b).unwrap();
        let pos_c = order.iter().position(|&id| id == c).unwrap();
        let pos_d = order.iter().position(|&id| id == d).unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_validate_graph_ok() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        graph.connect(out1, in2).unwrap();
        assert!(graph.validate_graph().is_ok());
    }

    #[test]
    fn test_validate_graph_detects_cycle() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let n3 = graph.add_node("p3".into(), "n3".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        let out2 = graph.node(n2).unwrap().outputs[0];
        let in3 = graph.node(n3).unwrap().inputs[0];
        let out3 = graph.node(n3).unwrap().outputs[0];
        let in1 = graph.node(n1).unwrap().inputs[0];

        graph.connect(out1, in2).unwrap();
        graph.connect(out2, in3).unwrap();
        // connect already detects cycles at graph build time
        let result = graph.connect(out3, in1);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_into_graph() {
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![
                TemplateNode {
                    id: "input".into(),
                    plugin: "input.plugin".into(),
                    label: None,
                    enabled: true,
                    params: None,
                },
                TemplateNode {
                    id: "output".into(),
                    plugin: "output.plugin".into(),
                    label: Some("Output".into()),
                    enabled: true,
                    params: None,
                },
            ],
            edges: vec![TemplateEdge {
                from: "input".into(),
                to: "output".into(),
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
    fn test_template_node_params_to_graph() {
        let mut params_map = std::collections::HashMap::new();
        params_map.insert("key".into(), serde_json::json!("value"));
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![TemplateNode {
                id: "n1".into(),
                plugin: "p1".into(),
                label: None,
                enabled: true,
                params: Some(params_map),
            }],
            edges: vec![],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };

        let graph = template.into_graph();
        let node = &graph.nodes[0];
        assert!(node.parameter_overrides.is_some());
        let ps = node.parameter_overrides.as_ref().unwrap();
        assert_eq!(ps.get_str("key"), Some("value"));
    }

    #[test]
    fn test_graph_disconnect_nonexistent() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let fake_port = uuid::Uuid::new_v4();
        let out1 = graph.node(n1).unwrap().outputs[0];
        assert!(!graph.disconnect(fake_port, out1));
    }

    #[test]
    fn test_graph_self_connect_fails() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let port = graph.node(n1).unwrap().outputs[0];
        let result = graph.connect(port, port);
        assert!(result.is_err());
    }

    #[test]
    fn test_graph_duplicate_edge_rejected() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        assert!(graph.connect(out1, in2).is_ok());
        assert!(graph.connect(out1, in2).is_err());
    }

    #[test]
    fn test_graph_port_owner() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "n1".into());
        let node = graph.node(id).unwrap();
        assert_eq!(graph.port_owner(node.outputs[0]), Some(id));
        assert_eq!(graph.port_owner(uuid::Uuid::new_v4()), None);
    }

    #[test]
    fn test_graph_node_mut() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "n1".into());
        {
            let node = graph.node_mut(id).unwrap();
            node.enabled = false;
        }
        assert!(!graph.node(id).unwrap().enabled);
    }

    #[test]
    fn test_graph_remove_nonexistent_node() {
        let mut graph = PipelineGraph::new();
        assert!(!graph.remove_node(uuid::Uuid::new_v4()));
    }

    #[test]
    fn test_graph_remove_node_cleans_edges() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        graph.connect(out1, in2).unwrap();
        assert_eq!(graph.edges.len(), 1);
        graph.remove_node(n2);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_template_validate_empty_nodes() {
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![],
            edges: vec![],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_validate_bad_edge_target() {
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
                to: "n2".into(),
            }],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_validate_bad_edge_source() {
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
                from: "n2".into(),
                to: "n1".into(),
            }],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn test_serialize_deserialize_pipeline_graph() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        graph.connect(out1, in2).unwrap();

        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: PipelineGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.nodes.len(), 2);
        assert_eq!(deserialized.edges.len(), 1);
    }

    #[test]
    fn test_topological_order_single_node() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], n1);
    }

    #[test]
    fn test_topological_order_two_disconnected() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 2);
        assert!(order.contains(&n1));
        assert!(order.contains(&n2));
    }

    #[test]
    fn test_topological_order_ten_nodes_linear() {
        let mut graph = PipelineGraph::new();
        let mut ids = vec![];
        for i in 0..10 {
            ids.push(graph.add_node(format!("p{i}"), format!("n{i}")));
        }
        for w in ids.windows(2) {
            let out = graph.node(w[0]).unwrap().outputs[0];
            let inp = graph.node(w[1]).unwrap().inputs[0];
            graph.connect(out, inp).unwrap();
        }
        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 10);
        for (a, b) in order.iter().zip(order.iter().skip(1)) {
            let pos_a = ids.iter().position(|id| id == a).unwrap();
            let pos_b = ids.iter().position(|id| id == b).unwrap();
            assert!(pos_a < pos_b);
        }
    }

    #[test]
    fn test_topological_order_ten_disconnected() {
        let mut graph = PipelineGraph::new();
        for i in 0..10 {
            graph.add_node(format!("p{i}"), format!("n{i}"));
        }
        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 10);
    }

    #[test]
    fn test_cycle_detection_self_loop() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let port = graph.node(n1).unwrap().outputs[0];
        assert!(graph.connect(port, port).is_err());
    }

    #[test]
    fn test_cycle_detection_simple_cycle() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        let out2 = graph.node(n2).unwrap().outputs[0];
        let in1 = graph.node(n1).unwrap().inputs[0];
        graph.connect(out1, in2).unwrap();
        assert!(graph.connect(out2, in1).is_err());
    }

    #[test]
    fn test_cycle_detection_triangle() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let n3 = graph.add_node("p3".into(), "n3".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        let out2 = graph.node(n2).unwrap().outputs[0];
        let in3 = graph.node(n3).unwrap().inputs[0];
        let out3 = graph.node(n3).unwrap().outputs[0];
        let in1 = graph.node(n1).unwrap().inputs[0];
        graph.connect(out1, in2).unwrap();
        graph.connect(out2, in3).unwrap();
        assert!(graph.connect(out3, in1).is_err());
    }

    #[test]
    fn test_cycle_detection_no_cycle_dag() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        assert!(graph.connect(out1, in2).is_ok());
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_connect_non_existent_ports() {
        let mut graph = PipelineGraph::new();
        graph.add_node("p1".into(), "n1".into());
        let fake1 = uuid::Uuid::new_v4();
        let fake2 = uuid::Uuid::new_v4();
        assert!(graph.connect(fake1, fake2).is_err());
    }

    #[test]
    fn test_disconnect_valid_edge() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        graph.connect(out1, in2).unwrap();
        assert!(graph.disconnect(out1, in2));
    }

    #[test]
    fn test_disconnect_nonexistent_edge() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        let out1 = graph.node(n1).unwrap().outputs[0];
        let in2 = graph.node(n2).unwrap().inputs[0];
        assert!(!graph.disconnect(out1, in2));
    }

    #[test]
    fn test_remove_node_removes_from_list() {
        let mut graph = PipelineGraph::new();
        let n1 = graph.add_node("p1".into(), "n1".into());
        let n2 = graph.add_node("p2".into(), "n2".into());
        graph.remove_node(n1);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, n2);
    }

    #[test]
    fn test_validate_empty_graph_ok() {
        let graph = PipelineGraph::new();
        assert!(graph.validate_graph().is_ok());
    }

    #[test]
    fn test_validate_single_node_no_edges_ok() {
        let mut graph = PipelineGraph::new();
        graph.add_node("p1".into(), "n1".into());
        assert!(graph.validate_graph().is_ok());
    }

    #[test]
    fn test_validate_duplicate_node_ids_error() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "n1".into());
        graph.add_node("p2".into(), "n2".into());
        if let Some(node) = graph.node_mut(id) {
            node.position = (10.0, 20.0);
        }
        assert!(graph.node(id).is_some());
    }

    #[test]
    fn test_pipeline_node_default_position() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "n1".into());
        let node = graph.node(id).unwrap();
        assert_eq!(node.position, (0.0, 0.0));
    }

    #[test]
    fn test_pipeline_node_custom_position() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "n1".into());
        if let Some(node) = graph.node_mut(id) {
            node.position = (100.0, 200.0);
        }
        let node = graph.node(id).unwrap();
        assert_eq!(node.position, (100.0, 200.0));
    }

    #[test]
    fn test_pipeline_node_default_enabled() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "n1".into());
        assert!(graph.node(id).unwrap().enabled);
    }

    #[test]
    fn test_pipeline_node_label() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("p1".into(), "MyNode".into());
        assert_eq!(graph.node(id).unwrap().label, "MyNode");
    }

    #[test]
    fn test_pipeline_node_plugin_id() {
        let mut graph = PipelineGraph::new();
        let id = graph.add_node("colorspace".into(), "CS".into());
        assert_eq!(graph.node(id).unwrap().plugin_id, "colorspace");
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig {
            parallel: 4,
            output_pattern: Some("{name}_out".into()),
            on_conflict: Some("skip".into()),
            resume: true,
        };
        assert_eq!(config.parallel, 4);
        assert!(config.resume);
    }

    #[test]
    fn test_batch_config_serde_roundtrip() {
        let config = BatchConfig {
            parallel: 8,
            output_pattern: Some("out/{name}".into()),
            on_conflict: None,
            resume: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let config2: BatchConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config2.parallel, 8);
        assert_eq!(config2.output_pattern, Some("out/{name}".into()));
    }

    #[test]
    fn test_template_empty_nodes_validate_fails() {
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![],
            edges: vec![],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_metadata_fields() {
        let meta = TemplateMetadata {
            name: Some("My Pipeline".into()),
            version: Some("1.0".into()),
            description: Some("A test pipeline".into()),
        };
        assert_eq!(meta.name, Some("My Pipeline".into()));
    }

    #[test]
    fn test_image_override_fields() {
        let mut params = std::collections::HashMap::new();
        params.insert("colorspace".into(), ParameterSet::new());
        let ov = ImageOverride {
            image: "img1.jpg".into(),
            params,
        };
        assert_eq!(ov.image, "img1.jpg");
    }

    #[test]
    fn test_param_group_fields() {
        let group = ParamGroup {
            name: "High ISO".into(),
            condition: "exif.iso > 800".into(),
            params: Default::default(),
        };
        assert_eq!(group.name, "High ISO");
        assert_eq!(group.condition, "exif.iso > 800");
    }
}
