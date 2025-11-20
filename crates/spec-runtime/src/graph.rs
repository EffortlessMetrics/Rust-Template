use anyhow::Result;
use serde::Serialize;

use crate::{DevExFlows, DocIndex, SpecLedger};

#[derive(Debug, Serialize, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct NodeMeta {
    #[serde(default)]
    pub must_have_ac: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub meta: NodeMeta,
}

#[derive(Debug, Serialize, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

// ... (Edge struct remains same)

pub fn build_graph(ledger: &SpecLedger, devex: &DevExFlows, docs: &DocIndex) -> Result<Graph> {
    let mut graph = Graph { nodes: Vec::new(), edges: Vec::new() };

    // Add stories, requirements, and ACs
    for story in &ledger.stories {
        graph.nodes.push(Node {
            id: story.id.clone(),
            label: story.title.clone(),
            node_type: "story".to_string(),
            url: None,
            meta: NodeMeta::default(),
        });

        for req in &story.requirements {
            graph.nodes.push(Node {
                id: req.id.clone(),
                label: req.title.clone(),
                node_type: "requirement".to_string(),
                url: None,
                meta: NodeMeta { must_have_ac: req.must_have_ac },
            });

            graph.edges.push(Edge {
                source: story.id.clone(),
                target: req.id.clone(),
                edge_type: "contains".to_string(),
            });

            for ac in &req.acceptance_criteria {
                graph.nodes.push(Node {
                    id: ac.id.clone(),
                    label: ac.text.clone(),
                    node_type: "ac".to_string(),
                    url: None,
                    meta: NodeMeta::default(),
                });

                graph.edges.push(Edge {
                    source: req.id.clone(),
                    target: ac.id.clone(),
                    edge_type: "contains".to_string(),
                });
            }
        }
    }

    // Add documents
    for doc in &docs.docs {
        graph.nodes.push(Node {
            id: doc.file.clone(),
            label: doc.id.clone(),
            node_type: "document".to_string(),
            url: Some(format!("file://{}", doc.file)),
            meta: NodeMeta::default(),
        });

        for req_id in &doc.requirements {
            graph.edges.push(Edge {
                source: doc.file.clone(),
                target: req_id.clone(),
                edge_type: "documents".to_string(),
            });
        }
    }

    // Add commands
    for cmd_name in devex.commands.keys() {
        graph.nodes.push(Node {
            id: format!("cmd:{}", cmd_name),
            label: format!("cargo xtask {}", cmd_name),
            node_type: "command".to_string(),
            url: None,
            meta: NodeMeta::default(),
        });
    }

    // Add flows
    for (flow_id, flow) in &devex.flows {
        let flow_node_id = format!("flow:{}", flow_id);
        graph.nodes.push(Node {
            id: flow_node_id.clone(),
            label: flow.name.clone(),
            node_type: "flow".to_string(),
            url: None,
            meta: NodeMeta::default(),
        });

        for step in &flow.steps {
            graph.edges.push(Edge {
                source: flow_node_id.clone(),
                target: format!("cmd:{}", step),
                edge_type: "executes".to_string(),
            });
        }
    }

    Ok(graph)
}

#[derive(Debug)]
pub struct InvariantViolation {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

pub fn check_invariants(graph: &Graph, devex: &DevExFlows) -> Result<(), Vec<InvariantViolation>> {
    let mut violations = Vec::new();

    // 1. REQ must_have_ac -> at least one AC
    for req in graph.nodes.iter().filter(|n| n.node_type == "requirement") {
        if !req.meta.must_have_ac {
            continue;
        }

        let has_ac = graph.edges.iter().any(|e| e.source == req.id && e.edge_type == "contains");

        if !has_ac {
            violations.push(InvariantViolation {
                code: "REQ_HAS_NO_AC".into(),
                message: format!("Requirement {} has no ACs in graph", req.id),
            });
        }
    }

    // 2. AC has tests (skipped for now)

    // 3. DevEx commands reachable
    for cmd_name in devex.commands.keys() {
        let cmd_id = format!("cmd:{}", cmd_name);
        let is_reachable = graph.edges.iter().any(|e| e.target == cmd_id);
        if !is_reachable {
            if let Some(cmd_spec) = devex.commands.get(cmd_name) {
                if cmd_spec.required {
                    violations.push(InvariantViolation {
                        code: "COMMAND_UNREACHABLE".into(),
                        message: format!(
                            "Required command '{}' is not used in any flow or task",
                            cmd_name
                        ),
                    });
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

impl Graph {
    pub fn to_mermaid(&self) -> String {
        let mut out = String::new();
        out.push_str("graph TD\n");

        // Emit node declarations
        for node in &self.nodes {
            let id_safe = mermaid_id(&node.id);
            let label = mermaid_label(&node.id, &node.label);
            out.push_str(&format!("  {id_safe}[\"{label}\"]\n"));
        }

        out.push('\n');

        // Emit edges
        for edge in &self.edges {
            let src = mermaid_id(&edge.source);
            let tgt = mermaid_id(&edge.target);
            let rel = &edge.edge_type;
            out.push_str(&format!("  {src} -->|\"{rel}\"| {tgt}\n"));
        }

        out
    }
}

fn mermaid_id(id: &str) -> String {
    // Mermaid node IDs must be simple identifiers: replace non-alnum with '_'
    id.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

fn mermaid_label(id: &str, label: &str) -> String {
    // Use "ID\nTitle" style; escape quotes and newlines
    let mut clean_label = label.replace('"', "\\\"");
    // Truncate overly long labels for readability
    if clean_label.len() > 80 {
        clean_label.truncate(77);
        clean_label.push_str("...");
    }
    format!("{id}\\n{clean_label}")
}
